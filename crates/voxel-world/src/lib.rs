//! voxel-world — grille logique du terrain de l'overworld (Système A).
//! Source de vérité voxel : block states, registry, sous-chunks palettés, chunks,
//! génération procédurale multi-noise et biomes.

pub mod biome;
pub mod block;
pub mod chunk;
pub mod palette;
pub mod registry;
pub mod texture;
pub mod world;
pub mod worldgen;

pub use biome::Biome;
pub use block::{BlockState, ChunkPos};
pub use chunk::{Aabb, Chunk, MeshState};
pub use palette::SubChunk;
pub use registry::{BlockKind, Face};
pub use texture::{TexDef, TexKind, TEXTURES, TEX_SIZE};
pub use world::{World, WORLD_HEIGHT};
pub use worldgen::Worldgen;
