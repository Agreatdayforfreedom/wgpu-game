use crate::{
    audio::Audio,
    camera::Camera,
    collider::{check_collision, Bounds},
    entities::{evil_ship::EvilShip, swift_ship::SwiftShip},
    explosion::{ExpansiveWave, Explosion, ExplosionManager},
    input::{self, Input},
    particle_system::{
        simulation_params::{Circle, SimulationParams},
        system::ParticleSystem,
    },
    player::Player,
    util::{distance, CompassDir, IdVendor},
    weapon,
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

    fn position(&self) -> Vector2<f32>;

    fn rotation(&self) -> cgmath::Deg<f32>;

    fn scale(&self) -> Vector2<f32>;

    fn id(&self) -> u32;

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

    /// Get the position of any point from its own center, regardless of its orientation.
    /// You can get the [point] parameter from the corner methods already implemented.
    /// To get the center between two corners, or the center of the entity itself, set an axis to 1.0 (or two to get the center).
    /// ### Example
    /// ```
    /// let top_left_corner = entity.get_orientation_point(entity.top_left());
    ///
    /// let entity_center = self.get_orientation_point((0.0, 0.0).into());
    /// ```
    fn get_orientation_point(&self, point: Vector2<f32>) -> Vector2<f32> {
        let rotation = self.rotation().0.to_radians();
        let point_oriented_x = point.x * rotation.cos() - point.y * rotation.sin();
        let point_oriented_y = point.x * rotation.sin() + point.y * rotation.cos();
        (
            self.position().x + point_oriented_x,
            self.position().y + point_oriented_y,
        )
            .into()
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
    pub tex_pos: cgmath::Vector2<f32>,
    pub position: cgmath::Vector2<f32>,
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
            pivot: (0.5, 0.5).into(),
            tex_pos: (1.0, 1.0).into(),
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
            * cgmath::Matrix4::from_translation(((0.5), (0.5), 0.0).into())
            * cgmath::Matrix4::from_angle_z(self.angle)
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
            * cgmath::Matrix4::from_translation((-(0.5), -(0.5), 0.0).into());
    }
}

#[allow(dead_code)]
pub struct EntityManager {
    id_vendor: IdVendor,
    pub player: Player,
    enemies: Vec<Box<dyn Entity>>,
    explosion_manager: ExplosionManager,
}

