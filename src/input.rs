use winit::event::ElementState;

#[derive(Default)]
pub struct Input {
    d_pressed: bool,
    a_pressed: bool,
    s_pressed: bool,
    w_pressed: bool,
    f_pressed: bool,
    c_pressed: bool,
}
//todo: better input  manager
impl Input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, key: &str, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        match key {
            "a" => {
                self.a_pressed = pressed;
            }
            "d" => {
                self.d_pressed = pressed;
            }
            "s" => {
                self.s_pressed = pressed;
            }
            "w" => {
                self.w_pressed = pressed;
            }
            "f" => {
                self.f_pressed = pressed;
            }
            "c" => {
                self.c_pressed = pressed;
            }

            _ => (),
        }
    }

    pub fn is_pressed(&self, key: &str) -> bool {
        if key == "a" {
            self.a_pressed
        } else if key == "d" {
            self.d_pressed
        } else if key == "s" {
            self.s_pressed
        } else if key == "w" {
            self.w_pressed
        } else if key == "f" {
            self.f_pressed
        } else if key == "c" {
            self.c_pressed
        } else {
            false
        }
    }
}
