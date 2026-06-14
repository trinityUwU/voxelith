//! voxelith — moteur voxel type Minecraft (Rust + wgpu).
//! Binaire : menus, mondes persistants, streaming infini, joueur + gamemodes, édition.

mod chat;
mod edit;
mod hud;
mod input;
mod menu;
mod player;
mod stream;
mod ui_kit;
mod world_save;

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use chat::Chat;
use edit::{raycast, touched_chunks};
use glam::Vec3;
use input::InputState;
use menu::{Field, MenuAction, MenuState, Screen};
use player::{GameMode, Player};
use stream::ChunkManager;
use ui_kit::Ui;
use voxel_render::{Renderer, UiBatch};
use voxel_world::registry::{AIR, DIRT, GRASS, GRAVEL, LEAVES, SAND, SNOW, STONE, WOOD};
use voxel_world::{BlockState, WorldStore};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const VIEW_DISTANCE: i32 = 32;
const UPLOAD_BUDGET: usize = 16;
const MOUSE_SENSITIVITY: f32 = 0.0025;
const PALETTE: [BlockState; 8] = [STONE, DIRT, GRASS, SAND, SNOW, GRAVEL, WOOD, LEAVES];

/// Monde actif (store partagé + streaming + joueur + métadonnées de sauvegarde).
struct Session {
    store: Arc<WorldStore>,
    manager: ChunkManager,
    player: Player,
    meta: world_save::WorldMeta,
}

