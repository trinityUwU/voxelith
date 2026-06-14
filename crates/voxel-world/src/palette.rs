//! Responsabilité : stockage compressé d'un sous-chunk 16³ par palette + indices bit-packés.
//! Édition O(1), mémoire proportionnelle au nombre d'états distincts présents.

use crate::block::{BlockState, SUBCHUNK_VOLUME};

/// Sous-chunk 16³ : palette des états distincts + indices compactés.
/// `bits/voxel = ceil(log2(palette.len()))`, recalculé à la croissance de palette.
#[derive(Debug, Clone)]
pub struct SubChunk {
    palette: Vec<BlockState>,
    /// Un index de palette par voxel. Forme dense ; le bit-packing réel est une
    /// optimisation mémoire de phase ultérieure (l'API publique reste stable).
    indices: Vec<u16>,
}

impl SubChunk {
    /// Sous-chunk plein d'air (palette = [AIR], coût mémoire minimal).
    pub fn empty() -> Self {
        Self {
            palette: vec![BlockState::AIR],
            indices: vec![0; SUBCHUNK_VOLUME],
        }
    }

    /// `true` si le sous-chunk ne contient que de l'air (skip meshing).
    pub fn is_empty(&self) -> bool {
        self.palette.iter().all(|b| b.is_air())
    }

    /// Lit l'état au voxel linéaire `i`.
    #[inline]
    pub fn get(&self, i: usize) -> BlockState {
        self.palette[self.indices[i] as usize]
    }

    /// Écrit un état au voxel linéaire `i`, étendant la palette si nécessaire.
    pub fn set(&mut self, i: usize, state: BlockState) {
        let idx = self.palette_index(state);
        self.indices[i] = idx;
    }

    /// Retourne l'index de `state` dans la palette, l'y insérant si absent.
    fn palette_index(&mut self, state: BlockState) -> u16 {
        if let Some(pos) = self.palette.iter().position(|&b| b == state) {
            return pos as u16;
        }
        self.palette.push(state);
        (self.palette.len() - 1) as u16
    }

    pub fn palette(&self) -> &[BlockState] {
        &self.palette
    }
}
