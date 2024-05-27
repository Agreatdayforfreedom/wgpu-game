use cgmath::{SquareMatrix, Vector2};

use crate::input::Input;

pub struct Player {
    pub position: cgmath::Vector2<f32>,
    scale: f32,
}

const SPEED: f32 = 500.0;
impl Player {
    pub fn new() -> Self {
        Self {
            position: cgmath::Vector2::new(400.0, 550.0),
            scale: 40.0,
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
