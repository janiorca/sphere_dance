#![no_std]
#![no_main]
#![windows_subsystem = "windows"]

// pub mod audio;
pub mod visual;

use intro_rs::audio::Audio;
use intro_rs::intro::Intro;
use intro_rs::visual::Visual;

pub struct Namekusei {
    time: f32,
    audio: Audio,
    visual: Visual
}

impl Intro for Namekusei {
    fn new() -> Self where Self: Sized {
        let time = 0.0;
        let audio = Audio::new();
        let visual = Visual::new();
        Self { time, audio, visual }
    }

    fn time(&mut self) -> &mut f32 {
        &mut self.time
    }

    fn audio(&self) -> &Audio {
        &self.audio
    }

    fn visual(&self) -> &Visual {
        &self.visual
    }
}

intro_rs::set_intro!(Namekusei);

