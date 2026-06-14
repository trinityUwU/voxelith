# STATE — voxelith

> Résumé vivant cross-session. Dernière mise à jour : 2026-06-14.

## Où on en est

**Batch « monde infini » — livré.** Streaming async + worldgen multi-noise + biomes + greedy meshing + textures procédurales. Validé en run release : ~1000 chunks streamés, zéro crash.

- **Streaming infini** (`app/src/stream.rs`) : `ChunkManager` charge/décharge les chunks autour du joueur, génération + greedy meshing sur thread pool **rayon**, résultats collectés par le main thread via channel crossbeam. Plus de bordure fixe. **View distance 32 chunks (512 blocs)**, budget 16 uploads/frame. ~3200 chunks à 60 FPS en release (CPU frustum cull encaisse).
- **Worldgen multi-noise** (`voxel-world/src/worldgen.rs`) : FBM Perlin (crate `noise`), heightmap via splines continentalness + erosion + peaks/valleys, sea level 62. Déterministe → le mesher sonde les voisins cross-chunk via `block_at`.
- **Biomes** (`biome.rs`) : sélection multi-noise temperature/humidity (plains, forest, savanna, desert, snowy, taiga), surface/filler + teinte d'herbe par biome.
- **Greedy meshing** (`voxel-mesh/src/greedy.rs`) : fusion des faces coplanaires, UV de tiling + teinte + index de texture par face.
- **Textures** (`voxel-render/src/texture.rs`) : texture array 16×16 procédural (10 layers), souverain/zéro asset externe. Fog de distance dans le shader pour fermer l'horizon.

### Historique


**Phase 00 (socle) — livrée et validée visuellement** (fenêtre ouverte, terrain à l'écran, Chris a confirmé).

**Phase 01a (culling CPU) — livrée.** Trois améliorations sur la render distance :
- **Cross-chunk face culling** : le mesher échantillonne les chunks voisins via `World::block_at`, plus aucun mur de bordure caché entre chunks pleins. Réduction massive de la géométrie.
- **Frustum culling CPU** : `Frustum::from_view_proj` (Gribb–Hartmann) + test AABB par chunk dans le render pass, les chunks hors champ ne sont pas dessinés.
- **Radius 24** (2401 chunks, 768 blocs de côté, ~3× l'ancienne distance). Meshing au démarrage quasi-instantané en release.
- Bonus : capture souris (curseur verrouillé/masqué) + Échap pour quitter.

Reste de la phase 01 : passage au `multi_draw_indirect` GPU-driven + frustum culling en compute shader (déplacer le cull du CPU vers le GPU).

## Stack figée

Rust 1.96 · wgpu 29 · winit 0.30 · glam 0.33 · bytemuck · rayon · pollster. Workspace 4 crates.

## Décisions clés

- **Rust + wgpu brut** (pas Bevy) — contrôle bas niveau imposé par le plan GPU-driven.
- **Nom voxelith** — évite le trademark Minecraft (repo public).
- Sous-chunk 16³ en palette+indices, chunk = colonne 24 sous-chunks, world = HashMap.
- Plan de référence : `docs/render-pipeline-v1.html`, séquence dans `docs/ROADMAP.md`.

## Prochaine étape

Candidats : caves 3D (density function noise 3D seuillé), arbres/structures, `multi_draw_indirect` GPU-driven (cull sur GPU au lieu du CPU), occlusion Hi-Z. À arbitrer avec Chris.

## Points ouverts

- Cull frustum côté CPU (1 `draw_indexed` par chunk visible) — candidat à passer GPU-driven.
- Pas de collision/édition : on ne conserve pas le voxel data après meshing (regénérable, déterministe). À ajouter pour le gameplay.
- Pas d'occlusion culling (Hi-Z) : terrain dense souterrain encore dessiné s'il est dans le frustum.
- Eau = bloc plein opaque pour l'instant (pas de transparence/blend).
- Pas de caves : terrain plein sous la surface (worldgen 2D heightmap, pas encore de density 3D).
