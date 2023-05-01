#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#![feature(core_intrinsics)]

pub mod util;
pub mod intro;
pub mod audio;
pub mod visual;

set_intro!(crate::intro::Intro);

