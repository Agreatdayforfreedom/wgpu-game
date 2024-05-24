use cgmath::{InnerSpace, SquareMatrix};

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
        let ortho = cgmath::ortho(
            -(800.0 / 2.0),
            800.0 / 2.0,
            600.0 / 2.0,
            -(600.0 / 2.0),
            -50.0,
            50.0,
        );
        // #[rustfmt::skip]

        // let ortho: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        //     2.0 / 800.0, 0.0, 0.0, 0.0,
        //     0.0, 2.0 / 600.0, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     -1.0, -1.0, 0.0, 1.0,
        // );
        // let proj = OPENGL_TO_WGPU_MATRIX * ortho;
        // Self { proj: proj.into() }

        let view = cgmath::Matrix4::look_at_rh(
            (0.0, 0.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
        );
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(45.0), 800.0 / 400.0, 0.1, 100.0);

        // 3.
        Self {
            proj: (OPENGL_TO_WGPU_MATRIX * ortho * view).into(),
        }
    }
}
