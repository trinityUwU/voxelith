//! Responsabilité : greedy meshing — fusionne les faces coplanaires de même
//! apparence en quads rectangulaires (bien moins de triangles que le face-culling).
//! Sonde le worldgen pour les voisins de bordure (culling cross-chunk déterministe).

use voxel_world::block::{BlockState, SUBCHUNK_SIZE};
use voxel_world::registry::{face_texture, renders_against, Face, GRASS, LEAVES};
use voxel_world::{Chunk, WorldStore, WORLD_HEIGHT};

use crate::vertex::{ChunkMesh, Vertex};

const WHITE: [u8; 3] = [255, 255, 255];

/// Quad fusionnable : deux cellules fusionnent si leur apparence est identique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mask {
    layer: u32,
    tint: [u8; 3],
    positive: bool,
}

/// Construit le maillage greedy d'un chunk (3 axes), bordures via `gen`.
pub fn mesh_chunk(chunk: &Chunk, gen: &WorldStore) -> ChunkMesh {
    let dims = [SUBCHUNK_SIZE as i32, WORLD_HEIGHT, SUBCHUNK_SIZE as i32];
    let (ox, oz) = chunk.pos.world_origin();
    let mut mesh = ChunkMesh::default();
    for d in 0..3 {
        mesh_axis(chunk, gen, (ox, oz), &dims, d, &mut mesh);
    }
    mesh
}

/// Échantillonne un bloc en coordonnées locales (hors chunk → worldgen).
fn sample(chunk: &Chunk, gen: &WorldStore, ox: i32, oz: i32, l: [i32; 3]) -> BlockState {
    let dims = [SUBCHUNK_SIZE as i32, WORLD_HEIGHT, SUBCHUNK_SIZE as i32];
    if (0..dims[0]).contains(&l[0]) && (0..dims[1]).contains(&l[1]) && (0..dims[2]).contains(&l[2]) {
        chunk.get(l[0] as usize, l[1] as usize, l[2] as usize)
    } else {
        gen.block_at(ox + l[0], l[1], oz + l[2])
    }
}

/// Greedy meshing le long d'un axe `d` : remplit puis fusionne le masque par tranche.
fn mesh_axis(chunk: &Chunk, gen: &WorldStore, o: (i32, i32), dims: &[i32; 3], d: usize, mesh: &mut ChunkMesh) {
    let (u, v) = ((d + 1) % 3, (d + 2) % 3);
    let (du, dv) = (dims[u] as usize, dims[v] as usize);
    let mut mask: Vec<Option<Mask>> = vec![None; du * dv];

    for s in -1..dims[d] {
        for j in 0..dv {
            for i in 0..du {
                let mut x = [0i32; 3];
                x[d] = s;
                x[u] = i as i32;
                x[v] = j as i32;
                mask[i + j * du] = face(chunk, gen, o, d, x);
            }
        }
        greedy_merge(&mut mask, dims, d, s + 1, o, mesh);
    }
}

/// Détermine la face entre le voxel à `x` et son voisin `x + 1` le long de `d`.
fn face(chunk: &Chunk, gen: &WorldStore, o: (i32, i32), d: usize, x: [i32; 3]) -> Option<Mask> {
    let mut xq = x;
    xq[d] += 1;
    let a = sample(chunk, gen, o.0, o.1, x);
    let b = sample(chunk, gen, o.0, o.1, xq);
    let (block, positive, owner) = if renders_against(a, b) {
        (a, true, x)
    } else if renders_against(b, a) {
        (b, false, xq)
    } else {
        return None;
    };
    let ny = if d == 1 { if positive { 1 } else { -1 } } else { 0 };
    let layer = face_texture(block, Face::from_normal(ny));
    let tint = if block == GRASS || block == LEAVES {
        gen.grass_tint(o.0 + owner[0], o.1 + owner[2])
    } else {
        WHITE
    };
    Some(Mask { layer, tint, positive })
}

/// Fusionne les cellules du masque en rectangles maximaux et émet les quads.
fn greedy_merge(mask: &mut [Option<Mask>], dims: &[i32; 3], d: usize, plane: i32, o: (i32, i32), mesh: &mut ChunkMesh) {
    let (u, v) = ((d + 1) % 3, (d + 2) % 3);
    let (du, dv) = (dims[u] as usize, dims[v] as usize);
    for j in 0..dv {
        let mut i = 0;
        while i < du {
            let Some(m) = mask[i + j * du] else { i += 1; continue };
            let mut w = 1;
            while i + w < du && mask[i + w + j * du] == Some(m) {
                w += 1;
            }
            let h = rect_height(mask, du, dv, i, j, w, m);
            emit_quad(mesh, (d, u, v), plane, (i, j, w, h), o, m);
            for hh in 0..h {
                for ww in 0..w {
                    mask[i + ww + (j + hh) * du] = None;
                }
            }
            i += w;
        }
    }
}

/// Hauteur maximale d'un rectangle de largeur `w` à partir de (i, j) dans le masque.
fn rect_height(mask: &[Option<Mask>], du: usize, dv: usize, i: usize, j: usize, w: usize, m: Mask) -> usize {
    let mut h = 1;
    while j + h < dv {
        if (0..w).any(|k| mask[i + k + (j + h) * du] != Some(m)) {
            break;
        }
        h += 1;
    }
    h
}

/// Émet un quad (4 vertices + 6 indices) à partir d'un rectangle fusionné du masque.
fn emit_quad(mesh: &mut ChunkMesh, axes: (usize, usize, usize), plane: i32, rect: (usize, usize, usize, usize), o: (i32, i32), m: Mask) {
    let (d, u, v) = axes;
    let (i, j, w, h) = rect;
    let mut base = [0f32; 3];
    base[d] = plane as f32;
    base[u] = i as f32;
    base[v] = j as f32;
    base[0] += o.0 as f32;
    base[2] += o.1 as f32;
    let mut du_vec = [0f32; 3];
    du_vec[u] = w as f32;
    let mut dv_vec = [0f32; 3];
    dv_vec[v] = h as f32;
    let mut normal = [0f32; 3];
    normal[d] = if m.positive { 1.0 } else { -1.0 };
    let tint = [m.tint[0] as f32 / 255.0, m.tint[1] as f32 / 255.0, m.tint[2] as f32 / 255.0];
    let start = mesh.vertices.len() as u32;
    let corners = [
        base,
        add(base, du_vec),
        add(add(base, du_vec), dv_vec),
        add(base, dv_vec),
    ];
    for corner in corners {
        let uv = corner_uv(corner, d);
        mesh.vertices.push(Vertex { position: corner, normal, uv, tint, layer: m.layer });
    }
    mesh.indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
}

/// Somme composante à composante de deux vecteurs.
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// UV de tiling dérivé des coordonnées monde : 1 répétition par bloc, axe vertical
/// = Y pour les faces latérales (frange d'herbe en haut), peu importe l'orientation.
fn corner_uv(p: [f32; 3], d: usize) -> [f32; 2] {
    match d {
        1 => [p[0], p[2]],
        0 => [p[2], -p[1]],
        _ => [p[0], -p[1]],
    }
}
