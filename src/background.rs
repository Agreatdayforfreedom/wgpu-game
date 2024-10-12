use std::{time::Duration, vec};

use cgmath::{vec4, InnerSpace, Vector2};

use crate::{
    camera,
    entity::{Entity, EntityUniform},
    input,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::distance,
};

fn create_layer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scale: Vector2<f32>,
    tex_scale: Vector2<f32>,
    layer_speed: f32,
    bytes: &[u8],
) -> (Sprite, f32, Uniform<EntityUniform>) {
    let sprite = Sprite::new(
        device,
        queue,
        wgpu::AddressMode::ClampToEdge,
        &create_bind_group_layout(device),
        bytes,
    );
    let mut uniform = Uniform::<EntityUniform>::new(&device);
    uniform
        .data
        .set_scale(scale)
        .set_tex_scale(tex_scale)
        .set_rotation(cgmath::Deg(180.0))
        .exec();

    (sprite, layer_speed, uniform)
}

pub struct Background {
    id: u32,
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: cgmath::Deg<f32>,
    uniforms: Vec<(Sprite, f32, Uniform<EntityUniform>)>,
    prev_pos: Vector2<f32>,
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
        for (sprite, _, uniform) in &mut self.uniforms {
            sprite.bind(rpass);
            rpass.set_bind_group(2, &uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}

impl Background {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let mut uniforms = vec![];

        let bytes = include_bytes!("./assets/nebula.png");
        let layer = create_layer(
            device,
            queue,
            (1200.0, 800.0).into(),
            (1.0, 1.0).into(),
            0.01,
            bytes,
        );
        uniforms.push(layer);

        let bytes = include_bytes!("./assets/planet.png");
        let layer = create_layer(
            device,
            queue,
            (128.0, 128.0).into(),
            (1.0, 1.0).into(),
            475.0,
            bytes,
        );
        uniforms.push(layer);

        let bytes = include_bytes!("./assets/deep-asteroids.png");
        let layer = create_layer(
            device,
            queue,
            (1200.0, 800.0).into(),
            (2.0, 2.0).into(),
            0.0125,
            bytes,
        );
        uniforms.push(layer);

        let bytes = include_bytes!("./assets/stars.png");
        let layer = create_layer(
            device,
            queue,
            (1200.0, 800.0).into(),
            (2.0, 2.0).into(),
            0.025,
            bytes,
        );
        uniforms.push(layer);

        let bytes = include_bytes!("./assets/big-stars.png");
        let layer = create_layer(
            device,
            queue,
            (1200.0, 800.0).into(),
            (2.0, 2.0).into(),
            0.05,
            bytes,
        );
        uniforms.push(layer);

        let bytes = include_bytes!("./assets/near-asteroids.png");
        let layer = create_layer(
            device,
            queue,
            (1200.0, 800.0).into(),
            (2.0, 2.0).into(),
            0.06,
            bytes,
        );
        uniforms.push(layer);

        Box::new(Self {
            id: 10000, //TODO
            uniforms,
            scale: (1200.0, 800.0).into(),
            position: (0.0, 0.0).into(),
            rotation: cgmath::Deg(0.0),
            prev_pos: (0.0, 0.0).into(),
        })
    }

    pub fn update(
        &mut self,
        queue: &mut wgpu::Queue,
        camera: &camera::Camera,
        input: &input::Input,
        dt: &Duration,
    ) {
        // let camera_pos = camera.position.xy();

        for (_, speed, uniform) in &mut self.uniforms {
            let dt = dt.as_secs_f32();
            let speed = *speed;
            let mut position = Vector2::new(0.0, 0.0);
            if input.is_pressed("d") {
                position.x += speed * dt;
            } else if input.is_pressed("a") {
                position.x -= speed * dt;
            } else if input.is_pressed("w") {
                position.y += speed * dt;
            } else if input.is_pressed("s") {
                position.y -= speed * dt;
            }

            if speed > 1.0 {
                // let position = position.magnitude().clamp(0.0, 1.0);
                if position.x != 0.0 && position.y != 0.0 {
                    position.normalize();
                }

                let px = uniform.data.position.x + position.x;
                let py = uniform.data.position.y + position.y;
                uniform.data.set_position((px, py).into()).exec();
            } else {
                uniform.data.tex_pos -= position;
                uniform
                    .data
                    .set_position(camera.position.xy() + (self.scale / 2.0))
                    .exec();
            }
            uniform.write(queue);
        }
    }
}
