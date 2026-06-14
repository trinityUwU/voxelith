//! Responsabilité : format de vertex GPU produit par le mesher, directement
//! uploadable (Pod) et décrit pour le vertex layout wgpu côté render.

use bytemuck::{Pod, Zeroable};

/// Vertex de terrain : position monde, normale, UV (tiling en blocs), teinte, layer.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tint: [f32; 3],
    pub layer: u32,
}

/// Maillage CPU d'un chunk : tampons prêts à être copiés en vertex/index buffers.
#[derive(Debug, Default, Clone)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}
