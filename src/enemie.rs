// use cgmath::{Vector2, Vector4};

// use crate::audio::{Audio, Sounds};
// use crate::collider::Bounds;
// use crate::entity::{Entity, EntityUniform};
// use crate::uniform::Uniform;
// use crate::util::CompassDir;
// use crate::weapon::projectile::Projectile;
// pub struct Enemy {
//     id: u32,
//     pub position: cgmath::Vector2<f32>,
//     pub scale: cgmath::Vector2<f32>,
//     pub alive: bool,
//     pub uniform: Uniform<EntityUniform>,
//     pub projectiles: Vec<Projectile>,
//     pub interval: instant::Instant,
// }

// impl Entity for Enemy {
//     fn update(
//         &mut self,
//         _dt: &instant::Duration,
//         _input: &crate::input::Input,
//         _audio: &mut Audio,
//         _device: &wgpu::Device,
//         _queue: &mut wgpu::Queue,
//     ) {
//         self.uniform.write(_queue);
//     }

//     fn alive(&self) -> bool {
//         self.alive
//     }

//     // fn position(&self) -> Vector2<f32> {
//     //     self.position
//     // }
//     // fn scale(&self) -> Vector2<f32> {
//     //     self.scale
//     // }
//     // fn set_colors(&mut self, color: Vector4<f32>) {
//     //     self.uniform.data.set_color(color);
//     // }
//     // fn id(&self) -> u32 {
//     //     self.id
//     // }

//     fn draw<'a, 'b>(&'a mut self, rpass: &'b mut wgpu::RenderPass<'a>) {
//         rpass.set_vertex_buffer(2, self.uniform.buffer.slice(..));
//         rpass.set_bind_group(2, &self.uniform.bind_group, &[]);
//         rpass.draw(0..6, 0..1);
//     }
// }

// impl Enemy {
//     pub fn new(
//         position: cgmath::Vector2<f32>,
//         scale: cgmath::Vector2<f32>,
//         uniform: Uniform<EntityUniform>,
//         id: u32,
//     ) -> Self {
//         Self {
//             id,
//             position,
//             scale,
//             alive: true,
//             uniform,
//             projectiles: vec![],
//             interval: instant::Instant::now(),
//         }
//     }

//     pub fn spawn_fire(
//         &mut self,
//         scale: cgmath::Vector2<f32>,
//         audio: &mut Audio,
//         device: &wgpu::Device,
//     ) -> Option<Projectile> {
//         if self.interval.elapsed().as_millis() >= 500 {
//             self.interval = instant::Instant::now();
//             let projectile_uniform = crate::uniform::Uniform::<EntityUniform>::new(&device);
//             audio.push(Sounds::Shoot, 1.0);
//             self.projectiles.push(Projectile::new(
//                 (
//                     self.position.x + (self.scale.x / 2.0) - (scale.x / 2.0),
//                     self.position.y,
//                 )
//                     .into(),
//                 scale,
//                 cgmath::Deg(90.0),
//                 Bounds {
//                     area: scale,
//                     origin: cgmath::Point2 {
//                         x: self.position.x,
//                         y: self.position.y,
//                     },
//                 },
//                 CompassDir::from_deg(270.0),
//                 projectile_uniform,
//             ));
//         }
//         None
//     }
// }
