use crate::audio::Audio;
use crate::log;
use crate::visual::Visual;

pub enum Command {
    Exit
}

pub trait Intro {
    fn new() -> Self where Self: Sized;
    fn run(&mut self) {
        log!("Entering loop\n");

        self.audio().play();
        loop {
            let time = *self.time();
            if let Some(Command::Exit) = self.visual().manage(time) {
                break;
            }
            *self.time() += 1.0 / 60.0f32;

            unsafe {
                if winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_ESCAPE) != 0 {
                    break;
                }
            }

            #[cfg(not(feature = "logger"))]
            if *self.time() > 120.0 {
                break;
            }
        }

        unsafe {
            // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
            winapi::um::processthreadsapi::ExitProcess(0);
        }
    }

    fn time(&mut self) -> &mut f32;
    fn audio(&self) -> &dyn Audio;
    fn visual(&self) -> &dyn Visual;
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