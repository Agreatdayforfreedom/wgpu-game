use std::time::Duration;

use cgmath::{InnerSpace, Point2, Vector2};
use rand::Rng;

use crate::{
    ai::patrol_area::PatrolArea,
    audio::Audio,
    entity::{Entity, EntityUniform},
    explosion::Explosion,
    particle_system::{self, system::ParticleSystem},
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::{distance, CompassDir, IdVendor},
    weapon::{cannon::Cannon, weapon::Weapon},
};

const MIN_DISTANCE_TO_ATTACK: f32 = 250.0;
const INITIAL_HIT_POINTS: i32 = 15;

pub struct EvilShip {
    id: u32,
    position: cgmath::Vector2<f32>,
    scale: cgmath::Vector2<f32>,
    alive: bool,
    pub uniform: Uniform<EntityUniform>,
    weapon: Box<dyn Weapon>,
    rotation: cgmath::Deg<f32>,
    sprite: Sprite, //todo
    hit_points: i32,
    targeting: bool,
    patrol: PatrolArea,
}

impl EvilShip {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u32,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Box<Self> {
        let uniform = Uniform::<EntityUniform>::new(&device);

        let bytes = include_bytes!("../assets/evil_ship.png");
        let sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToBorder,
            &create_bind_group_layout(device),
            bytes,
        );

        let points = vec![
            Vector2::new(
                rand::thread_rng().gen_range(800.0..1600.0),
                rand::thread_rng().gen_range(800.0..1600.0),
            ),
            Vector2::new(
                rand::thread_rng().gen_range(800.0..1600.0),
                rand::thread_rng().gen_range(800.0..1600.0),
            ),
            Vector2::new(
                rand::thread_rng().gen_range(800.0..1600.0),
                rand::thread_rng().gen_range(800.0..1600.0),
            ),
            Vector2::new(
                rand::thread_rng().gen_range(800.0..1600.0),
                rand::thread_rng().gen_range(800.0..1600.0),
            ),
        ];
        Box::new(Self {
            id,
            position,
            scale,
            alive: true,
            uniform,
            rotation: cgmath::Deg(0.0),
            hit_points: INITIAL_HIT_POINTS,
            weapon: Cannon::new(400, true, &device, &queue),
            targeting: false,
            sprite,
            patrol: PatrolArea::new(points),
        })
    }
}

impl Entity for EvilShip {
    fn update(
        &mut self,
        dt: &instant::Duration,
        input: &crate::input::Input,
        audio: &mut Audio,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
    ) {
        let pos = self.get_orientation_point((1.0, self.bottom_left().y).into());
        self.weapon.update(pos, queue, dt, particle_system);

        if self.patrol.is_over(self.position()) {
            self.patrol.next(self.position());
        }

        if self.alive() {
            if self.targeting {
                self.weapon.shoot(
                    device,
                    vec![pos],
                    CompassDir::from_deg(self.rotation.0),
                    input,
                    audio,
                    id_vendor,
                    particle_system,
                );
            }
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
        if self.hit_points <= 0 {
            self.destroy();
        }
    }

    fn destroy(&mut self) {
        self.alive = false;
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
            //DECOUPLE FROM THE PATH
            self.patrol.decouple();
            self.targeting = true;
            self.position.x -= 100.0 * dir.x * dt.as_secs_f32();
            self.position.y -= 100.0 * dir.y * dt.as_secs_f32();
            self.rotation = cgmath::Deg(angle - 90.0);
        } else {
            //COUPLE IT AGAIN
            self.patrol.couple(self.position());
            self.targeting = false;
            self.position.x -= 100.0 * self.patrol.get_direction().x * dt.as_secs_f32();
            self.position.y -= 100.0 * self.patrol.get_direction().y * dt.as_secs_f32();
            let dx = self.patrol.get_direction().x;
            let dy = self.patrol.get_direction().y;
            let angle = dy.atan2(dx);

            let angle = angle * 180.0 / std::f32::consts::PI;
            self.rotation = cgmath::Deg(angle - 90.0);
        }
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive() {
            self.sprite.bind(rpass);
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }

        self.weapon.draw(rpass);
    }
}
