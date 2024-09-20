use crate::state::game_state::GameState;

use pollster::block_on;
use std::sync::Arc;
use winit::{event::*, window::Window};

#[allow(dead_code)]
pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    game_state: GameState,
}

impl GpuState {
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
        let (device, mut queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
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

        let game_state = GameState::new(&device, &mut queue, &config);

        Self {
            surface,
            device,
            queue,
            config,
            game_state,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.game_state.input(event)
    }
    pub fn update(&mut self, dt: instant::Duration) {
        self.game_state.update(&mut self.queue, &self.device, dt);
    }
    pub fn render(&mut self) {
        self.game_state
            .render(&self.surface, &self.device, &mut self.queue);
    }
}
