use std::slice::IterMut;

use crate::{
    audio::Audio, collider::Bounds, entity::EntityUniform, input::Input,
    sprite_renderer::SpriteRenderer, util::CompassDir,
};

use super::{projectile::Projectile, weapon::Weapon};

pub struct Laser {
    projectiles: Vec<Projectile>,
    sprite: SpriteRenderer,
}

impl Laser {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/laser.png");
        let sprite = SpriteRenderer::new(&device, &queue, diffuse_bytes);

        Box::new(Self {
            projectiles: vec![],
            sprite,
        })
    }
}

impl Weapon for Laser {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        input: &Input,
        audio: &mut Audio,
    ) {
        if input.is_pressed("f") && self.projectiles.len() < 1 {
            let uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);

            self.projectiles.push(Projectile::new(
                position,
                scale,
                cgmath::Deg(0.0),
                Bounds {
                    area: scale,
                    origin: cgmath::Point2 {
                        x: position.x,
                        y: position.y,
                    },
                },
                CompassDir::from_deg(0.0),
                uniform,
            ))
        }
    }

    fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {
        for projectile in &mut self.projectiles {
            if projectile.alive {
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(
                        projectile.position.x,
                        projectile.position.y + projectile.scale.y,
                    ),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.update(&dt, 0.0, "laser");
                projectile.scale.y -= 500.0 * dt.as_secs_f32();
                projectile.uniform.write(queue);
            }
        }
    }

    fn get_projectiles(&mut self) -> IterMut<Projectile> {
        self.projectiles.iter_mut()
    }

    fn drain(&mut self) {
        self.projectiles = self
            .projectiles
            .drain(..)
            .filter(|p| p.alive != false)
            .collect();
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
