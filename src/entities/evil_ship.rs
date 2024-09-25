use std::time::Duration;

use cgmath::{Point2, Vector2};

use crate::{
    ai::patrol_area::PatrolArea,
    audio::Audio,
    entity::{Entity, EntityUniform},
    explosion::Explosion,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::{distance, CompassDir},
    weapon::{cannon::Cannon, weapon::Weapon},
};

const MIN_DISTANCE_TO_ATTACK: f32 = 250.0;
pub struct EvilShip {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    // pub projectiles: (Sprite, Vec<Projectile>),
    explosion: Explosion,
    weapon: Box<dyn Weapon>,
    pub rotation: cgmath::Deg<f32>,
    sprite: Sprite, //todo
    targeting: bool,
    patrol: PatrolArea,
}

impl EvilShip {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Self {
        let mut uniform = Uniform::<EntityUniform>::new(&device);
        uniform
            .data
            .set_pivot(Point2::new(scale.x * 0.5, scale.y * 0.5))
            .exec();

        let bytes = include_bytes!("../assets/evil_ship.png");
        let sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToBorder,
            &create_bind_group_layout(device),
            bytes,
        );

        let points = vec![
            Vector2::new(100.0, 100.0),
            Vector2::new(100.0, 200.0),
            Vector2::new(200.0, 200.0),
            Vector2::new(200.0, 100.0),
        ];
        Self {
            position,
            scale,
            alive: true,
            uniform,
            rotation: cgmath::Deg(0.0),
            explosion: Explosion::new((40.0, 40.0).into(), device, queue),
            weapon: Cannon::new(400, true, &device, &queue),
            targeting: false,
            sprite,
            patrol: PatrolArea::new(points),
        }
    }
}

impl EvilShip {
    //
    pub fn set_target_point(&mut self, target: Vector2<f32>, dt: &Duration) {
        use cgmath::InnerSpace;
        let dist = distance(target, self.position());
        let dir = self.position() - target;
        let dx = (self.position().x + self.scale.x * 0.5) - target.x;
        //set the point in the head
        let dy = (self.position().y + self.scale.y * 0.5) - (target.y - 0.5);

        let angle = dy.atan2(dx);

        let angle = angle * 180.0 / std::f32::consts::PI;

        // if dist < MIN_DISTANCE_TO_ATTACK {
        //     self.targeting = true;

        self.position.x -= 100.0 * self.patrol.get_direction(self.position()).x * dt.as_secs_f32();
        self.position.y -= 100.0 * self.patrol.get_direction(self.position()).y * dt.as_secs_f32();
        //     self.rotation = cgmath::Deg(angle + 180.0);
        // } else {
        // }
        // self.patrol.active(self.position());

        // self.patrol.deactive(self.position());
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
    ) {
        self.weapon.update(self.position, queue, dt);

        if self.patrol.is_over(self.position()) {
            self.patrol.next();
        }

        if self.alive() {
            let center = Vector2::new(
                self.position.x + (self.scale.x / 2.0),
                self.position.y + (self.scale.y / 2.0),
            ); // todo fix center position

            if self.targeting {
                self.weapon.shoot(
                    device,
                    center,
                    (40.0, 40.0).into(),
                    CompassDir::from_deg(self.rotation.0 + 90.0),
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
            //update IF NOT ALIVE
            self.explosion.update(audio, queue, dt);
        }
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn scale(&self) -> Vector2<f32> {
        self.scale
    }

    fn position(&self) -> cgmath::Vector2<f32> {
        self.position
    }

    fn destroy(&mut self) {
        if self.alive() {
            self.explosion.set_position(self.position());
        }
        self.alive = false;
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive() {
            self.sprite.bind(rpass);
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        } else {
            //draw IF NOT alive

            self.explosion.draw(rpass);
        }

        self.weapon.draw(rpass);
    }
}
