use std::{borrow::BorrowMut, ops::DerefMut};

use crate::{
    audio::Audio,
    camera::{self, Camera},
    collider::{check_collision, Bounds},
    enemie::{self, Enemy},
    entities::{
        evil_ship::{self, EvilShip},
        swift_ship::SwiftShip,
    },
    explosion::Explosion,
    input::{self, Input},
    player::Player,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::{distance, CompassDir},
    weapon::projectile::{self, Projectile},
};
use cgmath::{Angle, Deg, InnerSpace, Point2, SquareMatrix, Vector2, Vector3, Vector4};
use rand::Rng;

pub trait Entity {
    fn update(
        &mut self,
        _dt: &instant::Duration,
        _input: &Input,
        _audio: &mut Audio,
        _device: &wgpu::Device,
        _queue: &mut wgpu::Queue,
    ) {
    }

    fn alive(&self) -> bool {
        true
    }

    //todo: required
    fn position(&self) -> Vector2<f32> {
        Vector2::new(0.0, 0.0)
    }

    //todo: required
    fn rotation(&self) -> cgmath::Deg<f32> {
        cgmath::Deg(0.0)
    }

    fn center(&self) -> Vector2<f32> {
        Vector2::new(
            self.position().x + (self.scale().x / 2.0),
            self.position().y + (self.scale().y / 2.0),
        )
    }

    fn top_right(&self) -> Vector2<f32> {
        self.scale() / 2.0
    }
    fn top_left(&self) -> Vector2<f32> {
        Vector2::new(-self.scale().x, self.scale().y) / 2.0
    }
    fn bottom_right(&self) -> Vector2<f32> {
        Vector2::new(self.scale().x, -self.scale().y) / 2.0
    }
    fn bottom_left(&self) -> Vector2<f32> {
        -self.scale() / 2.0
    }

    /// get the position of any point from its own center, regardless of its orientation
    fn get_orientation_point(&self, point: Vector2<f32>) -> Vector2<f32> {
        let rotation = self.rotation().0.to_radians();
        let point_oriented_x = point.x * rotation.cos() - point.y * rotation.sin();
        let point_oriented_y = point.x * rotation.sin() + point.y * rotation.cos();

        (
            self.center().x + point_oriented_x,
            self.center().y + point_oriented_y,
        )
            .into()
    }

    //todo: required
    fn scale(&self) -> Vector2<f32> {
        Vector2::new(0.0, 0.0)
    }

    fn set_colors(&mut self, color: Vector4<f32>) {}

    fn destroy(&mut self) {}

    /// this method is used to set a target entity, move forward, shoot it, etc.
    fn set_target_point(&mut self, target: Vector2<f32>, dt: &instant::Duration) {}

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EntityUniform {
    pub model: cgmath::Matrix4<f32>,
    color: cgmath::Vector4<f32>,
    tex_scale: cgmath::Vector2<f32>,
    pub tex_pos: f32,
    position: cgmath::Vector2<f32>,
    angle: Deg<f32>,
    scale: cgmath::Vector2<f32>,
    pivot: cgmath::Point2<f32>,
}
unsafe impl bytemuck::Pod for EntityUniform {}
unsafe impl bytemuck::Zeroable for EntityUniform {}

impl Default for EntityUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity(),
            color: (1.0, 1.0, 1.0, 1.0).into(),
            tex_scale: (1.0, 1.0).into(), //TODO
            pivot: (0.5 * 24.0, 0.5 * 24.0).into(),
            tex_pos: 1.0,
            position: (0.0, 0.0).into(),
            angle: Deg(0.0),
            scale: (24.0, 24.0).into(),
        }
    }
}

impl EntityUniform {
    pub fn set_position(&mut self, position: cgmath::Vector2<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_rotation(&mut self, angle: Deg<f32>) -> &mut Self {
        self.angle = angle;
        self
    }

    pub fn set_scale(&mut self, scale: cgmath::Vector2<f32>) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn set_pivot(&mut self, pivot: cgmath::Point2<f32>) -> &mut Self {
        self.pivot = pivot;
        self
    }

    pub fn set_tex_scale(&mut self, scale: cgmath::Vector2<f32>) -> &mut Self {
        self.tex_scale = scale;
        self
    }

    pub fn set_color(&mut self, color: cgmath::Vector4<f32>) -> &mut Self {
        self.color = color;
        self
    }

    pub fn exec(&mut self) {
        self.model = cgmath::Matrix4::identity()
            * cgmath::Matrix4::from_translation((self.position.x, self.position.y, 0.0).into())
            * cgmath::Matrix4::from_translation((self.pivot.x, self.pivot.y, 0.0).into())
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_translation((-self.pivot.x, -self.pivot.y, 0.0).into())
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 0.0);
    }
}

