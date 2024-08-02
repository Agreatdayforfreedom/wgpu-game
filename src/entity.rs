use cgmath::{Deg, SquareMatrix, Vector2};

use crate::{audio::Audio, input::Input, uniform::Uniform};

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

    fn alive(&self) -> bool {
        true
    }

    fn id(&self) -> u32;

    fn position(&self) -> Vector2<f32>;

    fn rotate(&mut self, rotation: Deg<f32>) {}

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {}
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
    pivot: cgmath::Point2<f32>,
}
unsafe impl bytemuck::Pod for EntityUniform {}
unsafe impl bytemuck::Zeroable for EntityUniform {}

impl Default for EntityUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
            color: (1.0, 1.0, 1.0, 1.0).into(),
            tex_scale: (1.0, 1.0).into(), //TODO
            pivot: (0.5, 0.5).into(),
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

    pub fn set_pivot(&mut self, pivot: cgmath::Point2<f32>) -> &mut Self {
        self.pivot = pivot;
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
                (
                    self.pivot.x * self.scale.x,
                    self.pivot.y * self.scale.y,
                    0.0,
                )
                    .into(),
            )
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation(
                (
                    -self.pivot.x * self.scale.x,
                    -self.pivot.y * self.scale.y,
                    0.0,
                )
                    .into(),
            )
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 0.0);
    }
}
