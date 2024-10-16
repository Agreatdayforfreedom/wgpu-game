use cgmath::{Deg, Vector2};

use crate::{
    audio::Audio,
    collider::Bounds,
    entity::EntityUniform,
    explosion::{self, Explosion},
    uniform,
    util::{distance, CompassDir},
    weapon::projectile,
};

pub struct Projectile {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: Deg<f32>,

    /// When the projectile is active, it will be updated and drawn on the screen. </br>
    /// So we can avoid draw the projectile, but track the last position. </br>
    /// [default]: true
    active: bool,

    /// when the projectile is destroyed, it can be filtered. </br>
    /// [default]: false
    destroyed: bool,

    pub dir: CompassDir,
    pub bounds: Bounds,
    pub initial_position: Vector2<f32>,
    lifetime: instant::Instant,
    pub uniform: uniform::Uniform<EntityUniform>,
    target: Option<(u32, Vector2<f32>)>, // id, position
}

impl Projectile {
    pub fn new(
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: Deg<f32>,
        bounds: Bounds,
        dir: CompassDir,
        uniform: uniform::Uniform<EntityUniform>,
    ) -> Self {
        Self {
            position,
            scale,
            rotation: Deg(90.0) - rotation,
            bounds,
            dir,
            active: true,
            destroyed: false,
            initial_position: position,
            lifetime: instant::Instant::now(),
            uniform,
            target: None,
        }
    }

    // todo
    pub fn update(&mut self) {
        self.uniform
            .data
            .set_position(self.position)
            .set_rotation(self.rotation)
            .set_scale(self.scale)
            .exec();
    }

    pub fn set_direction<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        f(self);
    }

    pub fn set_target(&mut self, target_id: u32, target_pos: Vector2<f32>) {
        // is there a better way to do this? maybe
        let dist = distance(self.position, target_pos);
        if dist < 2.5 {
            self.active = false;
        }
        self.target = Some((target_id, target_pos));
    }

    pub fn has_target(&self) -> bool {
        if self.target.is_some() {
            self.target.unwrap().0 != 0
        } else {
            false
        }
    }

    /// get current target
    pub fn get_target(&self) -> (u32, Vector2<f32>) {
        self.target.unwrap()
    }

    /// set projectile bounds to collide
    pub fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    /// destroy the projectile
    pub fn destroy(&mut self) {
        self.destroyed = true;
    }

    pub fn is_active(&mut self) -> bool {
        self.active
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn lifetime(&self) -> u128 {
        self.lifetime.elapsed().as_millis()
    }

    /// desactive the projectile
    pub fn desactive(&mut self) {
        self.active = false;
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        if self.active {
            rpass.set_vertex_buffer(2, self.uniform.buffer.slice(..));
            rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }
}
