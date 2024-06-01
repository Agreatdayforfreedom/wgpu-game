use cgmath::SquareMatrix;

pub struct Projectile {
    position: cgmath::Vector2<f32>,
    scale: f32,
}

impl Projectile {
    pub fn new(position: cgmath::Vector2<f32>, scale: f32) -> Self {
        Self { position, scale }
    }

    pub fn update(&mut self, dt: &instant::Duration) -> cgmath::Matrix4<f32> {
        let model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_scale(self.scale);

        model
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ProjectileUniform {
    pub model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for ProjectileUniform {}
unsafe impl bytemuck::Zeroable for ProjectileUniform {}

impl Default for ProjectileUniform {
    fn default() -> Self {
        let model = cgmath::Matrix4::identity();
        Self { model }
    }
}

impl ProjectileUniform {
    pub fn update(&mut self, projectile: &mut Projectile, dt: &instant::Duration) {
        self.model = projectile.update(dt);
    }
}
