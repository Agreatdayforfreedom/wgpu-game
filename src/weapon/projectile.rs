use crate::{
    collider::Bounds, entity::EntityUniform, uniform, util::CompassDir, weapon::projectile,
};

pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub dir: CompassDir,
    pub bounds: Bounds,
    pub rotation: cgmath::Deg<f32>,
    pub initial_position: cgmath::Vector2<f32>,
    pub uniform: uniform::Uniform<EntityUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        rotation: cgmath::Deg<f32>,
        bounds: Bounds,
        dir: CompassDir,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            rotation: CompassDir::from_deg(dir.angle.0).angle,
            bounds,
            dir,
            alive: true,
            initial_position: position,
            uniform,
        }
    }
    // todo remove label :3
    pub fn update(
        &mut self,
        dt: &instant::Duration,
        fire_speed: f32,
        entity_position: cgmath::Vector2<f32>,
        label: &str,
    ) {
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
        if self.position.y < 0.0 || self.position.y > 600.0 {
            self.alive = false;
        }

        self.fire(dt, fire_speed, entity_position, label);
    }

    pub fn fire(
        &mut self,
        dt: &instant::Duration,
        fire_speed: f32,
        entity_position: cgmath::Vector2<f32>,
        label: &str,
    ) {
        if self.alive {
            if label == "laser" {
            } else {
                let spaceship_displacement = entity_position - self.initial_position;
                self.position.x +=
                    fire_speed * self.dir.dir.x * dt.as_secs_f32() + spaceship_displacement.x;
                self.position.y -=
                    fire_speed * self.dir.dir.y * dt.as_secs_f32() - spaceship_displacement.y;
                self.initial_position = entity_position;
            }
        }
    }

    pub fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    pub fn draw<'a, 'b>(&'a self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive {
            rpass.set_vertex_buffer(2, self.uniform.buffer.slice(..));
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}
