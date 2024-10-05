
use std::{collections::HashMap, ops::DerefMut};

use cgmath::{Angle, Vector2, Vector4};
use rand::Rng;
use rodio::Device;
use wgpu::util::DeviceExt;

use crate::{camera, post_processing::PostProcessing, rendering::{create_bind_group_layout, Sprite}, util::{distance, CompassDir}};

use super::simulation_params::{SimulationBuffer, SimulationParams};

const NUM_PARTICLES: wgpu::BufferAddress = 1000;
pub struct ParticleSystem {
    texture_view: Sprite,
    total_particles: u64,
    particles: HashMap<u32, Vec<f32>>,
    sim_params: Vec<(u32, SimulationParams)>,
    // distance_traveled: Vec<f32>,
    emitter_data: Vec<f32>,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    particle_buffer: wgpu::Buffer,
    emitter_buffer: wgpu::Buffer,
    simulation_buffer: SimulationBuffer,
    vertices_buffer: wgpu::Buffer,
    particle_bind_group: wgpu::BindGroup,
    bloom: PostProcessing,
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
            // wgpu::BindGroupLayoutEntry {
            //     binding: 2,
            //     visibility: wgpu::ShaderStages::COMPUTE,
            //     ty: wgpu::BindingType::Buffer {
            //         ty: wgpu::BufferBindingType::Storage { read_only: false },
            //         has_dynamic_offset: false,
            //         min_binding_size: None,
            //     },
            //     count: None,
            // }
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

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("Particle Buffer")),
                    size: 12 * 4,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

        let simulation_buffer = SimulationBuffer::new(&device);
        
        let emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Emitter Buffer"),
            size: 8,
            usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation:false,
            
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
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: emitter_buffer.as_entire_binding(),
                // }
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
            emitter_data: vec![],
            vertices_buffer,
            compute_pipeline,
            particle_buffer,
            emitter_buffer,
            simulation_buffer,
            render_pipeline,
        }
    }

    pub fn init(&mut self, device: &wgpu::Device) {
        self.push_group(0, device, 1000, (0.0, 0.0).into(), (1.0, 1.0, 1.0, 1.0).into());
    }

    pub fn update_sim_params(&mut self, 
        queue: &mut wgpu::Queue,
        id: u32,
        total: f32,
        position: &Vector2<f32>,
        dir: cgmath::Deg<f32>,
        color: Vector4<f32>,
        dt: &instant::Duration,
    ) {
        let dir = CompassDir::from_deg(dir.opposite().0).dir;


        let mut data = SimulationParams::new(
            dt.as_secs_f32(), 
            total, 
            *position, 
            color, 
            dir, 
            1.0, 
            15.0, 
            7.0,
            0.0
        );



        for t in &mut self.sim_params {
            if t.0 == id {
                let dist = distance(t.1.position(), *position);
                data.set_distance_traveled(dist);
                t.1 = data;
                break;
            }
        }        
    }


    pub fn push_group(&mut self, id: u32, device: &wgpu::Device, num_particles: wgpu::BufferAddress, position:Vector2<f32>, color: Vector4<f32>) {

        self.total_particles += num_particles;
        
        let mut particles = vec![0.0f32; (12 * num_particles) as usize];
        println!("particles size: {}", std::mem::size_of::<f32>() * (12 * num_particles) as usize);
        for chunk in particles.chunks_mut(12) {
            chunk[0] = rand::thread_rng().gen_range(-400..400) as f32;
            chunk[1] = rand::thread_rng().gen_range(-400..400) as f32;
    
            chunk[2] = 0.0;
            chunk[3] = 0.0;
    
            //color
            chunk[4] = 1.0;
            chunk[5] = 0.9;
            chunk[6] = 0.0;
            chunk[7] = 1.0;
    
            // velocity
            chunk[8] =  rand::thread_rng().gen_range(10.0..50.0);
            // chunk[8] =  rand::thread_rng().gen_range(100.0..500.0);
            // lifetime
            chunk[9] = 0.0 as f32;
    
            //padding
            // chunk[10] = rand::thread_rng().gen_range(0.0..30.0);
            // chunk[11] = 0.0;
        }

        let sim_params = SimulationParams::new(
            0.0, 
            num_particles as f32, 
            position, 
            color, 
            (0.0, 0.0).into(), 
            0.0, 
            0.0, 
            0.0, 
            0.0,
        );

        //extend the previous particles
        self.particles.insert(id, particles);
        //extend the sim params uniform
        self.sim_params.push((id, sim_params));

        //create a new buffer with the new particles
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&self.particles.values().cloned().flatten().collect::<Vec<f32>>()),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        // self.emitter_data.push(position.x);
        // self.emitter_data.push(position.y);
        // let emitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some(&format!("Particle Buffer")),
        //     contents: bytemuck::cast_slice(&self.emitter_data),
        //     usage: wgpu::BufferUsages::STORAGE
        //         | wgpu::BufferUsages::COPY_DST,
        // });
        
        //destroy previous buffers
        self.particle_buffer.destroy();
        self.simulation_buffer.destroy();
        // self.emitter_buffer.destroy();

        let simulation_buffer = self.simulation_buffer.with_contents(&device, bytemuck::cast_slice(&self.sim_params.iter().map(|t| { t.1 }).collect::<Vec<SimulationParams>>()));
        
        // replace the bind group with the new buffers
        println!("MAX BUFFER SIZE: {}", device.limits().max_buffer_size);
        self.particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: emitter_buffer.as_entire_binding(),
                // }
            ],
            label: None,
        });

        //save the new buffers
        self.particle_buffer = particle_buffer;
        // self.emitter_buffer = emitter_buffer;

    }

    pub fn render(
        &mut self,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::Texture,
        camera: &camera::Camera,
        player_position: &Vector2<f32>,
        dir: cgmath::Deg<f32>,
        dt: &instant::Duration,
    ) {
        queue.write_buffer(
            self.simulation_buffer.buffer(),
            0,
            bytemuck::cast_slice(&self.sim_params.iter().map(|t| { t.1 }).collect::<Vec<SimulationParams>>()),
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
            let mut rpass = encoder.begin_render_pass(&rpass_layout);
            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_bind_group(0, &camera.uniform.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.particle_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.vertices_buffer.slice(..));

            rpass.draw(0..6, 0..self.total_particles as u32);
        }
        {
            let mut rpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.compute_pipeline);
            rpass.set_bind_group(0, &self.particle_bind_group, &[]);
            rpass.dispatch_workgroups((self.total_particles as f32 / 64.0).ceil() as u32, 1, 1);
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
