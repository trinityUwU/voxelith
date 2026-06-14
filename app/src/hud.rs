//! Responsabilité : construit le HUD en jeu (crosshair, ligne d'info, hotbar)
//! dans un UiBatch à partir de l'état joueur.

use voxel_render::UiBatch;
use voxel_world::registry::name;
use voxel_world::BlockState;

use crate::player::Player;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.85];
const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.55];
const PANEL: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

/// Empile le HUD complet pour la frame.
pub fn build(ui: &mut UiBatch, w: u32, h: u32, player: &Player, held: BlockState) {
    crosshair(ui, w, h);
    info_line(ui, player, held);
}

/// Crosshair central (deux barres blanches).
fn crosshair(ui: &mut UiBatch, w: u32, h: u32) {
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    ui.rect(cx - 1.0, cy - 9.0, 2.0, 18.0, WHITE);
    ui.rect(cx - 9.0, cy - 1.0, 18.0, 2.0, WHITE);
}

/// Ligne d'info en haut à gauche (mode, position, bloc tenu).
fn info_line(ui: &mut UiBatch, player: &Player, held: BlockState) {
    let line = format!(
        "VOXELITH  {}  XYZ {:.0} {:.0} {:.0}  BLOC {}",
        player.mode.label().to_uppercase(),
        player.pos.x,
        player.pos.y,
        player.pos.z,
        name(held).to_uppercase(),
    );
    text_shadowed(ui, 10.0, 10.0, 2.0, &line, WHITE);
}

/// Texte avec fond translucide + ombre portée pour la lisibilité.
fn text_shadowed(ui: &mut UiBatch, x: f32, y: f32, scale: f32, s: &str, color: [f32; 4]) {
    let tw = UiBatch::text_width(s, scale);
    let th = UiBatch::text_height(scale);
    ui.rect(x - 4.0, y - 4.0, tw + 8.0, th + 8.0, PANEL);
    ui.text(x + scale, y + scale, scale, s, SHADOW);
    ui.text(x, y, scale, s, color);
}
