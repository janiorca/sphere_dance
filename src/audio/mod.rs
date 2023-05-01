static WAVE_FORMAT: winapi::shared::mmreg::WAVEFORMATEX = winapi::shared::mmreg::WAVEFORMATEX{
    wFormatTag : winapi::shared::mmreg::WAVE_FORMAT_IEEE_FLOAT,
    nChannels : 1,
    nSamplesPerSec : 44100,
    nAvgBytesPerSec : 44100*4,
    nBlockAlign : 4,
    wBitsPerSample: 32,
    cbSize:0
};

static mut WAVE_HEADER: winapi::um::mmsystem::WAVEHDR = winapi::um::mmsystem::WAVEHDR{
    lpData: 0 as *mut i8,
    dwBufferLength: 44100*4*120,
    dwBytesRecorded: 0,
    dwUser: 0,
    dwFlags: 0,
    dwLoops: 0,
    lpNext: 0 as *mut winapi::um::mmsystem::WAVEHDR,
    reserved: 0,
};

static mut MUSIC_DATA: [f32;44100*120] = [ 0.0;44100*120];

pub struct Audio;

impl Audio {
    pub fn new() -> Self {
        unsafe {
            make_music(&mut MUSIC_DATA);
            WAVE_HEADER.lpData = MUSIC_DATA.as_mut_ptr() as *mut i8;
        }
        Self
    }

    pub fn play(&self) {
        unsafe {
            let mut h_wave_out: winapi::um::mmsystem::HWAVEOUT = 0 as winapi::um::mmsystem::HWAVEOUT;
            winapi::um::mmeapi::waveOutOpen(&mut h_wave_out, winapi::um::mmsystem::WAVE_MAPPER, &WAVE_FORMAT, 0, 0, winapi::um::mmsystem::CALLBACK_NULL);
            winapi::um::mmeapi::waveOutPrepareHeader(h_wave_out, &mut WAVE_HEADER, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32);
            winapi::um::mmeapi::waveOutWrite(h_wave_out, &mut WAVE_HEADER, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32);
        }
    }
}


pub fn make_music( music: &mut [f32;44100*120]) {
    for (index, sample) in music.iter_mut().enumerate() {
        *sample = (index % 220) as f32 / 440.0f32 * 0.01f32;
    }
}
