use crate::rendering::{self, create_bind_group_layout, create_render_pipeline, Sprite};
const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'_> = wgpu::VertexBufferLayout {
    array_stride: 0, // No vertex buffer data needed
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[],
};
pub struct PostProcessing {
    brightness_target_texture: rendering::Sprite,
    horizontal_blur_target_texture: rendering::Sprite,
    vertical_blur_target_texture: rendering::Sprite,
    brightness_pipeline: wgpu::RenderPipeline,
    horizontal_blur_pipeline: wgpu::RenderPipeline,
    vertical_blur_pipeline: wgpu::RenderPipeline,
    blend_pipeline: wgpu::RenderPipeline,
}

impl PostProcessing {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = create_bind_group_layout(&device);

        let brightness_target_texture = rendering::Sprite::from_empty(
            &device,
            (800, 600),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );

        let horizontal_blur_target_texture = rendering::Sprite::from_empty(
            &device,
            (800, 600),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );

        let vertical_blur_target_texture = rendering::Sprite::from_empty(
            &device,
            (800, 600),
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            "offscreen",
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let final_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
            });
        let shader_fullscreen_quad = device
            .create_shader_module(wgpu::include_wgsl!("./shaders/fullscreen_quad_vertex.wgsl"));
        let shader_brightness =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/brightness.wgsl"));
        let shader_blend = device.create_shader_module(wgpu::include_wgsl!("./shaders/blend.wgsl"));
        let shader_blur = device.create_shader_module(wgpu::include_wgsl!("./shaders/blur.wgsl"));
        let brightness_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brightness pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_fullscreen_quad,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_brightness,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        let vertical_blur_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Verical blur pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_fullscreen_quad,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_blur,
                    entry_point: "vertical_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
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

        let horizontal_blur_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Horizontal blur pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_fullscreen_quad,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_blur,
                    entry_point: "horizontal_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
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

        let blend_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brightness pipeline"),
            layout: Some(&final_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_fullscreen_quad,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_blend,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        Self {
            brightness_pipeline,
            horizontal_blur_pipeline,
            vertical_blur_pipeline,
            blend_pipeline,

            horizontal_blur_target_texture,
            vertical_blur_target_texture,
            brightness_target_texture,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_texture: &Sprite,
        context_view: &wgpu::TextureView,
    ) {
        let mut brightness_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.brightness_target_texture.texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        brightness_rpass.set_pipeline(&self.brightness_pipeline);
        brightness_rpass.set_bind_group(0, &scene_texture.bind_group, &[]);
        brightness_rpass.draw(0..3, 0..1);

        drop(brightness_rpass);

        let mut vertical_blur_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.vertical_blur_target_texture.texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        vertical_blur_rpass.set_pipeline(&self.vertical_blur_pipeline);
        vertical_blur_rpass.set_bind_group(0, &self.brightness_target_texture.bind_group, &[]);
        vertical_blur_rpass.draw(0..3, 0..1);

        drop(vertical_blur_rpass);
        let mut horizontal_blur_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.horizontal_blur_target_texture.texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        horizontal_blur_rpass.set_pipeline(&self.horizontal_blur_pipeline);
        horizontal_blur_rpass.set_bind_group(0, &self.vertical_blur_target_texture.bind_group, &[]);
        horizontal_blur_rpass.draw(0..3, 0..1);

        drop(horizontal_blur_rpass);

        let mut final_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        final_rpass.set_pipeline(&self.blend_pipeline);
        final_rpass.set_bind_group(0, &self.horizontal_blur_target_texture.bind_group, &[]);
        final_rpass.set_bind_group(1, &scene_texture.bind_group, &[]);
        final_rpass.draw(0..3, 0..1);
    }
}
