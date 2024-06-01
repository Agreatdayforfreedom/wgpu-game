use cgmath::SquareMatrix;

pub struct Enemy {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
}

impl Enemy {
    pub fn new(position: cgmath::Vector2<f32>) -> Self {
        Self {
            position,
            size: 24.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EnemyUniform {
    pub model: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for EnemyUniform {}
unsafe impl bytemuck::Zeroable for EnemyUniform {}

impl Default for EnemyUniform {
    fn default() -> Self {
        let model = cgmath::Matrix4::identity();

        Self { model }
    }
}

impl EnemyUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((position.x, position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(24.0);
    }
}
