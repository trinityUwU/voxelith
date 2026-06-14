//! Responsabilité : conteneur du terrain statique de l'overworld — chunks streamés
//! indexés par position. L'octree de LOD (index spatial des bandes) viendra en phase 03.

use std::collections::HashMap;

use crate::block::{BlockState, ChunkPos, SUBCHUNKS_PER_CHUNK, SUBCHUNK_SIZE};
use crate::chunk::Chunk;

/// Hauteur totale du monde en blocs (24 sous-chunks de 16).
pub const WORLD_HEIGHT: i32 = (SUBCHUNKS_PER_CHUNK * SUBCHUNK_SIZE) as i32;

/// Monde voxel : ensemble des chunks chargés autour du joueur.
#[derive(Debug, Default)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.pos, chunk);
    }

    pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }

    /// État du bloc à une coordonnée monde. Hors monde (vertical ou chunk absent) = air.
    /// Permet au mesher de tester les voisins au-delà des bords de chunk.
    pub fn block_at(&self, wx: i32, wy: i32, wz: i32) -> BlockState {
        if wy < 0 || wy >= WORLD_HEIGHT {
            return BlockState::AIR;
        }
        let size = SUBCHUNK_SIZE as i32;
        let cx = wx.div_euclid(size);
        let cz = wz.div_euclid(size);
        match self.chunks.get(&ChunkPos::new(cx, cz)) {
            Some(chunk) => chunk.get(
                wx.rem_euclid(size) as usize,
                wy as usize,
                wz.rem_euclid(size) as usize,
            ),
            None => BlockState::AIR,
        }
    }
}
