use crate::{
    audio::Audio,
    camera::{self, Camera},
    collider::{check_collision, Bounds},
    enemie::Enemy,
    entities::evil_ship::{self, EvilShip},
    explosion::Explosion,
    input::{self, Input},
    player::Player,
    rendering::{create_bind_group_layout, Sprite},
    uniform::Uniform,
    util::distance,
    weapon::projectile::{self, Projectile},
};
use cgmath::{Angle, Deg, Point2, SquareMatrix, Vector2, Vector3, Vector4};
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

    // fn id(&self) -> u32;

    fn position(&self) -> Vector2<f32> {
        Vector2::new(0.0, 0.0)
    }
    fn scale(&self) -> Vector2<f32> {
        Vector2::new(0.0, 0.0)
    }
    fn set_colors(&mut self, color: Vector4<f32>) {}

    fn rotate(&mut self, rotation: Deg<f32>) {}

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
    enemies: Vec<EvilShip>,
    s: Sprite,
    // projectiles: Vec<Projectile>, // this refer to enemy projectiles,
    explosions: Vec<Explosion>,
}

impl EntityManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let player = Player::new(&device, &queue);

        let bytes = include_bytes!("./assets/evil_ship.png");
        let evil_ship_sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToBorder,
            &create_bind_group_layout(device),
            bytes,
        );
        let position = (0.0 as f32, 300.0);

        let mut enemy = EvilShip::new(device, queue, position.into(), (24.0, 24.0).into());
        enemy
            .uniform
            .data
            .set_position(position.into())
            .set_scale((64.0, 64.0).into())
            .set_color((1.0, 1.0, 0.0, 1.0).into())
            .exec();
        Self {
            player,
            enemies: vec![enemy],
            s: evil_ship_sprite,
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
            //todo:
            let dist = distance(self.player.position, e.position());

            if dist < min_dist {
                let dx = e.position().x - self.player.position.x;
                //set the point in the head
                let dy = e.position().y - (self.player.position.y - 0.5);

                let angle = dy.atan2(dx);

                let angle = angle * 180.0 / std::f32::consts::PI;
                self.player.rotation = cgmath::Deg(angle + 90.0); // adjust sprite rotation;

                min_dist = dist;

                //todo
                e.update(&dt, input_controller, audio, device, queue);
            }
        }

        self.player
            .update(dt, input_controller, audio, device, queue);
        // for e in &mut self.enemies {
        //     if rand::thread_rng().gen_range(0..10000) < 1 {
        //         e.spawn_fire((40.0, 40.0).into(), audio, device);
        //     }
        // for p in &mut e.projectiles {
        //     if p.alive {
        //         // p.update(&dt, 275.0, self.player.position, ":D");
        //         p.uniform.write(queue);
        //     }
        // }

        // e.projectiles = e
        //     .projectiles
        //     .drain(..)
        //     .filter(|p| p.alive != false)
        //     .collect();
        // }
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
            let mut min_dist = f32::MAX;
            for e in &mut self.enemies {
                let dist = distance(self.player.position, e.position());
                if dist < min_dist {
                    min_dist = dist;
                }

                if check_collision(
                    p.bounds,
                    Bounds {
                        origin: Point2::new(e.position().x, e.position().y),
                        area: Vector2::new(e.scale().x, e.scale().y),
                    },
                ) {
                    // p.alive = false;
                    e.set_colors((1.0, 0.0, 0.0, 1.0).into());
                    // let explosion = Explosion::new(
                    //     e.position.into(),
                    //     (40.0, 40.0).into(),
                    //     device,
                    //     &self.queue,
                    // );
                    // self.explosions.push(explosion);
                    // self.audio.push(Sounds::Explosion);
                    // e.alive = false;
                } else {
                    e.set_colors((0.0, 1.0, 0.0, 1.0).into());
                }
            }
        }

        for e in &mut self.explosions {
            e.update(&dt);
            e.uniform.write(queue);
        }

        for e in &mut self.enemies {
            // for p in &mut e.projectiles {
            //     if check_collision(
            //         Bounds {
            //             origin: Point2::new(
            //                 p.position.x + p.scale.x / 2.0,
            //                 p.position.y + p.scale.y / 2.0,
            //             ),
            //             area: Vector2::new(2.5, 2.5),
            //         },
            //         Bounds {
            //             origin: Point2::new(self.player.position.x, self.player.position.y),
            //             area: Vector2::new(self.player.scale.x, self.player.scale.y),
            //         },
            //     ) {
            //         p.alive = false;
            //         self.player.alive = false;
            //     }
            // }
        }

        self.player.active_weapon.drain();

        self.enemies = self
            .enemies
            .drain(..)
            .filter(|e| e.alive != false)
            .collect();

        self.explosions = self
            .explosions
            .drain(..)
            .filter(|e| e.end != true)
            .collect();
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        self.player.draw(rpass);

        self.s.bind(rpass);
        for e in &mut self.enemies {
            e.draw(rpass)
        }
        // for p in &mut self.projectiles {
        //     p.draw(rpass);
        // }
    }
}
