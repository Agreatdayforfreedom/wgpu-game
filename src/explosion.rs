use crate::entity::EntityUniform;
use crate::rendering;
use crate::rendering::create_bind_group_layout;
use crate::uniform;
use crate::uniform::Uniform;

const TIME_TO_NEXT_FRAME: f32 = 2.0 / 30.0;

pub struct Explosion {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub end: bool,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub i: u32,
    pub sprites: Vec<rendering::Sprite>,
    time_to_next_frame: f32,
}

impl Explosion {
    pub fn new(
        position: cgmath::Vector2<f32>,
        scale: cgmath::Vector2<f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let uniform = Uniform::<EntityUniform>::new(device);

        let bind_group_layout = create_bind_group_layout(device);

        let diffuse_bytes1 = include_bytes!("./assets/exp1.png");
        let diffuse_bytes2 = include_bytes!("./assets/exp2.png");
        let diffuse_bytes3 = include_bytes!("./assets/exp3.png");
        let diffuse_bytes4 = include_bytes!("./assets/exp4.png");
        let diffuse_bytes5 = include_bytes!("./assets/exp5.png");

        let sprites = vec![
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes1,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes2,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes3,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes4,
            ),
            rendering::Sprite::new(
                &device,
                &queue,
                wgpu::AddressMode::ClampToEdge,
                &bind_group_layout,
                diffuse_bytes5,
            ),
        ];
        Self {
            position,
            scale,
            uniform,
            i: 0,
            time_to_next_frame: 0.0,
            sprites,
            end: false,
        }
    }

    pub fn update(&mut self, dt: &instant::Duration) {
        self.uniform
            .data
            .set_position(self.position - cgmath::Vector2::new(8.0, 8.0))
            .set_scale(self.scale)
            .exec();
        self.time_to_next_frame += dt.as_secs_f32();

        if self.time_to_next_frame > TIME_TO_NEXT_FRAME {
            if self.i == 4 {
                self.end = true;
            } else {
                self.i += 1;
                self.time_to_next_frame = 0.0;
            }
        }
    }
}
