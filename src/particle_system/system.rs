use cgmath::Vector2;

use crate::{
    camera::Camera,
    entity::EntityUniform,
    rendering, texture,
    uniform::{self, Uniform},
    util::CompassDir,
};

use super::particle::Particle;

pub struct ParticleSystem {
    sprite: rendering::Sprite,
    particles: Vec<Particle>,
}
static mut n: u32 = 0;

impl ParticleSystem {
    pub fn new(sprite: rendering::Sprite, particles: Vec<Particle>) -> Self {
        Self { sprite, particles }
    }

    pub fn update(
        &mut self,
        start_position: Vector2<f32>,
        dir: CompassDir,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
    ) {
        for particle in &mut self.particles {
            particle.update(queue, dt);
        }
        println!("{}", self.particles.len());
        self.particles.push({
            let uniform = Uniform::<EntityUniform>::new(&device);
            Particle::new(
                start_position,
                (4.0, 4.0).into(),
                (1.0, 1.0, 0.0, 1.0).into(),
                20.0,
                2.75,
                dir,
                uniform,
            )
        });
        self.particles = self
            .particles
            .drain(..)
            .filter(|p| p.alive != false)
            .collect();
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        //todo batching
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for particle in &mut self.particles {
            particle.draw(rpass);
        }
    }
}
