use cgmath::SquareMatrix;

use crate::uniform;
pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub alive: bool,

    pub uniform: uniform::Uniform<ProjectileUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        size: f32,
        uniform: uniform::Uniform<ProjectileUniform>,
    ) -> Self {
        Self {
            position,
            size,
            alive: true,
            uniform,
        }
    }

    pub fn update(
        &mut self,
        dt: &instant::Duration,
        dir: f32,
        fire_speed: f32,
    ) -> cgmath::Matrix4<f32> {
        let model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(self.size);

        if self.position.y < 0.0 || self.position.y > 600.0 {
            self.alive = false;
        }
        self.uniform.data.model = model;
        self.fire(dt, dir, fire_speed);
        model
    }

    pub fn fire(&mut self, dt: &instant::Duration, dir: f32, fire_speed: f32) {
        if self.alive {
            self.position.y -= fire_speed * dir * dt.as_secs_f32();
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
