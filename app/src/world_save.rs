//! Responsabilité : persistance des mondes sur disque — métadonnées (seed, gamemode,
//! position joueur) + blocs édités, sous ~/.local/share/voxelith/saves/<slug>/.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Métadonnées persistées d'un monde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMeta {
    pub name: String,
    pub seed: u32,
    pub gamemode: String,
    pub player: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}

/// Blocs édités sérialisés (x, y, z, id).
#[derive(Default, Serialize, Deserialize)]
struct Edits {
    blocks: Vec<(i32, i32, i32, u16)>,
}

/// Dossier racine des sauvegardes (créé à la demande).
pub fn saves_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share/voxelith/saves")
}

/// Slug de dossier dérivé du nom (alphanumérique + tirets).
fn slug(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "monde".to_string()
    } else {
        s
    }
}

fn world_dir(name: &str) -> PathBuf {
    saves_dir().join(slug(name))
}

/// Liste les mondes existants (métadonnées parsées), triés par nom.
pub fn list_worlds() -> Vec<WorldMeta> {
    let mut worlds = Vec::new();
    let Ok(entries) = fs::read_dir(saves_dir()) else {
        return worlds;
    };
    for entry in entries.flatten() {
        let meta_path = entry.path().join("meta.json");
        match fs::read_to_string(&meta_path).map(|s| serde_json::from_str::<WorldMeta>(&s)) {
            Ok(Ok(meta)) => worlds.push(meta),
            _ => log::warn!("monde illisible : {}", meta_path.display()),
        }
    }
    worlds.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    worlds
}

/// Sauvegarde métadonnées + blocs édités d'un monde.
pub fn save_world(meta: &WorldMeta, blocks: Vec<(i32, i32, i32, u16)>) -> io::Result<()> {
    let dir = world_dir(&meta.name);
    fs::create_dir_all(&dir)?;
    let meta_json = serde_json::to_string_pretty(meta).map_err(io::Error::other)?;
    fs::write(dir.join("meta.json"), meta_json)?;
    let edits_json = serde_json::to_string(&Edits { blocks }).map_err(io::Error::other)?;
    fs::write(dir.join("edits.json"), edits_json)?;
    log::info!("monde sauvegardé : {}", dir.display());
    Ok(())
}

/// Charge les blocs édités d'un monde (vide si absent/illisible).
pub fn load_edits(name: &str) -> Vec<(i32, i32, i32, u16)> {
    let path = world_dir(name).join("edits.json");
    match fs::read_to_string(&path).map(|s| serde_json::from_str::<Edits>(&s)) {
        Ok(Ok(edits)) => edits.blocks,
        _ => Vec::new(),
    }
}