//we do this for simplicity
pub struct EntityManager {
    // sprite_entities_group: Vec<(Option<Sprite>, Vec<Box<dyn Entity>>)>,
    player: Player,
    enemies: Vec<Box<dyn Entity>>,
    // projectiles: Vec<Projectile>, // this refer to enemy projectiles,
    explosions: Vec<Explosion>,
}

impl EntityManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let player = Player::new(&device, &queue);

        let mut enemies: Vec<Box<dyn Entity>> = vec![];

        //evil_ships
        // for _ in 0..5 {
        //     let position = (
        //         rand::thread_rng().gen_range(-400.0..400.0),
        //         rand::thread_rng().gen_range(-400.0..400.0),
        //     );

        //     let enemy = EvilShip::new(device, queue, position.into(), (61.0, 19.0).into());
        //     enemies.push(enemy);
        // }

        //shifr_ships
        for _ in 0..1 {
            let position = (
                rand::thread_rng().gen_range(-400.0..400.0),
                rand::thread_rng().gen_range(-400.0..400.0),
            );

            // let enemy = SwiftShip::new(device, queue, position.into(), (17.5, 20.0).into());
            let enemy = SwiftShip::new(device, queue, position.into(), (100.0, 100.0).into());
            enemies.push(enemy);
        }

        Self {
            player,
            enemies,
            // projectiles: vec![],
            explosions: vec![],
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        audio: &mut Audio,
        input_controller: &input::Input,
        camera: &mut Camera,
        dt: &instant::Duration,
    ) {
        self.player.uniform.write(queue);
        camera.update(Vector3::new(
            self.player.position.x,
            self.player.position.y,
            1.0,
        ));
        let mut min_dist = f32::MAX;
        for e in &mut self.enemies {
            if e.alive() {
                //TODO: THE DIRECTION SHOULD BE POINTING TO THE MOUSE?
                let dist = distance(self.player.position, e.position());
                // let dir = e.position() - self.player.position;
                let dx = (e.position().x + e.scale().x * 0.5) - self.player.position.x;
                // //set the point in the head
                let dy = (e.position().y + e.scale().y * 0.5) - (self.player.position.y - 0.5);

                let angle = dy.atan2(dx);

                let angle = angle * 180.0 / std::f32::consts::PI;

                if dist < min_dist {
                    self.player.rotation = cgmath::Deg(angle + 90.0); // adjust sprite rotation;
                    min_dist = dist;
                }
                e.set_target_point(self.player.position(), dt);
            }

            e.update(&dt, input_controller, audio, device, queue);
        }

        self.player
            .update(dt, input_controller, audio, device, queue);

        if self.player.active_weapon.get_name() == "laser" {
            for p in self.player.active_weapon.get_projectiles() {
                let mut min_dist = f32::MAX;
                for e in &mut self.enemies {
                    let dist = distance(self.player.position, e.position());
                    if dist < min_dist {
                        min_dist = dist;
                        p.set_direction(|this| {
                            let center = Vector2::new(
                                self.player.position.x + (self.player.scale.x / 2.0) - 10.0,
                                self.player.position.y + (self.player.scale.y / 2.0),
                            );

                            this.position.x =
                                center.x + self.player.scale.x / 2.0 * self.player.rotation.sin();
                            this.position.y =
                                center.y - self.player.scale.y / 2.0 * self.player.rotation.cos();
                            // Apply the rotation

                            this.rotation = self.player.rotation;
                            this.uniform
                                .data
                                .set_pivot(cgmath::Point2::new(0.5 * 20.0, 1.0))
                                .exec();
                            this.scale.x = 20.0;
                            this.scale.y = -min_dist;
                        });
                    }
                }
            }
        }
        //check collsions
        for p in &mut self.player.active_weapon.get_projectiles() {
            // let mut min_dist = f32::MAX;
            for e in &mut self.enemies {
                if !e.alive() {
                    continue;
                }
                if check_collision(
                    p.bounds,
                    Bounds {
                        origin: Point2::new(e.position().x, e.position().y),
                        area: Vector2::new(e.scale().x, e.scale().y),
                    },
                ) {
                    p.alive = false;
                    e.destroy();
                }
            }
        }

        self.explosions = self
            .explosions
            .drain(..)
            .filter(|e| e.end != true)
            .collect();
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        self.player.draw(rpass);

        for e in &mut self.enemies {
            e.draw(rpass);
        }
    }
}
