//! Responsabilité : conteneur du terrain statique de l'overworld — chunks streamés
//! indexés par position. L'octree de LOD (index spatial des bandes) viendra en phase 03.

use std::collections::HashMap;

use crate::block::ChunkPos;
use crate::chunk::Chunk;

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
}
