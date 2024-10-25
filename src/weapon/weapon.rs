use std::slice::IterMut;

use super::projectile::Projectile;
use crate::{
    audio::Audio,
    input::Input,
    particle_system::{self, system::ParticleSystem},
    util::{CompassDir, IdVendor},
};

pub trait Weapon {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        position: cgmath::Vector2<f32>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
    ) {
    }

    fn update(
        &mut self,
        position: cgmath::Vector2<f32>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        particle_system: &mut ParticleSystem,
    ) {
    }

    fn get_projectiles(&mut self) -> IterMut<'_, Projectile>;

    fn set_target(&mut self, target_id: u32, target_pos: cgmath::Vector2<f32>) {}

    fn has_target(&self) -> bool {
        false
    }

    fn get_target(&self) -> (u32, cgmath::Vector2<f32>) {
        (0u32, (0.0, 0.0).into())
    }

    fn get_name(&self) -> &str {
        ""
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {}
}
