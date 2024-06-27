use crate::entity::EntityUniform;
use crate::sprite_renderer;
use crate::uniform;
use crate::uniform::Uniform;

const TIME_TO_NEXT_FRAME: f32 = 2.0 / 30.0;

pub struct Explosion {
    pub position: cgmath::Vector2<f32>,
    pub size: f32,
    pub play: bool,
    pub end: bool,
    pub uniform: uniform::Uniform<EntityUniform>,
    pub i: u32,
    pub sprites: Vec<sprite_renderer::SpriteRenderer>,
    time_to_next_frame: f32,
}

impl Explosion {
    pub fn new(
        position: cgmath::Vector2<f32>,
        size: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut uniform = Uniform::<EntityUniform>::new(&device);

        let diffuse_bytes1 = include_bytes!("./assets/exp1.png");
        let diffuse_bytes2 = include_bytes!("./assets/exp2.png");
        let diffuse_bytes3 = include_bytes!("./assets/exp3.png");
        let diffuse_bytes4 = include_bytes!("./assets/exp4.png");
        let diffuse_bytes5 = include_bytes!("./assets/exp5.png");

        let sprites = vec![
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes1),
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes2),
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes3),
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes4),
            sprite_renderer::SpriteRenderer::new(&device, &queue, diffuse_bytes5),
        ];
        Self {
            position,
            size: 40.0,
            play: false,
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
            .set_position(self.position - cgmath::Vector2::new(8.0, 8.0));
        self.uniform.data.set_size(self.size);
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
