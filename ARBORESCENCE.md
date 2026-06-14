# ARBORESCENCE — voxelith

```
voxelith/
├── Cargo.toml                      Workspace : membres, deps partagées, profils
├── README.md                       Présentation, stack, lancement
├── STATE.md                        Résumé vivant cross-session
├── TODO.md                         Tâches en cours + backlog
├── ARBORESCENCE.md                 Ce fichier
├── .echoforge.yml / .env.example / .gitignore
├── start.sh / stop.sh / restart.sh Scripts de lancement (PID + reset logs)
├── assets/
│   └── shaders/
│       └── terrain.wgsl            Shader terrain : texture array, teinte, éclairage, fog
├── docs/
│   ├── ROADMAP.md                  Séquence des phases
│   └── render-pipeline-v1.html     Plan technique de référence (Système A)
├── logs/                           Logs runtime (reset au start)
├── app/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 Binaire : boucle winit, App, câblage streaming
│       ├── input.rs                État clavier caméra fly + déplacement
│       └── stream.rs               ChunkManager : streaming async (rayon + channel)
└── crates/
    ├── voxel-world/                Grille logique + génération
    │   └── src/
    │       ├── lib.rs              Re-exports
    │       ├── block.rs            BlockState, ChunkPos, constantes de grille
    │       ├── palette.rs          SubChunk : palette + indices
    │       ├── chunk.rs            Chunk, Aabb, MeshState
    │       ├── world.rs            World : HashMap de chunks (édition future)
    │       ├── registry.rs         Registry blocs : kind, textures par face, visibilité
    │       ├── biome.rs            Registry biomes + sélection multi-noise
    │       ├── texture.rs          Catalogue textures (source de vérité layers)
    │       └── worldgen.rs         FBM + splines + biomes → terrain procédural
    ├── voxel-mesh/                 Grille → géométrie GPU
    │   └── src/
    │       ├── lib.rs              Re-exports
    │       ├── vertex.rs           Vertex (pos/normal/uv/tint/layer), ChunkMesh
    │       └── greedy.rs           Greedy meshing avec UV, teinte, texture index
    └── voxel-render/               Pipeline de rendu GPU
        └── src/
            ├── lib.rs              Re-exports (Camera, Renderer)
            ├── gpu.rs              Contexte wgpu : surface, device, queue, depth
            ├── pipeline.rs         Render pipeline, bind groups caméra + texture
            ├── camera.rs           Caméra perspective + CameraUniform (pos pour fog)
            ├── frustum.rs          Frustum (Gribb–Hartmann) + test AABB
            ├── texture.rs          Génération du texture array procédural
            └── renderer.rs         Frame : upload/retrait dynamique meshes, render pass
```
