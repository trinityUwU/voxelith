//! Responsabilité : mesher LOD0 par face-culling — émet une face uniquement
//! quand le voxel voisin est vide, voisins inter-chunks inclus (via World).
//! Base avant le passage au greedy/binary meshing.

use voxel_world::block::{SUBCHUNKS_PER_CHUNK, SUBCHUNK_SIZE};
use voxel_world::{BlockState, Chunk, World};

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

/// Construit le maillage LOD0 d'un chunk, voisins inter-chunks testés via `world`
/// pour ne pas émettre de murs de bordure cachés entre deux chunks pleins.
pub fn mesh_chunk(chunk: &Chunk, world: &World) -> ChunkMesh {
    let (ox, oz) = chunk.pos.world_origin();
    let mut mesh = ChunkMesh::default();

    for y in 0..MAX_Y {
        for z in 0..SUBCHUNK_SIZE {
            for x in 0..SUBCHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block.is_air() {
                    continue;
                }
                let world_pos = (ox + x as i32, y as i32, oz + z as i32);
                emit_block(world, &mut mesh, world_pos, block);
            }
        }
    }
    mesh
}

/// Émet les faces exposées d'un bloc plein, en coordonnées monde.
fn emit_block(world: &World, mesh: &mut ChunkMesh, world_pos: (i32, i32, i32), block: BlockState) {
    let (wx, wy, wz) = world_pos;
    let color = block_color(block);
    for (offset, corners) in FACES.iter() {
        if world.block_at(wx + offset[0], wy + offset[1], wz + offset[2]).is_solid() {
            continue;
        }
        let base = [wx as f32, wy as f32, wz as f32];
        push_quad(mesh, base, corners, *offset, color);
    }
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
