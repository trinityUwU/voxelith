//! Responsabilité : génération procédurale de terrain basique pour le socle (phase 00).
//! Heightmap sinusoïdale déterministe — remplacée par du bruit de worldgen en phase 03.

use crate::block::{BlockState, ChunkPos, SUBCHUNKS_PER_CHUNK, SUBCHUNK_SIZE};
use crate::chunk::Chunk;

const STONE: BlockState = BlockState(1);
const DIRT: BlockState = BlockState(2);
const GRASS: BlockState = BlockState(3);

/// Génère un chunk de terrain : heightmap douce, couche herbe/terre/pierre.
pub fn generate_chunk(pos: ChunkPos) -> Chunk {
    let mut chunk = Chunk::empty(pos);
    let (ox, oz) = pos.world_origin();
    let max_y = SUBCHUNKS_PER_CHUNK * SUBCHUNK_SIZE;

    for lx in 0..SUBCHUNK_SIZE {
        for lz in 0..SUBCHUNK_SIZE {
            let height = column_height(ox + lx as i32, oz + lz as i32);
            for ly in 0..height.min(max_y) {
                chunk.set(lx, ly, lz, block_at(ly, height));
            }
        }
    }
    chunk
}

/// Hauteur de terrain en blocs pour une colonne monde donnée.
fn column_height(wx: i32, wz: i32) -> usize {
    let fx = wx as f32 * 0.05;
    let fz = wz as f32 * 0.05;
    let h = 64.0 + (fx.sin() * 6.0) + (fz.cos() * 6.0);
    h.round().max(1.0) as usize
}

/// Sélection du bloc selon la profondeur sous la surface.
fn block_at(y: usize, surface: usize) -> BlockState {
    if y + 1 == surface {
        GRASS
    } else if y + 4 >= surface {
        DIRT
    } else {
        STONE
    }
}
