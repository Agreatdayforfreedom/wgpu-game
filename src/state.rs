use crate::audio::{Audio, Sounds};
use crate::camera::{Camera, CameraUniform};
use crate::collider::{check_collision, Bounds};
use crate::enemie::Enemy;
use crate::entity::{self, Entity, EntityUniform};
use crate::explosion::{self, Explosion};
use crate::input::Input;
use crate::particle_system::particle::Particle;
use crate::particle_system::system::ParticleSystem;
use crate::post_processing::{self, PostProcessing};
use crate::rendering::{create_bind_group_layout, create_render_pipeline};
use crate::uniform::Uniform;
use crate::util::CompassDir;
use crate::weapon::projectile;
use crate::{player, rendering, texture};
use cgmath::{
    Angle, InnerSpace, Matrix3, Point2, Quaternion, Rad, Rotation3, Transform, Vector2, Vector3,
    Vector4,
};
use rand::{self, Rng};

use pollster::block_on;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::sync::Arc;
use winit::{event::*, keyboard::Key, window::Window};

#[allow(dead_code)]
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,

    sprite: rendering::Sprite,
    enemie_sprites: Vec<rendering::Sprite>,
    projectile_sprite: rendering::Sprite,
    enemy_projectile_sprite: rendering::Sprite, // the same for all
    bg_sprite: rendering::Sprite,

    bg_uniform: Uniform<EntityUniform>,
    enemies: Vec<Enemy>,
    player: player::Player,
    projectile: Vec<projectile::Projectile>,
    explosions: Vec<explosion::Explosion>,
    entities: Vec<Box<dyn Entity>>,

    input_controller: Input,
    camera: Camera,
    audio: Audio,
    dt: instant::Duration,
    particle_system: ParticleSystem,
    post_processing: PostProcessing,
    time: f64,
}
//todo
fn distance(a: cgmath::Vector2<f32>, b: cgmath::Vector2<f32>) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
impl State {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        println!("w:{}, h: {}", size.width, size.height);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();
        println!("{:?}", adapter.features());
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader_particles =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/particles.wgsl"));

