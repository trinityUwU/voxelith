//! Responsabilité : écrans de menu (principal, sélection/création de monde, options,
//! pause) — rendu + interaction souris, retourne une action à exécuter par main.

use crate::ui_kit::Ui;
use crate::world_save::WorldMeta;

/// Écran courant de l'application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    WorldSelect,
    Create,
    Settings,
    InGame,
    Pause,
}

/// Champ de saisie focalisé dans l'écran de création.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    None,
    Name,
    Seed,
}

/// État mutable des menus (liste de mondes, champs de création).
pub struct MenuState {
    pub worlds: Vec<WorldMeta>,
    pub name: String,
    pub seed: String,
    pub focus: Field,
}

impl Default for MenuState {
    fn default() -> Self {
        Self { worlds: Vec::new(), name: String::new(), seed: String::new(), focus: Field::None }
    }
}

/// Action décidée par l'utilisateur dans un menu.
pub enum MenuAction {
    None,
    Goto(Screen),
    PlayNew { name: String, seed: String },
    PlayLoad(Box<WorldMeta>),
    Resume,
    SaveQuit,
    Quit,
}

const BTN_W: f32 = 460.0;
const BTN_H: f32 = 56.0;
const GAP: f32 = 70.0;
const MUTED: [f32; 4] = [0.7, 0.72, 0.78, 1.0];

/// Rend l'écran et retourne l'action utilisateur (mutant le focus des champs).
pub fn render(ui: &mut Ui, state: &mut MenuState, screen: Screen) -> MenuAction {
    match screen {
        Screen::Main => main_menu(ui),
        Screen::WorldSelect => world_select(ui, state),
        Screen::Create => create_world(ui, state),
        Screen::Settings => settings(ui),
        Screen::Pause => pause(ui),
        Screen::InGame => MenuAction::None,
    }
}

fn main_menu(ui: &mut Ui) -> MenuAction {
    ui.dim(0.55);
    ui.title(ui.h * 0.16, "VOXELITH", 9.0);
    let mut y = ui.h * 0.40;
    if ui.button(y, BTN_W, BTN_H, "JOUER") {
        return MenuAction::Goto(Screen::WorldSelect);
    }
    y += GAP;
    if ui.button(y, BTN_W, BTN_H, "OPTIONS") {
        return MenuAction::Goto(Screen::Settings);
    }
    y += GAP;
    if ui.button(y, BTN_W, BTN_H, "QUITTER") {
        return MenuAction::Quit;
    }
    MenuAction::None
}

fn world_select(ui: &mut Ui, state: &mut MenuState) -> MenuAction {
    ui.dim(0.6);
    ui.title(ui.h * 0.10, "SELECTIONNER UN MONDE", 4.0);
    let mut y = ui.h * 0.24;
    for meta in state.worlds.iter().take(6) {
        let label = format!("{}  [{}]", meta.name.to_uppercase(), meta.gamemode.to_uppercase());
        if ui.button(y, BTN_W, 48.0, &label) {
            return MenuAction::PlayLoad(Box::new(meta.clone()));
        }
        y += 58.0;
    }
    if state.worlds.is_empty() {
        ui.label((ui.w - 260.0) / 2.0, y + 8.0, 2.0, "AUCUN MONDE - CREES-EN UN", MUTED);
    }
    let y = ui.h - 150.0;
    if ui.button(y, BTN_W, BTN_H, "NOUVEAU MONDE") {
        return MenuAction::Goto(Screen::Create);
    }
    if ui.button(y + GAP, BTN_W, BTN_H, "RETOUR") {
        return MenuAction::Goto(Screen::Main);
    }
    MenuAction::None
}

fn create_world(ui: &mut Ui, state: &mut MenuState) -> MenuAction {
    ui.dim(0.6);
    ui.title(ui.h * 0.12, "NOUVEAU MONDE", 5.0);
    let y = ui.h * 0.34;
    if ui.field(y, BTN_W, 46.0, "NOM DU MONDE", &state.name, state.focus == Field::Name) {
        state.focus = Field::Name;
    }
    if ui.field(y + 100.0, BTN_W, 46.0, "SEED (VIDE = ALEATOIRE)", &state.seed, state.focus == Field::Seed) {
        state.focus = Field::Seed;
    }
    let by = ui.h - 150.0;
    if ui.button(by, BTN_W, BTN_H, "CREER") {
        let name = if state.name.trim().is_empty() { "Nouveau Monde".to_string() } else { state.name.trim().to_string() };
        return MenuAction::PlayNew { name, seed: state.seed.clone() };
    }
    if ui.button(by + GAP, BTN_W, BTN_H, "RETOUR") {
        return MenuAction::Goto(Screen::WorldSelect);
    }
    MenuAction::None
}

fn settings(ui: &mut Ui) -> MenuAction {
    ui.dim(0.6);
    ui.title(ui.h * 0.14, "OPTIONS", 6.0);
    let x = (ui.w - 520.0) / 2.0;
    let mut y = ui.h * 0.34;
    for line in [
        "DEPLACEMENT : ZQSD / WASD",
        "SAUTER / MONTER : ESPACE",
        "DESCENDRE (CREATIF) : SHIFT",
        "CASSER / POSER : CLIC G / CLIC D",
        "BLOCS : TOUCHES 1-8",
        "CHAT : T     PAUSE : ECHAP",
    ] {
        ui.label(x, y, 2.0, line, MUTED);
        y += 34.0;
    }
    if ui.button(ui.h - 110.0, BTN_W, BTN_H, "RETOUR") {
        return MenuAction::Goto(Screen::Main);
    }
    MenuAction::None
}

fn pause(ui: &mut Ui) -> MenuAction {
    ui.dim(0.5);
    ui.title(ui.h * 0.20, "PAUSE", 7.0);
    let y = ui.h * 0.44;
    if ui.button(y, BTN_W, BTN_H, "REPRENDRE") {
        return MenuAction::Resume;
    }
    if ui.button(y + GAP, BTN_W, BTN_H, "SAUVEGARDER ET QUITTER") {
        return MenuAction::SaveQuit;
    }
    MenuAction::None
}
