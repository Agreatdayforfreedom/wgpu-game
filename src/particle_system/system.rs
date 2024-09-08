use std::borrow::Borrow;

use cgmath::Vector2;
use wgpu::util::DeviceExt;

use crate::{camera, rendering, uniform::Uniform};

struct Particle {
    position: Vector2<f32>,
}

const NUM_PARTICLES: wgpu::BufferAddress = 50000;

//THIS STRUCT IS FOR TESTING
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    color: cgmath::Vector4<f32>,
}

unsafe impl bytemuck::Pod for Color {}
unsafe impl bytemuck::Zeroable for Color {}

impl Default for Color {
    fn default() -> Self {
        Self {
            color: cgmath::Vector4 {
                x: 1.0_f32,
                y: 0.0_f32,
                z: 0.0_f32,
                w: 1.0_f32,
            },
        }
    }
}

pub struct ParticleSystem {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particles_buffer: wgpu::Buffer,
    // simulation_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    compute_bind_group: wgpu::BindGroup,
    sprite: rendering::Sprite,
    color: Uniform<Color>,
    // bind_group: wgpu::BindGroup,
    // buffer: wgpu::Buffer,
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        sprite: rendering::Sprite,
        format: wgpu::TextureFormat,
        camera: &camera::Camera,
        bind_group_layout: &wgpu::BindGroupLayout,
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
            size: 4 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //pipelines
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/particles.wgsl"));

        let color = Uniform::<Color>::new(device);

        let p_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &color.bind_group_layout],
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
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
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
            cache: None,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particles compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: "simulate",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
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
            ],
        });

        Self {
            compute_bind_group,
            vertices_buffer,
            compute_pipeline,
            particles_buffer,
            // simulation_buffer,
            render_pipeline,
            color,
            sprite,
        }
    }

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
            let mut rpass = encoder.begin_render_pass(&rpass_layout);
            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_vertex_buffer(0, self.vertices_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.particles_buffer.slice(..));

            rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
            rpass.set_bind_group(1, &self.color.bind_group, &[]);
            rpass.draw(0..3, 0..NUM_PARTICLES as u32);
        }

        {
            let mut rpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.compute_pipeline);
            rpass.set_bind_group(0, &self.compute_bind_group, &[]);
            rpass.dispatch_workgroups(((NUM_PARTICLES + 63) / 64) as u32, 1, 1);
        }
    }
}
