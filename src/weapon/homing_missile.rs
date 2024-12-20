use cgmath::{InnerSpace, Vector2, VectorSpace};

use crate::{
    audio::Audio,
    collider::Bounds,
    entity::EntityUniform,
    explosion::ExplosionType,
    input::Input,
    particle_system::{
        simulation_params::{Cone, SimulationParams},
        system::ParticleSystem,
    },
    rendering::{create_bind_group_layout, Sprite},
    util::{CompassDir, IdVendor},
};

use super::{projectile::Projectile, weapon::Weapon};

const LIFETIME: u128 = 5000;
const SCALE: Vector2<f32> = Vector2::new(15.0, 21.0);
const DIRS: [f32; 5] = [110.0, 140.0, 180.0, 210.0, 240.0];

pub struct HomingMissile {
    pub projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128, // milliseconds
    sprite: Sprite,
    auto: bool,
}

impl HomingMissile {
    pub fn new(
        shooting_interval: u128,
        auto: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullets/brocket.png");
        let bind_group_layout = create_bind_group_layout(device);

        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::ClampToEdge,
            &bind_group_layout,
            diffuse_bytes,
        );

        Box::new(Self {
            projectiles: vec![],
            time: instant::Instant::now(),
            shooting_interval,
            sprite,
            auto,
        })
    }
}

impl Weapon for HomingMissile {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        positions: Vec<cgmath::Vector2<f32>>,

        dir: CompassDir,
        input: &Input,
        _audio: Option<&mut Audio>,
        id_vendor: &mut IdVendor,
        particle_system: &mut ParticleSystem,
    ) {
        let position = *positions.get(0).unwrap();
        if (input.is_pressed("f") || self.auto)
            && self.time.elapsed().as_millis() >= self.shooting_interval
        {
            self.time = instant::Instant::now();
            for _dir in DIRS {
                let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
                let pid = id_vendor.next_id();
                particle_system.push_group(
                    pid,
                    device,
                    SimulationParams {
                        total: 100.0,
                        color: (0.0, 1.0, 1.0, 1.0).into(),
                        position,
                        infinite: 1,
                        rate_over_distance: 7.0,
                        start_speed: 30.0,
                        lifetime_factor: 1.0,
                        shape_selected: 1,
                        cone: Cone {
                            angle: 90.0,
                            arc: 90.0,
                        },
                        ..Default::default()
                    },
                );
                // audio.push(Sounds::Shoot, 0.5);
                let dir = CompassDir::from_deg(_dir + dir.angle.0 + 90.0);
                let p = Projectile::new(
                    pid,
                    position,
                    SCALE,
                    dir.angle,
                    5,
                    Bounds {
                        area: SCALE,
                        origin: cgmath::Point2 {
                            x: position.x,
                            y: position.y,
                        },
                    },
                    dir,
                    ExplosionType::FireBlueWithWave,
                    projectile_uniform,
                );

                self.projectiles.push(p);
            }
        };
    }

    fn update(
        &mut self,
        _position: cgmath::Vector2<f32>,
        velocity: f32,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        particle_system: &mut ParticleSystem,
    ) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles.get_mut(i).unwrap();
            if projectile.lifetime() >= LIFETIME {
                projectile.deactivate(); // this will emit a explosion
            }

            if !projectile.is_destroyed() {
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(projectile.position.x, projectile.position.y),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });

                if !projectile.has_target() {
                    projectile.set_direction(|this| {
                        this.position.x += (200.0) * this.dir.dir.x * dt.as_secs_f32();
                        this.position.y -= (200.0) * this.dir.dir.y * dt.as_secs_f32();
                    });
                } else {
                    let (_, position) = projectile.get_target();

                    let linear_dir = position - projectile.position;

                    let dir = projectile
                        .dir
                        .dir
                        .lerp(linear_dir.normalize(), 0.1)
                        .normalize();

                    let angle = dir.y.atan2(dir.x);

                    projectile.set_direction(|this| {
                        this.position.x += (250.0) * dir.x * dt.as_secs_f32();
                        this.position.y += (250.0) * dir.y * dt.as_secs_f32();
                        this.dir.dir = dir;
                        this.rotation = cgmath::Deg(angle.to_degrees() + 90.0);
                    });
                }

                particle_system.update_sim_params(projectile.id, projectile.position, 1);
                projectile.update();
                projectile.uniform.write(queue);

                i += 1;
            } else {
                particle_system.update_sim_params(projectile.id, projectile.position, 0);
                self.projectiles.swap_remove(i);
            }
        }
    }

    fn get_projectiles(&mut self) -> std::slice::IterMut<'_, Projectile> {
        self.projectiles.iter_mut()
    }

    fn get_name(&self) -> &str {
        "homing_missile"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
