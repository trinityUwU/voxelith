# Roadmap voxelith — Système A (terrain statique)

Source : `docs/render-pipeline-v1.html`. Chaque phase est livrable et testable seule. On ne passe pas à la suivante sans la précédente verte.

| Phase | Nom | Contenu | Livrable |
|---|---|---|---|
| 00 | socle | Boucle wgpu, terrain procédural, meshing LOD0 face-culling, caméra fly | Terrain explorable, FPS mesuré ✅ |
| 01 | indirect | `multi_draw_indirect`, upload AABB, compute frustum culling, compaction | Render distance ×3 à FPS constant |
| 02 | occlusion | Pyramide Hi-Z post depth-pass, test occlusion (frame N−1) | Coût plat en intérieur/sous-terrain |
| 03 | lod | Octree de LOD, agrégation 2³/4³, imposteurs de chunk, sélection par bande | Render distance ≈512 chunks jouable |
| 04 | horizon | Skirts de bordure, geomorphing, fog volumétrique calé sur le dernier LOD | Aucun « vide » visible |
| 05 | stream | Génération + meshing sur thread pool rayon, priorisation par distance/regard | Déplacement rapide sans stutter |

## Décisions de structure de données (posées en phase 00)

- **Sous-chunk 16³** stocké en palette + indices (`voxel-world/src/palette.rs`). Bit-packing réel = optimisation mémoire ultérieure, API stable.
- **Chunk** = colonne 16 × 384 × 16 (24 sous-chunks), porte son `MeshState` et son AABB pour le futur culling GPU.
- **World** = `HashMap<ChunkPos, Chunk>`. L'octree de LOD (index spatial des bandes) arrive en phase 03.
- **Block state** sur `u16` : suffisant pour l'overworld vanilla.

## Données overworld (phase 03+)

À extraire depuis `PrismarineJS/minecraft-data` (MIT, exploitable en produit) : blocs, states, `blockCollisionShapes`, recettes, biomes. Le wiki (CC BY-NC-SA) reste une référence de lecture, pas une source à scraper en produit. Textures : assets maison ou CC0 pour rester clean en distribution — pas d'assets Mojang.
