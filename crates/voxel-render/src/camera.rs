//! Responsabilité : caméra fly libre + matrice view-projection uploadée au GPU.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// Uniform caméra : view-projection + position monde (pour le fog de distance).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
}

/// Caméra perspective contrôlée par position + yaw/pitch.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov_y: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(8.0, 80.0, 8.0),
            yaw: -std::f32::consts::FRAC_PI_4,
            pitch: -0.4,
            aspect,
            fov_y: 70f32.to_radians(),
        }
    }

    /// Vecteur de direction du regard à partir de yaw/pitch.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    /// Matrice view-projection courante (clip space wgpu, near 0.1).
    /// Le vecteur up est recalculé perpendiculaire au regard : pas de
    /// dégénérescence ni de flip 90° quand on regarde quasi à la verticale.
    pub fn view_proj(&self) -> Mat4 {
        let forward = self.forward();
        let mut right = forward.cross(Vec3::Y);
        if right.length_squared() < 1e-5 {
            right = Vec3::X;
        }
        let up = right.normalize().cross(forward);
        let view = Mat4::look_to_rh(self.position, forward, up);
        let proj = Mat4::perspective_rh(self.fov_y, self.aspect, 0.1, 4000.0);
        proj * view
    }

    pub fn uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_proj().to_cols_array_2d(),
            camera_pos: [self.position.x, self.position.y, self.position.z, 1.0],
        }
    }
}
