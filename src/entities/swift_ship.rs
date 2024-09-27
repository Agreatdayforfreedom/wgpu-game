use std::time::Duration;

use crate::{
    ai::patrol_area::PatrolArea,
    entity::{Entity, EntityUniform},
    explosion::Explosion,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::{distance, CompassDir},
    weapon::{cannon::Cannon, weapon::Weapon},
};
use cgmath::{InnerSpace, Point2, Vector2};
use rand::Rng;

const MIN_DISTANCE_TO_ATTACK: f32 = 500.0;

pub struct SwiftShip {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    explosion: Explosion,
    weapon: Box<dyn Weapon>,
    pub rotation: cgmath::Deg<f32>,
    sprite: Sprite, //todo
    targeting: bool,
    patrol: PatrolArea,
}

impl SwiftShip {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Box<Self> {
        let mut uniform = Uniform::<EntityUniform>::new(&device);
        uniform
            .data
            .set_pivot(Point2::new(scale.x * 0.5, scale.y * 0.5))
            .exec();

        let bytes = include_bytes!("../assets/fast_ship.png");
        let sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToBorder,
            &&create_bind_group_layout(device),
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
            position,
            scale,
            alive: true,
            uniform,
            rotation: cgmath::Deg(0.0),
            explosion: Explosion::new((20.0, 20.0).into(), device, queue),
            weapon: Cannon::new(100, true, &device, &queue),
            targeting: false,
            sprite,
            patrol: PatrolArea::new(points),
        })
    }
}

impl Entity for SwiftShip {
    fn update(
        &mut self,
        dt: &instant::Duration,
        input: &crate::input::Input,
        audio: &mut crate::audio::Audio,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) {
        let pos = self.get_orientation_point((1.0, self.top_left().y).into());

        self.weapon.update(pos, queue, dt);

        if self.patrol.is_over(self.position()) {
            println!("OVER");
            self.patrol.next(self.position());
        }

        if self.alive() {
            if self.targeting {
                self.weapon.shoot(
                    device,
                    pos,
                    (40.0, 40.0).into(),
                    CompassDir::from_deg(self.rotation.0 + 180.0),
                    input,
                    audio,
                );
            }
            self.uniform
                .data
                .set_position(self.position)
                .set_scale(self.scale)
                .set_rotation(self.rotation)
                .exec();
            self.uniform.write(queue);
        } else {
            self.explosion.update(audio, queue, dt);
        }
    }

    fn scale(&self) -> Vector2<f32> {
        self.scale
    }

    fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn rotation(&self) -> cgmath::Deg<f32> {
        self.rotation
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn destroy(&mut self) {
        if self.alive() {
            self.explosion.set_position(self.position());
        }
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
            self.position.x -= 200.0 * dir.x * dt.as_secs_f32();
            self.position.y -= 200.0 * dir.y * dt.as_secs_f32();
            self.rotation = cgmath::Deg(angle + 90.0);
        } else {
            //COUPLE IT AGAIN
            self.patrol.couple(self.position());
            self.targeting = false;
            self.position.x -= 450.0 * self.patrol.get_direction().x * dt.as_secs_f32();
            self.position.y -= 450.0 * self.patrol.get_direction().y * dt.as_secs_f32();
            let dx = self.patrol.get_direction().x;
            let dy = self.patrol.get_direction().y;
            let angle = dy.atan2(dx);

            let angle = angle * 180.0 / std::f32::consts::PI;
            self.rotation = cgmath::Deg(angle + 90.0);
        }
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive() {
            self.sprite.bind(rpass);
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        } else {
            self.explosion.draw(rpass);
        }

        self.weapon.draw(rpass);
    }
}
