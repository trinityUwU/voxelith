//! Responsabilité : frustum de la caméra extrait de la view-projection et test
//! d'intersection AABB (cull CPU des chunks hors champ). Étape CPU de la phase 01,
//! avant le passage au culling GPU-driven en compute shader.

use glam::{Mat4, Vec4};
use voxel_world::Aabb;

/// Les 6 plans du frustum (a, b, c, d) tels que `a·x + b·y + c·z + d >= 0` à l'intérieur.
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    /// Extrait les plans depuis la matrice view-projection (Gribb–Hartmann, depth [0,1]).
    pub fn from_view_proj(vp: &Mat4) -> Self {
        let r0 = vp.row(0);
        let r1 = vp.row(1);
        let r2 = vp.row(2);
        let r3 = vp.row(3);
        let planes = [
            normalize_plane(r3 + r0), // gauche
            normalize_plane(r3 - r0), // droite
            normalize_plane(r3 + r1), // bas
            normalize_plane(r3 - r1), // haut
            normalize_plane(r2),      // proche (z=0 en clip [0,1])
            normalize_plane(r3 - r2), // lointain
        ];
        Self { planes }
    }

    /// `true` si l'AABB est au moins partiellement dans le frustum (test conservateur).
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        for plane in &self.planes {
            // p-vertex : coin le plus positif le long de la normale du plan.
            let px = if plane.x >= 0.0 { aabb.max[0] } else { aabb.min[0] };
            let py = if plane.y >= 0.0 { aabb.max[1] } else { aabb.min[1] };
            let pz = if plane.z >= 0.0 { aabb.max[2] } else { aabb.min[2] };
            if plane.x * px + plane.y * py + plane.z * pz + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Normalise un plan par la longueur de sa normale (x, y, z).
fn normalize_plane(p: Vec4) -> Vec4 {
    let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
    if len > 0.0 {
        p / len
    } else {
        p
    }
}
