use std::slice::IterMut;

use super::projectile::Projectile;
use crate::{
    audio::Audio,
    collider::Bounds,
    input::Input,
    player::{self, Player},
    util::CompassDir,
};

pub trait Weapon {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: &Vec<cgmath::Vector2<f32>>,
        scale: cgmath::Vector2<f32>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
    ) {
    }

    fn update(
        &mut self,
        position: &Vec<cgmath::Vector2<f32>>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
    ) {
    }

    fn get_projectiles(&mut self) -> IterMut<'_, Projectile>;

    fn get_name(&self) -> &str {
        ""
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {}
}
