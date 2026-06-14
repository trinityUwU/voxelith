//! Responsabilité : registry des blocs — identifiants, catégorie de rendu et
//! textures par face. Sépare la grille logique (BlockState) de sa représentation.

use crate::block::BlockState;
use crate::texture::*;

pub const AIR: BlockState = BlockState(0);
pub const STONE: BlockState = BlockState(1);
pub const DIRT: BlockState = BlockState(2);
pub const GRASS: BlockState = BlockState(3);
pub const SAND: BlockState = BlockState(4);
pub const SNOW: BlockState = BlockState(5);
pub const WATER: BlockState = BlockState(6);
pub const GRAVEL: BlockState = BlockState(7);
pub const WOOD: BlockState = BlockState(8);
pub const LEAVES: BlockState = BlockState(9);

/// Catégorie de rendu d'un bloc — pilote la règle de visibilité des faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Solid,
    Liquid,
}

/// Face d'un bloc pour la sélection de texture (dérivée de la normale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Top,
    Bottom,
    Side,
}

impl Face {
    /// Déduit la face depuis l'offset de normale (uniquement le composant Y compte).
    pub fn from_normal(ny: i32) -> Self {
        match ny {
            1 => Face::Top,
            -1 => Face::Bottom,
            _ => Face::Side,
        }
    }
}

/// Catégorie de rendu d'un bloc.
pub fn kind(b: BlockState) -> BlockKind {
    match b {
        AIR => BlockKind::Air,
        WATER => BlockKind::Liquid,
        _ => BlockKind::Solid,
    }
}

/// Index de layer de texture pour une face donnée d'un bloc.
pub fn face_texture(b: BlockState, face: Face) -> u32 {
    match b {
        STONE => TEX_STONE,
        DIRT => TEX_DIRT,
        GRASS => match face {
            Face::Top => TEX_GRASS_TOP,
            Face::Bottom => TEX_DIRT,
            Face::Side => TEX_GRASS_SIDE,
        },
        SAND => TEX_SAND,
        SNOW => TEX_SNOW,
        WATER => TEX_WATER,
        GRAVEL => TEX_GRAVEL,
        WOOD => TEX_WOOD,
        LEAVES => TEX_LEAVES,
        _ => TEX_STONE,
    }
}

/// Nom lisible d'un bloc (HUD, chat).
pub fn name(b: BlockState) -> &'static str {
    match b {
        AIR => "air",
        STONE => "stone",
        DIRT => "dirt",
        GRASS => "grass",
        SAND => "sand",
        SNOW => "snow",
        WATER => "water",
        GRAVEL => "gravel",
        WOOD => "wood",
        LEAVES => "leaves",
        _ => "?",
    }
}

/// `true` si une face de `current` doit être dessinée face au voisin `neighbor`.
/// Solide visible contre air/liquide ; liquide visible seulement contre l'air (surface).
pub fn renders_against(current: BlockState, neighbor: BlockState) -> bool {
    match kind(current) {
        BlockKind::Air => false,
        BlockKind::Solid => kind(neighbor) != BlockKind::Solid,
        BlockKind::Liquid => kind(neighbor) == BlockKind::Air,
    }
}
