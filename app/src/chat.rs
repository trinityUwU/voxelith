//! Responsabilité : état et rendu du chat (touche T) — saisie de texte, historique
//! avec fondu temporel, et affichage. L'exécution des commandes est faite par main.

use std::collections::VecDeque;
use std::time::Instant;

use voxel_render::UiBatch;

const MAX_HISTORY: usize = 50;
const VISIBLE_LINES: usize = 10;
const FADE_SECS: f32 = 8.0;
const SCALE: f32 = 2.0;

const TEXT: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.6];
const PANEL: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// État du chat : ouvert/fermé, ligne en cours, historique horodaté.
#[derive(Default)]
pub struct Chat {
    pub open: bool,
    input: String,
    history: VecDeque<(String, Instant)>,
}

impl Chat {
    /// Ouvre le chat (saisie vide).
    pub fn open(&mut self) {
        self.open = true;
        self.input.clear();
    }

    /// Ouvre le chat avec un texte pré-rempli (ex. "/" pour une commande).
    pub fn open_with(&mut self, prefix: &str) {
        self.open = true;
        self.input.clear();
        self.input.push_str(prefix);
    }

    /// Ferme le chat sans envoyer.
    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
    }

    /// Ajoute du texte saisi (caractères imprimables uniquement).
    pub fn type_str(&mut self, s: &str) {
        for ch in s.chars() {
            if !ch.is_control() && self.input.len() < 200 {
                self.input.push(ch);
            }
        }
    }

    /// Efface le dernier caractère.
    pub fn backspace(&mut self) {
        self.input.pop();
    }

    /// Valide la ligne courante : la retourne et ferme le chat (vide = None).
    pub fn submit(&mut self) -> Option<String> {
        let line = self.input.trim().to_string();
        self.open = false;
        self.input.clear();
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    /// Pousse un message dans l'historique (réponse de commande, chat).
    pub fn push(&mut self, msg: impl Into<String>) {
        self.history.push_back((msg.into(), Instant::now()));
        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
    }

    /// Empile le rendu du chat : historique (fondu si fermé) + ligne de saisie.
    pub fn build(&self, ui: &mut UiBatch, _w: u32, h: u32) {
        let line_h = UiBatch::text_height(SCALE) + 4.0;
        let base_y = h as f32 - 64.0;
        for (i, (msg, ts)) in self.recent_lines().enumerate() {
            let alpha = self.line_alpha(ts);
            if alpha <= 0.0 {
                continue;
            }
            let y = base_y - (i as f32 + 1.0) * line_h;
            shadowed(ui, 8.0, y, SCALE, msg, [TEXT[0], TEXT[1], TEXT[2], alpha]);
        }
        if self.open {
            let y = h as f32 - 44.0;
            ui.rect(0.0, y - 6.0, _w as f32, line_h + 6.0, PANEL);
            let prompt = format!("> {}_", self.input);
            shadowed(ui, 8.0, y, SCALE, &prompt, TEXT);
        }
    }

    /// Les `VISIBLE_LINES` derniers messages, du plus récent (i=0) au plus ancien.
    fn recent_lines(&self) -> impl Iterator<Item = &(String, Instant)> {
        self.history.iter().rev().take(VISIBLE_LINES)
    }

    /// Opacité d'une ligne : pleine si chat ouvert, sinon fondu sur FADE_SECS.
    fn line_alpha(&self, ts: &Instant) -> f32 {
        if self.open {
            return 1.0;
        }
        (1.0 - ts.elapsed().as_secs_f32() / FADE_SECS).clamp(0.0, 1.0)
    }
}

/// Texte avec ombre portée.
fn shadowed(ui: &mut UiBatch, x: f32, y: f32, scale: f32, s: &str, color: [f32; 4]) {
    ui.text(x + scale, y + scale, scale, s, [SHADOW[0], SHADOW[1], SHADOW[2], color[3] * 0.6]);
    ui.text(x, y, scale, s, color);
}
