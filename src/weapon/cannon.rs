use std::slice::IterMut;

use cgmath::{Angle, Quaternion, Rotation3, Vector2, Vector3};

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    player,
    rendering::Sprite,
    util::CompassDir,
};

use super::projectile::Projectile;
use super::weapon::Weapon;
pub struct Cannon {
    pub projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128, // milliseconds
    sprite: Sprite,
}

impl Cannon {
    pub fn new(shooting_interval: u128, device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullet.png");
        let sprite = Sprite::new(
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

impl Weapon for Cannon {
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
            let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            audio.push(Sounds::Shoot);
            let mut p = Projectile::new(
                (position.x, position.y).into(),
                scale,
                dir.angle.opposite(),
                Bounds {
                    area: scale,
                    origin: cgmath::Point2 {
                        x: position.x,
                        y: position.y,
                    },
                },
                dir,
                projectile_uniform,
            );
            p.uniform
                .data
                .set_pivot((0.5 * scale.x, 0.5 * scale.y).into())
                .exec();

            self.projectiles.push(p);
        };
    }

    fn update(
        &mut self,
        position: cgmath::Vector2<f32>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        time: f64,
    ) {
        for projectile in &mut self.projectiles {
            if projectile.alive {
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(
                        projectile.position.x + projectile.scale.x / 2.0,
                        projectile.position.y + projectile.scale.y / 2.0,
                    ),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.update(&dt, 500.0, position);
                projectile.set_direction(|this| {
                    if this.alive {
                        let q = Quaternion::from_angle_z(this.dir.angle);

                        // let offset = Vector3::new(15.0, 15.0, 0.0);
                        let spaceship_displacement = position - this.initial_position;
                        let spaceship_displacement = 0.0;
                        this.position.x +=
                            500.0 * this.dir.dir.x * dt.as_secs_f32() + spaceship_displacement;
                        this.position.y -=
                            500.0 * this.dir.dir.y * dt.as_secs_f32() - spaceship_displacement;
                        this.initial_position = position;
                    }
                });
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

    fn get_name(&self) -> &str {
        "cannon"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
