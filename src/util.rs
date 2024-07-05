use cgmath::{Angle, InnerSpace};

//45.0
const COS_SIN_PI_4: f32 = 0.70710677;

//22.5
const SIN_PI_8: f32 = 0.38268343;
const COS_PI_8: f32 = 0.92387953;

#[derive(Debug, Clone, Copy)]
pub struct CompassDir {
    pub dir: cgmath::Vector2<f32>,
    pub angle: cgmath::Deg<f32>,
}

impl CompassDir {
    pub fn from_deg(angle: f32) -> Self {
        let angle = cgmath::Deg(angle);
        let dir = cgmath::Vector2 {
            x: angle.cos(),
            y: angle.sin(),
        }
        .normalize();

        Self { dir, angle }
    }
}
