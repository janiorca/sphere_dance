use super::random;

static mut SOUNDS: [[f32;44100*9];7] = [[0.0;44100*9];7];
static FREQUENCIES: [ f32; 7] = [
    349.0,     //F4   
    415.0,     //Ab4            1.189
    523.0,     //C5             1.26
    554.0,     //Db5            1.059
    622.0,     //Eb5            1.122
    698.0,     //F5             1.22
    831.0];     //Ab5

fn play( dst: &mut [f32;44100*120], dst_offset : usize, signal : &[f32;44100*9], sample_duration : f32 ) {
    let mut dst_pos = 0;
    let mut position : f32 = 0.0;
    unsafe{
        loop{
            let src_val = signal.get_unchecked(dst_pos);
            let in_pos = position/4.5-2f32;
            let val = (in_pos*in_pos)*position/4.5;
            *dst.get_unchecked_mut( dst_pos + dst_offset) += src_val*val;

            position += sample_duration;
            dst_pos += 1;
            if dst_pos == 44100*9 {
                return;
            }
        }
    }
}

pub fn make_music( music: &mut [f32;44100*120]) {
    let mut vrng = random::Rng::new_unseeded();

    super::log!( "Make instruments!");

    let mut i = 0;
    loop{
        let mut scale = 1.0;
        // # Could combine into a single loop that doubles the scales when loop % 11 == 0. Possibly slightly shorter
        unsafe{
            loop{
                let mut d = 0;
                loop{
                    let frequency : f32 = FREQUENCIES.get_unchecked(i)/scale+6.0*vrng.next_f32();
                    let mut position : f32 = 0.0;
                    let mut sample_no = 0;
                    loop {
                        let sample_duration : f32 = frequency / 44100.0f32;
                        position = position + sample_duration;
                        if position > 0.5 {
                            position -= 1.0f32;
                        }
                        let val = core::intrinsics::fabsf32(position)*4f32-1.0f32;
                        *SOUNDS.get_unchecked_mut(i).get_unchecked_mut(sample_no) += val/55.0f32;
                        sample_no += 1;
                        if sample_no == 44100*9 {
                            break;
                        }
                    }
                    d += 1;
                    if d == 11 {
                        break;
                    }
                }
                scale *= 2.0f32;
                if scale >= 32.0 {
                    break;
                }
            }
        }
        i += 1;
        if i == 7 {
            break;
        }
    }

    unsafe{ 
        let mut mrng : random::Rng = random::Rng{seed: core::num::Wrapping(1161249)};

        let mut dst : usize = 0;
        let mut s = 0;
        loop {
            let mut i = 0;
            loop{
                let nt = mrng.next_f32();
                if nt > 0.9 {
                    play(music, dst, &SOUNDS[i], 1.0 / 44100.0 );
                }
                i += 1;
                if i == 7 { 
                    break;
                }
            }
            dst += 44100;
            s += 1;
            if s == 110 { 
                break; 
            }
        }
    }
}
