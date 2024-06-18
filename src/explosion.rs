use crate::entity::EntityUniform;
use crate::uniform;

const TIME_TO_NEXT_FRAME: f32 = 2.0 / 30.0;

pub struct Explosion {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub play: bool,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub i: u32,
    time_to_next_frame: f32,
}

impl Explosion {
    pub fn new(
        position: cgmath::Vector2<f32>,
        size: f32,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            size,
            play: false,
            uniform,
            i: 0,
            time_to_next_frame: 0.0,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration) {
        self.time_to_next_frame += dt.as_secs_f32();
        if self.time_to_next_frame > TIME_TO_NEXT_FRAME {
            if self.i == 4 {
                self.i = 0;
            } else {
                self.i += 1;
                self.time_to_next_frame = 0.0;
            }
        }
    }
}
