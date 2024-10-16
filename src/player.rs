use crate::audio::Audio;
use crate::entity::Entity;
use crate::rendering::{create_bind_group_layout, Sprite};
use crate::uniform::{self, Uniform};
use crate::util::CompassDir;
use crate::weapon;
// use crate::weapon::cannon::Cannon;
// use crate::weapon::double_connon::DoubleCannon;
use crate::weapon::homing_missile::HomingMissile;
// use crate::weapon::laser::Laser;
// use crate::weapon::rail_gun::RailGun;
use crate::weapon::weapon::Weapon;
use crate::{entity::EntityUniform, input::Input};

use cgmath::{Angle, Point2, Vector2, Vector4};

pub struct Player {
    id: u32,
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub rotation: cgmath::Deg<f32>,
    pub uniform: uniform::Uniform<EntityUniform>,
    sprite: Sprite,
    pub active_weapons: Vec<Box<dyn Weapon>>,
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

        let positions = [
            self.get_orientation_point((8.0, self.bottom_left().y).into()),
            self.get_orientation_point((-10.0, self.bottom_right().y).into()),
        ];

        let mut i = 0usize;
        // println!("{}, {:?}", self.active_weapons.len(), positions);
        while i < self.active_weapons.len() {
            let weapon = self.active_weapons.get_mut(i).unwrap();
            let position = positions[i];
            weapon.update(position, queue, dt);
            weapon.shoot(
                device,
                position,
                CompassDir::from_deg(self.rotation.0),
                input,
                audio,
            );
            i += 1;
        }
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
        self.uniform.write(queue);
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn scale(&self) -> Vector2<f32> {
        self.scale
    }
    fn rotation(&self) -> cgmath::Deg<f32> {
        self.rotation
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        // also draw the weapon :P
        for weapon in &mut self.active_weapons {
            weapon.draw(rpass);
        }

        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        self.sprite.bind(rpass);
        rpass.draw(0..6, 0..1);
    }
}

impl Player {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, id: u32) -> Self {
        let mut uniform = Uniform::<EntityUniform>::new(&device);

        let diffuse_bytes = include_bytes!("./assets/spaceship.png");
        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &create_bind_group_layout(&device),
            diffuse_bytes,
        );

        let scale: cgmath::Vector2<f32> = (44.0, 33.0).into();
        let position = cgmath::Vector2::new(0.0, 0.0);

        Self {
            id,
            position,
            scale,
            alive: true,
            rotation: cgmath::Deg(360.0),
            uniform,
            sprite,
            active_weapons: vec![
                HomingMissile::new(100, false, device, queue),
                // DoubleCannon::new(100, false, device, queue),
            ],
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
