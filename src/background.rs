use std::{time::Duration, vec};

use cgmath::{InnerSpace, Vector2};

use crate::{
    camera,
    entity::{Entity, EntityUniform},
    input,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};

const LAYERS_SPEED: [f32; 3] = [0.01, 0.025, 0.05];

pub struct Background {
    id: u32,
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: cgmath::Deg<f32>,
    uniforms: Vec<(Sprite, Uniform<EntityUniform>)>,
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
        for (sprite, uniform) in &mut self.uniforms {
            sprite.bind(rpass);
            rpass.set_bind_group(2, &uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}

impl Background {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let mut uniforms = vec![];
        let diffuse_bytes = include_bytes!("./assets/nebula.png");
        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &create_bind_group_layout(device),
            diffuse_bytes,
        );
        let mut uniform = Uniform::<EntityUniform>::new(&device);
        uniform.data.set_scale((1200.0, 800.0).into()).exec();
        uniform.data.set_tex_scale((1.0, 1.0).into()).exec();
        uniform.data.set_rotation(cgmath::Deg(180.0)).exec();

        uniforms.push((sprite, uniform));
        let diffuse_bytes = include_bytes!("./assets/stars.png");

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

        uniforms.push((sprite, uniform));

        let diffuse_bytes = include_bytes!("./assets/big-stars.png");

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

        uniforms.push((sprite, uniform));

        Box::new(Self {
            id: 10000, //TODO
            uniforms,
            scale: (1200.0, 800.0).into(),
            position: (0.0, 0.0).into(),
            rotation: cgmath::Deg(0.0),
        })
    }

    pub fn update(
        &mut self,
        queue: &mut wgpu::Queue,
        camera: &camera::Camera,
        input: &input::Input,
        dt: &Duration,
    ) {
        for (i, (_, uniform)) in &mut self.uniforms.iter_mut().enumerate() {
            let dt = dt.as_secs_f32();
            let mut position = Vector2::new(0.0, 0.0);
            if input.is_pressed("d") {
                position.x += LAYERS_SPEED[i] * dt;
            }
            if input.is_pressed("a") {
                position.x -= LAYERS_SPEED[i] * dt;
            }
            if input.is_pressed("w") {
                position.y += LAYERS_SPEED[i] * dt;
            }
            if input.is_pressed("s") {
                position.y -= LAYERS_SPEED[i] * dt;
            }
            uniform.data.tex_pos -= position;
            uniform
                .data
                .set_position(camera.position.xy() + (self.scale / 2.0))
                .exec();

            uniform.write(queue);
        }
        // self.position = ; //se te position in the center
    }
}
