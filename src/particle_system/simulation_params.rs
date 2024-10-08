use cgmath::{Array, Vector2, Vector3, Vector4};

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
pub struct Cone {
    pub arc: f32,
    pub angle: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub radius: f32,
    pub emit_from_edge: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SimulationParams {
    pub interval: f32,
    pub total: f32,
    pub position: Vector2<f32>,
    pub color: Vector4<f32>,
    pub dir: Vector2<f32>,
    pub color_over_lifetime: f32,
    pub rate_over_distance: f32,
    pub distance_traveled: f32,
    pub lifetime_factor: f32,
    pub start_speed: f32,
    pub mode: u32,
    pub shape_selected: u32,
    pub cone: Cone,
    pub circle: Circle,
    pub _pad: Vector3<u32>,
}

unsafe impl bytemuck::Pod for SimulationParams {}
unsafe impl bytemuck::Zeroable for SimulationParams {}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            interval: 0.0,
            total: 1000.0,
            position: (0.0, 0.0).into(),
            dir: (0.0, 0.0).into(),
            color: (1.0, 1.0, 1.0, 1.0).into(),
            color_over_lifetime: 1.0,
            rate_over_distance: 0.0,
            distance_traveled: 0.0,
            lifetime_factor: 1.0,
            start_speed: 1.0,
            mode: 0,
            shape_selected: 0,
            cone: Cone::default(),
            circle: Circle::default(),
            _pad: Vector3::from_value(0),
        }
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            emit_from_edge: 0,
            radius: 5.0,
        }
    }
}
impl Default for Cone {
    fn default() -> Self {
        Self {
            angle: 0.0,
            arc: 45.0,
        }
    }
}

impl SimulationParams {
    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn set_distance_traveled(&mut self, dist: f32) {
        self.distance_traveled = dist;
    }
}
