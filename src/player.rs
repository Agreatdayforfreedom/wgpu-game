use crate::input::Input;
use crate::projectile;
use cgmath::{SquareMatrix, Vector2};

pub struct Player {
    pub position: cgmath::Vector2<f32>,
    scale: f32,
    interval: instant::Instant,
}

const SPEED: f32 = 500.0;
impl Player {
    pub fn new(position: cgmath::Vector2<f32>) -> Self {
        Self {
            position,
            scale: 40.0,
            interval: instant::Instant::now(),
        }
    }

    pub fn update(&mut self, dt: &instant::Duration, input: &Input) -> cgmath::Matrix4<f32> {
        let model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(self.scale);
        if input.is_pressed("d") {
            self.movement("d", dt);
        } else if input.is_pressed("a") {
            self.movement("a", dt);
        } else if input.is_pressed("s") {
            self.movement("s", dt);
        } else if input.is_pressed("w") {
            self.movement("w", dt);
        }

        model
    }

    pub fn movement(&mut self, key: &str, dt: &instant::Duration) {
        let dt = dt.as_secs_f32();
        let mut position = Vector2::new(0.0, 0.0);
        if key == "d" {
            position.x += SPEED * dt;
        }
        if key == "a" {
            position.x -= SPEED * dt;
        }
        if key == "w" {
            position.y -= SPEED * dt;
        }
        if key == "s" {
            position.y += SPEED * dt;
        }
        self.position += position;
    }

    pub fn spawn_fire(
        &mut self,
        device: &wgpu::Device,
        input: &Input,
    ) -> Option<projectile::Projectile> {
        if input.is_pressed("f") && self.interval.elapsed().as_millis() >= 500 {
            self.interval = instant::Instant::now();
            let projectile_uniform =
                crate::uniform::Uniform::<projectile::ProjectileUniform>::new(&device);
            return Some(projectile::Projectile::new(
                (self.position.x + (self.scale / 2.0) - 5.0, self.position.y).into(),
                10.0,
                projectile_uniform,
            ));
        }
        None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PlayerUniform {
    pub model: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for PlayerUniform {}
unsafe impl bytemuck::Zeroable for PlayerUniform {}

impl Default for PlayerUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
        }
    }
}

impl PlayerUniform {
    pub fn update(&mut self, player: &mut Player, dt: &instant::Duration, input: &Input) {
        self.model = player.update(dt, input); // ??????
    }
}
