use std::slice::IterMut;

use cgmath::{Angle, Quaternion, Rotation3, Vector2, Vector3};

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    player,
    rendering::{create_bind_group_layout, Sprite},
    util::CompassDir,
};

use super::projectile::Projectile;
use super::weapon::Weapon;

const LIFETIME: u128 = 5000;
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
        positions: &Vec<cgmath::Vector2<f32>>,
        scale: cgmath::Vector2<f32>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
    ) {
        if (input.is_pressed("f") || self.auto)
            && self.time.elapsed().as_millis() >= self.shooting_interval
        {
            self.time = instant::Instant::now();
            audio.push(Sounds::Shoot, 0.5);
            for i in 0..2 {
                let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
                let position = positions.get(i).unwrap();
                let p = Projectile::new(
                    ((position.x - scale.x / 2.0), (position.y - scale.y / 2.0)).into(),
                    scale,
                    dir.angle,
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

                self.projectiles.push(p);
            }
        };
    }
    fn update(
        &mut self,
        positions: &Vec<cgmath::Vector2<f32>>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
    ) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles.get_mut(i).unwrap();
            let position = positions.get(1 & i).unwrap();
            if projectile.alive && projectile.lifetime.elapsed().as_millis() <= LIFETIME {
                projectile.update(&dt, 500.0, *position, queue);
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(
                        projectile.position.x + projectile.scale.x / 2.0,
                        projectile.position.y + projectile.scale.y / 2.0,
                    ),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.set_direction(|this| {
                    this.position.x += (500.0) * this.dir.dir.x * dt.as_secs_f32();
                    this.position.y -= (500.0) * this.dir.dir.y * dt.as_secs_f32();
                    // this.position.x = position.x - this.scale.x / 2.0;
                    // this.position.y = position.y - this.scale.y / 2.0;
                    this.initial_position = *position;
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
