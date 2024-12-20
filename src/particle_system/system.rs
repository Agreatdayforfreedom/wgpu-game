
use std::collections::HashMap;

use cgmath::Vector2;
use wgpu::util::DeviceExt;

use crate::{camera, post_processing::PostProcessing, rendering::{create_bind_group_layout, Sprite}, util::distance};

use super::simulation_params::{SimulationBuffer, SimulationParams};

const PARTICLE_POOLING: wgpu::BufferAddress = 100000;
pub struct ParticleSystem {
    texture_view: Sprite,
    total_particles: u64,
    particles: HashMap<u32, Vec<f32>>,
    sim_params: Vec<(u32, SimulationParams)>,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particle_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    simulation_buffer: SimulationBuffer,
    particle_bind_group: wgpu::BindGroup,
    bloom: PostProcessing,
    time: f64,
}

fn create_compute_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
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

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Compute Buffer"),
            contents: bytemuck::bytes_of(&[0.0, 0.0]),//delta_time, time
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST 
            ,

        });

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("Particle Buffer")),
                    size: 12 * 4 * PARTICLE_POOLING,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

        let simulation_buffer = SimulationBuffer::new(&device);
        
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


        

        let particle_bind_group= device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &create_compute_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: simulation_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&create_compute_bind_group_layout(device)],
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
            total_particles: 0,
            particle_bind_group,
            particles: HashMap::with_capacity(0),
            sim_params: vec![],
            vertices_buffer,
            particle_buffer,
            simulation_buffer,
            uniform_buffer,
            compute_pipeline,
            render_pipeline,
            time: 0.0,
        }
    }

    pub fn update_sim_params(&mut self,
        id: u32,
        position: Vector2<f32>,
        infinite: u32,
    ) {
        

        for t in &mut self.sim_params {
            if t.0 == id {
                let dist = distance(t.1.position(), position);
                t.1.set_distance_traveled(dist);
                t.1.position = position;
                if infinite != t.1.infinite {
                    t.1.infinite = infinite; // so we do this so that when a projectile dies, it stops emitting particles
                }
                break;
            }
        }        
    }

    pub fn update(&mut self, 
        queue: &mut wgpu::Queue,
        dt:&instant::Duration
    ) {
        let dt = dt.as_secs_f64();
        self.time += dt;
        let mut i = 0;
        for (_, data) in &mut self.sim_params {
            i += 1;
            data.interval += dt as f32;
        }



        queue.write_buffer(
            self.simulation_buffer.buffer(),
            0,
            bytemuck::cast_slice(&self.sim_params.iter().map(|t| { t.1 }).collect::<Vec<SimulationParams>>()),
        );
        queue.write_buffer(
            &self.uniform_buffer, 0, bytemuck::cast_slice(&[dt as f32, self.time as f32]));

    }


    pub fn push_group(&mut self, id: u32, device: &wgpu::Device, params: SimulationParams) {

        self.total_particles = PARTICLE_POOLING;
        
        
        self.sim_params.push((id, params));
        
        //destroy previous buffers
        self.simulation_buffer.destroy();

        let simulation_buffer = self.simulation_buffer.with_contents(&device, bytemuck::cast_slice(&self.sim_params.iter().map(|t| { t.1 }).collect::<Vec<SimulationParams>>()));
        
        // replace the bind group with the new buffers
        println!("MAX BUFFER SIZE: {}", device.limits().max_buffer_size);
        self.particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &create_compute_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: simulation_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });


    }

    

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::Texture,
        camera: &camera::Camera,
    ) {
        
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
            rpass.set_bind_group(0, &self.particle_bind_group, &[]);
            rpass.dispatch_workgroups((self.total_particles as f32 / 64.0).ceil() as u32, 1, 1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&rpass_layout);
            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_bind_group(0, &camera.uniform.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.particle_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.vertices_buffer.slice(..));

            rpass.draw(0..6, 0..self.total_particles as u32);
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
