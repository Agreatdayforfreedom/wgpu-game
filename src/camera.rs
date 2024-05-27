#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub proj: [[f32; 4]; 4],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            proj: cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0).into(),
        }
    }
}
