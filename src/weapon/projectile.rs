use crate::{
    collider::Bounds, entity::EntityUniform, uniform, util::CompassDir, weapon::projectile,
};

pub struct Projectile {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub dir: CompassDir,
    pub bounds: Bounds,
    pub rotation: cgmath::Deg<f32>,
    pub initial_position: cgmath::Vector2<f32>,
    pub lifetime: instant::Instant,
    pub uniform: uniform::Uniform<EntityUniform>,
}

impl Projectile {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        rotation: cgmath::Deg<f32>,
        bounds: Bounds,
        dir: CompassDir,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            rotation: cgmath::Deg(90.0) - rotation,
            bounds,
            dir,
            alive: true,
            initial_position: position,
            lifetime: instant::Instant::now(),
            uniform,
        }
    }
    // todo
    pub fn update(
        &mut self,
        dt: &instant::Duration,
        fire_speed: f32,
        entity_position: cgmath::Vector2<f32>,
    ) {
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
        // if self.position.y < 0.0 || self.position.y > 600.0 {
        //     self.alive = false; // todo life bounds
    }

    pub fn set_direction<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        f(self);
    }

    pub fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    pub fn draw<'a, 'b>(&'a self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.alive {
            rpass.set_vertex_buffer(2, self.uniform.buffer.slice(..));
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}
