use std::time::Duration;

use cgmath::{InnerSpace, Vector2};

use crate::{
    camera::{self, Camera},
    entity::{Entity, EntityUniform},
    input,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};

const LAYER_1_SPEED: f32 = 0.01;

pub struct Background {
    id: u32,
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: cgmath::Deg<f32>,
    sprite: Sprite,
    pub uniform: Uniform<EntityUniform>,
}

impl Entity for Background {
    fn id(&self) -> u32 {
        self.id
    }

    fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn scale(&self) -> cgmath::Vector2<f32> {
        self.scale
    }

    fn rotation(&self) -> cgmath::Deg<f32> {
        self.rotation
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        self.sprite.bind(rpass);
        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}

impl Background {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./assets/bg.png");

        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &create_bind_group_layout(device),
            diffuse_bytes,
        );
        let mut uniform = Uniform::<EntityUniform>::new(&device);
        uniform.data.set_scale((1200.0, 800.0).into()).exec();
        uniform.data.set_tex_scale((2.0, 2.0).into()).exec();
        uniform.data.set_rotation(cgmath::Deg(180.0)).exec();

        Box::new(Self {
            id: 10000, //TODO
            sprite,
            uniform,
            scale: (1200.0, 800.0).into(),
            position: (0.0, 0.0).into(),
            rotation: cgmath::Deg(0.0),
        })
    }

    pub fn update(&mut self, camera: &camera::Camera, input: &input::Input, dt: &Duration) {
        let dt = dt.as_secs_f32();
        let mut position = Vector2::new(0.0, 0.0);
        if input.is_pressed("d") {
            position.x += LAYER_1_SPEED * dt;
        }
        if input.is_pressed("a") {
            position.x -= LAYER_1_SPEED * dt;
        }
        if input.is_pressed("w") {
            position.y += LAYER_1_SPEED * dt;
        }
        if input.is_pressed("s") {
            position.y -= LAYER_1_SPEED * dt;
        }

        self.uniform.data.tex_pos -= position;
        self.position = camera.position.xy() + (self.scale / 2.0); //se te position in the center
        self.uniform.data.set_position(self.position).exec();
    }
}
