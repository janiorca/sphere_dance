pub mod gl;
mod window;

use crate::intro::Command;
pub use window::*;

pub trait Visual {
    fn new() -> Self where Self: Sized;

    fn window(&self) -> &Window;

    fn manage(&self, time: f32) -> Option<Command> {
        if let Some(Command::Exit) = self.window().manage() {
            Some(Command::Exit)
        } else {
            self.draw(time);
            self.window().present();
            None
        }
    }

    fn draw(&self, time: f32);
}
