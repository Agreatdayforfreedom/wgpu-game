#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Player {
    pub position: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Player {}
unsafe impl bytemuck::Zeroable for Player {}

impl Player {
    pub fn new() -> Self {
        Self {
            position: cgmath::Vector2::new(10.0, 10.0),
        }
    }

    pub fn update(&mut self) {
        self.position += (0.02, 0.02).into();
        // println!("{:?}", &self.position);
    }
}
