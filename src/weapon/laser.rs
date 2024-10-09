use std::slice::IterMut;

use crate::{
    audio::Audio,
    collider::Bounds,
    entity::EntityUniform,
    input::Input,
    rendering::{create_bind_group_layout, Sprite},
    util::CompassDir,
};

use super::{projectile::Projectile, weapon::Weapon};

const SPEED_LASER_MOVEMENT: f32 = 1.5;

pub struct Laser {
    projectiles: Vec<Projectile>,
    sprite: Sprite,
}

impl Laser {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Box<Self> {
        let diffuse_bytes = include_bytes!("./../assets/laser.png");
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
        positions: &Vec<cgmath::Vector2<f32>>,
        scale: cgmath::Vector2<f32>,
        dir: CompassDir,
        input: &Input,
        _audio: &mut Audio,
    ) {
        if input.is_pressed("f") && self.projectiles.len() < 1 {
            let mut uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
            let position = *positions.get(0).unwrap();
            uniform.data.set_tex_scale((-1.0, -3.0).into()).exec();
            self.projectiles.push(Projectile::new(
                position,
                scale,
                cgmath::Deg(0.0),
                Bounds {
                    area: scale,
                    origin: cgmath::Point2 {
                        x: position.x,
                        y: position.y,
                    },
                },
                CompassDir::from_deg(0.0),
                uniform,
            ))
        }
    }

    fn update(
        &mut self,
        positions: &Vec<cgmath::Vector2<f32>>,
        queue: &mut wgpu::Queue,
        dt: &instant::Duration,
    ) {
        for projectile in &mut self.projectiles {
            let position = *positions.get(0).unwrap();
            if projectile.alive {
                //todo remove laser (swap_remove)
                projectile.update(&dt, 0.0, position);

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
