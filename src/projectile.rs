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

    pub fn update(
        &mut self,
        dt: &instant::Duration,
        dir: f32,
        deg: f32,
        fire_speed: f32,
    ) -> cgmath::Matrix4<f32> {
        let model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_angle_z(cgmath::Deg(deg))
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
