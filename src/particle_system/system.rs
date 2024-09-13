use std::{borrow::Borrow, mem};

use cgmath::{vec4, Vector2, Vector4};
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::{camera, rendering, uniform::Uniform, util::CompassDir};

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
struct Particle {
    position: Vector2<f32>,
    color: Vector4<f32>,
}

unsafe impl bytemuck::Pod for Particle {}
unsafe impl bytemuck::Zeroable for Particle {}

const NUM_PARTICLES: wgpu::BufferAddress = 1000;
const PARTICLES_PER_GROUP: wgpu::BufferAddress = 64;
const BYTES: u32 = 4 * 6;
pub struct ParticleSystem {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    // simulation_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    sprite: rendering::Sprite,
    // bind_group: wgpu::BindGroup,
    // buffer: wgpu::Buffer,
    frame_num: usize,
}

//todo
const GRADIENT: [Vector4<f32>; 20] = [
    Vector4::new(0.0, 0.0, 1.0, 1.0),
    Vector4::new(0.05, 0.05, 1.0, 1.0),
    Vector4::new(0.10, 0.10, 1.0, 1.0),
    Vector4::new(0.15, 0.15, 1.0, 1.0),
    Vector4::new(0.20, 0.20, 1.0, 1.0),
    Vector4::new(0.25, 0.25, 1.0, 1.0),
    Vector4::new(0.30, 0.30, 1.0, 1.0),
    Vector4::new(0.35, 0.35, 1.0, 1.0),
    Vector4::new(0.40, 0.40, 1.0, 1.0),
    Vector4::new(0.45, 0.45, 1.0, 1.0),
    Vector4::new(0.50, 0.50, 1.0, 1.0),
    Vector4::new(0.55, 0.55, 1.0, 1.0),
    Vector4::new(0.60, 0.60, 1.0, 1.0),
    Vector4::new(0.65, 0.65, 1.0, 1.0),
    Vector4::new(0.70, 0.70, 1.0, 1.0),
    Vector4::new(0.75, 0.75, 1.0, 1.0),
    Vector4::new(0.80, 0.80, 1.0, 1.0),
    Vector4::new(0.85, 0.85, 1.0, 1.0),
    Vector4::new(0.90, 0.90, 1.0, 1.0),
    Vector4::new(1.0, 1.0, 1.0, 1.0),
];

fn create_particles() -> Vec<Particle> {
    let mut particles = Vec::<Particle>::new();
    let mut g = 0;
    for i in 0..NUM_PARTICLES {
        particles.push(Particle {
            position: ((rand::thread_rng().gen_range(0..400)) as f32, 0.0).into(),
            color: GRADIENT[g as usize],
        });
        if g == 19 {
            g = 0;
            continue;
        }
        g += 1;
    }
    particles
}

fn create_particles_bytes() -> Vec<f32> {
    let mut particles = vec![0.0f32; (12 * NUM_PARTICLES) as usize];
    for chunk in particles.chunks_mut(12) {
        //left position at x: 0.0, y: 0.0
        // chunk[0] = 0.0;
        // chunk[1] = 0.0;

        let dir = CompassDir::from_deg(rand::thread_rng().gen_range(0..360) as f32).dir;
        chunk[2] = dir.x;
        chunk[3] = dir.y;

        //color
        chunk[4] = 0.2;
        chunk[5] = 1.0;
        chunk[6] = 1.0;
        chunk[7] = 1.0;

        // velocity
        chunk[8] = rand::thread_rng().gen_range(10..100) as f32;
        // lifetime
        chunk[9] = rand::thread_rng().gen_range(0..12) as f32;

        //padding
        // chunk[10] = 0.0;
        // chunk[11] = 0.0;
    }

    particles
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sprite: rendering::Sprite,
        format: wgpu::TextureFormat,
        camera: &camera::Camera,
    ) -> Self {
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
        // let vertex_buffer_data = [-0.01f32, -0.02, 0.01, -0.02, 0.00, 0.02];

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
            contents: bytemuck::bytes_of(&[0.04]),
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
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: wgpu::ShaderStages::COMPUTE,
                    //     ty: wgpu::BindingType::Buffer {
                    //         ty: wgpu::BufferBindingType::Storage { read_only: false },
                    //         has_dynamic_offset: false,
                    //         min_binding_size: wgpu::BufferSize::new((NUM_PARTICLES * (4 * 2)) as _),
                    //     },
                    //     count: None,
                    // },
                ],
            });

        for i in 0..1 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    // wgpu::BindGroupEntry {
                    //     binding: 0,
                    //     resource: sim_param_buffer.as_entire_binding(),
                    // },
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    // wgpu::BindGroupEntry {
                    //     binding: 1,
                    //     resource: particle_buffers[(i + 1) % 2].as_entire_binding(),
                    // },
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

        // let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Particles compute bind group"),
        //     layout: &bind_group_layout,
        //     entries: &[
        //         // wgpu::BindGroupEntry {
        //         //     binding: 0,
        //         //     resource: simulation_buffer.as_entire_binding(),
        //         // },
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                 buffer: &particles_buffer,
        //                 offset: 0,
        //                 size: wgpu::BufferSize::new(particles_buffer.size()),
        //             }),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                 buffer: &particles_buffer_dst,
        //                 offset: 0,
        //                 size: wgpu::BufferSize::new(particles_buffer_dst.size()),
        //             }),
        //         },
        //     ],
        // });

        Self {
            particle_bind_groups,
            vertices_buffer,
            compute_pipeline,
            particle_buffers,
            // simulation_buffer,
            render_pipeline,
            sprite,
            frame_num: 0,
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

            // rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
            rpass.draw(0..6, 0..NUM_PARTICLES as u32);
        }

        self.frame_num += 1;
    }
}
