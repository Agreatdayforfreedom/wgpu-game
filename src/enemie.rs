use crate::entity::EntityUniform;
use crate::projectile;
use crate::uniform::Uniform;
pub struct Enemy {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    pub projectiles: Vec<projectile::Projectile>,
    pub interval: instant::Instant,
}

impl Enemy {
    pub fn new(position: cgmath::Vector2<f32>, uniform: Uniform<EntityUniform>) -> Self {
        Self {
            position,
            size: 24.0,
            alive: true,
            uniform,
            projectiles: vec![],
            interval: instant::Instant::now(),
        }
    }

    pub fn spawn_fire(&mut self, device: &wgpu::Device) -> Option<projectile::Projectile> {
        if self.interval.elapsed().as_millis() >= 500 {
            self.interval = instant::Instant::now();
            let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            self.projectiles.push(projectile::Projectile::new(
                (self.position.x + (self.size / 2.0) - 5.0, self.position.y).into(),
                10.0,
                projectile_uniform,
            ));
        }
        None
    }
}
