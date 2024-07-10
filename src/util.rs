use cgmath::{Angle, InnerSpace};

#[derive(Debug, Clone, Copy)]
pub struct CompassDir {
    pub dir: cgmath::Vector2<f32>,
    #[allow(dead_code)]
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
