//! Responsabilité : orchestration du rendu d'une frame — caméra, texture array,
//! upload/retrait dynamique des maillages de chunk (streaming) et render pass
//! terrain avec frustum culling CPU.

use std::collections::HashMap;
use std::sync::Arc;

use voxel_mesh::ChunkMesh;
use voxel_world::{Aabb, ChunkPos};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::frustum::Frustum;
use crate::gpu::Gpu;
use crate::pipeline::{build_pipeline, camera_bind_group_layout, texture_bind_group_layout};
use crate::texture::BlockTextures;
use crate::ui::{Overlay, UiBatch};

/// Maillage d'un chunk résident sur le GPU, avec son AABB pour le frustum culling.
struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    aabb: Aabb,
}

/// État de rendu : contexte GPU, pipeline, caméra, textures, chunks streamés.
pub struct Renderer {
    gpu: Gpu,
    pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    overlay: Overlay,
    meshes: HashMap<ChunkPos, GpuMesh>,
}

impl Renderer {
    /// Construit le renderer (contexte GPU, textures procédurales, pipeline).
    pub async fn new(window: Arc<Window>) -> Self {
        let gpu = Gpu::new(window).await;
        let camera = Camera::new(gpu.aspect());

        let camera_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-uniform"),
            contents: bytemuck::bytes_of(&camera.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bgl = camera_bind_group_layout(&gpu.device);
        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
        });

        let textures = BlockTextures::new(&gpu.device, &gpu.queue);
        let texture_bgl = texture_bind_group_layout(&gpu.device);
        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture-bg"),
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&textures.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&textures.sampler) },
            ],
        });

        let pipeline = build_pipeline(&gpu.device, gpu.config.format, (&camera_bgl, &texture_bgl));
        let overlay = Overlay::new(&gpu.device, gpu.config.format);

        Self {
            gpu,
            pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            texture_bind_group,
            overlay,
            meshes: HashMap::new(),
        }
    }

    /// `true` si un chunk est déjà résident sur le GPU.
    pub fn has_chunk(&self, pos: ChunkPos) -> bool {
        self.meshes.contains_key(&pos)
    }

    /// Téléverse (ou remplace) le maillage GPU d'un chunk.
    pub fn upload_chunk(&mut self, pos: ChunkPos, mesh: &ChunkMesh, aabb: Aabb) {
        let vertex_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk-vertices"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk-indices"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.meshes.insert(pos, GpuMesh { vertex_buffer, index_buffer, index_count: mesh.index_count(), aabb });
    }

    /// Retire un chunk du GPU (déchargement par distance).
    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        self.meshes.remove(&pos);
    }

    /// Vide tous les chunks GPU (changement/fermeture de monde).
    pub fn clear_chunks(&mut self) {
        self.meshes.clear();
    }

    pub fn loaded_count(&self) -> usize {
        self.meshes.len()
    }

    /// Taille courante de la surface (pour la mise en page de l'UI).
    pub fn size(&self) -> (u32, u32) {
        (self.gpu.config.width, self.gpu.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.camera.aspect = self.gpu.aspect();
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Pousse l'état caméra courant dans son uniform buffer.
    fn update_camera(&mut self) {
        self.gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&self.camera.uniform()));
    }

    /// Dessine une frame : terrain frustum-cullé puis overlay UI 2D.
    pub fn render(&mut self, ui: &UiBatch) {
        self.update_camera();
        let frame = match self.gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);
                return;
            }
            _ => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        let frustum = Frustum::from_view_proj(&self.camera.view_proj());
        self.terrain_pass(&mut encoder, &view, &frustum);
        self.overlay.render(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &view,
            ui,
            (self.gpu.config.width, self.gpu.config.height),
        );
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    /// Render pass unique du terrain (clear ciel + depth + draws indexés frustum-cullés).
    fn terrain_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, frustum: &Frustum) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("terrain-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.62, g: 0.74, b: 0.92, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.gpu.depth_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.texture_bind_group, &[]);
        for mesh in self.meshes.values() {
            if !frustum.intersects_aabb(&mesh.aabb) {
                continue;
            }
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
