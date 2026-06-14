//! Responsabilité : colonne de chunk (16 × 384 × 16) — pile de sous-chunks,
//! état de meshing et AABB uploadée au GPU pour le culling.

use crate::block::{
    local_index, BlockState, ChunkPos, SUBCHUNKS_PER_CHUNK, SUBCHUNK_SIZE,
};
use crate::palette::SubChunk;

/// Boîte englobante alignée aux axes, en coordonnées monde. Source du frustum/Hi-Z culling.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// État du mesh d'un chunk dans le pipeline de streaming asynchrone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshState {
    /// Données voxel modifiées : mesh à (re)construire.
    Dirty,
    /// Meshing en cours sur le thread pool.
    Building,
    /// Mesh prêt pour le LOD indiqué.
    Ready(u8),
}

/// Colonne de chunk : pile verticale de sous-chunks 16³.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub subchunks: Vec<SubChunk>,
    pub mesh_state: MeshState,
    pub aabb: Aabb,
}

impl Chunk {
    /// Chunk vide (air) prêt à recevoir la génération procédurale.
    pub fn empty(pos: ChunkPos) -> Self {
        let (ox, oz) = pos.world_origin();
        let height = (SUBCHUNKS_PER_CHUNK * SUBCHUNK_SIZE) as f32;
        Self {
            pos,
            subchunks: (0..SUBCHUNKS_PER_CHUNK).map(|_| SubChunk::empty()).collect(),
            mesh_state: MeshState::Dirty,
            aabb: Aabb {
                min: [ox as f32, 0.0, oz as f32],
                max: [
                    ox as f32 + SUBCHUNK_SIZE as f32,
                    height,
                    oz as f32 + SUBCHUNK_SIZE as f32,
                ],
            },
        }
    }

    /// Lit l'état à la coordonnée locale `(x, y, z)` du chunk (y sur toute la hauteur).
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockState {
        let sub = y / SUBCHUNK_SIZE;
        self.subchunks[sub].get(local_index(x, y % SUBCHUNK_SIZE, z))
    }

    /// Écrit un état à la coordonnée locale et marque le chunk à re-mesher.
    pub fn set(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let sub = y / SUBCHUNK_SIZE;
        self.subchunks[sub].set(local_index(x, y % SUBCHUNK_SIZE, z), state);
        self.mesh_state = MeshState::Dirty;
    }
}
