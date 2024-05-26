#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new() -> Self {
        // let ortho = cgmath::ortho(
        //     -(800.0 / 2.0),
        //     800.0 / 2.0,
        //     600.0 / 2.0,
        //     -(600.0 / 2.0),
        //     -50.0,
        //     50.0,
        // );

        let ortho = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0);
        // let view = cgmath::Matrix4::look_at_rh(
        //     (0.0, 0.0, 2.0).into(),
        //     (0.0, 0.0, 0.0).into(),
        //     cgmath::Vector3::unit_y(),
        // );

        Self {
            proj: (ortho).into(),
        }
    }
}
