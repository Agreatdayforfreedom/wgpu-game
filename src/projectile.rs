use crate::{entity::EntityUniform, uniform};
pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,

    pub uniform: uniform::Uniform<EntityUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            alive: true,
            uniform,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, dir: f32, deg: f32, fire_speed: f32) {
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(cgmath::Deg(deg))
            .set_scale(self.scale)
            .exec();
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
