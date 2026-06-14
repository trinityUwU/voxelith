//! Responsabilité : état logique d'un bloc et coordonnées spatiales du monde voxel.
//! La grille logique (BlockState) est la source de vérité ; le rendu en est dérivé.

/// Taille d'arête d'un sous-chunk (16³ voxels).
pub const SUBCHUNK_SIZE: usize = 16;
/// Nombre de voxels dans un sous-chunk.
pub const SUBCHUNK_VOLUME: usize = SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE;
/// Nombre de sous-chunks empilés dans une colonne de chunk (hauteur 384).
pub const SUBCHUNKS_PER_CHUNK: usize = 24;

/// État d'un bloc : identifiant + propriétés encodés sur 16 bits.
/// `0` = air, jamais meshé. L'overworld vanilla tient largement dans un `u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u16);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);

    #[inline]
    pub fn is_air(self) -> bool {
        self.0 == 0
    }

    /// Bloc plein opaque : éligible au greedy meshing et à l'agrégation LOD.
    #[inline]
    pub fn is_solid(self) -> bool {
        !self.is_air()
    }
}

/// Position d'un chunk dans la grille horizontale du monde (clé de streaming).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Coordonnée monde du coin (0,0) du chunk, en blocs.
    #[inline]
    pub fn world_origin(self) -> (i32, i32) {
        (self.x * SUBCHUNK_SIZE as i32, self.z * SUBCHUNK_SIZE as i32)
    }
}

/// Index local `(x, y, z)` → offset linéaire dans un sous-chunk.
#[inline]
pub fn local_index(x: usize, y: usize, z: usize) -> usize {
    (y * SUBCHUNK_SIZE + z) * SUBCHUNK_SIZE + x
}
