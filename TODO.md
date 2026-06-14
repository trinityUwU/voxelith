# TODO — voxelith

## Prochains candidats (à arbitrer)

- [ ] Caves & overhangs : density function 3D (noise seuillé) dans le worldgen
- [ ] Arbres / structures (feature placement par biome)
- [ ] Eau transparente (pass de rendu trié, blend)
- [ ] Collision + édition de blocs (conserver le voxel data des chunks proches)
- [ ] GPU-driven `multi_draw_indirect` + frustum culling en compute shader
- [ ] Occlusion culling Hi-Z (phase 02 du plan)
- [ ] LOD par bandes de distance (imposteurs lointains, render distance ≫)

## Backlog

- [ ] Bit-packing réel des indices de palette
- [ ] Mipmaps sur le texture array (anti-aliasing des textures lointaines)
- [ ] Extraction des données overworld depuis minecraft-data (MIT) pour un registry complet
- [ ] Sérialisation/sauvegarde des chunks édités

## Fait

- [x] Scaffold workspace 4 crates, compile vert
- [x] Pipeline wgpu + caméra fly + shader terrain + depth
- [x] Phase 01a : frustum culling CPU, cross-chunk culling, capture souris + Échap
- [x] **Streaming infini** : ChunkManager async (rayon + channel), load/unload autour du joueur
- [x] **Worldgen multi-noise** : FBM, splines continentalness/erosion/peaks-valleys, sea level
- [x] **Biomes** : sélection temperature/humidity, surface/filler + teinte par biome
- [x] **Greedy meshing** : fusion de faces, UV tiling + teinte + texture index
- [x] **Textures** : texture array procédural souverain + fog de distance
- [x] Docs (README, STATE, ROADMAP, ARBORESCENCE), scripts start/stop/restart
