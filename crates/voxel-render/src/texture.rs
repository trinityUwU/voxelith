//! Responsabilité : génération procédurale du texture array de blocs (16×16 par
//! layer, RGBA) depuis le catalogue `voxel_world::TEXTURES`. Souverain, zéro asset
//! externe : chaque texture est synthétisée par motif depuis sa couleur de base.

use voxel_world::{TexDef, TexKind, TEXTURES, TEX_SIZE};

/// Texture array de blocs prête pour le bind group (vue + sampler).
pub struct BlockTextures {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl BlockTextures {
    /// Construit le texture array et y écrit chaque layer synthétisé.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let layers = TEXTURES.len() as u32;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("block-textures"),
            size: wgpu::Extent3d { width: TEX_SIZE, height: TEX_SIZE, depth_or_array_layers: layers },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (layer, def) in TEXTURES.iter().enumerate() {
            write_layer(queue, &texture, layer as u32, def);
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("block-sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self { view, sampler }
    }
}

/// Synthétise et téléverse un layer (16×16 RGBA) du catalogue.
fn write_layer(queue: &wgpu::Queue, texture: &wgpu::Texture, layer: u32, def: &TexDef) {
    let n = TEX_SIZE as usize;
    let mut data = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let [r, g, b] = pixel(def, x as u32, y as u32);
            let o = (y * n + x) * 4;
            data[o..o + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * TEX_SIZE),
            rows_per_image: Some(TEX_SIZE),
        },
        wgpu::Extent3d { width: TEX_SIZE, height: TEX_SIZE, depth_or_array_layers: 1 },
    );
}

/// Couleur d'un pixel selon le motif procédural de la texture.
fn pixel(def: &TexDef, x: u32, y: u32) -> [u8; 3] {
    let base = def.rgb;
    match def.kind {
        TexKind::Grain => shade(base, hash(x, y, 7) * 0.22 - 0.11),
        TexKind::Smooth => shade(base, hash(x, y, 3) * 0.08 - 0.04),
        TexKind::Strata => shade(base, if y % 4 == 0 { -0.14 } else { hash(x, y, 5) * 0.1 - 0.05 }),
        TexKind::GrassTop => shade(base, hash(x, y, 11) * 0.28 - 0.12),
        TexKind::GrassSide => grass_side(base, x, y),
    }
}

/// Côté de bloc d'herbe : frange verte en haut, terre en dessous.
fn grass_side(dirt: [u8; 3], x: u32, y: u32) -> [u8; 3] {
    let grass = [98, 154, 64];
    let fringe = 3 + (hash(x, 0, 9) * 2.0) as u32;
    if y < fringe {
        shade(grass, hash(x, y, 11) * 0.24 - 0.1)
    } else {
        shade(dirt, hash(x, y, 7) * 0.22 - 0.11)
    }
}

/// Applique un facteur de luminosité signé à une couleur.
fn shade(c: [u8; 3], delta: f32) -> [u8; 3] {
    let f = (1.0 + delta).clamp(0.0, 2.0);
    [
        (c[0] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * f).clamp(0.0, 255.0) as u8,
    ]
}

/// Hash déterministe (x, y, seed) → [0,1).
fn hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(2246822519));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}
