//! voxel-render — pipeline de rendu GPU du terrain statique.
//! Phase 00 : render pass direct LOD0. Phases suivantes : indirect draw,
//! frustum + occlusion Hi-Z culling, LOD par bandes (cf. docs/ROADMAP.md).

pub mod camera;
pub mod frustum;
pub mod gpu;
pub mod pipeline;
pub mod renderer;

pub use camera::Camera;
pub use renderer::Renderer;
