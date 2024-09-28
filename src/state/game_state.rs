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
    entity::{Entity, EntityManager, EntityUniform},
    explosion::Explosion,
    input::Input,
    particle_system::system::ParticleSystem,
    player::Player,
    rendering::{create_bind_group_layout, create_render_pipeline, Sprite},
    uniform::Uniform,
    weapon::projectile::Projectile,
};

pub struct GameState {
    entity_manager: EntityManager,

    render_pipeline: wgpu::RenderPipeline,
    final_pipeline: wgpu::RenderPipeline,

    background: Box<Background>,

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
        queue: &mut wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let entity_manager = EntityManager::new(&device, &queue);

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/sprite.wgsl"));

        let camera_uniform = Uniform::<CameraUniform>::new(&device);
        let camera = Camera::new(camera_uniform);
        //BG
        let bind_group_layout = create_bind_group_layout(&device);

        let background = Background::new(&device, &queue);
        let player = Player::new(&device, &queue);
        background.uniform.write(queue);

        // entity_manager.add(None, vec![background]);
        //ENEMIES
        let mut entities: Vec<Box<dyn Entity>> = vec![];
        let mut enemie_sprites = Vec::<Sprite>::new();

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
            entity_manager,

            render_pipeline,
            background,
            camera,

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

        println!("FPS: {}", 1.0 / dt.as_secs_f64());

        self.background.uniform.write(queue);

        self.entity_manager.update(
            device,
            queue,
            &mut self.audio,
            &self.input_controller,
            &mut self.camera,
            &dt,
        );
        self.camera.uniform.write(queue);
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

    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

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
            self.entity_manager.draw(&mut rpass);
        }

        self.particle_system.render(
            queue,
            &mut encoder,
            &frame.texture,
            &self.camera,
            &self
                .entity_manager
                .player
                .get_orientation_point((0.0, self.entity_manager.player.top_right().y).into()),
            self.entity_manager.player.rotation(),
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
