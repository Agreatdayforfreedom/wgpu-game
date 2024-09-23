use std::ops::DerefMut;

use cgmath::{Point2, Vector2};

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::{Entity, EntityUniform},
    explosion::Explosion,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::CompassDir,
    weapon::{cannon::Cannon, projectile::Projectile, weapon::Weapon},
};

pub struct EvilShip {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    // pub projectiles: (Sprite, Vec<Projectile>),
    explosion: Explosion,
    weapon: Box<dyn Weapon>,
    pub rotation: cgmath::Deg<f32>,
    pub interval: instant::Instant,
    sprite: Sprite, //todo
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
        Self {
            position,
            scale,
            alive: true,
            uniform,
            rotation: cgmath::Deg(0.0),
            explosion: Explosion::new((40.0, 40.0).into(), device, queue),
            weapon: Cannon::new(400, true, &device, &queue),
            interval: instant::Instant::now(),
            sprite,
        }
    }
}

impl EvilShip {
    pub fn set_target_point(target: Vector2<f32>) {
        // self.po
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
        // let delta_ = dt.as_secs_f32();
        self.weapon.update(self.position, queue, dt);
        if self.alive() {
            let center = Vector2::new(
                self.position.x + (self.scale.x / 2.0),
                self.position.y + (self.scale.y / 2.0),
            ); // todo fix center position
            self.weapon.shoot(
                device,
                center,
                (40.0, 40.0).into(),
                CompassDir::from_deg(self.rotation.0 + 90.0),
                input,
                audio,
            );
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