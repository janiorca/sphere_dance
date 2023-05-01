pub struct Audio;

static mut MUSIC_DATA: [f32; 44100*120] = [0.0; 44100*120];

impl intro_rs::Audio for Audio {
    fn new() -> Self where Self: Sized {
        unsafe {
            for (index, sample) in MUSIC_DATA.iter_mut().enumerate() {
                *sample = (index % 220) as f32 / 440.0f32 * 0.01f32;
            }
        }
        Self
    }
    
    fn data_mut(&self) -> &mut [f32] {
        unsafe {
            &mut MUSIC_DATA
        }
    }
}