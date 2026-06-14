# TODO — voxelith

## Phase 01b — GPU-driven indirect

- [ ] Upload des AABB de chunk dans un buffer GPU persistant
- [ ] Compute shader de frustum culling (test AABB vs 6 plans)
- [ ] Compaction du buffer d'arguments de draw
- [ ] Migration vers `multi_draw_indirect`

## Backlog

- [ ] Greedy/binary meshing à la place du face-culling naïf
- [ ] Bit-packing réel des indices de palette
- [ ] Occlusion Hi-Z (phase 02)
- [ ] Octree de LOD + agrégation 2³/4³ + imposteurs (phase 03)
- [ ] Skirts + geomorphing + fog (phase 04)
- [ ] Streaming async sur rayon (phase 05)
- [ ] Extraction des données overworld depuis minecraft-data (MIT)
- [ ] Capture souris (grab) + relâche sur Échap

## Fait

- [x] Scaffold workspace 4 crates, compile vert
- [x] Structures de données : BlockState, SubChunk palette, Chunk, World
- [x] Génération procédurale heightmap
- [x] Meshing LOD0 face-culling
- [x] Pipeline wgpu + caméra fly + shader terrain éclairé
- [x] Docs (README, STATE, ROADMAP), scripts start/stop/restart
- [x] Phase 01a : cross-chunk face culling, frustum culling CPU, radius 24, capture souris + Échap
