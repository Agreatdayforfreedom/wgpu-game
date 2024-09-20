mod audio;
mod background;
mod camera;
mod collider;
mod enemie;
mod entities;
mod entity;
mod explosion;
mod input;
mod particle_system;
mod player;
mod post_processing;
mod rendering;
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
