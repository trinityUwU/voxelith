//! voxel-mesh — conversion grille logique → géométrie GPU par greedy meshing,
//! avec UV de tiling, teinte de biome et index de texture par face.

pub mod greedy;
pub mod vertex;

pub use greedy::mesh_chunk;
pub use vertex::{ChunkMesh, Vertex};
