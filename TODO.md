# TODO — voxelith

## En cours

- [ ] Valider visuellement le socle phase 00 (lancer la fenêtre, observer terrain + FPS)

## Phase 01 — indirect + frustum culling

- [ ] Upload des AABB de chunk dans un buffer GPU persistant
- [ ] Compute shader de frustum culling (test AABB vs 6 plans)
- [ ] Compaction du buffer d'arguments de draw
- [ ] Migration vers `multi_draw_indirect`

## Backlog

- [ ] Greedy/binary meshing à la place du face-culling naïf (culling cross-chunk inclus)
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
