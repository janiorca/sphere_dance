use intro_rs::Window;

pub mod functions;
use functions::*;

pub struct Visual {
    window: Window
}

impl intro_rs::Visual for Visual {
    fn new() -> Self where Self: Sized {
        initialize_functions();
        let window = Window::new();
        Self { window }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn draw(&self, time: f32) {
        glClearColor(time, 0.0, 0.0, 1.0);
        glClear(gl::COLOR_BUFFER_BIT);
    }
}