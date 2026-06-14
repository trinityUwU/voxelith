//! Responsabilité : génération procédurale du terrain — bruit fractal (FBM) mappé
//! par splines vers une heightmap (continentalness/erosion/peaks-valleys), climat
//! (temperature/humidity) pour les biomes, et placement des blocs par colonne.
//! Déterministe et sans état : `block_at` permet au mesher de sonder les voisins.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::biome::{select_biome, Biome};
use crate::block::{BlockState, ChunkPos, SUBCHUNK_SIZE};
use crate::chunk::Chunk;
use crate::registry::{AIR, SAND, STONE, WATER};

/// Niveau de la mer en blocs.
pub const SEA_LEVEL: i32 = 62;

/// Spline continentalness : bruit [-1,1] → offset de hauteur (océan profond → plateau).
const CONT_SPLINE: &[(f32, f32)] = &[
    (-1.0, -34.0),
    (-0.4, -12.0),
    (-0.15, 2.0),
    (0.1, 14.0),
    (0.5, 26.0),
    (1.0, 42.0),
];

/// Spline érosion : bruit [-1,1] → facteur d'aplatissement [0,1] (1 = plat).
const ERO_SPLINE: &[(f32, f32)] = &[(-1.0, 0.0), (-0.3, 0.25), (0.2, 0.7), (1.0, 1.0)];

/// Générateur de monde : un FBM par canal climatique/relief, dérivés d'une seed.
pub struct Worldgen {
    continental: Fbm<Perlin>,
    erosion: Fbm<Perlin>,
    peaks_valleys: Fbm<Perlin>,
    temperature: Fbm<Perlin>,
    humidity: Fbm<Perlin>,
}

impl Worldgen {
    /// Construit le générateur depuis une seed (canaux décorrélés par seed+offset).
    pub fn new(seed: u32) -> Self {
        Self {
            continental: fbm(seed, 5, 0.0016),
            erosion: fbm(seed + 1, 4, 0.0021),
            peaks_valleys: fbm(seed + 2, 4, 0.0090),
            temperature: fbm(seed + 3, 3, 0.0008),
            humidity: fbm(seed + 4, 3, 0.0008),
        }
    }

    /// Hauteur de surface en blocs pour une colonne monde.
    pub fn height(&self, wx: i32, wz: i32) -> i32 {
        let p = [wx as f64, wz as f64];
        let base = spline(self.continental.get(p) as f32, CONT_SPLINE);
        let ero_flat = spline(self.erosion.get(p) as f32, ERO_SPLINE);
        let relief = self.peaks_valleys.get(p) as f32 * 42.0 * (1.0 - ero_flat);
        SEA_LEVEL + (base + relief).round() as i32
    }

    /// Biome de la colonne, déduit du climat (temperature, humidity) en [0,1].
    pub fn biome(&self, wx: i32, wz: i32) -> &'static Biome {
        let p = [wx as f64, wz as f64];
        let temp = (self.temperature.get(p) as f32) * 0.5 + 0.5;
        let hum = (self.humidity.get(p) as f32) * 0.5 + 0.5;
        select_biome(temp, hum)
    }

    /// Teinte d'herbe de la colonne (pour les faces d'herbe au meshing).
    pub fn grass_tint(&self, wx: i32, wz: i32) -> [u8; 3] {
        self.biome(wx, wz).grass_tint
    }

    /// État du bloc à une coordonnée monde — recalcule hauteur/biome (usage bordure).
    pub fn block_at(&self, wx: i32, wy: i32, wz: i32) -> BlockState {
        if wy < 0 {
            return STONE;
        }
        let h = self.height(wx, wz);
        block_for(wy, h, self.biome(wx, wz))
    }

    /// Génère le contenu voxel complet d'un chunk.
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::empty(pos);
        let (ox, oz) = pos.world_origin();
        for lz in 0..SUBCHUNK_SIZE {
            for lx in 0..SUBCHUNK_SIZE {
                let (wx, wz) = (ox + lx as i32, oz + lz as i32);
                let h = self.height(wx, wz);
                let biome = self.biome(wx, wz);
                let top = h.max(SEA_LEVEL);
                for y in 0..=top {
                    let block = block_for(y, h, biome);
                    if !block.is_air() {
                        chunk.set(lx, y as usize, lz, block);
                    }
                }
            }
        }
        chunk
    }
}

/// Bloc à la hauteur `y` d'une colonne de surface `h` pour un biome donné.
fn block_for(y: i32, h: i32, biome: &Biome) -> BlockState {
    if y > h {
        return if y <= SEA_LEVEL { WATER } else { AIR };
    }
    if y == h {
        return if h <= SEA_LEVEL { SAND } else { biome.surface };
    }
    if y >= h - 3 {
        return biome.filler;
    }
    STONE
}

/// Construit un FBM Perlin paramétré (octaves, fréquence de base).
fn fbm(seed: u32, octaves: usize, frequency: f64) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed)
        .set_octaves(octaves)
        .set_frequency(frequency)
        .set_persistence(0.5)
        .set_lacunarity(2.0)
}

/// Interpolation linéaire par morceaux d'une spline (points triés par x croissant).
fn spline(t: f32, points: &[(f32, f32)]) -> f32 {
    if t <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if t <= x1 {
            let f = (t - x0) / (x1 - x0);
            return y0 + (y1 - y0) * f;
        }
    }
    points[points.len() - 1].1
}
