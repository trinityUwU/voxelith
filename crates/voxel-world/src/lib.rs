//! voxel-world — grille logique du terrain statique de l'overworld (Système A).
//! Source de vérité voxel : block states, sous-chunks palettés, chunks, monde streamé.

pub mod block;
pub mod chunk;
pub mod gen;
pub mod palette;
pub mod world;

pub use block::{BlockState, ChunkPos};
pub use chunk::{Aabb, Chunk, MeshState};
pub use palette::SubChunk;
pub use world::{World, WORLD_HEIGHT};
