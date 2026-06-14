# ARBORESCENCE — voxelith

```
voxelith/
├── Cargo.toml                      Workspace : membres, deps partagées, profils
├── README.md                       Présentation, stack, lancement
├── STATE.md                        Résumé vivant cross-session
├── TODO.md                         Tâches en cours + backlog par phase
├── ARBORESCENCE.md                 Ce fichier
├── .echoforge.yml                  Config EchoForge (type desktop)
├── .env.example                    Variables d'env (RUST_LOG, WGPU_BACKEND)
├── .gitignore                      target/, logs, fichiers tmp
├── start.sh / stop.sh / restart.sh Scripts de lancement avec PID + reset logs
├── assets/
│   └── shaders/
│       └── terrain.wgsl            Shader terrain LOD0 (vs + fs, éclairage Lambert)
├── docs/
│   ├── ROADMAP.md                  Séquence des phases 00→05
│   └── render-pipeline-v1.html     Plan technique de référence (Système A)
├── logs/                           Logs runtime (reset au start)
├── app/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 Binaire voxelith : boucle winit, App, génération monde
│       └── input.rs                État clavier caméra fly + application déplacement
└── crates/
    ├── voxel-world/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs              Re-exports du crate
    │       ├── block.rs            BlockState, ChunkPos, constantes de grille
    │       ├── palette.rs          SubChunk : palette + indices
    │       ├── chunk.rs            Chunk, Aabb, MeshState
    │       ├── world.rs            World : HashMap de chunks
    │       └── gen.rs              Génération procédurale heightmap
    ├── voxel-mesh/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs              Re-exports
    │       ├── vertex.rs           Vertex (Pod), ChunkMesh
    │       └── cull.rs             Mesher LOD0 par face-culling
    └── voxel-render/
        ├── Cargo.toml
        └── src/
            ├── lib.rs              Re-exports (Camera, Renderer)
            ├── gpu.rs              Contexte wgpu : surface, device, queue, depth
            ├── pipeline.rs         Render pipeline terrain, bind group caméra
            ├── camera.rs           Caméra perspective + CameraUniform
            ├── frustum.rs          Frustum (Gribb–Hartmann) + test AABB pour cull CPU
            └── renderer.rs         Orchestration frame : upload meshes, render pass cullé
```
