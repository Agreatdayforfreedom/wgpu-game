use crate::{
    audio::{Audio, Sounds},
    entity::EntityUniform,
    input::Input,
    sprite_renderer::SpriteRenderer,
    util::CompassDir,
};

use super::projectile::Projectile;
use super::weapon::Weapon;
pub struct Cannon {
    pub projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128, // milliseconds
    sprite: SpriteRenderer,
}

impl Cannon {
    pub fn new(shooting_interval: u128, device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullet.png");
        let sprite = SpriteRenderer::new(&device, &queue, diffuse_bytes);

        Box::new(Self {
            projectiles: vec![],
            time: instant::Instant::now(),
            shooting_interval,
            sprite,
        })
    }
}

impl Weapon for Cannon {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        input: &Input,
        audio: &mut Audio,
    ) {
        if input.is_pressed("f") && self.time.elapsed().as_millis() >= self.shooting_interval {
            self.time = instant::Instant::now();
            let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            audio.push(Sounds::Shoot);
            let p = Projectile::new(
                (position.x - 2.0, position.y).into(),
                scale,
                cgmath::Deg(-90.0),
                CompassDir::from_deg(90.0),
                projectile_uniform,
            );

            self.projectiles.push(p);
        };
    }

    fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {
        println!("len: {}", self.projectiles.len());
        for projectile in &mut self.projectiles {
            if projectile.alive {
                projectile.update(&dt, 500.0);
                projectile.uniform.write(queue);
            }
        }
    }
    fn drain(&mut self) {
        self.projectiles = self
            .projectiles
            .drain(..)
            .filter(|p| p.alive != false)
            .collect();
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
