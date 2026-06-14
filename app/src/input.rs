//! Responsabilité : état clavier de la caméra fly et application du déplacement.

use std::collections::HashSet;

use glam::Vec3;
use voxel_render::Camera;
use winit::keyboard::KeyCode;

/// Touches actuellement enfoncées pour le déplacement fly.
#[derive(Default)]
pub struct InputState {
    pressed: HashSet<KeyCode>,
}

impl InputState {
    pub fn set(&mut self, key: KeyCode, down: bool) {
        if down {
            self.pressed.insert(key);
        } else {
            self.pressed.remove(&key);
        }
    }

    /// Déplace la caméra selon les touches WASD/espace/shift sur `dt` secondes.
    pub fn apply(&self, camera: &mut Camera, dt: f32) {
        let speed = 40.0 * dt;
        let forward = camera.forward();
        let right = forward.cross(Vec3::Y).normalize();
        let mut delta = Vec3::ZERO;

        if self.pressed.contains(&KeyCode::KeyW) {
            delta += forward;
        }
        if self.pressed.contains(&KeyCode::KeyS) {
            delta -= forward;
        }
        if self.pressed.contains(&KeyCode::KeyD) {
            delta += right;
        }
        if self.pressed.contains(&KeyCode::KeyA) {
            delta -= right;
        }
        if self.pressed.contains(&KeyCode::Space) {
            delta += Vec3::Y;
        }
        if self.pressed.contains(&KeyCode::ShiftLeft) {
            delta -= Vec3::Y;
        }
        if delta != Vec3::ZERO {
            camera.position += delta.normalize() * speed;
        }
    }
}
