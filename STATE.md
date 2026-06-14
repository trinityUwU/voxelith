# STATE — voxelith

> Résumé vivant cross-session. Dernière mise à jour : 2026-06-14.

## Où on en est

**Phase 00 (socle) — livrée.** Le workspace compile (`cargo check` vert), le binaire ouvre une fenêtre wgpu, génère un disque de chunks de terrain procédural autour de l'origine et le rend en LOD0 avec une caméra fly libre.

Non encore validé : run visuel réel de la fenêtre (build release lourd, à faire au prochain lancement). Le code compile mais le rendu à l'écran n'a pas encore été observé.

## Stack figée

Rust 1.96 · wgpu 29 · winit 0.30 · glam 0.33 · bytemuck · rayon · pollster. Workspace 4 crates.

## Décisions clés

- **Rust + wgpu brut** (pas Bevy) — contrôle bas niveau imposé par le plan GPU-driven.
- **Nom voxelith** — évite le trademark Minecraft (repo public).
- Sous-chunk 16³ en palette+indices, chunk = colonne 24 sous-chunks, world = HashMap.
- Plan de référence : `docs/render-pipeline-v1.html`, séquence dans `docs/ROADMAP.md`.

## Prochaine étape

Phase 01 : passage à `multi_draw_indirect` + frustum culling en compute shader. Avant ça, valider visuellement le socle (lancer la fenêtre, vérifier le terrain et le FPS).

## Points ouverts

- Faces de bordure de chunk émises sans culling cross-chunk (acceptable phase 00, à régler au greedy meshing).
- Palette non bit-packée pour l'instant (forme dense `Vec<u16>`).
