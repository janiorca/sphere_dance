use crate::audio::Audio;
use crate::log;
use crate::visual::{gl, Visual};

pub enum Command {
    Exit
}

pub fn tick(_now: f32 )  {
    unsafe {
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub struct Intro {
    time: f32,
    audio: Audio,
    visual: Visual
}

impl Intro {
    pub fn new() -> Self {
        log!("Initializing\n");
        let time = 0.0f32;
        let audio = Audio::new();
        let visual = Visual::new();
        Self { time, audio, visual }
    }

    pub fn run(&mut self) {
        log!("Entering loop\n");

        self.audio.play();
        loop {
            if let Some(Command::Exit) = self.visual.tick(self.time) {
                break;
            }
            self.time += 1.0 / 60.0f32;

            unsafe{
                if winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_ESCAPE) != 0 {
                    break;
                }
            }

            #[cfg(not(feature = "logger"))]
            if self.time > 120.0 {
                break;
            }
        }

        unsafe {
            // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
            winapi::um::processthreadsapi::ExitProcess(0);
        }
    }
}

#[macro_export]
macro_rules! set_intro {
    ($i:ty) => {
        #[no_mangle]
        pub extern "system" fn mainCRTStartup() {
            let mut intro = <$i>::new();
            intro.run();
        }

        // Compiling with no_std seems to require the following symbol to be set if there is any floating point code anywhere in the code
        #[no_mangle]
        pub static _fltused : i32 = 1;

    }
}