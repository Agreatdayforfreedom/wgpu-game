use crate::audio::Audio;
use crate::entity::Entity;
use crate::particle_system::simulation_params::{Circle, SimulationParams};
use crate::particle_system::system::ParticleSystem;
use crate::rendering::{create_bind_group_layout, Sprite};
use crate::uniform::{self, Uniform};
use crate::util::{CompassDir, IdVendor};
use crate::weapon;
use crate::weapon::double_connon::DoubleCannon;
// use crate::weapon::cannon::Cannon;
// use crate::weapon::double_connon::DoubleCannon;
use crate::weapon::homing_missile::HomingMissile;
// use crate::weapon::laser::Laser;
use crate::weapon::rail_gun::RailGun;
use crate::weapon::weapon::Weapon;
use crate::{entity::EntityUniform, input::Input};

use cgmath::{Angle, Array, Point2, Vector2, Vector4};

const SPEED: f32 = 500.0;
const EVASION_COOLDOWN: u128 = 5000;
const EVASION_TIME: u128 = 2500;

pub struct Player {
    id: u32,
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    color: Vector4<f32>,
    pub alive: bool,
    pub velocity: f32,
    pub rotation: cgmath::Deg<f32>,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub evading: bool,
    sprite: Sprite,
    pub active_weapons: Vec<Box<dyn Weapon>>,

    pub evasion_cd_timer: instant::Instant,
    pub evading_timer: instant::Instant,
}

impl Entity for Player {
    fn update(
        &mut self,
        dt: &instant::Duration,
        input: &Input,
        audio: &mut Audio,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
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

        if input.is_pressed("v") && self.evasion_cd_timer.elapsed().as_millis() >= EVASION_COOLDOWN
        {
            particle_system.push_group(
                self.id + 3,
                device,
                SimulationParams {
                    total: 100.0,
                    color: (1.0, 0.0, 0.0, 1.0).into(),
                    position: self.position,
                    infinite: 1,
                    rate_over_distance: 7.0,
                    start_speed: 30.0,
                    lifetime_factor: 1.0,
                    shape_selected: 0,
                    circle: Circle {
                        radius: 3.0,
                        emit_from_edge: 0,
                    },
                    ..Default::default()
                },
            );
            self.color = Vector4::new(1.0, 0.0, 0.0, 1.0);
            self.evasion_cd_timer = instant::Instant::now();
            self.velocity = SPEED + 200.0;
            self.evading = true;
        }

        if self.evading {
            particle_system.update_sim_params(self.id + 3, self.position, 1);
        }

        if self.evading_timer.elapsed().as_millis() >= EVASION_TIME {
            particle_system.update_sim_params(self.id + 3, self.position, 0);

            self.evading_timer = instant::Instant::now();
            self.velocity = SPEED;
            self.color = Vector4::from_value(1.0);

            self.evading = false;
        }

        let double_cannon_positions = [
            self.get_orientation_point((8.0, self.bottom_left().y).into()),
            self.get_orientation_point((-10.0, self.bottom_right().y).into()),
        ]
        .to_vec();

        let mut i = 0usize;

        while i < self.active_weapons.len() {
            let weapon = self.active_weapons.get_mut(i).unwrap();
            weapon.update(self.position, queue, dt, particle_system);
            weapon.shoot(
                device,
                vec![self.position],
                // positions.clone(),
                CompassDir::from_deg(self.rotation.0),
                input,
                audio,
                id_vendor,
                particle_system,
            );
            i += 1;
        }
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_color(self.color)
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
        let uniform = Uniform::<EntityUniform>::new(&device);

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
            color: Vector4::from_value(1.0),
            alive: true,
            rotation: cgmath::Deg(360.0),
            uniform,
            sprite,
            velocity: SPEED,
            evading: false,
            evasion_cd_timer: instant::Instant::now(),
            evading_timer: instant::Instant::now(),
            active_weapons: vec![
                // HomingMissile::new(100, false, device, queue),
                // DoubleCannon::new(100, false, device, queue),
                RailGun::new(1000, device, queue),
            ],
        }
    }

    pub fn movement(&mut self, key: &str, dt: &instant::Duration) {
        let dt = dt.as_secs_f32();
        let mut position = Vector2::new(0.0, 0.0);
        if key == "d" {
            position.x += self.velocity * dt;
        }
        if key == "a" {
            position.x -= self.velocity * dt;
        }
        if key == "w" {
            position.y += self.velocity * dt;
        }
        if key == "s" {
            position.y -= self.velocity * dt;
        }
        self.position += position;
    }
}
