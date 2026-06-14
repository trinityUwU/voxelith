//! Responsabilité : raycast voxel (DDA Amanatides & Woo) pour viser un bloc, et
//! calcul des chunks à re-mesher après une édition (bloc + voisins de bordure).

use glam::{IVec3, Vec3};
use voxel_world::block::SUBCHUNK_SIZE;
use voxel_world::registry::{kind, BlockKind};
use voxel_world::{ChunkPos, WorldStore};

/// Résultat d'un raycast : bloc visé + cellule libre devant (pour la pose).
pub struct RayHit {
    pub block: IVec3,
    pub place: IVec3,
}

/// Lance un rayon et retourne le premier bloc solide rencontré dans la portée.
pub fn raycast(store: &WorldStore, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RayHit> {
    let mut block = origin.floor().as_ivec3();
    let step = IVec3::new(sign(dir.x), sign(dir.y), sign(dir.z));
    let t_delta = Vec3::new(inv_abs(dir.x), inv_abs(dir.y), inv_abs(dir.z));
    let mut t_max = Vec3::new(
        boundary(origin.x, dir.x),
        boundary(origin.y, dir.y),
        boundary(origin.z, dir.z),
    );
    let mut prev = block;

    let mut t = 0.0;
    while t <= max_dist {
        if is_solid(store, block) {
            return Some(RayHit { block, place: prev });
        }
        prev = block;
        if t_max.x < t_max.y && t_max.x < t_max.z {
            block.x += step.x;
            t = t_max.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            block.y += step.y;
            t = t_max.y;
            t_max.y += t_delta.y;
        } else {
            block.z += step.z;
            t = t_max.z;
            t_max.z += t_delta.z;
        }
    }
    None
}

/// Chunks à re-mesher après une édition à (wx, wz) : le chunk + voisins si en bordure.
pub fn touched_chunks(wx: i32, wz: i32) -> Vec<ChunkPos> {
    let size = SUBCHUNK_SIZE as i32;
    let (cx, cz) = (wx.div_euclid(size), wz.div_euclid(size));
    let (lx, lz) = (wx.rem_euclid(size), wz.rem_euclid(size));
    let mut chunks = vec![ChunkPos::new(cx, cz)];
    if lx == 0 {
        chunks.push(ChunkPos::new(cx - 1, cz));
    }
    if lx == size - 1 {
        chunks.push(ChunkPos::new(cx + 1, cz));
    }
    if lz == 0 {
        chunks.push(ChunkPos::new(cx, cz - 1));
    }
    if lz == size - 1 {
        chunks.push(ChunkPos::new(cx, cz + 1));
    }
    chunks
}

fn is_solid(store: &WorldStore, b: IVec3) -> bool {
    kind(store.block_at(b.x, b.y, b.z)) == BlockKind::Solid
}

/// Distance paramétrique jusqu'au premier plan de voxel le long d'un axe.
fn boundary(origin: f32, dir: f32) -> f32 {
    if dir == 0.0 {
        return f32::INFINITY;
    }
    let cell = origin.floor();
    let next = if dir > 0.0 { cell + 1.0 } else { cell };
    ((next - origin) / dir).abs()
}

fn inv_abs(d: f32) -> f32 {
    if d == 0.0 {
        f32::INFINITY
    } else {
        1.0 / d.abs()
    }
}

fn sign(d: f32) -> i32 {
    if d > 0.0 {
        1
    } else if d < 0.0 {
        -1
    } else {
        0
    }
}
