use crate::audio::{Audio, Sounds};
use crate::collider::Bounds;
use crate::entity::EntityUniform;
use crate::uniform::Uniform;
use crate::util::CompassDir;
use crate::weapon::projectile;
use crate::weapon::projectile::Projectile;
pub struct Enemy {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    pub projectiles: Vec<Projectile>,
    pub interval: instant::Instant,
}

impl Enemy {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        uniform: Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            alive: true,
            uniform,
            projectiles: vec![],
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
            self.projectiles.push(Projectile::new(
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
