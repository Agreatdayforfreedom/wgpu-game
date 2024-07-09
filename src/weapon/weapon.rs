use std::slice::IterMut;

use super::projectile::Projectile;
use crate::{audio::Audio, collider::Bounds, input::Input};

pub trait Weapon {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        input: &Input,
        audio: &mut Audio,
    ) {
    }

    fn update(&mut self, queue: &mut wgpu::Queue, dt: &instant::Duration) {}

    fn drain(&mut self) {}

    fn get_projectiles(&mut self) -> IterMut<'_, Projectile>;

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {}
}
