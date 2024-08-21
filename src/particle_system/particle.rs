use cgmath::{Array, Vector2, Vector4};
use rand::Rng;
use wgpu::Queue;

use crate::{entity::EntityUniform, uniform, util::CompassDir};

pub struct Particle {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    color: Vector4<f32>,
    velocity: f32,
    life: f32,
    dir: CompassDir,
    pub alive: bool,
    uniform: uniform::Uniform<EntityUniform>,
}

impl Particle {
    pub fn new(
        position: Vector2<f32>,
        scale: Vector2<f32>,
        color: Vector4<f32>,
        velocity: f32,
        life: f32,
        dir: CompassDir,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        let random = ((rand::thread_rng().gen_range(0..10000) as f32 % 100.0) - 50.0) / 10.0;
        Self {
            position: (position.x + random * 20.0, position.y + random).into(),
            scale,
            color,
            velocity,
            life,
            dir,
            alive: true,
            uniform,
        }
    }

    pub fn update(&mut self, queue: &mut Queue, dt: &instant::Duration) {
        let dt = dt.as_secs_f32();
        self.life -= dt;
        if self.life <= 0.0 {
            self.alive = false
        } else {
            self.position.x += self.velocity * self.dir.dir.x * dt;
            self.position.y -= self.velocity * self.dir.dir.y * dt;
            // self.color.w -= dt * 1.5;
        };

        self.uniform
            .data
            .set_position(self.position)
            .set_scale(self.scale)
            .set_color(self.color)
            .exec();
        self.uniform.write(queue);
    }

    pub fn draw<'a, 'b>(&'a self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(2, self.uniform.buffer.slice(..));
        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
