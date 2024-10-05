use std::borrow::Borrow;

use bytemuck::AnyBitPattern;
use cgmath::{Vector2, Vector3, Vector4};

pub struct SimulationBuffer {
    buffer: wgpu::Buffer,
}

impl SimulationBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        use std::mem;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particles compute buffer"),
            size: mem::size_of::<SimulationParams>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer }
    }

    pub fn with_contents(&mut self, device: &wgpu::Device, content: &[u8]) -> &wgpu::Buffer {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particles compute buffer"),
            contents: bytemuck::cast_slice(content),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.buffer = buffer;

        &self.buffer
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub fn destroy(&self) {
        self.buffer.destroy();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SimulationParams {
    delta_time: f32,
    total: f32,
    position: Vector2<f32>,
    color: Vector4<f32>,
    dir: Vector2<f32>,
    color_over_lifetime: f32,
    arc: f32,
    rate_over_distance: f32,
    distance_traveled: f32,
    _pad: f32,
    __pad: f32,
    //updated in the shader
}

impl SimulationParams {
    pub fn new(
        delta_time: f32,
        total: f32,
        position: Vector2<f32>,
        color: Vector4<f32>,
        dir: Vector2<f32>,
        color_over_lifetime: f32,
        arc: f32,
        rate_over_distance: f32,
        distance_traveled: f32,
    ) -> Self {
        Self {
            delta_time,
            total,
            position,
            color,
            dir,
            color_over_lifetime,
            arc,
            rate_over_distance,
            distance_traveled,
            _pad: 0.0,
            __pad: 0.0,
        }
    }
}

unsafe impl bytemuck::Pod for SimulationParams {}
unsafe impl bytemuck::Zeroable for SimulationParams {}

impl SimulationParams {
    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn set_distance_traveled(&mut self, dist: f32) {
        self.distance_traveled = dist;
    }
}
