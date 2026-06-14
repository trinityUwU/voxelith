//! Responsabilité : registry des biomes et sélection multi-noise (temperature,
//! humidity). Le biome décide les blocs de surface/sous-sol et la teinte d'herbe.

use crate::block::BlockState;
use crate::registry::{DIRT, GRASS, SAND, SNOW};

/// Biome de l'overworld : blocs de surface/remplissage + teinte d'herbe.
#[derive(Debug, Clone, Copy)]
pub struct Biome {
    pub name: &'static str,
    pub surface: BlockState,
    pub filler: BlockState,
    pub grass_tint: [u8; 3],
    /// Point climatique de référence (temperature, humidity) dans [0,1].
    climate: [f32; 2],
}

/// Table des biomes. Sélection par plus proche voisin dans l'espace climatique.
pub const BIOMES: &[Biome] = &[
    Biome { name: "plains", surface: GRASS, filler: DIRT, grass_tint: [120, 184, 88], climate: [0.45, 0.45] },
    Biome { name: "forest", surface: GRASS, filler: DIRT, grass_tint: [86, 156, 64], climate: [0.40, 0.78] },
    Biome { name: "savanna", surface: GRASS, filler: DIRT, grass_tint: [186, 178, 96], climate: [0.85, 0.32] },
    Biome { name: "desert", surface: SAND, filler: SAND, grass_tint: [200, 190, 120], climate: [0.92, 0.08] },
    Biome { name: "snowy", surface: SNOW, filler: DIRT, grass_tint: [158, 196, 176], climate: [0.06, 0.55] },
    Biome { name: "taiga", surface: GRASS, filler: DIRT, grass_tint: [96, 150, 110], climate: [0.18, 0.70] },
];

/// Sélectionne le biome dont le point climatique est le plus proche de (temp, hum).
pub fn select_biome(temperature: f32, humidity: f32) -> &'static Biome {
    let mut best = &BIOMES[0];
    let mut best_d = f32::MAX;
    for biome in BIOMES {
        let dt = biome.climate[0] - temperature;
        let dh = biome.climate[1] - humidity;
        let d = dt * dt + dh * dh;
        if d < best_d {
            best_d = d;
            best = biome;
        }
    }
    best
}
