#![no_std]
#![no_main]
#![windows_subsystem = "windows"]

pub mod audio;
pub mod visual;

use intro_rs::Audio;
use intro_rs::Intro;
use intro_rs::Visual;

pub struct Namekusei {
    time: f32,
    audio: audio::Audio,
    visual: visual::Visual
}

impl Intro for Namekusei {
    fn new() -> Self where Self: Sized {
        let time = 0.0;
        let audio = audio::Audio::new();
        let visual = visual::Visual::new();
        Self { time, audio, visual }
    }

    fn time(&mut self) -> &mut f32 {
        &mut self.time
    }

    fn audio(&self) -> &dyn Audio {
        &self.audio
    }

    fn visual(&self) -> &dyn Visual {
        &self.visual
    }
}

intro_rs::set_intro!(Namekusei);

