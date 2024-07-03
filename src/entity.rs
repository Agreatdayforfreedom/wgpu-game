use cgmath::{Deg, SquareMatrix};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EntityUniform {
    pub model: cgmath::Matrix4<f32>,
    color: cgmath::Vector4<f32>,
    position: cgmath::Vector2<f32>,
    size: f32,
    angle: Deg<f32>,
    scale: cgmath::Vector2<f32>,
}
unsafe impl bytemuck::Pod for EntityUniform {}
unsafe impl bytemuck::Zeroable for EntityUniform {}

impl Default for EntityUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
            color: (1.0, 1.0, 1.0, 1.0).into(),
            position: (0.0, 0.0).into(),
            angle: Deg(0.0),
            size: 24.0,
            scale: (24.0, 24.0).into(),
        }
    }
}

impl EntityUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.position = position;
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((position.x, position.y, 0.0).into())
            * cgmath::Matrix4::from_translation((0.5 * self.size, 0.5*self.size, 0.0).into()) //move origin
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation((-0.5 * self.size, -0.5*self.size, 0.0).into()) //move origin back
            * cgmath::Matrix4::from_scale(self.size);
    }

    pub fn set_rotation(&mut self, angle: Deg<f32>) {
        self.angle = angle;
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_translation((0.5 * self.size, 0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_angle_z(angle)
            * cgmath::Matrix4::from_translation((-0.5 * self.size, -0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_scale(self.size);
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_translation((0.5 * self.size, 0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation((-0.5 * self.size, -0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_scale(size);
    }

    pub fn set_scale(&mut self, scale: cgmath::Vector2<f32>) {
        self.scale = scale;
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_translation((0.5 * self.size, 0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation((-0.5 * self.size, -0.5 * self.size, 0.0).into())
            * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, 0.0);
    }

    pub fn set_color(&mut self, color: cgmath::Vector4<f32>) {
        self.color = color;
    }
}
