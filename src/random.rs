use core::mem::transmute;
use core::num::Wrapping;

pub struct Rng {
    pub seed: Wrapping<u32>,
}

impl Rng {
    pub fn new_unseeded() -> Rng {
        Rng {
            seed: Wrapping(0x66126c8d),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.seed = self.seed*Wrapping(214013) + Wrapping(2531011);
        self.seed.0
    }

    pub fn next_f32(&mut self) -> f32 {
        const UPPER_MASK: u32 = 0x3F800000;
        const LOWER_MASK: u32 = 0x7FFFFF;
        let tmp = UPPER_MASK | (self.next_u32() & LOWER_MASK);
        let result: f32 = unsafe { transmute(tmp) };
        result - 1.0
    }
}