use std::slice::IterMut;

use cgmath::Vector2;

use crate::{
    audio::{Audio, Sounds},
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    particle_system::system::ParticleSystem,
    rendering::{create_bind_group_layout, Sprite},
    util::{CompassDir, IdVendor},
    weapon::projectile::Projectile,
};

use super::weapon::Weapon;

const LIFETIME: u128 = 5000;
const PER_SHOOT_INTERVAL: u128 = 50;
const SCALE: Vector2<f32> = Vector2::new(40.0, 40.0);
// rotation applied to each bullet to achieve a cone effect
const MAX_WAVES: u16 = 5;
const ROTATION_DIRS: [f32; MAX_WAVES as usize] = [0.0, 10.0, 20.0, -10.0, -20.0];
pub struct RailGun {
    projectiles: Vec<Projectile>,
    time: instant::Instant,
    shooting_interval: u128,     // milliseconds
    wave_time: instant::Instant, // interval per wave
    sprite: Sprite,
    actived: bool,
    current_wave_emitted: u16,
}

impl RailGun {
    pub fn new(shooting_interval: u128, device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/blue_bullet.png");
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
            wave_time: instant::Instant::now(),
            shooting_interval,
            sprite,
            actived: false,
            current_wave_emitted: 0,
        })
    }
}

impl Weapon for RailGun {
    fn shoot(
        &mut self,
        device: &wgpu::Device,
        positions: Vec<cgmath::Vector2<f32>>,
        dir: CompassDir,
        input: &Input,
        audio: &mut Audio,
        id_vendor: &mut IdVendor,
        _particle_system: &mut ParticleSystem,
    ) {
        if self.actived
            || (input.is_pressed("f") && self.time.elapsed().as_millis() >= self.shooting_interval)
        {
            let position = *positions.get(0).unwrap();
            self.actived = true;

            self.time = instant::Instant::now();

            if self.current_wave_emitted < MAX_WAVES
                && self.wave_time.elapsed().as_millis() >= PER_SHOOT_INTERVAL
            {
                audio.push(Sounds::Shoot, 1.5);

                self.wave_time = instant::Instant::now();
                self.current_wave_emitted += 1;

                for i in 0..5 {
                    let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);

                    self.projectiles.push(Projectile::new(
                        id_vendor.next_id(),
                        ((position.x), position.y).into(),
                        SCALE,
                        dir.angle,
                        2,
                        Bounds {
                            area: (2.5, 2.5).into(),
                            origin: cgmath::Point2 {
                                x: position.x,
                                y: position.y,
                            },
                        },
                        dir.rotate(ROTATION_DIRS[i]),
                        projectile_uniform,
                    ));
                }
                if self.current_wave_emitted == 5 {
                    self.actived = false;
                    self.current_wave_emitted = 0;
                }
            }
        }
    }

    fn update(
        &mut self,
        position: cgmath::Vector2<f32>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        _particle_system: &mut ParticleSystem,
    ) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles.get_mut(i).unwrap();
            if projectile.lifetime() > LIFETIME {
                projectile.destroy();
            }
            if !projectile.is_destroyed() {
                projectile.set_bounds(Bounds {
                    origin: cgmath::Point2::new(projectile.position.x, projectile.position.y),
                    area: cgmath::Vector2::new(2.5, 2.5),
                });
                projectile.update();
                projectile.set_direction(|this| {
                    this.position.x += 500.0 * this.dir.dir.x * dt.as_secs_f32();
                    this.position.y -= 500.0 * this.dir.dir.y * dt.as_secs_f32();
                    this.initial_position = position;
                });
                projectile.uniform.write(queue);
                i += 1;
            } else {
                self.projectiles.swap_remove(i);
            }
        }
    }

    fn get_projectiles(&mut self) -> IterMut<Projectile> {
        self.projectiles.iter_mut()
    }

    fn get_name(&self) -> &str {
        "rail_gun"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
