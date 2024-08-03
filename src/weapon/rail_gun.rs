use std::slice::IterMut;

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    rendering::Sprite,
    util::CompassDir,
    weapon::projectile::Projectile,
};

use super::weapon::Weapon;

pub struct RailGun {
    projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128, // milliseconds
    sprite: Sprite,
}

impl RailGun {
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
                    ((position.x), position.y).into(),
                    scale,
                    dir.angle + cgmath::Deg(180.0),
                    Bounds {
                        area: scale,
                        origin: cgmath::Point2 {
                            x: position.x,
                            y: position.y,
                        },
                    },
                    if i == -2 {
                        dir.rotate(0.0)
                    } else if i == -1 {
                        dir.rotate(10.0)
                    } else if i == 0 {
                        dir.rotate(20.0)
                    } else if i == 1 {
                        dir.rotate(-10.0)
                    } else {
                        dir.rotate(-20.0)
                    },
                    projectile_uniform,
                ));
            }
        }
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
                        let spaceship_displacement = position - this.initial_position;
                        this.position.x +=
                            500.0 * this.dir.dir.x * dt.as_secs_f32() + spaceship_displacement.x;
                        this.position.y -=
                            500.0 * this.dir.dir.y * dt.as_secs_f32() - spaceship_displacement.y;
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
        "rail_gun"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
