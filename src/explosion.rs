use cgmath::{Vector2, Vector4};

use crate::entity::EntityUniform;
use crate::particle_system::simulation_params::{Circle, SimulationParams};
use crate::particle_system::system::ParticleSystem;
use crate::rendering::{create_bind_group_layout, Sprite};
use crate::uniform;
use crate::uniform::Uniform;
use crate::{particle_system, rendering};
const TIME_TO_NEXT_FRAME: f32 = 2.0 / 30.0;

pub enum ExplosionType {
    Fire,
    Particles,
}

pub struct Explosion {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    start: bool,
    pub end: bool,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub i: u32,
    time_to_next_frame: f32,
}

impl Explosion {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        device: &wgpu::Device,
    ) -> Self {
        let uniform = Uniform::<EntityUniform>::new(device);

        Self {
            position,
            scale,
            uniform,
            i: 0,
            time_to_next_frame: 0.0,
            // sprites,
            end: false,
            start: true,
        }
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {
        self.uniform
            .data
            .set_position(self.position)
            .set_scale(self.scale)
            .exec();
        self.time_to_next_frame += dt.as_secs_f32();

        if self.start {
            self.start = false;
            // audio.push(Sounds::Explosion, 1.0);
        }

        if self.time_to_next_frame > TIME_TO_NEXT_FRAME {
            if self.i == 6 {
                self.end = true;
            } else {
                self.i += 1;
                self.time_to_next_frame = 0.0;
            }
        }

        self.uniform.write(queue);
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if !self.end {
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}

pub struct ExpansiveWave {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    uniform: Uniform<EntityUniform>,
    color: Vector4<f32>,
    end: bool,
}

impl ExpansiveWave {
    pub fn new_at(position: Vector2<f32>, device: &wgpu::Device) -> Self {
        let initial_scale = Vector2::new(50.0, 50.0);
        let mut uniform = Uniform::<EntityUniform>::new(device);
        uniform.data.set_position(position).exec();
        Self {
            scale: initial_scale,
            position,
            uniform,
            color: Vector4::new(0.0, 1.0, 1.0, 1.0),
            end: false,
        }
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {
        let dt = dt.as_secs_f32();

        if self.scale.x >= 175.0 {
            self.color.w -= 2.5 * dt;
            if self.color.w < 0.0 {
                self.end = true;
            }
        } else {
            self.scale.x += 200.0 * dt;
            self.scale.y += 200.0 * dt;
        }

        self.uniform
            .data
            .set_scale(self.scale)
            .set_color(self.color)
            .exec();

        self.uniform.write(queue);
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}

pub struct ExplosionManager {
    sprites: Vec<Sprite>,
    explosions: Vec<Explosion>,
    waves: (Sprite, Vec<ExpansiveWave>),
}

impl ExplosionManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bind_group_layout = create_bind_group_layout(device);

        let diffuse_bytes1 = include_bytes!("./assets/bexp1.png");
        let diffuse_bytes2 = include_bytes!("./assets/bexp2.png");
        let diffuse_bytes3 = include_bytes!("./assets/bexp3.png");
        let diffuse_bytes4 = include_bytes!("./assets/bexp4.png");
        let diffuse_bytes5 = include_bytes!("./assets/bexp5.png");
        let diffuse_bytes6 = include_bytes!("./assets/bexp6.png");
        let diffuse_bytes7 = include_bytes!("./assets/bexp7.png");

        let sprites = vec![
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes1,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes2,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes3,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes4,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes5,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes6,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes7,
            ),
        ];

        let bytes = include_bytes!("./assets/expansive_wave.png");

        let wave_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            bytes,
        );

        let waves = (wave_sprite, vec![]);
        Self {
            sprites,
            explosions: vec![],
            waves,
        }
    }

    pub fn add(
        &mut self,
        id: u32,
        explosios_type: ExplosionType,
        position: Vector2<f32>,
        particle_system: &mut ParticleSystem,
        device: &wgpu::Device,
    ) {
        match explosios_type {
            ExplosionType::Fire => {
                let explosion = Explosion::new(position, (40.0, 40.0).into(), device);
                let wave = Some(ExpansiveWave::new_at(position, device));
                if let Some(wave) = wave {
                    self.waves.1.push(wave);
                }
                self.explosions.push(explosion);
            }
            ExplosionType::Particles => {
                particle_system.push_group(
                    id,
                    device,
                    SimulationParams {
                        total: 100.0,
                        color: (1.0, 0.466, 0.0, 1.0).into(),
                        position: position,
                        infinite: 0,
                        start_speed: 10.0,
                        circle: Circle {
                            radius: 3.0,
                            emit_from_edge: 0,
                        },
                        ..Default::default()
                    },
                );
            }

            _ => particle_system.push_group(
                id,
                device,
                SimulationParams {
                    total: 100.0,
                    color: (1.0, 0.466, 0.0, 1.0).into(),
                    position: position,
                    infinite: 0,
                    start_speed: 10.0,
                    circle: Circle {
                        radius: 3.0,
                        emit_from_edge: 0,
                    },
                    ..Default::default()
                },
            ),
        }
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {
        let mut i = 0usize;
        while i < self.explosions.len() {
            let explosion = self.explosions.get_mut(i).unwrap();
            if !explosion.end {
                explosion.update(queue, dt);

                i += 1;
            } else {
                self.explosions.swap_remove(i);
            }
        }

        let mut i = 0usize;
        while i < self.waves.1.len() {
            let wave = self.waves.1.get_mut(i).unwrap();
            if !wave.end {
                wave.update(queue, dt);
                i += 1;
            } else {
                self.waves.1.swap_remove(i);
            }
        }
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        for explosion in &mut self.explosions {
            let sprite = self.sprites.get(explosion.i as usize).unwrap();
            sprite.bind(rpass);
            explosion.draw(rpass);
        }
        self.waves.0.bind(rpass);
        for wave in &mut self.waves.1 {
            wave.draw(rpass);
        }
    }
}
