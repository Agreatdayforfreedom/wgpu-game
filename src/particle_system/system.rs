
use cgmath::Vector2;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::{camera, post_processing::{self, PostProcessing}, rendering::{create_bind_group_layout, Sprite}, util::CompassDir};

const NUM_PARTICLES: wgpu::BufferAddress = 1500;
pub struct ParticleSystem {
    texture_view: Sprite,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    simulation_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    bloom: PostProcessing,
}


fn create_particles_bytes() -> Vec<f32> {
    let mut particles = vec![0.0f32; (12 * NUM_PARTICLES) as usize];
    for chunk in particles.chunks_mut(12) {
        chunk[0] = rand::thread_rng().gen_range(-400..400) as f32;
        chunk[1] = rand::thread_rng().gen_range(-400..400) as f32;

        let dir = CompassDir::from_deg(rand::thread_rng().gen_range(0..360) as f32).dir;
        chunk[2] = dir.x;
        chunk[3] = dir.y;

        //color
        chunk[4] = 0.2;
        chunk[5] = 1.0;
        chunk[6] = 1.0;
        chunk[7] = 1.0;

        // velocity
        chunk[8] = rand::thread_rng().gen_range(5..35) as f32;
        // lifetime
        chunk[9] = rand::thread_rng().gen_range(0..20) as f32;

        //padding
        // chunk[10] = 0.0;
        // chunk[11] = 0.0;
    }

    particles
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera: &camera::Camera,
    ) -> Self {

        let texture_view = Sprite::from_empty(device, (800, 600), wgpu::AddressMode::ClampToEdge, &create_bind_group_layout(device), "Particles_target_texture");

        let bloom = PostProcessing::new(device, format);

        //buffers
        #[rustfmt::skip]
        let vertex_buffer_data = [
            -1.0f32, -1.0,
            1.0, -1.0,   
           -1.0,  1.0,   
           -1.0,  1.0,   
            1.0,  1.0,   
            1.0, -1.0
        ];

        let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let particles_src = create_particles_bytes();

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        for i in 0..1 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {i}")),
                    contents: bytemuck::cast_slice(&particles_src),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        let simulation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particles compute buffer"),
            contents: bytemuck::bytes_of(&[0.04, 800.0, 600.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        //pipelines
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/particles.wgsl"));

        let p_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&camera.uniform.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particles render pipeline"),
            layout: Some(&p_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 12 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 1,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        //bind groups

        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        for i in 0..1 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: simulation_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particles compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "simulate",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            texture_view,
            bloom,
            particle_bind_groups,
            vertices_buffer,
            compute_pipeline,
            particle_buffers,
            simulation_buffer,
            render_pipeline,
        }
    }

    pub fn render(
        &mut self,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::Texture,
        camera: &camera::Camera,
        player_position: &Vector2<f32>,
    ) {
        queue.write_buffer(
            &self.simulation_buffer,
            0,
            bytemuck::cast_slice(&[
                0.04, //delta
                0.0, //padding
                800.0, 600.0,
                player_position.x, 
                player_position.y
            ]),
        );

        // let view = &view.create_view(&wgpu::TextureViewDescriptor::default());
        let rpass_layout = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view.texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        {
            let mut rpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.compute_pipeline);
            rpass.set_bind_group(0, &self.particle_bind_groups[0], &[]);
            rpass.dispatch_workgroups((NUM_PARTICLES as f32 / 64.0).ceil() as u32, 1, 1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&rpass_layout);
            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_bind_group(0, &camera.uniform.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.particle_buffers[0].slice(..));
            rpass.set_vertex_buffer(1, self.vertices_buffer.slice(..));

            rpass.draw(0..6, 0..NUM_PARTICLES as u32);
        }
        let v = view.create_view(&wgpu::TextureViewDescriptor::default());
        self.bloom.render(encoder, &self.texture_view, &v);

    }

    pub fn blend(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        extra_texture: &Sprite,
        target_texture: &wgpu::TextureView, 
        pipeline: &wgpu::RenderPipeline
    ) {
        // blend particles and normal pass
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_texture,
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

        
        rpass.set_pipeline(pipeline);

        rpass.set_bind_group(0, &extra_texture.bind_group, &[]);
        rpass.set_bind_group(1, &self.bloom.get_final_texture().bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
