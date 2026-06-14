//! Responsabilité : état et physique du joueur — déplacement, gravité, collision
//! AABB contre les blocs solides, et bascule de gamemode (survival/creative).

use glam::{IVec3, Vec3};
use voxel_world::WorldStore;

use crate::input::InputState;
use winit::keyboard::KeyCode;

/// Demi-largeur du joueur (boîte 0.6 × 1.8 × 0.6).
const HALF_W: f32 = 0.3;
const HEIGHT: f32 = 1.8;
const EYE: f32 = 1.62;
const WALK_SPEED: f32 = 5.0;
const FLY_SPEED: f32 = 16.0;
const GRAVITY: f32 = 28.0;
const JUMP_VELOCITY: f32 = 9.0;
const REACH: f32 = 5.0;

/// Mode de jeu : survie (gravité) ou créatif (vol libre).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Survival => "survival",
            GameMode::Creative => "creative",
        }
    }
}

/// Joueur : position des pieds, vitesse, orientation et gamemode.
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub mode: GameMode,
    on_ground: bool,
}

impl Player {
    pub fn new(pos: Vec3, mode: GameMode) -> Self {
        Self { pos, vel: Vec3::ZERO, yaw: -std::f32::consts::FRAC_PI_4, pitch: -0.3, mode, on_ground: false }
    }

    /// Position de l'œil (caméra).
    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, EYE, 0.0)
    }

    /// Vecteur de visée (depuis yaw/pitch).
    pub fn look_dir(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn reach(&self) -> f32 {
        REACH
    }

    /// Bascule le gamemode et remet la vitesse à zéro.
    pub fn set_mode(&mut self, mode: GameMode) {
        self.mode = mode;
        self.vel = Vec3::ZERO;
    }

    /// Téléporte le joueur (commande /tp).
    pub fn teleport(&mut self, pos: Vec3) {
        self.pos = pos;
        self.vel = Vec3::ZERO;
    }

    /// `true` si le bloc fait partie de l'AABB du joueur (interdit d'y poser un bloc).
    pub fn occupies(&self, b: IVec3) -> bool {
        let min = (self.pos - Vec3::new(HALF_W, 0.0, HALF_W)).floor().as_ivec3();
        let max = (self.pos + Vec3::new(HALF_W, HEIGHT, HALF_W)).floor().as_ivec3();
        b.x >= min.x && b.x <= max.x && b.y >= min.y && b.y <= max.y && b.z >= min.z && b.z <= max.z
    }

    /// Avance la physique d'un pas de temps selon l'input et le monde.
    pub fn update(&mut self, input: &InputState, dt: f32, store: &WorldStore) {
        let wish = self.wish_horizontal(input);
        match self.mode {
            GameMode::Creative => self.update_creative(input, wish),
            GameMode::Survival => self.update_survival(input, wish),
        }
        self.apply_velocity(dt, store);
    }

    /// Direction de déplacement horizontale voulue (WASD, normalisée).
    fn wish_horizontal(&self, input: &InputState) -> Vec3 {
        let forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let mut wish = Vec3::ZERO;
        if input.is(KeyCode::KeyW) {
            wish += forward;
        }
        if input.is(KeyCode::KeyS) {
            wish -= forward;
        }
        if input.is(KeyCode::KeyD) {
            wish += right;
        }
        if input.is(KeyCode::KeyA) {
            wish -= right;
        }
        wish.normalize_or_zero()
    }

    /// Vol libre : vitesse directe, vertical via espace/shift, pas de gravité.
    fn update_creative(&mut self, input: &InputState, wish: Vec3) {
        let mut v = wish * FLY_SPEED;
        if input.is(KeyCode::Space) {
            v.y += FLY_SPEED;
        }
        if input.is(KeyCode::ShiftLeft) {
            v.y -= FLY_SPEED;
        }
        self.vel = v;
    }

    /// Survie : déplacement au sol, gravité, saut si au sol.
    fn update_survival(&mut self, input: &InputState, wish: Vec3) {
        self.vel.x = wish.x * WALK_SPEED;
        self.vel.z = wish.z * WALK_SPEED;
        if self.on_ground && input.is(KeyCode::Space) {
            self.vel.y = JUMP_VELOCITY;
            self.on_ground = false;
        }
    }

    /// Applique la vitesse axe par axe avec résolution de collision.
    fn apply_velocity(&mut self, dt: f32, store: &WorldStore) {
        if self.mode == GameMode::Survival {
            self.vel.y -= GRAVITY * dt;
        }
        self.on_ground = false;
        self.move_axis(1, self.vel.y * dt, store);
        self.move_axis(0, self.vel.x * dt, store);
        self.move_axis(2, self.vel.z * dt, store);
    }

    /// Déplace le joueur d'un delta sur un axe et résout la pénétration des blocs.
    /// Snappe contre la face la plus proche dans le sens du mouvement.
    fn move_axis(&mut self, axis: usize, delta: f32, store: &WorldStore) {
        if delta == 0.0 {
            return;
        }
        self.pos[axis] += delta;
        let (min, max) = self.bounds();
        let mut bound: Option<i32> = None;
        for by in min.y..=max.y {
            for bx in min.x..=max.x {
                for bz in min.z..=max.z {
                    if !store.is_solid(bx, by, bz) {
                        continue;
                    }
                    let coord = [bx, by, bz][axis];
                    bound = Some(match bound {
                        None => coord,
                        Some(b) if delta > 0.0 => b.min(coord),
                        Some(b) => b.max(coord),
                    });
                }
            }
        }
        if let Some(b) = bound {
            self.resolve(axis, b, delta);
        }
    }

    /// Repousse le joueur contre la face `b` selon le sens du mouvement sur l'axe.
    fn resolve(&mut self, axis: usize, b: i32, delta: f32) {
        let b = b as f32;
        let size_pos = if axis == 1 { HEIGHT } else { HALF_W };
        let size_neg = if axis == 1 { 0.0 } else { HALF_W };
        if delta > 0.0 {
            self.pos[axis] = b - size_pos;
        } else {
            self.pos[axis] = b + 1.0 + size_neg;
            if axis == 1 {
                self.on_ground = true;
            }
        }
        self.vel[axis] = 0.0;
    }

    /// Plage de blocs (inclusifs) couverte par l'AABB du joueur.
    fn bounds(&self) -> (IVec, IVec) {
        let min = Vec3::new(self.pos.x - HALF_W, self.pos.y, self.pos.z - HALF_W);
        let max = Vec3::new(self.pos.x + HALF_W, self.pos.y + HEIGHT, self.pos.z + HALF_W);
        (
            IVec { x: min.x.floor() as i32, y: min.y.floor() as i32, z: min.z.floor() as i32 },
            IVec { x: max.x.floor() as i32, y: max.y.floor() as i32, z: max.z.floor() as i32 },
        )
    }
}

/// Coordonnée de bloc entière.
struct IVec {
    x: i32,
    y: i32,
    z: i32,
}