        let camera_uniform = Uniform::<CameraUniform>::new(&device);
        let camera = Camera::new(camera_uniform);
        //BG
        let bind_group_layout = create_bind_group_layout(&device);
        let diffuse_bytes = include_bytes!("./assets/bg.png");
        let bg_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );
        let mut bg_uniform = Uniform::<EntityUniform>::new(&device);
        bg_uniform
            .data
            // .set_tex_scale((8.0, 8.0).into())
            .set_scale((856.0 * 2.0, 375.0 * 2.0).into())
            .exec();

        //PLAYER

        let diffuse_bytes = include_bytes!("./assets/spaceship.png");
        let sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );
        let player_uniform = Uniform::<EntityUniform>::new(&device);
        let player = player::Player::new(
            cgmath::Vector2::new(0.0, 0.0),
            (32.0, 32.0).into(),
            player_uniform,
            &device,
            &queue,
        );
        //ENEMIES
        let mut entities: Vec<Box<dyn Entity>> = vec![];
        let mut enemie_sprites = Vec::<rendering::Sprite>::new();
        let mut enemies = Vec::<Enemy>::new();

        let enemie_bytes = include_bytes!("./assets/alien1.png");
        let enemie_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            enemie_bytes,
        );
        enemie_sprites.push(enemie_sprite);
        for i in 0..2 {
            // let position = ((i as f32 + 1.0) * 40.0, (j as f32 + 1.0) * 25.0);
            let position = (0.0 * 0 as f32, 300.0 * i as f32);
            let uniform = Uniform::<EntityUniform>::new(&device);

            let mut enemy = Box::new(Enemy::new(
                position.into(),
                (24.0, 24.0).into(),
                uniform,
                100 + i + 1,
            ));
            enemy
                .uniform
                .data
                .set_position(position.into())
                .set_scale((24.0, 24.0).into())
                .set_color((0.0, 1.0, 0.0, 1.0).into())
                .exec();
            entities.push(enemy);
        }
        //PROJECTILES

        let diffuse_bytes = include_bytes!("./assets/bullet.png");
        let projectile_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );

        let diffuse_bytes = include_bytes!("./assets/alien_bullet.png");
        let enemy_projectile_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );

        let input_controller = Input::new();
        // let bind_groups_layouts = %
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bind_group_layout,
                &player.uniform.bind_group_layout,
                &camera.uniform.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        ////! PARTICLES
        let particle_bytes = include_bytes!("./assets/spaceship.png");
        let particle_sprite = rendering::Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            particle_bytes,
        );

        // let mut particles: Vec<Particle> = vec![];
        // let center = Vector2::new(
        //     player.position.x + (player.scale.x / 2.0),
        //     player.position.y + (player.scale.y / 2.0),
        // );
        // for n in 0..=100 {
        //     let uniform = Uniform::<EntityUniform>::new(&device);
        //     let particle = Particle::new(
        //         center,
        //         (8.0, 8.0).into(),
        //         (0.0, 1.0, 1.0, 1.0).into(),
        //         100.0,
        //         1.0,
        //         uniform,
        //     );

        //     particles.push(particle);
        // }

        let offscreen_texture = rendering::Sprite::from_empty(
            &device,
            (config.width, config.height),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );
        // let sprite = rendering::Sprite::new(
        //     &device,
        //     &queue,
        //     wgpu::AddressMode::ClampToEdge,
        //     &bind_group_layout,
        //     diffuse_bytes,
        // );

        // let render_pipeline = create_render_pipeline(
        //     &device,
        //     &device.create_shader_module(wgpu::include_wgsl!("./shaders/particles.wgsl")),
        //     &config,
        //     &pipeline_layout,
        // );

        let particle_system = ParticleSystem::new(
            &device,
            particle_sprite,
            config.format,
            &camera,
            &bind_group_layout, // render_pipeline,
        );

        let render_pipeline = create_render_pipeline(&device, &shader, &config, &pipeline_layout);
        // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: None,
        //     bind_group_layouts: &[&bind_group_layout],
        //     push_constant_ranges: &[],
        // });
        // let render_pipeline_particles =
        //     create_render_pipeline(&device, &shader_particles, &config, &pipeline_layout);
        let post_processing = PostProcessing::new(&device, &config);

        //audio
        let audio = Audio::new();
        audio.start_track(Sounds::MainTheme);

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            enemie_sprites,
            enemies,
            enemy_projectile_sprite,

            sprite,
            camera,
            player,

            bg_sprite,
            bg_uniform,

            projectile: vec![],
            explosions: vec![],
            entities,

            projectile_sprite,

            input_controller,
            audio,
            dt: instant::Instant::now().elapsed(),
            time: 0.0,

            particle_system,
            post_processing, // offscreen_texture,
        }
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.dt = dt;
        self.time += dt.as_secs_f64();

        self.audio.update();
        //todo
        if !self.player.alive {
            println!("YOU LOST!!!");
            return;
        }
        println!("FPS: {}", 1.0 / dt.as_secs_f64());

        self.camera.update(Vector3::new(
            self.player.position.x,
            self.player.position.y,
            1.0,
        ));
        self.bg_uniform.write(&mut self.queue);

        self.player.uniform.write(&mut self.queue);

        let mut min_dist = f32::MAX;
        for e in &mut self.entities {
            //todo:
            let dist = distance(self.player.position, e.position());

            if dist < min_dist {
                let dx = e.position().x - self.player.position.x;
                //set the point in the head
                let dy = e.position().y - (self.player.position.y - 0.5);

                let angle = dy.atan2(dx);

                let angle = angle * 180.0 / std::f32::consts::PI;
                self.player.rotation = cgmath::Deg(angle + 90.0); // adjust sprite rotation;

                min_dist = dist;

                e.update(
                    &dt,
                    &self.input_controller,
                    &mut self.audio,
                    &self.device,
                    &mut self.queue,
                    self.time,
                );
            }
        }

        self.player.update(
            &dt,
            &self.input_controller,
            &mut self.audio,
            &self.device,
            &mut self.queue,
            self.time,
        );
        self.camera.uniform.write(&mut self.queue);
        for e in &mut self.enemies {
            if rand::thread_rng().gen_range(0..10000) < 1 {
                e.spawn_fire((40.0, 40.0).into(), &mut self.audio, &self.device);
            }
            for p in &mut e.projectiles {
                if p.alive {
                    // p.update(&dt, 275.0, self.player.position, ":D");
                    p.uniform.write(&mut self.queue);
                }
            }

            e.projectiles = e
                .projectiles
                .drain(..)
                .filter(|p| p.alive != false)
                .collect();
        }
        if self.player.active_weapon.get_name() == "laser" {
            for p in self.player.active_weapon.get_projectiles() {
                let mut min_dist = f32::MAX;
                for e in &mut self.entities {
                    let dist = distance(self.player.position, e.position());
                    if dist < min_dist {
                        min_dist = dist;
                        p.set_direction(|this| {
                            let center = Vector2::new(
                                self.player.position.x + (self.player.scale.x / 2.0) - 10.0,
                                self.player.position.y + (self.player.scale.y / 2.0),
                            );

                            this.position.x =
                                center.x + self.player.scale.x / 2.0 * self.player.rotation.sin();
                            this.position.y =
                                center.y - self.player.scale.y / 2.0 * self.player.rotation.cos();
                            // Apply the rotation

                            this.rotation = self.player.rotation;
                            this.uniform
                                .data
                                .set_pivot(cgmath::Point2::new(0.5 * 20.0, 1.0))
                                .exec();
                            this.scale.x = 20.0;
                            this.scale.y = -min_dist;
                        });
                    }
                }
            }
        }
        //check collsions
        for p in &mut self.player.active_weapon.get_projectiles() {
            let mut min_dist = f32::MAX;
            for e in &mut self.entities {
                let dist = distance(self.player.position, e.position());
                if dist < min_dist {
                    min_dist = dist;
                }

                if check_collision(
                    p.bounds,
                    Bounds {
                        origin: Point2::new(e.position().x, e.position().y),
                        area: Vector2::new(e.scale().x, e.scale().y),
                    },
                ) {
                    // p.alive = false;
                    e.set_colors((1.0, 0.0, 0.0, 1.0).into());
                    // let explosion = Explosion::new(
                    //     e.position.into(),
                    //     (40.0, 40.0).into(),
                    //     &self.device,
                    //     &self.queue,
                    // );
                    // self.explosions.push(explosion);
                    // self.audio.push(Sounds::Explosion);
                    // e.alive = false;
                } else {
                    e.set_colors((0.0, 1.0, 0.0, 1.0).into());
                }
            }
        }

        for e in &mut self.explosions {
            e.update(&dt);
            e.uniform.write(&mut self.queue);
        }

        for e in &mut self.enemies {
            for p in &mut e.projectiles {
                if check_collision(
                    Bounds {
                        origin: Point2::new(
                            p.position.x + p.scale.x / 2.0,
                            p.position.y + p.scale.y / 2.0,
                        ),
                        area: Vector2::new(2.5, 2.5),
                    },
                    Bounds {
                        origin: Point2::new(self.player.position.x, self.player.position.y),
                        area: Vector2::new(self.player.scale.x, self.player.scale.y),
                    },
                ) {
                    p.alive = false;
                    self.player.alive = false;
                }
            }
        }

        self.player.active_weapon.drain();
        // self.particle_system.update(
        //     Vector2::new(
        //         self.player.position.x + (self.player.scale.x / 2.0) - 4.0,
        //         self.player.position.y + (self.player.scale.y / 2.0) - 4.0,
        //     ),
        //     CompassDir::from_deg(self.player.rotation.opposite().0),
        //     &mut self.device,
        //     &mut self.queue,
        //     &self.dt,
        // );

        self.enemies = self
            .enemies
            .drain(..)
            .filter(|e| e.alive != false)
            .collect();

        self.explosions = self
            .explosions
            .drain(..)
            .filter(|e| e.end != true)
            .collect();
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let action = if let Key::Character(ch) = event.logical_key.as_ref() {
                    Some(ch)
                } else {
                    None
                };
                if let Some(key) = action {
                    self.input_controller.update(key, event.state);
                }
                true
            }
            _ => false,
        }
    }

    // pub fn set_dest_texture(&mut self, texture: wgpu::TextureView) {
    //     self.destination_texture = Some(texture);
    // }

    pub fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        // let view = if let Some(texture) = &self.destination_texture {
        //     texture
        // } else {
        //     &frame
        //         .texture
        //         .create_view(&wgpu::TextureViewDescriptor::default())
        // };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            //pipeline
            rpass.set_pipeline(&self.render_pipeline);
            // self.set_dest_texture(
            //     self.particle_system
            //         .offscreen
            //         .texture
            //         .texture
            //         .create_view(&wgpu::TextureViewDescriptor::default()),
            // );

            rpass.set_bind_group(1, &self.camera.uniform.bind_group, &[]);
            // rpass.set_vertex_buffer(1, self.camera.uniform.buffer.slice(..));

            rpass.set_bind_group(0, &self.bg_sprite.bind_group, &[]);
            rpass.set_bind_group(2, &self.bg_uniform.bind_group, &[]);

            rpass.set_vertex_buffer(0, self.bg_sprite.buffer.slice(..));
            rpass.set_vertex_buffer(2, self.bg_uniform.buffer.slice(..));

            rpass.draw(0..6, 0..1);

            // self.particle_system.draw(&mut rpass);

            rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
            rpass.set_bind_group(2, &self.player.uniform.bind_group, &[]);
            //buffers

            rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
            rpass.set_vertex_buffer(2, self.player.uniform.buffer.slice(..));

            rpass.draw(0..6, 0..1);

            rpass.set_vertex_buffer(0, self.enemie_sprites[0].buffer.slice(..));
            rpass.set_bind_group(0, &self.enemie_sprites[0].bind_group, &[]);
            //bind_groups
            for e in &mut self.entities {
                if e.alive() {
                    e.draw(&mut rpass);
                }
            }

            //draw enemy projectiles
            rpass.set_vertex_buffer(0, self.enemy_projectile_sprite.buffer.slice(..));
            rpass.set_bind_group(0, &self.enemy_projectile_sprite.bind_group, &[]);
            for e in &self.enemies {
                for p in &e.projectiles {
                    rpass.set_vertex_buffer(2, p.uniform.buffer.slice(..));
                    rpass.set_bind_group(2, &p.uniform.bind_group, &[]);
                    rpass.draw(0..6, 0..1);
                }
            }

            for e in &self.explosions {
                //explosions
                if e.end {
                    continue;
                }
                rpass.set_vertex_buffer(2, e.uniform.buffer.slice(..));
                rpass.set_bind_group(2, &e.uniform.bind_group, &[]);

                rpass.set_vertex_buffer(0, e.sprites.get(e.i as usize).unwrap().buffer.slice(..));
                rpass.set_bind_group(0, &e.sprites.get(e.i as usize).unwrap().bind_group, &[]);
                rpass.draw(0..6, 0..1);
            }

            self.player.active_weapon.draw(&mut rpass);
        }

        self.particle_system.render(
            &self.device,
            &mut self.queue,
            &mut encoder,
            &frame.texture,
            &self.camera,
        );
        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
