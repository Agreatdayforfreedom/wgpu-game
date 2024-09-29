use cgmath::Vector2;

use crate::{
    entity::{Entity, EntityUniform},
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};

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
        uniform
            .data
            .set_scale((856.0 * 2.0, 375.0 * 2.0).into())
            .exec();

        Box::new(Self {
            id: 10000, //TODO
            sprite,
            uniform,
            scale: (856.0 * 2.0, 375.0 * 2.0).into(),
            position: (0.0, 0.0).into(),
            rotation: cgmath::Deg(0.0),
        })
    }
}