impl EntityManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        particle_system: &mut ParticleSystem,
    ) -> Self {
        let mut id_vendor = IdVendor::default();
        let player = Player::new(&device, &queue, id_vendor.next_id());
        particle_system.push_group(
            player.id(),
            device,
            100,
            (0.0, 0.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        );
        particle_system.push_group(
            player.id() + 1,
            device,
            100,
            (0.0, 0.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        );
        let mut enemies: Vec<Box<dyn Entity>> = vec![];

        //evil_ships
        for _ in 0..5 {
            let position = (
                rand::thread_rng().gen_range(-400.0..400.0),
                rand::thread_rng().gen_range(-400.0..400.0),
            );

            let enemy = EvilShip::new(
                device,
                queue,
                id_vendor.next_id(),
                position.into(),
                (61.0, 19.0).into(),
            );

            // particle_system.push_group(
            //     enemy.id(),
            //     device,
            //     1000,
            //     (0.0, 0.0).into(),
            //     (0.0, 1.0, 0.5, 1.0).into(),
            // );

            enemies.push(enemy);
        }

        particle_system.push_group(
            10,
            device,
            1000,
            (-200.0, -200.0).into(),
            (1.0, 1.0, 1.0, 1.0).into(),
        );

        //swift_ships
        for _ in 0..5 {
            let position = (
                rand::thread_rng().gen_range(-400.0..400.0),
                rand::thread_rng().gen_range(-400.0..400.0),
            );

            let enemy = SwiftShip::new(
                device,
                queue,
                id_vendor.next_id(),
                position.into(),
                (17.5, 20.0).into(),
            );
            // particle_system.push_group(
            //     enemy.id(),
            //     device,
            //     1000,
            //     (0.0, 0.0).into(),
            //     (1.0, 0.5, 0.0, 1.0).into(),
            // );
            enemies.push(enemy);
        }

        Self {
            id_vendor,
            player,
            enemies,
            explosion_manager: ExplosionManager::new(device, queue),
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
        particle_system: &mut ParticleSystem,
    ) {
        self.player
            .update(dt, input_controller, audio, device, queue);

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
            // particle_system.update_sim_params(
            //     queue,
            //     e.id(),
            //     &e.get_orientation_point((0.0, e.top_right().y).into()),
            //     e.rotation(),
            //     (0.0, 1.0, 0.0, 1.0).into(),
            //     dt,
            // );
        }

        for weapon in &mut self.player.active_weapons {
            if weapon.get_name() == "laser" {
                for p in weapon.get_projectiles() {
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

                                this.position.x = center.x
                                    + self.player.scale.x / 2.0 * self.player.rotation.sin();
                                this.position.y = center.y
                                    - self.player.scale.y / 2.0 * self.player.rotation.cos();
                                // Apply the rotation

                                this.rotation = self.player.rotation;

                                this.scale.x = 20.0;
                                this.scale.y = -min_dist;
                            });
                        }
                    }
                }
            }
            if weapon.get_name() == "homing_missile" {
                for p in weapon.get_projectiles() {
                    // p.set_explosion(device, queue, false);
                    let mut min_dist = f32::MAX;
                    // if there is no target, we set the closer enemy as target.
                    // we keep (in the else statement) the same target regardless if the it's far away then any other enemy
                    if !p.has_target() {
                        for e in &mut self.enemies {
                            if e.alive() {
                                let dist = distance(self.player.position, e.position());
                                if dist < min_dist {
                                    min_dist = dist;
                                    p.set_target(e.id(), e.position());
                                }
                            }
                        }
                    } else {
                        for e in &mut self.enemies {
                            if e.id() == p.get_target().0 {
                                p.set_target(e.id(), e.position());
                            }
                        }
                    }
                }
            }

            for p in &mut weapon.get_projectiles() {
                if !p.alive && !p.destroyed {
                    audio.push(crate::audio::Sounds::Explosion, 0.2);
                    self.explosion_manager.add(
                        Explosion::new(p.position, (40.0, 40.0).into(), device, queue),
                        Some(ExpansiveWave::new_at(p.position, device)),
                    );
                    p.destroy();
                }
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

                        if !e.alive() {
                            audio.push(crate::audio::Sounds::Explosion, 0.2);
                            self.explosion_manager.add(
                                Explosion::new(e.position(), (40.0, 40.0).into(), device, queue),
                                None,
                            );
                        }
                    }
                }
            }
        }

        self.explosion_manager.update(queue, dt);

        let pos = self.player.get_orientation_point(
            (
                self.player.top_right().x - 12.0,
                self.player.top_right().y - 15.0,
            )
                .into(),
        );

        particle_system.update_sim_params(
            self.player.id(),
            SimulationParams {
                total: 100.0,
                position: pos,
                dir: CompassDir::from_deg(self.player.rotation().opposite().0).dir,
                color: (52.0 / 255.0, 76.0 / 255.0, 235.0 / 255.0, 1.0).into(),
                rate_over_distance: 7.0,
                color_over_lifetime: 1.0,
                // interval: dt.as_secs_f32(),
                lifetime_factor: 0.25,
                start_speed: 0.0,
                shape_selected: 0,
                circle: Circle {
                    radius: 1.5,
                    emit_from_edge: 0,
                },
                ..Default::default()
            },
        );
        let pos = self.player.get_orientation_point(
            (
                self.player.top_left().x + 12.0,
                self.player.top_left().y - 15.0,
            )
                .into(),
        );
        particle_system.update_sim_params(
            self.player.id() + 1,
            SimulationParams {
                total: 100.0,
                position: pos,
                dir: CompassDir::from_deg(self.player.rotation().opposite().0).dir,
                color: (52.0 / 255.0, 76.0 / 255.0, 235.0 / 255.0, 1.0).into(),
                rate_over_distance: 7.0,
                color_over_lifetime: 1.0,
                // delta_time: dt.as_secs_f32(),
                lifetime_factor: 0.25,
                start_speed: 0.0,
                shape_selected: 0,
                circle: Circle {
                    radius: 1.5,
                    emit_from_edge: 0,
                },
                ..Default::default()
            },
        );

        particle_system.update_sim_params(
            10,
            SimulationParams {
                total: 1000.0,
                position: (0.0, 0.0).into(),
                dir: (0.0, 1.0).into(),
                color: (225.0 / 255.0, 69.0 / 255.0, 0.0 / 255.0, 1.0).into(),
                rate_over_distance: -1.0,
                color_over_lifetime: 1.0,
                // interval: dt.as_secs_f32(),
                lifetime_factor: 10000.0,
                start_speed: 0.0,
                mode: 1,
                shape_selected: 0,
                circle: Circle {
                    radius: 10.0,
                    emit_from_edge: 1,
                },
                ..*particle_system.get_sim_params(10) // ..Default::default()
            },
        );
    }

    pub fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        self.explosion_manager.draw(rpass);
        self.player.draw(rpass);

        for e in &mut self.enemies {
            e.draw(rpass);
        }
    }
}
