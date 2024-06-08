use cgmath::{SquareMatrix, Vector2};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EntityUniform {
    pub model: cgmath::Matrix4<f32>,
    position: cgmath::Vector2<f32>,
    size: f32,
}
unsafe impl bytemuck::Pod for EntityUniform {}
unsafe impl bytemuck::Zeroable for EntityUniform {}

impl Default for EntityUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
            position: (0.0, 0.0).into(),
            size: 24.0,
        }
    }
}

impl EntityUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((position.x, position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(self.size);
    }

    pub fn set_size(&mut self, size: f32) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(size);
    }

    pub fn set_scale(&mut self, w: f32, h: f32) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_nonuniform_scale(w, h, 0.0);
    }
}
