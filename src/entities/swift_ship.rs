use crate::{
    ai::patrol_area::PatrolArea, entity::EntityUniform, explosion::Explosion, rendering::Sprite,
    uniform::Uniform, weapon::weapon::Weapon,
};

pub struct SwiftShip {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub alive: bool,
    pub uniform: Uniform<EntityUniform>,
    // pub projectiles: (Sprite, Vec<Projectile>),
    explosion: Explosion,
    weapon: Box<dyn Weapon>,
    pub rotation: cgmath::Deg<f32>,
    sprite: Sprite, //todo
    targeting: bool,
    patrol: PatrolArea,
}
