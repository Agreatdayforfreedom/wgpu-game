use cgmath::SquareMatrix;

use crate::input::{self, Input};
use crate::uniform;
pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub scale: f32,
    pub alive: bool,

    pub uniform: uniform::Uniform<ProjectileUniform>,
}
const FIRE_SPEED: f32 = 600.0;
impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: f32,
        uniform: uniform::Uniform<ProjectileUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            alive: true,
            uniform,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, input: &Input) -> cgmath::Matrix4<f32> {
        let model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(self.scale);

        if input.is_pressed("f") {
            self.alive = true;
        }

        if self.position.y < 0.0 {
            self.alive = false;
        }
        self.uniform.data.model = model;
        self.fire(dt);
        model
    }

    pub fn fire(&mut self, dt: &instant::Duration) {
        if self.alive {
            self.position.y -= FIRE_SPEED * dt.as_secs_f32();
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ProjectileUniform {
    pub model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for ProjectileUniform {}
unsafe impl bytemuck::Zeroable for ProjectileUniform {}

impl Default for ProjectileUniform {
    fn default() -> Self {
        let model = cgmath::Matrix4::identity();
        Self { model }
    }
}
