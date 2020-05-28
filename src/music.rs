use super::random;

type AdsrEnvelope = [(f32,f32);5];

struct MiniADSR<'a>{
    signal : &'a [f32;44100*9],
    sample_duration : f32, // length of each sample in secs
    position : f32,
    triggered : bool,
    signal_pos : usize
}

static miniEnvelope : AdsrEnvelope = [ (0.0, 0.01), (3.6, 1.0), (5.1,0.7), (6.6,0.3), (8.9, 0.0)];

impl <'a>MiniADSR<'a>{
    fn new(  signal : &'a [f32;44100*9], sample_duration : f32 ) -> MiniADSR{
        MiniADSR{
            signal : signal,
            sample_duration : sample_duration,
            position: 0.0,
            triggered : false,
            signal_pos : 0
        }
    }
    fn next(&mut self) -> f32 {
        let val = interpolate( self.position, &miniEnvelope );
        if self.triggered {
            let src_val = self.signal[self.signal_pos];
            self.signal_pos += 1;
                self.position += self.sample_duration;
            if self.position > miniEnvelope[ 4 ].0 {
                self.triggered = false;
            }
            return src_val*val;
        }
        return 0.0;
    }
    fn trigger(&mut self) {
        self.triggered = true;
        self.position = 0.0;       
        self.signal_pos = 0;
    }
}

static mut sounds : [[f32;44100*9];7] = [[0.0;44100*9];7];

fn interpolate( t: f32, envelope: &AdsrEnvelope) -> f32 {
    let mut loc = 1;
    loop{
        if t < envelope[ loc ].0 {
            let width = envelope[ loc ].0 - envelope[ loc-1 ].0;
            let r_pos = ( t - envelope[ loc-1 ].0 ) / width;
            return (1.0-r_pos)*envelope[ loc-1 ].1 + r_pos*envelope[ loc ].1;
        }
        loc += 1;
        if loc == 5 {
            return 0.0;
        }
    }
}

pub fn make_music( music: &mut [f32;44100*60]) {
    let frequencies : [ f32; 7]= [
    349.0,     //F4   
    415.0,     //Ab4
    523.0,     //C5
    554.0,     //Db5
    622.0,     //Eb5
    698.0,     //F5
    831.0];     //Ab5

    let mut vrng = random::Rng::new_unseeded();

    unsafe{ super::log!( "Make instruments!"); };

    for i in 0..7 {
        let mut scale = 1.0;
        let mut harmonic = 0;
        loop{
            let mut d = 0;
            loop{
                let angle : f32 = 0.5f32;
                let frequency : f32 = frequencies[ i ]/scale+6.0*vrng.next_f32();
                let amplitude : f32 = 0.94f32;
                let mut position : f32 = 0.0;
                for sample in 0..44100*9 {
                    let sample_duration : f32 = frequency / 44100.0f32;
                    position = position + sample_duration;
                    if position > 1.0 {
                        position -= 1.0f32;
                    }
            
                    let val;
                    if position <= angle {
                        val = position / angle;
                    } else {
                        val = ( 1f32 - position ) / ( 1f32 - angle );
            
                    }
                    unsafe{
                        sounds[i][sample] += (val * amplitude)/55.0f32;
                    }
                }
                d += 1;
                if d == 11 {
                    break;
                }
            }
            scale *= 2.0f32;
            harmonic += 1;
            if harmonic == 5 {
                break;
            }
        }
    }

    unsafe{ 
        let mut instruments : [MiniADSR;7] = core::mem::zeroed();
        let mut i = 0;
        loop {
            instruments[i] = MiniADSR::new( &sounds[i], 1.0 / 44100.0);
            i += 1;
            if i == 7 {
                break;
            }
        }

//    let mut mrng : Rng = Rng{seed: core::num::Wrapping(31249)};
//    let mut mrng : Rng = Rng{seed: core::num::Wrapping(6731249)};
        let mut mrng : random::Rng = random::Rng{seed: core::num::Wrapping(1161249)};

        let mut dst : usize = 0;
        let mut s = 0;
        loop {
            super::log!( "#", s as f32);
            let mut i = 0;
            loop{
                let nt = mrng.next_f32();
                if nt > 0.95-(s as f32 * 0.1) && instruments[ i ].triggered == false {
                    instruments[ i ].trigger();
                    super::log!( "#", nt as f32);
                }
                i += 1;
                if i == 7 { 
                    break;
                }
            }
            i = 0;
            loop{
                let mut value : f32 = 0.0;
                for instrument in &mut instruments {
                    value += instrument.next();
                }
                value /= 2.0;
                music[dst ] = value;
                dst += 1;
                i += 1;
                if i == 44100 { 
                    break;
                }
            }
            s += 1;
            if s == 60 { 
                break; 
            }
        }
    }
}
