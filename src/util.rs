use cgmath::{Angle, InnerSpace};

#[derive(Debug, Clone, Copy)]
pub struct CompassDir {
    pub dir: cgmath::Vector2<f32>,
    #[allow(dead_code)]
    pub angle: cgmath::Deg<f32>,
}

impl CompassDir {
    pub fn from_deg(angle: f32) -> Self {
        let mut angle = cgmath::Deg(90.0 - angle);
        let dir = cgmath::Vector2 {
            x: angle.cos(),
            y: angle.sin(),
        }
        .normalize();

        if angle < cgmath::Deg(0.0) {
            angle += cgmath::Deg(360.0);
        }

        Self { dir, angle }
    }
    //todo
    pub fn rotate(&self, angle: f32) -> Self {
        let mut angle = self.angle + cgmath::Deg(angle);
        let dir = cgmath::Vector2 {
            x: angle.cos(),
            y: angle.sin(),
        }
        .normalize();

        if angle < cgmath::Deg(0.0) {
            angle += cgmath::Deg(360.0);
        }

        Self { dir, angle }
    }
}
