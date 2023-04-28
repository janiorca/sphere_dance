use crate::gl;

pub fn init() {

}

pub fn tick(_now: f32 )  {
    unsafe {
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}