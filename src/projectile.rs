use cgmath::SquareMatrix;

use crate::{entity::EntityUniform, uniform};
pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub alive: bool,

    pub uniform: uniform::Uniform<EntityUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        size: f32,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            size,
            alive: true,
            uniform,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, dir: f32, deg: f32, fire_speed: f32) {
        self.uniform.data.set_position(self.position);
        self.uniform.data.set_rotation(cgmath::Deg(deg));
        self.uniform.data.set_size(self.size);
        if self.position.y < 0.0 || self.position.y > 600.0 {
            self.alive = false;
        }
        self.fire(dt, dir, fire_speed);
    }

    pub fn fire(&mut self, dt: &instant::Duration, dir: f32, fire_speed: f32) {
        if self.alive {
            self.position.y -= fire_speed * dir * dt.as_secs_f32();
        }
    }
}
