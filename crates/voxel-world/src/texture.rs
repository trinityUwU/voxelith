//! Responsabilité : catalogue ordonné des textures de bloc — source de vérité unique
//! partagée par le registry (indices de layer) et le générateur de texture array du
//! crate render. L'index dans `TEXTURES` EST le layer dans le texture array GPU.

/// Famille de motif procédural appliquée à la couleur de base d'une texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexKind {
    /// Bruit léger uniforme (pierre, terre, sable).
    Grain,
    /// Touffes verticales façon herbe sur le dessus.
    GrassTop,
    /// Bande de terre en bas + herbe en haut (côté de bloc d'herbe).
    GrassSide,
    /// Stries horizontales (rondins, strates).
    Strata,
    /// Surface lisse légèrement bruitée (eau).
    Smooth,
}

/// Définition d'une texture : nom, couleur de base RGB, motif procédural.
#[derive(Debug, Clone, Copy)]
pub struct TexDef {
    pub name: &'static str,
    pub rgb: [u8; 3],
    pub kind: TexKind,
}

/// Catalogue des textures. L'ordre fixe les indices de layer (cf. constantes ci-dessous).
pub const TEXTURES: &[TexDef] = &[
    TexDef { name: "stone", rgb: [122, 122, 128], kind: TexKind::Grain },
    TexDef { name: "dirt", rgb: [110, 78, 50], kind: TexKind::Grain },
    TexDef { name: "grass_top", rgb: [88, 148, 66], kind: TexKind::GrassTop },
    TexDef { name: "grass_side", rgb: [110, 78, 50], kind: TexKind::GrassSide },
    TexDef { name: "sand", rgb: [216, 204, 152], kind: TexKind::Grain },
    TexDef { name: "snow", rgb: [236, 240, 246], kind: TexKind::Grain },
    TexDef { name: "water", rgb: [54, 96, 178], kind: TexKind::Smooth },
    TexDef { name: "gravel", rgb: [128, 122, 120], kind: TexKind::Grain },
    TexDef { name: "wood", rgb: [104, 80, 48], kind: TexKind::Strata },
    TexDef { name: "leaves", rgb: [62, 112, 50], kind: TexKind::GrassTop },
];

pub const TEX_STONE: u32 = 0;
pub const TEX_DIRT: u32 = 1;
pub const TEX_GRASS_TOP: u32 = 2;
pub const TEX_GRASS_SIDE: u32 = 3;
pub const TEX_SAND: u32 = 4;
pub const TEX_SNOW: u32 = 5;
pub const TEX_WATER: u32 = 6;
pub const TEX_GRAVEL: u32 = 7;
pub const TEX_WOOD: u32 = 8;
pub const TEX_LEAVES: u32 = 9;

/// Résolution d'une texture (16×16, look blocky vanilla).
pub const TEX_SIZE: u32 = 16;
