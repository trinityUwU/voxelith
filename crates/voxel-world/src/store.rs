//! Responsabilité : monde jouable = worldgen déterministe + overrides d'édition.
//! Source unique de blocs pour le meshing, la collision et le raycast. Les blocs
//! cassés/posés vivent dans une table d'overrides consultée avant le worldgen.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::block::{BlockState, ChunkPos, SUBCHUNK_SIZE};
use crate::chunk::Chunk;
use crate::world::WORLD_HEIGHT;
use crate::worldgen::Worldgen;

/// Monde éditable : terrain procédural + modifications du joueur.
pub struct WorldStore {
    gen: Worldgen,
    seed: u32,
    overrides: RwLock<HashMap<(i32, i32, i32), BlockState>>,
}

impl WorldStore {
    /// Crée un monde pour une seed (overrides vides).
    pub fn new(seed: u32) -> Self {
        Self { gen: Worldgen::new(seed), seed, overrides: RwLock::new(HashMap::new()) }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// `true` si le bloc à (x, y, z) est solide (collision).
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        crate::registry::kind(self.block_at(x, y, z)) == crate::registry::BlockKind::Solid
    }

    /// État du bloc : override d'édition si présent, sinon worldgen.
    pub fn block_at(&self, wx: i32, wy: i32, wz: i32) -> BlockState {
        if let Some(b) = self.overrides.read().unwrap().get(&(wx, wy, wz)) {
            return *b;
        }
        self.gen.block_at(wx, wy, wz)
    }

    /// Pose/casse un bloc (override persistant en mémoire).
    pub fn set_block(&self, wx: i32, wy: i32, wz: i32, block: BlockState) {
        self.overrides.write().unwrap().insert((wx, wy, wz), block);
    }

    /// Exporte les overrides pour la sauvegarde : (x, y, z, id de bloc).
    pub fn export_overrides(&self) -> Vec<(i32, i32, i32, u16)> {
        self.overrides
            .read()
            .unwrap()
            .iter()
            .map(|(&(x, y, z), &b)| (x, y, z, b.0))
            .collect()
    }

    /// Charge des overrides depuis une sauvegarde.
    pub fn import_overrides(&self, data: &[(i32, i32, i32, u16)]) {
        let mut overrides = self.overrides.write().unwrap();
        for &(x, y, z, id) in data {
            overrides.insert((x, y, z), BlockState(id));
        }
    }

    /// Teinte d'herbe d'une colonne (déléguée au worldgen).
    pub fn grass_tint(&self, wx: i32, wz: i32) -> [u8; 3] {
        self.gen.grass_tint(wx, wz)
    }

    /// Hauteur de surface d'une colonne (déléguée au worldgen).
    pub fn height(&self, wx: i32, wz: i32) -> i32 {
        self.gen.height(wx, wz)
    }

    /// Génère un chunk puis applique les overlays d'édition de sa colonne.
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let mut chunk = self.gen.generate_chunk(pos);
        let overrides = self.overrides.read().unwrap();
        if overrides.is_empty() {
            return chunk;
        }
        let (ox, oz) = pos.world_origin();
        let size = SUBCHUNK_SIZE as i32;
        for (&(wx, wy, wz), &block) in overrides.iter() {
            if wx >= ox && wx < ox + size && wz >= oz && wz < oz + size && wy >= 0 && wy < WORLD_HEIGHT {
                chunk.set((wx - ox) as usize, wy as usize, (wz - oz) as usize, block);
            }
        }
        chunk
    }
}
