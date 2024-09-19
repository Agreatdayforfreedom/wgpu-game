use cgmath::{Angle, Point2, Vector2, Vector3};
use rand::{self, Rng};
use winit::{event::WindowEvent, keyboard::Key};

use crate::{
    audio::{
        Audio::{self},
        Sounds,
    },
    background::Background,
    camera::{Camera, CameraUniform},
    collider::{check_collision, Bounds},
    enemie::Enemy,
    entity::{Entity, EntityUniform},
    explosion::Explosion,
    input::Input,
    particle_system::system::ParticleSystem,
    player::Player,
    rendering::{create_bind_group_layout, create_render_pipeline, Sprite},
    uniform::Uniform,
    weapon::projectile::Projectile,
};
fn distance(a: cgmath::Vector2<f32>, b: cgmath::Vector2<f32>) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
pub struct GameState {
    render_pipeline: wgpu::RenderPipeline,
    final_pipeline: wgpu::RenderPipeline,

    player: Player,
    background: Background,

    sprite: Sprite,
    enemie_sprites: Vec<Sprite>,
    projectile_sprite: Sprite,
    enemy_projectile_sprite: Sprite, // the same for all

    enemies: Vec<Enemy>,
    projectile: Vec<Projectile>,
    explosions: Vec<Explosion>,
    entities: Vec<Box<dyn Entity>>,

    input_controller: Input,
    camera: Camera,
    audio: Audio,
    dt: instant::Duration,
    particle_system: ParticleSystem,
    render_target_texture: Sprite,
    // post_processing: PostProcessing,
    time: f64,
}

impl GameState {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/sprite.wgsl"));

        let camera_uniform = Uniform::<CameraUniform>::new(&device);
        let camera = Camera::new(camera_uniform);
        //BG
        let bind_group_layout = create_bind_group_layout(&device);

        let background = Background::new(&device, &queue);
        //PLAYER

        let diffuse_bytes = include_bytes!("../assets/spaceship.png");
        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );
        let player_uniform = Uniform::<EntityUniform>::new(&device);
        let player = Player::new(
            cgmath::Vector2::new(0.0, 0.0),
            (32.0, 32.0).into(),
            player_uniform,
            &device,
            &queue,
        );
        //ENEMIES
        let mut entities: Vec<Box<dyn Entity>> = vec![];
        let mut enemie_sprites = Vec::<Sprite>::new();
        let mut enemies = Vec::<Enemy>::new();

        let enemie_bytes = include_bytes!("../assets/alien1.png");
        let enemie_sprite = Sprite::new(
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

        let diffuse_bytes = include_bytes!("../assets/bullet.png");
        let projectile_sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );

        let diffuse_bytes = include_bytes!("../assets/alien_bullet.png");
        let enemy_projectile_sprite = Sprite::new(
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
        let particle_bytes = include_bytes!("../assets/spaceship.png");
        let particle_sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            particle_bytes,
        );

        let offscreen_texture = Sprite::from_empty(
            &device,
            (config.width, config.height),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );

        let particle_system = ParticleSystem::new(&device, config.format, &camera);

        let render_pipeline = create_render_pipeline(&device, &shader, &config, &pipeline_layout);
        let render_target_texture = Sprite::from_empty(
            &device,
            (800, 600),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );
        let final_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
            });
        let blend_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/blend.wgsl"));
        let shader_fullscreen_quad = device.create_shader_module(wgpu::include_wgsl!(
            "../shaders/fullscreen_quad_vertex.wgsl"
        ));

        let final_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Final pipeline"),
            layout: Some(&final_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_fullscreen_quad,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blend_shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        //audio
        let audio = Audio::new();
        audio.start_track(Sounds::MainTheme);

        Self {
            render_pipeline,
            enemie_sprites,
            enemies,
            enemy_projectile_sprite,

            sprite,
            camera,

            player,
            background,

            projectile: vec![],
            explosions: vec![],
            entities,

            projectile_sprite,

            input_controller,
            audio,
            dt: instant::Instant::now().elapsed(),
            time: 0.0,

            particle_system,
            render_target_texture,
            final_pipeline,
        }
    }

    pub fn update(
        &mut self,
        queue: &mut wgpu::Queue,
        device: &wgpu::Device,
        dt: instant::Duration,
    ) {
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

        self.background.uniform.write(queue);
        self.player.uniform.write(queue);

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
                    device,
                    queue,
                    self.time,
                );
            }
        }

        self.player.update(
            &dt,
            &self.input_controller,
            &mut self.audio,
            device,
            queue,
            self.time,
        );
        self.camera.uniform.write(queue);
        for e in &mut self.enemies {
            if rand::thread_rng().gen_range(0..10000) < 1 {
                e.spawn_fire((40.0, 40.0).into(), &mut self.audio, device);
            }
            for p in &mut e.projectiles {
                if p.alive {
                    // p.update(&dt, 275.0, self.player.position, ":D");
                    p.uniform.write(queue);
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
                    //     device,
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
            e.uniform.write(queue);
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
        //     queue,
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

    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        // let view = if let Some(texture) = &self.destination_texture {
        //     texture
        // } else {
        //     &frame
        //         .texture
        //         .create_view(&wgpu::TextureViewDescriptor::default())
        // };

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target_texture.texture.view,
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

            rpass.set_bind_group(1, &self.camera.uniform.bind_group, &[]);

            self.background.draw(&mut rpass);
            self.player.draw(&mut rpass);

            //todo enemies
            rpass.set_vertex_buffer(0, self.enemie_sprites[0].buffer.slice(..));
            rpass.set_bind_group(0, &self.enemie_sprites[0].bind_group, &[]);
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

            for e in &mut self.explosions {
                e.draw(&mut rpass);
            }
        }

        self.particle_system.render(
            queue,
            &mut encoder,
            &frame.texture,
            &self.camera,
            &self.player.position,
            &self.dt,
        );

        self.particle_system.blend(
            &mut encoder,
            &self.render_target_texture,
            &context_view,
            &self.final_pipeline,
        );

        queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
