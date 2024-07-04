use cgmath::InnerSpace;

const SIN_PI_4: f32 = 0.70710677;
const SIN_PI_3: f32 = 0.16602540;

#[derive(Debug, Clone, Copy)]
pub enum CompassDir {
    North,
    NorthEast,
    NNE,
    East,
    NorthWest,
    South,
    West,
}

impl CompassDir {
    pub fn to_dir(self) -> cgmath::Vector2<f32> {
        match self {
            Self::North => cgmath::Vector2::new(0.0, 1.0),
            Self::NorthEast => cgmath::Vector2::new(SIN_PI_4, SIN_PI_4).normalize(),
            Self::NNE => cgmath::Vector2::new(SIN_PI_3, SIN_PI_3).normalize(),
            Self::East => cgmath::Vector2::new(1.0, 0.0),
            Self::NorthWest => cgmath::Vector2::new(-SIN_PI_4, SIN_PI_4).normalize(),
            Self::South => cgmath::Vector2::new(0.0, -1.0),
            Self::West => cgmath::Vector2::new(-1.0, 0.0),
        }
    }
}
