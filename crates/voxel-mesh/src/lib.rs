//! voxel-mesh — conversion grille logique → géométrie GPU.
//! Phase 00 : face-culling LOD0. Phases suivantes : greedy/binary meshing, agrégation LOD.

pub mod cull;
pub mod vertex;

pub use cull::mesh_chunk;
pub use vertex::{ChunkMesh, Vertex};
