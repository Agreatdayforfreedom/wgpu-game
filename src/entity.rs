use cgmath::{Deg, SquareMatrix};

use crate::{audio::Audio, input::Input};

pub trait Entity {
    fn update(
        &mut self,
        _dt: &instant::Duration,
        _input: &Input,
        _audio: &mut Audio,
        _device: &wgpu::Device,
        _queue: &mut wgpu::Queue,
        _time: f64,
    ) {
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EntityUniform {
    pub model: cgmath::Matrix4<f32>,
    color: cgmath::Vector4<f32>,
    tex_scale: cgmath::Vector2<f32>,
    pub tex_pos: f32,
    position: cgmath::Vector2<f32>,
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
            tex_scale: (1.0, 1.0).into(), //TODO
            tex_pos: 1.0,
            position: (0.0, 0.0).into(),
            angle: Deg(0.0),
            scale: (24.0, 24.0).into(),
        }
    }
}

impl EntityUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_rotation(&mut self, angle: Deg<f32>) -> &mut Self {
        self.angle = angle;
        self
    }

    pub fn set_scale(&mut self, scale: cgmath::Vector2<f32>) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn set_tex_scale(&mut self, scale: cgmath::Vector2<f32>) -> &mut Self {
        self.tex_scale = scale;
        self
    }

    pub fn set_color(&mut self, color: cgmath::Vector4<f32>) -> &mut Self {
        self.color = color;
        self
    }

    pub fn exec(&mut self) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_translation(
                (0.5 * self.scale.x, 0.5 * self.scale.y, 0.0).into(),
            )
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation(
                (-0.5 * self.scale.x, -0.5 * self.scale.y, 0.0).into(),
            )
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 0.0);
    }
}
