use std::slice::IterMut;

use cgmath::Vector2;

use crate::{
    audio::Audio,
    collider::Bounds,
    entity::EntityUniform,
    explosion::ExplosionType,
    input::Input,
    particle_system::system::ParticleSystem,
    rendering::{create_bind_group_layout, Sprite},
    util::{CompassDir, IdVendor},
};

use super::{projectile::Projectile, weapon::Weapon};

const SPEED_LASER_MOVEMENT: f32 = 1.5;
const SCALE: Vector2<f32> = Vector2::new(4.0, 40.0);

pub struct Laser {
    projectiles: Vec<Projectile>,
    sprite: Sprite,
}

impl Laser {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/bullets/laser.png");
        let bind_group_layout = create_bind_group_layout(device);

        let sprite = Sprite::new(
            &device,
            &queue,
            wgpu::AddressMode::Repeat,
            &bind_group_layout,
            diffuse_bytes,
        );

        Box::new(Self {
            projectiles: vec![],
            sprite,
        })
    }
}

impl Weapon for Laser {
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
        if input.is_pressed("f") && self.projectiles.len() < 1 {
            let position = *positions.get(0).unwrap();
            let mut uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            uniform.data.set_tex_scale((-1.0, -3.0).into()).exec();
            self.projectiles.push(Projectile::new(
                id_vendor.next_id(),
                position,
                SCALE,
                cgmath::Deg(0.0),
                1,
                Bounds {
                    area: SCALE,
                    origin: cgmath::Point2 {
                        x: position.x,
                        y: position.y,
                    },
                },
                CompassDir::from_deg(0.0),
                ExplosionType::Particles,
                uniform,
            ))
        }
    }

    fn update(
        &mut self,
        position: cgmath::Vector2<f32>,
        velocity: f32,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
        particle_system: &mut ParticleSystem,
    ) {
        for projectile in &mut self.projectiles {
            if projectile.is_active() {
                //todo remove laser (swap_remove)
                projectile.update();

                // projectile.set_bounds(Bounds {
                //     origin: cgmath::Point2::new(projectile.position.x, projectile.position.y),
                //     area: cgmath::Vector2::new(1000.0, 1000.0), //todo
                // });
                projectile
                    .uniform
                    .data
                    .set_color((1.0, 0.5, 0.0, 1.0).into());
                projectile.uniform.data.tex_pos.y -= SPEED_LASER_MOVEMENT * dt.as_secs_f32();
                projectile.uniform.write(queue);
            }
        }
    }

    fn get_projectiles(&mut self) -> IterMut<Projectile> {
        self.projectiles.iter_mut()
    }

    fn get_name(&self) -> &str {
        "laser"
    }

    fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.sprite.buffer.slice(..));
        rpass.set_bind_group(0, &self.sprite.bind_group, &[]);
        for projectile in &mut self.projectiles {
            projectile.draw(rpass);
        }
    }
}
