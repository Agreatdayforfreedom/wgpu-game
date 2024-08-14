use crate::audio::Audio;
use crate::entity::Entity;
use crate::uniform::{self, Uniform};
use crate::util::CompassDir;
use crate::weapon::cannon::Cannon;
use crate::weapon::laser::Laser;
use crate::weapon::rail_gun::RailGun;
use crate::weapon::weapon::Weapon;
use crate::{entity::EntityUniform, input::Input};

use cgmath::{Angle, Point2, Vector2, Vector4};
//todo

pub struct Player {
    id: u32,
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
        self.uniform
            .data
            .set_pivot(Point2::new(self.scale.x * 0.5, self.scale.y * 0.5));

        let center = Vector2::new(
            self.position.x + (self.scale.x / 2.0) - 20.0,
            self.position.y + (self.scale.y / 2.0) - 20.0,
        );
        self.active_weapon.shoot(
            device,
            center,
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

    fn rotate(&mut self, rotation: cgmath::Deg<f32>) {
        self.rotation = rotation;
    }
    fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn scale(&self) -> Vector2<f32> {
        self.scale
    }
    fn set_colors(&mut self, color: Vector4<f32>) {
        self.uniform.data.set_color(color);
    }
    fn id(&self) -> u32 {
        self.id
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
            id: 100,
            position,
            scale,
            alive: true,
            rotation: cgmath::Deg(360.0),
            uniform,
            active_weapon: Laser::new(device, queue),
        }
    }

    // pub fn dir(&mut self) {
    //     {
    //         //     //todo: set origin correctly
    //         if self.player.active_weapon.get_name() == "laser" {
    //             for p in self.player.active_weapon.get_projectiles() {
    //                 p.set_direction(|this| {
    //                     this.rotation = cgmath::Deg(angle + 90.0);
    //                     this.position = (
    //                         self.player.position.x,
    //                         self.player.position.y - min_dist + self.player.scale.y,
    //                     )
    //                         .into();
    //                     this.scale.y = min_dist;
    //                 });
    //             }
    //         }
    //     }
    // }

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
