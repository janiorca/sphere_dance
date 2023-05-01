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

pub trait Audio {
    fn new() -> Self where Self: Sized;

    fn data_mut(&self) -> &mut [f32];

    fn play(&self) {
        unsafe {
            WAVE_HEADER.lpData = self.data_mut().as_mut_ptr() as *mut i8;
            let mut h_wave_out: winapi::um::mmsystem::HWAVEOUT = 0 as winapi::um::mmsystem::HWAVEOUT;
            winapi::um::mmeapi::waveOutOpen(&mut h_wave_out, winapi::um::mmsystem::WAVE_MAPPER, &WAVE_FORMAT, 0, 0, winapi::um::mmsystem::CALLBACK_NULL);
            winapi::um::mmeapi::waveOutPrepareHeader(h_wave_out, &mut WAVE_HEADER, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32);
            winapi::um::mmeapi::waveOutWrite(h_wave_out, &mut WAVE_HEADER, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32);
        }
    }
}
