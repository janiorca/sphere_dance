use intro_rs::gl;
use intro_rs::Window;

pub struct Visual {
    window: Window
}

impl intro_rs::Visual for Visual {
    fn new() -> Self where Self: Sized {
        let window = Window::new();
        Self { window }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn draw(&self, _time: f32) {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}