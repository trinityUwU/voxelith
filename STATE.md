# STATE — voxelith

> Résumé vivant cross-session. Dernière mise à jour : 2026-06-14.

## Où on en est

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

Phase 01b : `multi_draw_indirect` GPU-driven — upload des AABB dans un buffer persistant, compute shader de frustum culling, compaction du buffer de draw. Le CPU cesse de piloter la liste de draw.

## Points ouverts

- Cull frustum encore côté CPU (1 `draw_indexed` par chunk visible) — à déplacer sur GPU en phase 01b.
- Pas encore de greedy meshing : faces individuelles (face-culling). Vertex count élevé vs greedy.
- Palette non bit-packée (forme dense `Vec<u16>`).
- `block_at` fait 6 lookups HashMap par bloc plein au meshing — OK en synchrone one-shot, à revoir au streaming async (phase 05).
- Pas de fog : les chunks lointains apparaissent net jusqu'à la far plane (4000). Fog = phase 04.
