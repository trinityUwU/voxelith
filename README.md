# voxelith

Moteur voxel type Minecraft écrit en Rust, rendu GPU-driven sur wgpu. L'objectif est une render distance quasi-infinie sans jamais dessiner le vide : LOD par bandes de distance, culling à deux étages (frustum + occlusion Hi-Z) et streaming asynchrone du terrain.

Le projet suit le plan technique de `docs/render-pipeline-v1.html` (Système A — terrain statique de l'overworld). Les contraptions dynamiques type Create sont un système séparé, hors scope pour l'instant.

## État actuel — phase 00 (socle)

- Fenêtre wgpu (Vulkan/DX12/Metal selon la plateforme)
- Terrain procédural autour de l'origine (heightmap déterministe)
- Meshing LOD0 par face-culling
- Caméra fly libre : WASD pour bouger, souris pour regarder, espace/shift pour monter/descendre

Pas encore de culling GPU, pas de LOD multi-résolution, pas de streaming. C'est le terrain dessiné, rien de plus — la base sur laquelle les phases suivantes s'appuient.

## Stack

| Couche | Choix |
|---|---|
| Rendu | wgpu 29 |
| Fenêtrage / events | winit 0.30 |
| Maths | glam |
| Parallélisme | rayon (phases ultérieures) |
| Buffers GPU | bytemuck |

## Lancer

```bash
# build + run en release
cargo run --release --bin voxelith

# ou via les scripts (logs dans logs/)
./start.sh
./stop.sh
```

Régler la verbosité avec `RUST_LOG` (voir `.env.example`).

## Architecture

Workspace Cargo en quatre crates :

- `voxel-world` — grille logique : block states, sous-chunks palettés, chunks, monde streamé, génération
- `voxel-mesh` — conversion grille → géométrie GPU (face-culling LOD0, puis greedy meshing)
- `voxel-render` — contexte wgpu, pipeline, caméra, render pass
- `app` (`voxelith`) — binaire : boucle winit, input, assemblage

Détail des phases dans `docs/ROADMAP.md`.

## Licence

MIT.
