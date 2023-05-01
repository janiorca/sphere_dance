#![no_std]
#![no_main]
#![windows_subsystem = "windows"]

pub mod audio;
pub mod visual;

use audio::Audio;
use visual::Visual;

intro_rs::set_intro!(intro_rs::IntroTemplate<Audio, Visual>);