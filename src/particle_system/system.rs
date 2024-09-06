use cgmath::Vector2;
use wgpu::util::DeviceExt;

use crate::{camera, entity::EntityUniform, rendering};

struct Particle {
    position: Vector2<f32>,
}

const NUM_PARTICLES: wgpu::BufferAddress = 50000;

pub struct ParticleSystem {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particles_buffer: wgpu::Buffer,
    // simulation_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    compute_bind_group: wgpu::BindGroup,
    sprite: rendering::Sprite,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct _Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

const VERTICES: &[_Vertex] = &[
    _Vertex {
        position: [0.0, 0.5],
        tex_coords: [1.0, 0.0],
    },
    _Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    _Vertex {
        position: [0.5, -0.5],
        tex_coords: [0.0, 0.0],
    },
];
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        sprite: rendering::Sprite,
        format: wgpu::TextureFormat,
        texture: wgpu::Texture,
        camera: &camera::Camera,
        bind_group_layout: &wgpu::BindGroupLayout, // bind_group_layout: &BindGroupLayout,
    ) -> Self {
        //buffers

        let vertex_buffer_data = [-0.01f32, -0.02, 0.01, -0.02, 0.00, 0.02];
        let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let particles_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particles buffer"),
            size: std::mem::size_of::<Particle>() as wgpu::BufferAddress * NUM_PARTICLES,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let simulation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particles compute buffer"),
            size: 1 * 4 + 3 * 4 + 4 * 4 + 0,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //pipelines
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/particles.wgsl"));

        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particles render pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),

                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,

                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                    }, // wgpu::VertexBufferLayout {
                       //     array_stride: 2 * 4,
                       //     step_mode: wgpu::VertexStepMode::Vertex,
                       //     attributes: &[VertexAttribute {
                       //         shader_location: 2,
                       //         offset: 0,
                       //         format: wgpu::VertexFormat::Float32x2,
                       //     }],
                       // },
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
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particles compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: "simulate",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        //bind groups

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particles compute bind group"),
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[
                // wgpu::BindGroupEntry {
                //     binding: 0,
                //     resource: simulation_buffer.as_entire_binding(),
                // },
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &particles_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(particles_buffer.size()),
                    }),
                },
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: wgpu::BindingResource::TextureView(
                //         &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                //     ),
                // },
            ],
        });

        Self {
            compute_bind_group,
            vertices_buffer,
            compute_pipeline,
            particles_buffer,
            // simulation_buffer,
            render_pipeline,

            sprite,
        }
    }

    // pub fn update(
    //     &mut self,
    //     start_position: Vector2<f32>,
    //     dir: CompassDir,
    //     device: &mut wgpu::Device,
    //     queue: &mut wgpu::Queue,
    //     dt: &instant::Duration,
    // ) {
    //     for particle in &mut self.particles {
    //         particle.update(queue, dt);
    //     }
    //     println!("{}", self.particles.len());
    //     self.particles.push({
    //         let uniform = Uniform::<EntityUniform>::new(&device);
    //         Particle::new(
    //             start_position,
    //             (4.0, 4.0).into(),
    //             (1.0, 1.0, 0.0, 1.0).into(),
    //             20.0,
    //             50.0,
    //             dir,
    //             uniform,
    //         )
    //     });
    //     self.particles = self
    //         .particles
    //         .drain(..)
    //         .filter(|p| p.alive != false)
    //         .collect();
    // }

    // pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
    //     //todo batching
    //     rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
    //     rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
    //     for particle in &mut self.particles {
    //         particle.draw(rpass);
    //     }
    // }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::Texture,
        camera: &camera::Camera,
    ) {
        // queue.write_buffer(
        //     &self.simulation_buffer,
        //     0,
        //     bytemuck::cast_slice(&[
        //         0.04, //delta
        //         0.0,
        //         0.0,
        //         0.0, // padding
        //         rand::thread_rng().gen_range(0..=100) as f32,
        //         rand::thread_rng().gen_range(0..=100) as f32,
        //         1.0 + rand::thread_rng().gen_range(0..=1) as f32,
        //         1.0 + rand::thread_rng().gen_range(0..=1) as f32,
        //     ]),
        // );

        let view = &view.create_view(&wgpu::TextureViewDescriptor::default());
        let rpass_layout = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
            rpass.set_bind_group(0, &self.compute_bind_group, &[]);
            rpass.dispatch_workgroups(((NUM_PARTICLES + 63) / 64) as u32, 1, 1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&rpass_layout);
            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_vertex_buffer(0, self.vertices_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.particles_buffer.slice(..));

            rpass.draw(0..3, 0..NUM_PARTICLES as u32);
        }
    }
}
