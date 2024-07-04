use crate::{entity::EntityUniform, uniform, util::CompassDir};

pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub dir: CompassDir,
    pub rotation: cgmath::Deg<f32>,
    pub uniform: uniform::Uniform<EntityUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        rotation: cgmath::Deg<f32>,
        dir: CompassDir,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            rotation,
            dir,
            alive: true,
            uniform,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, fire_speed: f32) {
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
        if self.position.y < 0.0 || self.position.y > 600.0 {
            self.alive = false;
        }
        self.fire(dt, fire_speed);
    }

    pub fn fire(&mut self, dt: &instant::Duration, fire_speed: f32) {
        if self.alive {
            self.position.x += fire_speed * self.dir.to_dir().x * dt.as_secs_f32();
            self.position.y -= fire_speed * self.dir.to_dir().y * dt.as_secs_f32();
        }
    }
}
