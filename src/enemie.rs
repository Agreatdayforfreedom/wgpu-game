use cgmath::SquareMatrix;

use crate::projectile;
use crate::uniform::Uniform;
pub struct Enemy {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub alive: bool,
    pub uniform: Uniform<EnemyUniform>,
    pub projectiles: Vec<projectile::Projectile>,
    pub interval: instant::Instant,
}

impl Enemy {
    pub fn new(position: cgmath::Vector2<f32>, uniform: Uniform<EnemyUniform>) -> Self {
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
            let projectile_uniform =
                crate::uniform::Uniform::<projectile::ProjectileUniform>::new(&device);
            self.projectiles.push(projectile::Projectile::new(
                (self.position.x + (self.size / 2.0) - 5.0, self.position.y).into(),
                10.0,
                projectile_uniform,
            ));
        }
        None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EnemyUniform {
    pub model: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for EnemyUniform {}
unsafe impl bytemuck::Zeroable for EnemyUniform {}

impl Default for EnemyUniform {
    fn default() -> Self {
        let model = cgmath::Matrix4::identity();

        Self { model }
    }
}

impl EnemyUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((position.x, position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(24.0);
    }
}
