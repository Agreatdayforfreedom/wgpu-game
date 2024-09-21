use cgmath::{Point2, Vector2};

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
    pub rotation: cgmath::Deg<f32>,
    pub interval: instant::Instant,
}

impl EvilShip {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
    ) -> Self {
        let mut uniform = Uniform::<EntityUniform>::new(&device);
        uniform
            .data
            .set_pivot(Point2::new(scale.x * 0.5, scale.y * 0.5))
            .exec();

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
            rotation: cgmath::Deg(0.0),
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
                CompassDir::from_deg(self.rotation.0),
                projectile_uniform,
            ));
        }
        None
    }
}

impl EvilShip {
    pub fn set_target_point(target: Vector2<f32>) {
        // self.po
    }
}

impl Entity for EvilShip {
    fn update(
        &mut self,
        dt: &instant::Duration,
        _input: &crate::input::Input,
        _audio: &mut Audio,
        _device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) {
        let dt = dt.as_secs_f32();
        // let dir = CompassDir::from_deg(self.rotation.0 + 180.0);
        // self.position.x += 50.0 * dir.dir.x * dt;
        // self.position.y -= 50.0 * dir.dir.y * dt;

        self.uniform
            .data
            .set_position(self.position)
            .set_scale(self.scale)
            .set_rotation(self.rotation)
            .exec();
        self.uniform.write(queue);
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn position(&self) -> cgmath::Vector2<f32> {
        self.position
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
