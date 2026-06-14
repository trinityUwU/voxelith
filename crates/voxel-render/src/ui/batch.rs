//! Responsabilité : accumulation de la géométrie 2D d'une frame d'UI (rectangles
//! pleins + texte bitmap) en coordonnées pixels. Consommé par l'overlay au rendu.

use bytemuck::{Pod, Zeroable};

use super::font5x7::{glyph, GLYPH_H, GLYPH_W};

/// Vertex d'UI : position pixels (origine haut-gauche), UV atlas, couleur, mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct UiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    /// 0 = rectangle plein, 1 = texte (alpha depuis l'atlas de police).
    pub mode: u32,
}

/// Nombre de glyphes imprimables de l'atlas (ASCII 32..126).
pub const ATLAS_GLYPHS: usize = 95;

/// Géométrie 2D accumulée pour une frame d'UI.
#[derive(Default)]
pub struct UiBatch {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
}

impl UiBatch {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Rectangle plein opaque/translucide.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.quad([x, y], [x + w, y + h], [0.0, 0.0], [0.0, 0.0], color, 0);
    }

    /// Écrit une chaîne ; retourne la largeur pixels avancée.
    pub fn text(&mut self, x: f32, y: f32, scale: f32, s: &str, color: [f32; 4]) -> f32 {
        let mut cx = x;
        let cell = GLYPH_W as f32 * scale;
        for ch in s.chars() {
            self.glyph_quads(ch, cx, y, scale, color);
            cx += cell + scale; // 1 px de gouttière entre glyphes
        }
        cx - x
    }

    /// Largeur pixels qu'occuperait une chaîne (sans l'écrire).
    pub fn text_width(s: &str, scale: f32) -> f32 {
        let cell = (GLYPH_W as f32 + 1.0) * scale;
        s.chars().count() as f32 * cell
    }

    /// Hauteur pixels d'une ligne de texte.
    pub fn text_height(scale: f32) -> f32 {
        GLYPH_H as f32 * scale
    }

    /// Émet les quads des pixels allumés d'un glyphe.
    fn glyph_quads(&mut self, ch: char, x: f32, y: f32, scale: f32, color: [f32; 4]) {
        let bits = glyph(ch);
        for (row, mask) in bits.iter().enumerate() {
            for col in 0..GLYPH_W {
                if mask & (1 << (GLYPH_W - 1 - col)) != 0 {
                    let px = x + col as f32 * scale;
                    let py = y + row as f32 * scale;
                    self.quad([px, py], [px + scale, py + scale], [0.0, 0.0], [0.0, 0.0], color, 0);
                }
            }
        }
    }

    /// Quad générique (deux triangles) entre deux coins.
    fn quad(&mut self, min: [f32; 2], max: [f32; 2], uv0: [f32; 2], uv1: [f32; 2], color: [f32; 4], mode: u32) {
        let start = self.vertices.len() as u32;
        let corners = [
            ([min[0], min[1]], [uv0[0], uv0[1]]),
            ([max[0], min[1]], [uv1[0], uv0[1]]),
            ([max[0], max[1]], [uv1[0], uv1[1]]),
            ([min[0], max[1]], [uv0[0], uv1[1]]),
        ];
        for (pos, uv) in corners {
            self.vertices.push(UiVertex { pos, uv, color, mode });
        }
        self.indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
    }
}
