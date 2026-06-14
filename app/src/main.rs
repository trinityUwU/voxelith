//! voxelith — moteur voxel type Minecraft (Rust + wgpu, GPU-driven).
//! Binaire : fenêtre, streaming infini, joueur physique + gamemodes, caméra.

mod edit;
mod input;
mod player;
mod stream;

use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use edit::{raycast, touched_chunks};
use input::InputState;
use player::{GameMode, Player};
use stream::ChunkManager;
use voxel_render::Renderer;
use voxel_world::registry::{AIR, DIRT, GRASS, GRAVEL, LEAVES, SAND, SNOW, STONE, WOOD};
use voxel_world::{BlockState, WorldStore};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const SEED: u32 = 1337;
const VIEW_DISTANCE: i32 = 32;
const UPLOAD_BUDGET: usize = 16;
const MOUSE_SENSITIVITY: f32 = 0.0025;

/// Application winit : fenêtre, rendu, monde partagé, streaming, joueur.
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    store: Option<Arc<WorldStore>>,
    manager: Option<ChunkManager>,
    player: Option<Player>,
    input: InputState,
    held: BlockState,
    last_frame: Option<Instant>,
    frame: u64,
}

/// Blocs sélectionnables par les touches 1..8.
const PALETTE: [BlockState; 8] = [STONE, DIRT, GRASS, SAND, SNOW, GRAVEL, WOOD, LEAVES];

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("voxelith");
        let window = Arc::new(event_loop.create_window(attrs).expect("création fenêtre"));
        grab_cursor(&window);

        let store = Arc::new(WorldStore::new(SEED));
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        let manager = ChunkManager::new(store.clone(), VIEW_DISTANCE);
        let spawn_y = store.height(0, 0) as f32 + 2.0;
        let player = Player::new(Vec3::new(0.5, spawn_y, 0.5), GameMode::Creative);

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.store = Some(store);
        self.manager = Some(manager);
        self.player = Some(player);
        self.held = STONE;
        self.last_frame = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event_loop, event),
            WindowEvent::MouseInput { state, button, .. } if state == ElementState::Pressed => {
                match button {
                    MouseButton::Left => self.edit(true),
                    MouseButton::Right => self.edit(false),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let (DeviceEvent::MouseMotion { delta }, Some(player)) = (event, self.player.as_mut()) {
            player.yaw += delta.0 as f32 * MOUSE_SENSITIVITY;
            player.pitch = (player.pitch - delta.1 as f32 * MOUSE_SENSITIVITY).clamp(-1.54, 1.54);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    /// Traite une touche : Échap quitte, G bascule le gamemode (temporaire avant le chat).
    fn on_key(&mut self, event_loop: &ActiveEventLoop, event: winit::event::KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let pressed = event.state == ElementState::Pressed;
        if pressed && code == KeyCode::Escape {
            event_loop.exit();
            return;
        }
        if pressed && code == KeyCode::KeyG {
            if let Some(player) = self.player.as_mut() {
                let next = match player.mode {
                    GameMode::Creative => GameMode::Survival,
                    GameMode::Survival => GameMode::Creative,
                };
                player.set_mode(next);
                log::info!("gamemode → {}", next.label());
            }
        }
        if pressed {
            if let Some(i) = digit_index(code) {
                self.held = PALETTE[i];
            }
        }
        self.input.set(code, pressed);
    }

    /// Casse (clic gauche) ou pose (clic droit) le bloc visé, puis re-meshe.
    fn edit(&mut self, break_block: bool) {
        let (Some(store), Some(player), Some(manager)) =
            (self.store.as_ref(), self.player.as_ref(), self.manager.as_ref())
        else {
            return;
        };
        let Some(hit) = raycast(store, player.eye(), player.look_dir(), player.reach()) else {
            return;
        };
        let target = if break_block { hit.block } else { hit.place };
        if break_block {
            store.set_block(target.x, target.y, target.z, AIR);
        } else {
            if player.occupies(target) {
                return;
            }
            store.set_block(target.x, target.y, target.z, self.held);
        }
        for pos in touched_chunks(target.x, target.z) {
            manager.remesh(pos);
        }
    }

    /// Avance physique + streaming + caméra d'une frame et redessine.
    fn draw(&mut self) {
        let (Some(renderer), Some(manager), Some(player), Some(store)) = (
            self.renderer.as_mut(),
            self.manager.as_mut(),
            self.player.as_mut(),
            self.store.as_ref(),
        ) else {
            return;
        };
        let now = Instant::now();
        let dt = self.last_frame.map_or(0.016, |t| (now - t).as_secs_f32()).min(0.1);
        self.last_frame = Some(now);

        player.update(&self.input, dt, store);
        for pos in manager.update(player.pos) {
            renderer.remove_chunk(pos);
        }
        for result in manager.drain(UPLOAD_BUDGET) {
            if result.mesh.is_empty() {
                renderer.remove_chunk(result.pos);
            } else {
                renderer.upload_chunk(result.pos, &result.mesh, result.aabb);
            }
        }

        let cam = renderer.camera_mut();
        cam.position = player.eye();
        cam.yaw = player.yaw;
        cam.pitch = player.pitch;
        renderer.render();

        self.frame += 1;
        if self.frame % 120 == 0 {
            log::info!("{} chunks | {} | pos {:.0},{:.0},{:.0}", renderer.loaded_count(), player.mode.label(), player.pos.x, player.pos.y, player.pos.z);
        }
    }
}

/// Index de palette (0..7) pour les touches 1..8, sinon None.
fn digit_index(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        _ => None,
    }
}

/// Verrouille et masque le curseur pour le contrôle caméra FPS (fallback Confined sur X11).
fn grab_cursor(window: &Window) {
    let grabbed = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
    if grabbed.is_err() {
        log::warn!("capture du curseur indisponible sur cette plateforme");
    }
    window.set_cursor_visible(false);
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let event_loop = EventLoop::new().expect("création event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run_app");
}
