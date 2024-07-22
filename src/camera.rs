// aspect = w / h
// half_h = h / 2.0
// half_w = w * aspect

use cgmath::{
    Array, EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Transform, Vector3,
};
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0,
    0.0, 0.0, 0.0, 1.0,
);
use crate::uniform::Uniform;

pub struct Camera {
    pub uniform: Uniform<CameraUniform>,
}

impl Camera {
    pub fn new(uniform: Uniform<CameraUniform>) -> Self {
        Self { uniform }
    }
    pub fn update(&mut self, position: Vector3<f32>) {
        self.uniform.data.update(position);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub proj: [[f32; 4]; 4],
}
const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const ASPECT: f32 = WIDTH / HEIGHT;
impl CameraUniform {
    fn update(&mut self, position: Vector3<f32>) {
        let view = Matrix4::from_translation(-position);
        let ortho = OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -WIDTH / 2.0,
                WIDTH / 2.0,
                -HEIGHT / 2.0,
                HEIGHT / 2.0,
                -50.0,
                50.0,
            );
        self.proj = (ortho * view).into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        let position = Vector3::new(0.0, 0.0, 1.0);

        let view = Matrix4::from_translation(-position);
        let ortho = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0);
        Self {
            proj: (ortho * view).into(),
        }
    }
}
