use std::slice::IterMut;

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    player::{self, Player},
    sprite_renderer::SpriteRenderer,
    util::CompassDir,
    weapon::projectile::Projectile,
};

use super::weapon::Weapon;

pub struct RailGun {
    projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128, // milliseconds
    sprite: SpriteRenderer,
}

impl RailGun {
    pub fn new(shooting_interval: u128, device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullet.png");
        let sprite = SpriteRenderer::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            diffuse_bytes,
        );

        Box::new(Self {
            projectiles: vec![],
            time: instant::Instant::now(),
            shooting_interval,
            sprite,
        })
    }
}

impl Weapon for RailGun {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
    ) {
        if input.is_pressed("f") && self.time.elapsed().as_millis() >= self.shooting_interval {
            self.time = instant::Instant::now();
            audio.push(Sounds::Shoot);
            audio.push(Sounds::Shoot);
            //todo
            for i in -2..=2 {
                let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
                self.projectiles.push(Projectile::new(
                    ((position.x - 2.0) + i as f32 * 5.0, position.y).into(),
                    scale,
                    (dir.angle + cgmath::Deg(180.0)),
                    Bounds {
                        area: scale,
                        origin: cgmath::Point2 {
                            x: position.x,
                            y: position.y,
                        },
                    },
                    if i == -2 {
                        dir
                    } else if i == -1 {
                        dir
                    } else if i == 0 {
                        dir
                    } else if i == 1 {
                        dir
                    } else {
                        dir
                    },
                    projectile_uniform,
                ));
            }
        }
    }

    fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration, time: f64) {
        for projectile in &mut self.projectiles {
            // println!("p: {:?}", projectile.dir);
            if projectile.alive {
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(
                        projectile.position.x + projectile.scale.x / 2.0,
                        projectile.position.y + projectile.scale.y / 2.0,
                    ),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.update(&dt, 500.0, ":D");
                projectile.uniform.write(queue);
            }
        }
    }

    fn drain(&mut self) {
        self.projectiles = self
            .projectiles
            .drain(..)
            .filter(|p| p.alive != false)
            .collect();
    }

    fn get_projectiles(&mut self) -> IterMut<Projectile> {
        self.projectiles.iter_mut()
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
