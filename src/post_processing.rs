use crate::rendering::{self, create_bind_group_layout, create_render_pipeline};

pub struct PostProcessing {
    offscreen: rendering::Sprite,
    pipeline: wgpu::RenderPipeline,
}

impl PostProcessing {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let bind_group_layout = create_bind_group_layout(&device);

        let offscreen = rendering::Sprite::from_empty(
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
        let shader_particles =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/particles.wgsl"));
        let pipeline =
            create_render_pipeline(&device, &shader_particles, &config, &pipeline_layout);

        Self {
            offscreen,
            pipeline,
        }
    }

    //TODO
    // pub fn create_view(&self) -> &wgpu::TextureView {
    //     &self.offscreen_texture.view
    // }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.offscreen.buffer.slice(..));
        rpass.set_bind_group(0, &self.offscreen.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
