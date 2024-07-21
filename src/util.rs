use cgmath::{Angle, InnerSpace};

#[derive(Debug, Clone, Copy)]
pub struct CompassDir {
    pub dir: cgmath::Vector2<f32>,
    #[allow(dead_code)]
    pub angle: cgmath::Deg<f32>,
}

impl CompassDir {
    pub fn from_deg(angle: f32) -> Self {
        let angle = cgmath::Deg(90.0 - angle);
        Self::from_angle(angle)
    }

    pub fn rotate(&self, angle: f32) -> Self {
        let angle = self.angle + cgmath::Deg(angle);
        Self::from_angle(angle)
    }

    #[inline]
    fn from_angle(mut angle: cgmath::Deg<f32>) -> Self {
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
