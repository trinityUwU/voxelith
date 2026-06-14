//! voxelith — moteur voxel type Minecraft (Rust + wgpu, GPU-driven).
//! Binaire socle (phase 00) : fenêtre, terrain procédural, caméra fly libre.

mod input;

use std::sync::Arc;
use std::time::Instant;

use input::InputState;
use voxel_render::Renderer;
use voxel_world::{ChunkPos, World};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

/// Rayon de génération de chunks autour de l'origine (49×49 = 2401 chunks).
const SOCLE_RADIUS: i32 = 24;
const MOUSE_SENSITIVITY: f32 = 0.0025;

/// Application winit : fenêtre + renderer + état d'entrée, créés au `resumed`.
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    last_frame: Option<Instant>,
}

impl App {
    /// Génère un disque de chunks de terrain autour de l'origine.
    fn build_world() -> World {
        let mut world = World::new();
        for x in -SOCLE_RADIUS..=SOCLE_RADIUS {
            for z in -SOCLE_RADIUS..=SOCLE_RADIUS {
                world.insert(voxel_world::gen::generate_chunk(ChunkPos::new(x, z)));
            }
        }
        world
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("voxelith — phase 01");
        let window = Arc::new(event_loop.create_window(attrs).expect("création fenêtre"));
        grab_cursor(&window);
        let world = Self::build_world();
        let renderer = pollster::block_on(Renderer::new(window.clone(), &world));
        log::info!("monde généré : {} chunks", world.loaded_count());
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.last_frame = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape {
                        event_loop.exit();
                        return;
                    }
                    self.input.set(code, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(renderer) = self.renderer.as_mut() {
                let cam = renderer.camera_mut();
                cam.yaw += delta.0 as f32 * MOUSE_SENSITIVITY;
                cam.pitch = (cam.pitch - delta.1 as f32 * MOUSE_SENSITIVITY).clamp(-1.54, 1.54);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    /// Avance la simulation d'une frame et redessine.
    fn draw(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let now = Instant::now();
        let dt = self.last_frame.map_or(0.016, |t| (now - t).as_secs_f32());
        self.last_frame = Some(now);

        self.input.apply(renderer.camera_mut(), dt);
        renderer.render();
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
