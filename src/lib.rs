mod audio;
mod camera;
mod collider;
mod enemie;
mod entity;
mod explosion;
mod input;
mod player;
mod sprite_renderer;
mod state;
mod texture;
mod uniform;
mod util;
mod weapon;
mod window;

use window::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