/// Application : fenêtre, rendu, écran courant, menus et session de jeu éventuelle.
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    session: Option<Session>,
    screen: Screen,
    menu: MenuState,
    input: InputState,
    empty_input: InputState,
    chat: Chat,
    held: BlockState,
    ui: UiBatch,
    mouse_pos: [f32; 2],
    mouse_clicked: bool,
    should_exit: bool,
    last_frame: Option<Instant>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Main
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("voxelith");
        let window = Arc::new(event_loop.create_window(attrs).expect("création fenêtre"));
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.held = STONE;
        self.menu.worlds = world_save::list_worlds();
        self.last_frame = Some(Instant::now());
    }

    fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.should_exit = true,
            WindowEvent::Resized(size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = [position.x as f32, position.y as f32];
            }
            WindowEvent::MouseInput { state, button, .. } if state == ElementState::Pressed => {
                self.on_click(button);
            }
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event),
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if self.screen != Screen::InGame || self.chat.open {
            return;
        }
        if let (DeviceEvent::MouseMotion { delta }, Some(s)) = (event, self.session.as_mut()) {
            s.player.yaw += delta.0 as f32 * MOUSE_SENSITIVITY;
            s.player.pitch = (s.player.pitch - delta.1 as f32 * MOUSE_SENSITIVITY).clamp(-1.54, 1.54);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_exit {
            event_loop.exit();
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    /// Clic souris : édition en jeu, sinon marque un clic pour les menus.
    fn on_click(&mut self, button: MouseButton) {
        if self.screen == Screen::InGame && !self.chat.open {
            match button {
                MouseButton::Left => self.edit(true),
                MouseButton::Right => self.edit(false),
                _ => {}
            }
        } else if button == MouseButton::Left {
            self.mouse_clicked = true;
        }
    }

    /// Routage clavier selon l'écran courant.
    fn on_key(&mut self, event: winit::event::KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let pressed = event.state == ElementState::Pressed;
        match self.screen {
            Screen::InGame => self.on_game_key(code, pressed, event.text.as_deref()),
            Screen::Create => self.on_create_key(code, pressed, event.text.as_deref()),
            _ => {
                if pressed && code == KeyCode::Escape {
                    self.menu_escape();
                }
            }
        }
    }

    /// Clavier en jeu : chat prioritaire, Échap ouvre la pause, sinon contrôles.
    fn on_game_key(&mut self, code: KeyCode, pressed: bool, text: Option<&str>) {
        if self.chat.open {
            if pressed {
                self.on_chat_key(code, text);
            }
            return;
        }
        if pressed {
            match code {
                KeyCode::KeyT => {
                    self.chat.open();
                    self.input.clear();
                    return;
                }
                KeyCode::Escape => {
                    self.set_screen(Screen::Pause);
                    return;
                }
                _ => {}
            }
            if let Some(i) = digit_index(code) {
                self.held = PALETTE[i];
            }
        }
        self.input.set(code, pressed);
    }

    /// Saisie clavier dans le chat.
    fn on_chat_key(&mut self, code: KeyCode, text: Option<&str>) {
        match code {
            KeyCode::Escape => self.chat.close(),
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if let Some(line) = self.chat.submit() {
                    self.run_command(line);
                }
            }
            KeyCode::Backspace => self.chat.backspace(),
            _ => {
                if let Some(t) = text {
                    self.chat.type_str(t);
                }
            }
        }
    }

    /// Saisie clavier dans l'écran de création de monde.
    fn on_create_key(&mut self, code: KeyCode, pressed: bool, text: Option<&str>) {
        if !pressed {
            return;
        }
        match code {
            KeyCode::Escape => self.set_screen(Screen::WorldSelect),
            KeyCode::Enter | KeyCode::NumpadEnter => self.create_from_menu(),
            KeyCode::Backspace => {
                self.focused_field().map(String::pop);
            }
            _ => {
                if let (Some(t), Some(field)) = (text, self.focused_field()) {
                    push_printable(field, t);
                }
            }
        }
    }

    /// Champ de saisie actuellement focalisé (création).
    fn focused_field(&mut self) -> Option<&mut String> {
        match self.menu.focus {
            Field::Name => Some(&mut self.menu.name),
            Field::Seed => Some(&mut self.menu.seed),
            Field::None => None,
        }
    }

    /// Échap dans un menu : revient en arrière selon l'écran.
    fn menu_escape(&mut self) {
        let target = match self.screen {
            Screen::Pause => Screen::InGame,
            Screen::Create => Screen::WorldSelect,
            Screen::WorldSelect | Screen::Settings => Screen::Main,
            other => other,
        };
        self.set_screen(target);
    }

    /// Change d'écran et ajuste la capture du curseur.
    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        if screen == Screen::WorldSelect {
            self.menu.worlds = world_save::list_worlds();
        }
        self.set_cursor_grabbed(screen == Screen::InGame);
    }

    /// Verrouille/masque (jeu) ou libère/affiche (menu) le curseur.
    fn set_cursor_grabbed(&self, grabbed: bool) {
        let Some(window) = &self.window else {
            return;
        };
        if grabbed {
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
        }
        window.set_cursor_visible(!grabbed);
    }

    /// Casse (clic gauche) ou pose (clic droit) le bloc visé, puis re-meshe.
    fn edit(&mut self, break_block: bool) {
        let held = self.held;
        let Some(s) = self.session.as_ref() else {
            return;
        };
        let Some(hit) = raycast(&s.store, s.player.eye(), s.player.look_dir(), s.player.reach()) else {
            return;
        };
        let target = if break_block { hit.block } else { hit.place };
        if break_block {
            s.store.set_block(target.x, target.y, target.z, AIR);
        } else {
            if s.player.occupies(target) {
                return;
            }
            s.store.set_block(target.x, target.y, target.z, held);
        }
        for pos in touched_chunks(target.x, target.z) {
            s.manager.remesh(pos);
        }
    }

    /// Traite une ligne de chat : commande (/...) ou message.
    fn run_command(&mut self, line: String) {
        if let Some(cmd) = line.strip_prefix('/') {
            let response = self.exec(cmd);
            self.chat.push(response);
        } else {
            self.chat.push(format!("<joueur> {line}"));
        }
    }

    /// Exécute une commande et retourne la réponse.
    fn exec(&mut self, cmd: &str) -> String {
        let mut args = cmd.split_whitespace();
        match args.next() {
            Some("gamemode") | Some("gm") => self.cmd_gamemode(args.next()),
            Some("tp") => self.cmd_tp(args.next(), args.next(), args.next()),
            Some("seed") => match self.session.as_ref() {
                Some(s) => format!("seed: {}", s.store.seed()),
                None => "monde indisponible".into(),
            },
            Some("help") => "commandes: /gamemode <creative|survival> /tp <x y z> /seed /help".into(),
            Some(other) => format!("commande inconnue: {other}"),
            None => "tape une commande apres /".into(),
        }
    }

    fn cmd_gamemode(&mut self, arg: Option<&str>) -> String {
        let mode = match arg {
            Some("creative") | Some("c") | Some("1") => GameMode::Creative,
            Some("survival") | Some("s") | Some("0") => GameMode::Survival,
            _ => return "usage: /gamemode <creative|survival>".into(),
        };
        if let Some(s) = self.session.as_mut() {
            s.player.set_mode(mode);
        }
        format!("gamemode -> {}", mode.label())
    }

    fn cmd_tp(&mut self, x: Option<&str>, y: Option<&str>, z: Option<&str>) -> String {
        let coords = (|| Some(Vec3::new(x?.parse().ok()?, y?.parse().ok()?, z?.parse().ok()?)))();
        match (coords, self.session.as_mut()) {
            (Some(pos), Some(s)) => {
                s.player.teleport(pos);
                format!("teleporte en {:.0} {:.0} {:.0}", pos.x, pos.y, pos.z)
            }
            _ => "usage: /tp <x> <y> <z>".into(),
        }
    }

    /// Avance le jeu + le streaming d'un pas de temps (uniquement en jeu).
    fn tick_game(&mut self, dt: f32) {
        let input = if self.chat.open { &self.empty_input } else { &self.input };
        let Some(s) = self.session.as_mut() else {
            return;
        };
        s.player.update(input, dt, &s.store);
        for pos in s.manager.update(s.player.pos) {
            if let Some(r) = self.renderer.as_mut() {
                r.remove_chunk(pos);
            }
        }
        let results = s.manager.drain(UPLOAD_BUDGET);
        if let Some(r) = self.renderer.as_mut() {
            for result in results {
                if result.mesh.is_empty() {
                    r.remove_chunk(result.pos);
                } else {
                    r.upload_chunk(result.pos, &result.mesh, result.aabb);
                }
            }
            let cam = r.camera_mut();
            cam.position = s.player.eye();
            cam.yaw = s.player.yaw;
            cam.pitch = s.player.pitch;
        }
    }

    /// Construit l'UI de la frame et exécute l'action de menu éventuelle.
    fn draw(&mut self) {
        let now = Instant::now();
        let dt = self.last_frame.map_or(0.016, |t| (now - t).as_secs_f32()).min(0.1);
        self.last_frame = Some(now);

        if self.screen == Screen::InGame {
            self.tick_game(dt);
        }
        let Some(size) = self.renderer.as_ref().map(|r| r.size()) else {
            return;
        };
        self.ui.clear();
        if matches!(self.screen, Screen::InGame | Screen::Pause) {
            if let Some(s) = self.session.as_ref() {
                hud::build(&mut self.ui, size.0, size.1, &s.player, self.held);
            }
            self.chat.build(&mut self.ui, size.0, size.1);
        }
        let action = self.build_menu(size);
        if let Some(r) = self.renderer.as_mut() {
            r.render(&self.ui);
        }
        self.mouse_clicked = false;
        self.handle_action(action);
    }

    /// Empile l'UI de menu et retourne l'action utilisateur (None en jeu).
    fn build_menu(&mut self, size: (u32, u32)) -> MenuAction {
        if self.screen == Screen::InGame {
            return MenuAction::None;
        }
        let mut ui = Ui::new(&mut self.ui, self.mouse_pos, self.mouse_clicked, size);
        menu::render(&mut ui, &mut self.menu, self.screen)
    }

    /// Applique l'action de menu : navigation, démarrage/sauvegarde de monde, quitter.
    fn handle_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::None => {}
            MenuAction::Goto(screen) => self.set_screen(screen),
            MenuAction::Resume => self.set_screen(Screen::InGame),
            MenuAction::Quit => self.should_exit = true,
            MenuAction::PlayNew { name, seed } => self.start_new_world(name, &seed),
            MenuAction::PlayLoad(meta) => self.start_world(*meta),
            MenuAction::SaveQuit => self.save_and_quit(),
        }
    }

    /// Crée et démarre un monde neuf depuis les champs de création.
    fn create_from_menu(&mut self) {
        let name = if self.menu.name.trim().is_empty() {
            "Nouveau Monde".to_string()
        } else {
            self.menu.name.trim().to_string()
        };
        let seed = self.menu.seed.clone();
        self.start_new_world(name, &seed);
    }

    /// Démarre un monde neuf (seed résolue, spawn sur le terrain).
    fn start_new_world(&mut self, name: String, seed_str: &str) {
        let seed = parse_seed(seed_str);
        let store = WorldStore::new(seed);
        let spawn_y = store.height(0, 0) as f32 + 2.0;
        let meta = world_save::WorldMeta {
            name,
            seed,
            gamemode: "creative".into(),
            player: [0.5, spawn_y, 0.5],
            yaw: -std::f32::consts::FRAC_PI_4,
            pitch: -0.3,
        };
        self.boot_session(Arc::new(store), meta);
    }

    /// Charge un monde existant (overrides + état joueur sauvegardés).
    fn start_world(&mut self, meta: world_save::WorldMeta) {
        let store = WorldStore::new(meta.seed);
        store.import_overrides(&world_save::load_edits(&meta.name));
        self.boot_session(Arc::new(store), meta);
    }

    /// Initialise la session (manager + joueur) et entre en jeu.
    fn boot_session(&mut self, store: Arc<WorldStore>, meta: world_save::WorldMeta) {
        let manager = ChunkManager::new(store.clone(), VIEW_DISTANCE);
        let mode = if meta.gamemode == "survival" { GameMode::Survival } else { GameMode::Creative };
        let mut player = Player::new(Vec3::from(meta.player), mode);
        player.yaw = meta.yaw;
        player.pitch = meta.pitch;
        self.menu.name.clear();
        self.menu.seed.clear();
        self.menu.focus = Field::None;
        self.session = Some(Session { store, manager, player, meta });
        self.set_screen(Screen::InGame);
    }

    /// Sauvegarde l'état courant et revient au menu principal.
    fn save_and_quit(&mut self) {
        if let Some(s) = self.session.as_ref() {
            let mut meta = s.meta.clone();
            meta.player = s.player.pos.into();
            meta.yaw = s.player.yaw;
            meta.pitch = s.player.pitch;
            meta.gamemode = s.player.mode.label().to_string();
            if let Err(e) = world_save::save_world(&meta, s.store.export_overrides()) {
                log::error!("échec sauvegarde : {e}");
            }
        }
        if let Some(r) = self.renderer.as_mut() {
            r.clear_chunks();
        }
        self.session = None;
        self.set_screen(Screen::Main);
    }
}

/// Index de palette (0..7) pour les touches 1..8.
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

/// Ajoute des caractères imprimables à un champ de saisie (max 32).
fn push_printable(field: &mut String, text: &str) {
    for ch in text.chars() {
        if !ch.is_control() && field.chars().count() < 32 {
            field.push(ch);
        }
    }
}

/// Résout une seed : nombre, sinon hash du texte, sinon aléatoire (horloge).
fn parse_seed(s: &str) -> u32 {
    let s = s.trim();
    if s.is_empty() {
        return (SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u32).unwrap_or(1)) | 1;
    }
    if let Ok(n) = s.parse::<u32>() {
        return n;
    }
    s.bytes().fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619))
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let event_loop = EventLoop::new().expect("création event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run_app");
}
