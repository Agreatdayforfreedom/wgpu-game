use crate::collider::{check_collision, check_collision_ep};
use crate::enemie::{self, Enemy, EnemyUniform};
use crate::input::Input;
use crate::sprite_renderer::create_render_pipeline;
use crate::uniform::Uniform;
use crate::{camera::Camera, player::PlayerUniform};
use crate::{player, projectile, sprite_renderer};
use rand::{self, Rng};

use pollster::block_on;
use std::sync::Arc;
use winit::{event::*, keyboard::Key, window::Window};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,

    // vertex_buffer: wgpu::Buffer,
    // diffuse_bind_group: wgpu::BindGroup,
    sprite: sprite_renderer::SpriteRenderer,
    enemie_sprites: Vec<sprite_renderer::SpriteRenderer>,
    projectile_sprite: sprite_renderer::SpriteRenderer,
    enemy_projectile_sprite: sprite_renderer::SpriteRenderer, // the same for all
    //uniforms
    enemies: Vec<Enemy>,
    camera_uniform: Uniform<Camera>,
    player_uniform: Uniform<PlayerUniform>,

    player: player::Player,
    projectile: Vec<projectile::Projectile>,

    input_controller: Input,
    dt: instant::Duration,
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

        let diffuse_bytes = include_bytes!("./assets/spaceship.png");
        let sprite = sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes);
        let camera_uniform = Uniform::<Camera>::new(&device);

        let player = player::Player::new(cgmath::Vector2::new(400.0, 550.0));
        let player_uniform = Uniform::<PlayerUniform>::new(&device);

        let diffuse_bytes = include_bytes!("./assets/bullet.png");
        let projectile_sprite =
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes);
        //ENEMIES
        let mut enemie_sprites = Vec::<sprite_renderer::SpriteRenderer>::new();
        let mut enemies = Vec::<Enemy>::new();
        let diffuse_bytes = include_bytes!("./assets/alien_bullet.png");
        let enemy_projectile_sprite =
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes);
        let enemie_bytes = include_bytes!("./assets/alien1.png");
        let enemie_sprite = sprite_renderer::SpriteRenderer::new(&device, &queue, enemie_bytes);
        enemie_sprites.push(enemie_sprite);

        for i in 0..(config.width / 36) {
            for j in 0..(config.height / 80) {
                let position = ((i as f32 + 1.0) * 40.0, (j as f32 + 1.0) * 25.0);
                let mut uniform = Uniform::<EnemyUniform>::new(&device);
                uniform.data.set_position(position.into());

                let enemie = Enemy::new(position.into(), uniform);
                enemies.push(enemie);
            }
        }

        let input_controller = Input::new();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &sprite.bind_group_layout,
                &camera_uniform.bind_group_layout,
                &player_uniform.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(&device, &shader, &config, &pipeline_layout);

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
            camera_uniform,
            player_uniform,
            player,

            projectile: vec![],

            projectile_sprite,

            input_controller,
            dt: instant::Instant::now().elapsed(),
        }
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.dt = dt;
        //todo
        if !self.player.alive {
            println!("YOU LOST!!!");
            return;
        }
        println!("FPS: {}", 1.0 / dt.as_secs_f64());
        self.player_uniform
            .data
            .update(&mut self.player, &dt, &self.input_controller);
        self.player_uniform.write(&mut self.queue);

        for e in &self.enemies {
            if e.alive {
                e.uniform.write(&mut self.queue);
            }
        }

        let new_projectile = self.player.spawn_fire(&self.device, &self.input_controller);

        if let Some(projectile) = new_projectile {
            self.projectile.push(projectile);
        }

        for p in &mut self.projectile {
            if p.alive {
                p.update(&dt, 1.0);
                p.uniform.write(&mut self.queue);
            }
        }

        for e in &mut self.enemies {
            if rand::thread_rng().gen_range(0..10000) < 1 {
                e.spawn_fire(&self.device);
            }
            for p in &mut e.projectiles {
                if p.alive {
                    p.update(&dt, -1.0);
                    p.uniform.write(&mut self.queue);
                }
            }

            e.projectiles = e
                .projectiles
                .drain(..)
                .filter(|p| p.alive != false)
                .collect();
        }

        //check collsions
        for p in &mut self.projectile {
            for e in &mut self.enemies {
                if check_collision(p, e) {
                    p.alive = false;
                    e.alive = false;
                }
            }
        }

        for e in &mut self.enemies {
            for p in &mut e.projectiles {
                if check_collision_ep(p, &self.player) {
                    p.alive = false;
                    self.player.alive = false;
                }
            }
        }

        self.projectile = self
            .projectile
            .drain(..)
            .filter(|p| p.alive != false)
            .collect();

        self.enemies = self
            .enemies
            .drain(..)
            .filter(|e| e.alive != false)
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

    pub fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
            rpass.set_bind_group(1, &self.camera_uniform.bind_group, &[]);
            rpass.set_bind_group(2, &self.player_uniform.bind_group, &[]);
            //buffers

            rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
            rpass.set_vertex_buffer(1, self.camera_uniform.buffer.slice(..));
            rpass.set_vertex_buffer(2, self.player_uniform.buffer.slice(..));

            rpass.draw(0..6, 0..1);

            rpass.set_vertex_buffer(0, self.enemie_sprites[0].buffer.slice(..));
            rpass.set_bind_group(0, &self.enemie_sprites[0].bind_group, &[]);
            //bind_groups
            for e in &self.enemies {
                if e.alive {
                    rpass.set_vertex_buffer(2, e.uniform.buffer.slice(..));
                    rpass.set_bind_group(2, &e.uniform.bind_group, &[]);
                    rpass.draw(0..6, 0..1);
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

            // draw player projectiles
            rpass.set_vertex_buffer(0, self.projectile_sprite.buffer.slice(..));
            rpass.set_bind_group(0, &self.projectile_sprite.bind_group, &[]);
            for p in &self.projectile {
                if p.alive {
                    rpass.set_vertex_buffer(2, p.uniform.buffer.slice(..));
                    rpass.set_bind_group(2, &p.uniform.bind_group, &[]);
                    rpass.draw(0..6, 0..1);
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
