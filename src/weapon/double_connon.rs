use std::slice::IterMut;

use cgmath::{Angle, Quaternion, Rotation3, Vector2, Vector3};

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    particle_system::system::ParticleSystem,
    player,
    rendering::{create_bind_group_layout, Sprite},
    util::{CompassDir, IdVendor},
};

use super::projectile::Projectile;
use super::weapon::Weapon;

const LIFETIME: u128 = 5000;
const SCALE: Vector2<f32> = Vector2::new(40.0, 40.0);

pub struct DoubleCannon {
    pub projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128,
    sprite: Sprite,
    velocity: f32,
    auto: bool,
}

impl DoubleCannon {
    pub fn new(
        shooting_interval: u128,
        auto: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullet.png");
        let bind_group_layout = create_bind_group_layout(device);

        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );

        Box::new(Self {
            projectiles: vec![],
            time: instant::Instant::now(),
            shooting_interval,
            sprite,
            velocity: 500.0,
            auto,
        })
    }
}

impl Weapon for DoubleCannon {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        positions: Vec<cgmath::Vector2<f32>>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
    ) {
        if (input.is_pressed("f") || self.auto)
            && self.time.elapsed().as_millis() >= self.shooting_interval
        {
            self.time = instant::Instant::now();
            audio.push(Sounds::Shoot, 0.5);

            for position in positions {
                let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
                let p = Projectile::new(
                    id_vendor.next_id(),
                    position,
                    SCALE,
                    dir.angle,
                    2,
                    Bounds {
                        area: SCALE,
                        origin: cgmath::Point2 {
                            x: position.x,
                            y: position.y,
                        },
                    },
                    dir,
                    projectile_uniform,
                );

                self.projectiles.push(p);
            }
        };
    }
    fn update(
        &mut self,
        position: cgmath::Vector2<f32>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        particle_system: &mut ParticleSystem,
    ) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles.get_mut(i).unwrap();
            if projectile.lifetime() > LIFETIME {
                projectile.destroy();
            }

            if !projectile.is_destroyed() {
                projectile.update();
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(projectile.position.x, projectile.position.y),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.set_direction(|this| {
                    this.position.x += (500.0) * this.dir.dir.x * dt.as_secs_f32();
                    this.position.y -= (500.0) * this.dir.dir.y * dt.as_secs_f32();
                    // this.position.x = position.x - this.scale.x / 2.0;
                    // this.position.y = position.y - this.scale.y / 2.0;
                    this.initial_position = position;
                });
                projectile.uniform.write(queue);
                i += 1;
            } else {
                self.projectiles.swap_remove(i);
            }
        }
    }

    fn get_projectiles(&mut self) -> IterMut<Projectile> {
        self.projectiles.iter_mut()
    }

    fn get_name(&self) -> &str {
        "double_cannon"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
