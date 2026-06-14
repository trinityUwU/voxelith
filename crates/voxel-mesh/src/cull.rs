//! Responsabilité : mesher LOD0 par face-culling — émet une face uniquement
//! quand le voxel voisin est vide. Base avant le passage au greedy/binary meshing.

use voxel_world::block::{SUBCHUNKS_PER_CHUNK, SUBCHUNK_SIZE};
use voxel_world::{BlockState, Chunk};

use crate::vertex::{ChunkMesh, Vertex};

/// Les 6 faces d'un cube : direction de la normale + offset du voisin à tester.
const FACES: [([i32; 3], [[f32; 3]; 4]); 6] = [
    ([1, 0, 0], [[1., 0., 0.], [1., 1., 0.], [1., 1., 1.], [1., 0., 1.]]),
    ([-1, 0, 0], [[0., 0., 1.], [0., 1., 1.], [0., 1., 0.], [0., 0., 0.]]),
    ([0, 1, 0], [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]]),
    ([0, -1, 0], [[0., 0., 1.], [0., 0., 0.], [1., 0., 0.], [1., 0., 1.]]),
    ([0, 0, 1], [[1., 0., 1.], [1., 1., 1.], [0., 1., 1.], [0., 0., 1.]]),
    ([0, 0, -1], [[0., 0., 0.], [0., 1., 0.], [1., 1., 0.], [1., 0., 0.]]),
];

const MAX_Y: usize = SUBCHUNKS_PER_CHUNK * SUBCHUNK_SIZE;

/// Construit le maillage LOD0 d'un chunk par face-culling interne.
pub fn mesh_chunk(chunk: &Chunk) -> ChunkMesh {
    let (ox, oz) = chunk.pos.world_origin();
    let mut mesh = ChunkMesh::default();

    for y in 0..MAX_Y {
        for z in 0..SUBCHUNK_SIZE {
            for x in 0..SUBCHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block.is_air() {
                    continue;
                }
                emit_block(chunk, &mut mesh, (x, y, z), (ox, oz), block);
            }
        }
    }
    mesh
}

/// Émet les faces exposées d'un bloc plein.
fn emit_block(
    chunk: &Chunk,
    mesh: &mut ChunkMesh,
    local: (usize, usize, usize),
    origin: (i32, i32),
    block: BlockState,
) {
    let (x, y, z) = local;
    let color = block_color(block);
    for (offset, corners) in FACES.iter() {
        if !is_face_exposed(chunk, x as i32 + offset[0], y as i32 + offset[1], z as i32 + offset[2]) {
            continue;
        }
        let base = [origin.0 as f32 + x as f32, y as f32, origin.1 as f32 + z as f32];
        push_quad(mesh, base, corners, *offset, color);
    }
}

/// `true` si le voisin est hors-chunk ou vide → la face doit être dessinée.
fn is_face_exposed(chunk: &Chunk, nx: i32, ny: i32, nz: i32) -> bool {
    if nx < 0 || nz < 0 || ny < 0 {
        return true;
    }
    let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
    if nx >= SUBCHUNK_SIZE || nz >= SUBCHUNK_SIZE || ny >= MAX_Y {
        return true;
    }
    chunk.get(nx, ny, nz).is_air()
}

/// Ajoute un quad (2 triangles) au maillage.
fn push_quad(mesh: &mut ChunkMesh, base: [f32; 3], corners: &[[f32; 3]; 4], normal: [i32; 3], color: [f32; 3]) {
    let start = mesh.vertices.len() as u32;
    let n = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    for c in corners {
        mesh.vertices.push(Vertex {
            position: [base[0] + c[0], base[1] + c[1], base[2] + c[2]],
            normal: n,
            color,
        });
    }
    mesh.indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
}

/// Couleur de placeholder par état de bloc (remplacée par l'atlas de textures).
fn block_color(block: BlockState) -> [f32; 3] {
    match block.0 {
        1 => [0.45, 0.45, 0.47],
        2 => [0.42, 0.30, 0.18],
        3 => [0.30, 0.62, 0.25],
        _ => [0.8, 0.2, 0.8],
    }
}
