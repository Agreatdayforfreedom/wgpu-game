use crate::{
    entity::EntityUniform,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};

pub struct Background {
    sprite: Sprite,
    pub uniform: Uniform<EntityUniform>,
}

impl Background {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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

        Self { sprite, uniform }
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        self.sprite.bind(rpass);
        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
