use rodio::queue;
use wgpu::core::device;

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::{Entity, EntityUniform},
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::CompassDir,
    weapon::projectile::Projectile,
};

pub struct EvilShip {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    pub projectiles: (Sprite, Vec<Projectile>),
    pub interval: instant::Instant,
}

impl EvilShip {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Self {
        let uniform = Uniform::<EntityUniform>::new(&device);

        let bytes = include_bytes!("../assets/bullet.png");
        let projectile_sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &create_bind_group_layout(device),
            bytes,
        );
        Self {
            position,
            scale,
            alive: true,
            uniform,
            projectiles: (projectile_sprite, vec![]),
            interval: instant::Instant::now(),
        }
    }

    pub fn spawn_fire(
        &mut self,
        scale: cgmath::Vector2<f32>,
        audio: &mut Audio,
        device: &wgpu::Device,
    ) -> Option<Projectile> {
        if self.interval.elapsed().as_millis() >= 500 {
            self.interval = instant::Instant::now();
            let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            audio.push(Sounds::Shoot);
            self.projectiles.1.push(Projectile::new(
                (
                    self.position.x + (self.scale.x / 2.0) - (scale.x / 2.0),
                    self.position.y,
                )
                    .into(),
                scale,
                cgmath::Deg(90.0),
                Bounds {
                    area: scale,
                    origin: cgmath::Point2 {
                        x: self.position.x,
                        y: self.position.y,
                    },
                },
                CompassDir::from_deg(270.0),
                projectile_uniform,
            ));
        }
        None
    }
}

impl Entity for EvilShip {
    fn update(
        &mut self,
        _dt: &instant::Duration,
        _input: &crate::input::Input,
        _audio: &mut Audio,
        _device: &wgpu::Device,
        _queue: &mut wgpu::Queue,
    ) {
        self.uniform.write(_queue);
    }

    fn alive(&self) -> bool {
        self.alive
    }
    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
        rpass.draw(0..6, 0..1);
        self.projectiles.0.bind(rpass);
        for p in &mut self.projectiles.1 {
            p.draw(rpass);
        }
    }
}
