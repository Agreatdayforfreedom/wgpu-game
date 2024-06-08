use cgmath::SquareMatrix;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EntityUniform {
    pub model: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for EntityUniform {}
unsafe impl bytemuck::Zeroable for EntityUniform {}

impl Default for EntityUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
        }
    }
}

impl EntityUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((position.x, position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(24.0);
    }
}
