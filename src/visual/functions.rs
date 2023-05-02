#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub use intro_rs::gl;
use intro_rs::load_functions;

load_functions! {
    fn glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) -> (),
    fn glClear(mask: GLenum) -> ()
}
