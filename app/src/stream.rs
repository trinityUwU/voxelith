//! Responsabilité : streaming infini des chunks — charge/décharge autour du joueur.
//! Génération + greedy meshing sur le thread pool rayon, résultats collectés par le
//! main thread via un channel. Le monde devient illimité (plus de bordure fixe).

use std::collections::HashSet;
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};
use glam::Vec3;
use voxel_mesh::{mesh_chunk, ChunkMesh};
use voxel_world::block::SUBCHUNK_SIZE;
use voxel_world::{Aabb, ChunkPos, WorldStore};

/// Maillage d'un chunk généré sur un thread de fond, prêt à être uploadé.
pub struct ChunkResult {
    pub pos: ChunkPos,
    pub mesh: ChunkMesh,
    pub aabb: Aabb,
}

/// Gère l'ensemble des chunks chargés et l'émission des jobs de génération.
pub struct ChunkManager {
    store: Arc<WorldStore>,
    view_distance: i32,
    handled: HashSet<ChunkPos>,
    center: ChunkPos,
    tx: Sender<ChunkResult>,
    rx: Receiver<ChunkResult>,
}

impl ChunkManager {
    /// Crée le manager pour un monde partagé et une distance de vue (en chunks).
    pub fn new(store: Arc<WorldStore>, view_distance: i32) -> Self {
        let (tx, rx) = unbounded();
        Self {
            store,
            view_distance,
            handled: HashSet::new(),
            center: ChunkPos::new(i32::MAX, i32::MAX),
            tx,
            rx,
        }
    }

    /// Re-meshe un chunk déjà chargé (après une édition de bloc).
    pub fn remesh(&self, pos: ChunkPos) {
        let (store, tx) = (self.store.clone(), self.tx.clone());
        rayon::spawn(move || {
            let chunk = store.generate_chunk(pos);
            let aabb = chunk.aabb;
            let mesh = mesh_chunk(&chunk, &store);
            let _ = tx.send(ChunkResult { pos, mesh, aabb });
        });
    }

    /// Met à jour l'ensemble chargé selon la position joueur : émet les jobs des
    /// nouveaux chunks (proches d'abord), retourne les chunks à décharger.
    pub fn update(&mut self, player: Vec3) -> Vec<ChunkPos> {
        let center = world_to_chunk(player);
        if center == self.center {
            return Vec::new();
        }
        self.center = center;
        self.spawn_missing(center);
        self.collect_unloads(center)
    }

    /// Émet un job de génération pour chaque chunk manquant dans le rayon (proches d'abord).
    fn spawn_missing(&mut self, center: ChunkPos) {
        let r = self.view_distance;
        let mut wanted: Vec<ChunkPos> = Vec::new();
        for dz in -r..=r {
            for dx in -r..=r {
                let pos = ChunkPos::new(center.x + dx, center.z + dz);
                if dx * dx + dz * dz <= r * r && !self.handled.contains(&pos) {
                    wanted.push(pos);
                }
            }
        }
        wanted.sort_by_key(|p| dist2(*p, center));
        for pos in wanted {
            self.handled.insert(pos);
            let (store, tx) = (self.store.clone(), self.tx.clone());
            rayon::spawn(move || {
                let chunk = store.generate_chunk(pos);
                let aabb = chunk.aabb;
                let mesh = mesh_chunk(&chunk, &store);
                let _ = tx.send(ChunkResult { pos, mesh, aabb });
            });
        }
    }

    /// Retire et retourne les chunks chargés hors du rayon (+marge d'hystérésis).
    fn collect_unloads(&mut self, center: ChunkPos) -> Vec<ChunkPos> {
        let limit = self.view_distance + 2;
        let dropped: Vec<ChunkPos> = self
            .handled
            .iter()
            .filter(|p| (p.x - center.x).abs() > limit || (p.z - center.z).abs() > limit)
            .copied()
            .collect();
        for pos in &dropped {
            self.handled.remove(pos);
        }
        dropped
    }

    /// Récupère jusqu'à `max` chunks terminés, en ignorant ceux devenus hors rayon.
    pub fn drain(&mut self, max: usize) -> Vec<ChunkResult> {
        let mut out = Vec::new();
        while out.len() < max {
            match self.rx.try_recv() {
                Ok(result) if self.handled.contains(&result.pos) => out.push(result),
                Ok(_) => {}
                Err(_) => break,
            }
        }
        out
    }
}

/// Distance chunk au carré (priorisation de chargement).
fn dist2(p: ChunkPos, c: ChunkPos) -> i32 {
    let (dx, dz) = (p.x - c.x, p.z - c.z);
    dx * dx + dz * dz
}

/// Convertit une position monde en position de chunk.
fn world_to_chunk(p: Vec3) -> ChunkPos {
    let s = SUBCHUNK_SIZE as f32;
    ChunkPos::new((p.x / s).floor() as i32, (p.z / s).floor() as i32)
}
