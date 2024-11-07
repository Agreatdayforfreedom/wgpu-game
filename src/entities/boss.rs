use std::time::Duration;

use cgmath::{InnerSpace, Point2, Vector2};
use rand::Rng;

use crate::{
    ai::patrol_area::PatrolArea,
    audio::Audio,
    collider::Bounds,
    entity::{Entity, EntityUniform},
    explosion::{Explosion, ExplosionType},
    particle_system::{self, system::ParticleSystem},
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::{distance, CompassDir, IdVendor},
    weapon::{
        cannon::{BulletType, Cannon},
        weapon::Weapon,
    },
};

const MIN_DISTANCE_TO_ATTACK: f32 = 250.0;
const INITIAL_HIT_POINTS: i32 = 500;
pub struct Boss {
    id: u32,
    position: cgmath::Vector2<f32>,
    scale: cgmath::Vector2<f32>,
    alive: bool,
    pub uniform: Uniform<EntityUniform>,
    weapon: Vec<Box<dyn Weapon>>,
    rotation: cgmath::Deg<f32>,
    sprite: Sprite,
    hit_points: i32,
    targeting: bool,
    time: f64,
    spiral_interval: instant::Instant,
}

impl Boss {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u32,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Box<Self> {
        let uniform = Uniform::<EntityUniform>::new(&device);

        let bytes = include_bytes!("../assets/entities/boss.png");
        let sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToBorder,
            &create_bind_group_layout(device),
            bytes,
        );

        Box::new(Self {
            id,
            position,
            scale,
            alive: true,
            uniform,
            rotation: cgmath::Deg(0.0),
            hit_points: INITIAL_HIT_POINTS,
            weapon: vec![
                Cannon::new(50, true, BulletType::BulletViolet, &device, &queue),
                Cannon::new(50, true, BulletType::BulletViolet, &device, &queue),
                Cannon::new(50, true, BulletType::BulletViolet, &device, &queue),
                Cannon::new(50, true, BulletType::BulletViolet, &device, &queue),
            ],
            targeting: false,
            sprite,
            time: 0.0,
            spiral_interval: instant::Instant::now(),
        })
    }
}

impl Entity for Boss {
    fn update(
        &mut self,
        dt: &instant::Duration,
        input: &crate::input::Input,
        _audio: &mut Audio,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
    ) {
        let pos = vec![
            self.get_orientation_point(
                (self.bottom_left().x + 40.0, self.bottom_left().y + 87.5).into(),
            ),
            self.get_orientation_point(
                (self.bottom_right().x - 40.0, self.bottom_right().y + 87.5).into(),
            ),
        ];
        let alive = self.alive;

        let mut i = 0usize;
        let mut pos_index = 0;
        while i < self.weapon.len() {
            if i > 1 {
                pos_index = 1;
            }
            let weapon = self.weapon.get_mut(i).unwrap();
            weapon.update(pos[pos_index], 300.0, queue, dt, particle_system);
            if alive {
                if self.spiral_interval.elapsed().as_millis() >= 5000 {
                    weapon.shoot(
                        device,
                        vec![pos[pos_index]],
                        CompassDir::from_deg(self.rotation.0)
                            .rotate(self.time as f32 + (180 * i) as f32),
                        input,
                        None,
                        id_vendor,
                        particle_system,
                    );
                }
            }
            i += 1;
        }
        if self.spiral_interval.elapsed().as_millis() >= 5000 {
            self.time += dt.as_secs_f64() * 180.0;
            if self.time >= 500.0 {
                self.spiral_interval = instant::Instant::now();
                self.time = 0.0;
            }
        }

        if self.alive() {
            self.uniform
                .data
                .set_position(self.position)
                .set_scale(self.scale)
                .set_rotation(self.rotation)
                .exec();
            self.uniform.write(queue);
        }
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn rotation(&self) -> cgmath::Deg<f32> {
        self.rotation
    }

    fn scale(&self) -> Vector2<f32> {
        self.scale
    }

    fn position(&self) -> cgmath::Vector2<f32> {
        self.position
    }

    fn hit(&mut self, hits: i32) {
        self.hit_points -= hits;
    }

    fn get_hit_points(&self) -> i32 {
        self.hit_points
    }

    fn destroy(&mut self) {
        self.alive = false;
    }

    fn get_bounds(&self) -> Bounds {
        Bounds {
            origin: Point2::new(self.position().x - 80.0, self.position().y - 80.0),
            area: Vector2::new(160.0, 80.0),
        }
    }

    fn set_target_point(&mut self, target: Vector2<f32>, dt: &Duration) {
        let dist = distance(target, self.position());
        let dir = (self.position() - target).normalize();
        let dx = (self.position().x + self.scale.x * 0.5) - target.x;
        //set the point in the head
        let dy = (self.position().y + self.scale.y * 0.5) - (target.y - 0.5);

        let angle = dy.atan2(dx);

        let angle = angle * 180.0 / std::f32::consts::PI;

        if dist < MIN_DISTANCE_TO_ATTACK {
            self.targeting = true;
        } else {
            //COUPLE IT AGAIN
            self.targeting = false;
        }
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive() {
            self.sprite.bind(rpass);
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }

        for weapon in &mut self.weapon {
            weapon.draw(rpass);
        }
    }
}
