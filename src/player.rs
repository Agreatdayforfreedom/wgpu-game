use crate::audio::Audio;
use crate::entity::Entity;
use crate::uniform;
use crate::util::CompassDir;
use crate::weapon::cannon::Cannon;
use crate::weapon::laser::Laser;
use crate::weapon::rail_gun::RailGun;
use crate::weapon::weapon::Weapon;
use crate::{entity::EntityUniform, input::Input};

use cgmath::Vector2;
//todo

pub struct Player {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub rotation: cgmath::Deg<f32>,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub active_weapon: Box<dyn Weapon>,
}

const SPEED: f32 = 500.0;

impl Entity for Player {
    fn update(
        &mut self,
        dt: &instant::Duration,
        input: &Input,
        audio: &mut Audio,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        time: f64,
    ) {
        if input.is_pressed("d") {
            self.movement("d", dt);
        } else if input.is_pressed("a") {
            self.movement("a", dt);
        } else if input.is_pressed("s") {
            self.movement("s", dt);
        } else if input.is_pressed("w") {
            self.movement("w", dt);
        }
        self.active_weapon.shoot(
            device,
            (self.position.x + self.scale.x / 3.0, self.position.y).into(),
            (40.0, 40.0).into(),
            CompassDir::from_deg(self.rotation.0),
            input,
            audio,
        );
        self.active_weapon.update(self.position, queue, dt, time);

        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
    }
}

impl Player {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        uniform: uniform::Uniform<EntityUniform>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            position,
            scale,
            alive: true,
            rotation: cgmath::Deg(360.0),
            uniform,
            active_weapon: Laser::new(device, queue),
        }
    }

    pub fn movement(&mut self, key: &str, dt: &instant::Duration) {
        let dt = dt.as_secs_f32();
        let mut position = Vector2::new(0.0, 0.0);
        if key == "d" {
            position.x += SPEED * dt;
        }
        if key == "a" {
            position.x -= SPEED * dt;
        }
        if key == "w" {
            position.y += SPEED * dt;
        }
        if key == "s" {
            position.y -= SPEED * dt;
        }
        self.position += position;
    }
}
