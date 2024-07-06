use crate::audio;
use crate::audio::Audio;
use crate::projectile;
use crate::uniform;
use crate::util::CompassDir;
use crate::weapon::cannon::Cannon;
use crate::weapon::weapon::Weapon;
use crate::{entity::EntityUniform, input::Input};

use cgmath::Angle;
use cgmath::Vector2;
//todo
// pub enum Weapon {
//     Cannon,
//     RailGun,
// }

// impl Weapon {
// pub fn fire(
//     &self,
//     device: &wgpu::Device,
//     scale: cgmath::Vector2<f32>,
//     position: cgmath::Vector2<f32>,
// ) {
// }

//todo
// fn cannon_fire(
//     &self,
//     device: &wgpu::Device,
//     scale: cgmath::Vector2<f32>,
//     position: cgmath::Vector2<f32>,
//     // input: &Input,
//     // audio: &mut Audio,
// ) -> projectile::Projectile {
//     let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
//     return projectile::Projectile::new(
//         (position.x - 2.0, position.y).into(),
//         scale,
//         projectile_uniform,
//     );
// }
// }

pub struct Player {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub active_weapon: Box<dyn Weapon>,
    interval: instant::Instant,
}

const SPEED: f32 = 500.0;
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
            uniform,
            active_weapon: Cannon::new(500, device, queue),
            interval: instant::Instant::now(),
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, input: &Input) {
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
            .set_position(self.position)
            .set_scale(self.scale)
            .exec();
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
            position.y -= SPEED * dt;
        }
        if key == "s" {
            position.y += SPEED * dt;
        }
        self.position += position;
    }

    pub fn spawn_fire(
        &mut self,
        device: &wgpu::Device,
        scale: cgmath::Vector2<f32>,
        input: &Input,
        audio: &mut Audio,
    ) -> Vec<Option<projectile::Projectile>> {
        if input.is_pressed("f") && self.interval.elapsed().as_millis() >= 500 {
            self.interval = instant::Instant::now();
            let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            audio.push(audio::Sounds::Shoot);
            return vec![Some(projectile::Projectile::new(
                (self.position.x - 2.0, self.position.y).into(),
                scale,
                cgmath::Deg(-90.0),
                CompassDir::from_deg(0.0),
                projectile_uniform,
            ))];
        };
        vec![None]
    }

    pub fn spawn_rail_gun(
        &mut self,
        device: &wgpu::Device,
        scale: cgmath::Vector2<f32>,
        input: &Input,
        audio: &mut Audio,
    ) -> Vec<Option<projectile::Projectile>> {
        if input.is_pressed("c") && self.interval.elapsed().as_millis() >= 25 {
            self.interval = instant::Instant::now();
            let mut vec = vec![];

            for i in -2..=2 {
                let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
                // audio.push(audio::Sounds::Shoot);
                println!("{:?}", cgmath::Deg(45.0).sin());
                vec.push(Some(projectile::Projectile::new(
                    ((self.position.x - 2.0) + i as f32 * 5.0, self.position.y).into(),
                    scale,
                    cgmath::Deg(-90.0 + (i as f32 * 7.5)),
                    if i == -2 {
                        CompassDir::from_deg(110.0)
                    } else if i == -1 {
                        CompassDir::from_deg(100.0)
                    } else if i == 0 {
                        CompassDir::from_deg(90.0)
                    } else if i == 1 {
                        CompassDir::from_deg(80.0)
                    } else {
                        CompassDir::from_deg(70.0)
                    },
                    projectile_uniform,
                )));
            }
            return vec;
        };
        vec![None]
    }
}
