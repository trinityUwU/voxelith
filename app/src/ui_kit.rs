//! Responsabilité : widgets d'UI immediate-mode (boutons, libellés, panneaux) avec
//! interaction souris, par-dessus UiBatch. Utilisé par les menus.

use voxel_render::UiBatch;

const BTN: [f32; 4] = [0.22, 0.22, 0.26, 0.92];
const BTN_HOVER: [f32; 4] = [0.36, 0.36, 0.42, 0.96];
const BORDER: [f32; 4] = [0.58, 0.58, 0.64, 1.0];
const TEXT: [f32; 4] = [0.96, 0.96, 0.96, 1.0];
const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.6];
const FIELD: [f32; 4] = [0.08, 0.08, 0.10, 0.95];
const FIELD_FOCUS: [f32; 4] = [0.14, 0.14, 0.20, 0.97];

/// Contexte d'UI immediate-mode pour une frame de menu.
pub struct Ui<'a> {
    batch: &'a mut UiBatch,
    mouse: [f32; 2],
    clicked: bool,
    pub w: f32,
    pub h: f32,
}

impl<'a> Ui<'a> {
    pub fn new(batch: &'a mut UiBatch, mouse: [f32; 2], clicked: bool, size: (u32, u32)) -> Self {
        Self { batch, mouse, clicked, w: size.0 as f32, h: size.1 as f32 }
    }

    /// Voile de fond plein écran (assombrit le jeu sous le menu).
    pub fn dim(&mut self, alpha: f32) {
        self.batch.rect(0.0, 0.0, self.w, self.h, [0.04, 0.05, 0.07, alpha]);
    }

    /// Titre centré horizontalement.
    pub fn title(&mut self, y: f32, text: &str, scale: f32) {
        let tw = UiBatch::text_width(text, scale);
        let x = (self.w - tw) / 2.0;
        self.batch.text(x + scale, y + scale, scale, text, SHADOW);
        self.batch.text(x, y, scale, text, TEXT);
    }

    /// Libellé aligné à gauche.
    pub fn label(&mut self, x: f32, y: f32, scale: f32, text: &str, color: [f32; 4]) {
        self.batch.text(x, y, scale, text, color);
    }

    /// Bouton centré horizontalement ; retourne `true` au clic.
    pub fn button(&mut self, y: f32, w: f32, h: f32, label: &str) -> bool {
        let x = (self.w - w) / 2.0;
        self.button_at(x, y, w, h, label)
    }

    /// Bouton positionné ; retourne `true` au clic.
    pub fn button_at(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str) -> bool {
        let hover = self.hit(x, y, w, h);
        self.outlined(x, y, w, h, if hover { BTN_HOVER } else { BTN });
        let scale = 3.0;
        let tw = UiBatch::text_width(label, scale);
        let th = UiBatch::text_height(scale);
        let (tx, ty) = (x + (w - tw) / 2.0, y + (h - th) / 2.0);
        self.batch.text(tx + 1.0, ty + 1.0, scale, label, SHADOW);
        self.batch.text(tx, ty, scale, label, TEXT);
        hover && self.clicked
    }

    /// Champ de saisie (libellé + valeur) ; retourne `true` au clic (pour le focus).
    pub fn field(&mut self, y: f32, w: f32, h: f32, caption: &str, value: &str, focused: bool) -> bool {
        let x = (self.w - w) / 2.0;
        self.label(x, y - 22.0, 2.0, caption, TEXT);
        let hover = self.hit(x, y, w, h);
        self.outlined(x, y, w, h, if focused { FIELD_FOCUS } else { FIELD });
        let shown = if focused { format!("{value}_") } else { value.to_string() };
        let th = UiBatch::text_height(3.0);
        self.batch.text(x + 10.0, y + (h - th) / 2.0, 3.0, &shown, TEXT);
        hover && self.clicked
    }

    /// `true` si la souris est dans le rectangle.
    fn hit(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.mouse[0] >= x && self.mouse[0] <= x + w && self.mouse[1] >= y && self.mouse[1] <= y + h
    }

    /// Rectangle plein avec bordure 2 px.
    fn outlined(&mut self, x: f32, y: f32, w: f32, h: f32, fill: [f32; 4]) {
        self.batch.rect(x - 2.0, y - 2.0, w + 4.0, h + 4.0, BORDER);
        self.batch.rect(x, y, w, h, fill);
    }
}
