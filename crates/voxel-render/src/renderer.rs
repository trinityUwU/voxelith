//! Responsabilité : orchestration du rendu d'une frame — caméra, upload des
//! maillages de chunk en buffers GPU, render pass terrain avec depth test.

use std::sync::Arc;

use voxel_mesh::{mesh_chunk, ChunkMesh};
use voxel_world::{Aabb, World};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::frustum::Frustum;
use crate::gpu::Gpu;
use crate::pipeline::{build_pipeline, camera_bind_group_layout};

/// Maillage d'un chunk résident sur le GPU, avec son AABB pour le frustum culling.
struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    aabb: Aabb,
}

/// État de rendu complet : contexte GPU, pipeline, caméra et chunks meshés.
pub struct Renderer {
    gpu: Gpu,
    pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    meshes: Vec<GpuMesh>,
}

impl Renderer {
    /// Construit le renderer et téléverse les chunks déjà présents dans `world`.
    pub async fn new(window: Arc<Window>, world: &World) -> Self {
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        let pipeline = build_pipeline(&gpu.device, gpu.config.format, &camera_bgl);

        let mut renderer = Self {
            gpu,
            pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            meshes: Vec::new(),
        };
        renderer.upload_world(world);
        renderer
    }

    /// Meshe et téléverse tous les chunks chargés (synchrone ; async = phase 05).
    pub fn upload_world(&mut self, world: &World) {
        self.meshes.clear();
        for chunk in world.iter() {
            let mesh = mesh_chunk(chunk, world);
            if !mesh.is_empty() {
                self.meshes.push(self.upload_mesh(&mesh, chunk.aabb));
            }
        }
        log::info!("{} chunks meshés (non vides)", self.meshes.len());
    }

    /// Crée les vertex/index buffers GPU d'un maillage de chunk.
    fn upload_mesh(&self, mesh: &ChunkMesh, aabb: Aabb) -> GpuMesh {
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
        GpuMesh { vertex_buffer, index_buffer, index_count: mesh.index_count(), aabb }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.camera.aspect = self.gpu.aspect();
    }

    /// Pousse l'état caméra courant dans son uniform buffer.
    pub fn update_camera(&mut self) {
        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera.uniform()),
        );
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Dessine une frame complète du terrain visible.
    pub fn render(&mut self) {
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
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.52, g: 0.70, b: 0.92, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.gpu.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for mesh in &self.meshes {
            if !frustum.intersects_aabb(&mesh.aabb) {
                continue;
            }
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
