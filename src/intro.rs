use super::math_util;
use super::gl;
use super::gl_util;
use super::random;
use core::arch::x86;

use gl::CVoid;
use core::mem::{size_of,transmute};
use core::ops::{Add,Sub,Mul};

// Floating point constants picked for compressibility
pub const FP_0_01 :f32 = 0.0100097656f32;  //    0.01
pub const FP_0_02 :f32 = 0.0200195313f32;     // 0.02f    0x3ca40000
pub const FP_0_05 :f32 = 0.0500488281f32;  //
pub const FP_0_20 : f32 = 0.2001953125f32;
pub const FP_1_32  : f32 = 1.3203125000f32;     // 1.32f    0x3fa90000
pub const FP_1_54 : f32 = 1.5390625000f32;

// The dynamic part of the world is 80 spheres + camera and light
pub const CAMERA_POS_IDX : usize = 80*2;
pub const CAMERA_ROT_IDX : usize = 80*2+1;
pub const CAMERA_CUT_INFO : usize = 80*2+2;
pub const num_spheres : usize = 80;
pub const sphere_extras : usize = 2;

static mut shader_prog : gl::GLuint = 0;
static mut vertex_array_id : gl::GLuint = 0;

static mut rng : random::Rng = random::Rng{seed: core::num::Wrapping(21431249)};

static mut global_spheres: [ [ f32; 4]; (num_spheres+sphere_extras)*2] = [ [ 0f32; 4]; (num_spheres+sphere_extras)*2 ];  

fn smooth( pixels: &mut [ f32; 512*513*4 ]) {
    unsafe{
        let mut xy = 0;
        loop{
            let offset = xy*4;
            let mut val  = *pixels.get_unchecked( offset );
            val += pixels.get_unchecked( offset+4 );
            val += pixels.get_unchecked( offset+2048 );
            val += pixels.get_unchecked( offset+2052 );
            *pixels.get_unchecked_mut( offset ) = val / 4.0;
            xy += 1;
            if xy == 511*511 { break; }
        }
    }
}

static mut src_terrain  : [ f32; 512*513*4 ] = [ 0.0; 512*513*4 ];
static mut tex_buffer_id : gl::GLuint = 0;

#[cfg(feature = "logger")]
static mut glbl_shader_code : [ u8;25000] = [0; 25000];

static mut old_x : i32 = 0;
static mut old_y : i32 = 0;
static mut moving_camera : bool  = false;
static mut rotating_camera : bool  = false;

static mut camera_velocity : [ f32; 4] = [ 0.0; 4];
static mut camera_rot_speed : [ f32; 4] = [ 0.0; 4];

static mut camera_mode : u32 = 0;
static mut sphere_scale : f32 = 0.0;

#[cfg(feature = "logger")]
pub fn set_pos( x: i32, y: i32, ctrl : bool ) {
    unsafe{
        if moving_camera {
            if ctrl{
                global_spheres[ CAMERA_POS_IDX ][ 1 ] += ( y-old_y) as f32 / 32.0;
            } else {
                global_spheres[ CAMERA_POS_IDX ][ 0 ] += ( x-old_x) as f32 / 32.0;
                global_spheres[ CAMERA_POS_IDX ][ 2 ] += ( y-old_y) as f32 / 32.0;
            }
        } else if rotating_camera {
            global_spheres[ CAMERA_ROT_IDX ][ 0 ] += ( y-old_y) as f32 / 1024.0;
            global_spheres[ CAMERA_ROT_IDX ][ 1 ] += ( x-old_x) as f32 / 1024.0;
        }
        old_x = x;
        old_y = y;
    }
}

#[cfg(feature = "logger")]
pub fn rbutton_down( x: i32, y: i32 ) {
    unsafe{ 
        old_x = x;
        old_y = y;
        moving_camera = false;
        rotating_camera = true;
    }
}

#[cfg(feature = "logger")]
pub fn rbutton_up( ) {
    setup_random_camera();
    unsafe{ 
        rotating_camera = false;
    }
}

#[cfg(feature = "logger")]
pub fn lbutton_down( x: i32, y: i32 ) {
    unsafe{ 
        old_x = x;
        old_y = y;
        moving_camera = true;
        rotating_camera = false;
    }
}

#[cfg(feature = "logger")]
pub fn lbutton_up( ) {
    unsafe{ 
        moving_camera = false;
    }
    unsafe{ super::log!( "Camera: ", global_spheres[ CAMERA_POS_IDX ][ 0 ], global_spheres[ CAMERA_POS_IDX ][ 1 ], global_spheres[ CAMERA_POS_IDX ][ 2 ]); }
}

static mut r3_pos : usize = 0;

fn set_r3( dest : &mut[ f32 ; 4 ], crng : &mut random::Rng, a: f32, b: f32, c: f32, offset: f32 ) {
    // tried turning into a loop -> crinkled version grew 60bytes!
    let x = crng.next_f32();
    let z = crng.next_f32();
    dest[ 0 ] = (x-offset)*a;
    dest[ 1 ] = (crng.next_f32()-offset)*b;
    dest[ 2 ] = (z-offset)*c;
    unsafe{
        // we only ever calculate the position scaled by 512 ( by the unoffset values )
        r3_pos = (((z*512f32) as usize *512)+(x*512f32) as usize)*4;
    }
}

static mut sphere_delta : f32  = 0.0;
fn set_sphere_positions(now: f32) -> ( ) {
    let mut rng_terrain : random::Rng = random::Rng{seed: core::num::Wrapping(7923129)};

    let mut idx = 0;
    unsafe{
        let mut offset = math_util::sin( (now-sphere_delta)*0.02 )*sphere_scale;
        if offset < 0.0 {
            offset = -offset;
        }
        loop {
            loop{
                let y_offset = offset;
                set_r3( global_spheres.get_unchecked_mut(idx*2), &mut rng_terrain,512f32,512f32,512f32, 0.0 );
                if *src_terrain.get_unchecked( r3_pos ) > 0.3f32 {
                    global_spheres.get_unchecked_mut(idx*2)[ 1 ] = *src_terrain.get_unchecked( r3_pos )*60.0-12.1 + y_offset;
                    global_spheres.get_unchecked_mut(idx*2)[ 3 ] = 18.0f32;
                    global_spheres.get_unchecked_mut(idx*2+1)[ 0 ] = FP_0_02;
                    global_spheres.get_unchecked_mut(idx*2+1)[ 1 ] = FP_0_02;
                    global_spheres.get_unchecked_mut(idx*2+1)[ 2 ] = FP_0_02;
                    global_spheres.get_unchecked_mut(idx*2+1)[ 3 ] = FP_1_32;
                    break;
                }
                offset *= 1.003;
            }
            idx += 1;
            if idx == num_spheres { break;}
        }
    }
}

pub fn prepare() -> () {
    let mut error_message : [i8;100] = [ 0; 100];
     let vtx_shader_src : &'static str = "#version 330 core
    layout (location = 0) in vec3 Pos;
    void main()
    {
     gl_Position = vec4(Pos, 1.0);
    }\0";

    let spheres : &mut[ [ f32; 4]; (num_spheres+sphere_extras)*2];  
    unsafe{
        spheres  = &mut global_spheres;
    }
    
    let vtx_shader : u32;
    let frag_shader : u32;
    unsafe{ super::log!( "Load shader !"); };

    #[cfg(not(feature = "logger"))]
    {
        vtx_shader = gl_util::shader_from_source( vtx_shader_src.as_ptr(), gl::VERTEX_SHADER, &mut error_message ).unwrap();
        frag_shader  = gl_util::shader_from_source( super::shaders::frag_shader_src.as_ptr(), gl::FRAGMENT_SHADER,  &mut error_message ).unwrap();
        unsafe{
            shader_prog = gl_util::program_from_shaders(vtx_shader, frag_shader, &mut error_message ).unwrap();
        }
    }

    #[cfg(feature = "logger")]
    {
        vtx_shader = match gl_util::shader_from_source( vtx_shader_src.as_ptr(), gl::VERTEX_SHADER, &mut error_message ) {
            Some( shader ) => shader,
            None => { super::show_error( error_message.as_ptr()  ); 0 }
        };
        unsafe{  
            super::util::read_file( "shader.glsl\0", &mut glbl_shader_code); 
            frag_shader  = match gl_util::shader_from_source( glbl_shader_code.as_ptr(), gl::FRAGMENT_SHADER,  &mut error_message ) {
                Some( shader ) => shader,
                None => { super::show_error( error_message.as_ptr() ); 0 }
            };
        }
        unsafe{
            shader_prog = match gl_util::program_from_shaders(vtx_shader, frag_shader, &mut error_message ) {
                Some( prog ) => prog,
                None => { super::show_error( error_message.as_ptr() ); 0 }
            };
        }
    }

    unsafe{
        super::log!( "Build terrain!");
        // Create the terrains by dropping some lumps and randomly aggregating points around them
        let mut rng_terrain : random::Rng = random::Rng{seed: core::num::Wrapping(7923129)};
        let mut lumps : [[f32;4];50] = [[0f32;4];50];
        let num_lumps = 50;

        let mut nl = 0;
        loop{
            // do not put the lumps too close to the edges to avoid ugly discontinuities
            set_r3( lumps.get_unchecked_mut(nl), &mut rng_terrain,0.8f32,0.8f32,0.8f32, -0.1 );
            nl += 1;
            if nl == num_lumps {break}
        }

        let  mut i = 0;
        loop{
            set_r3( spheres.get_unchecked_mut(0), &mut rng_terrain,1f32,1f32,1f32, 0.0 );
            let x = spheres.get_unchecked_mut(0)[0];
            let z = spheres.get_unchecked_mut(0)[2];

            let mut charge = 0.0;
            nl = 0;
            loop{
                let lmp = lumps.get(nl).unwrap();
                let dist = (x-lmp[0])*(x-lmp[0]) + (z-lmp[2])*(z-lmp[2]);
                charge += lmp[1]*0.0001/dist;
                nl += 1;
                if nl == num_lumps { break;}
            }
            *src_terrain.get_unchecked_mut( r3_pos ) += charge;
            if *src_terrain.get_unchecked( r3_pos ) > 1.0  {
                *src_terrain.get_unchecked_mut( r3_pos ) = 1.0
            }
            i += 1;
            if i== 700_000 { break}
        }

        // Smooth the terrain once to make it less 'craggy'
        smooth( &mut src_terrain); 
    }

    let mut vertex_buffer_id : gl::GLuint = 0;
    unsafe{
        // Create the map texture
        gl::GenTextures( 1, &mut tex_buffer_id );
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture( gl::TEXTURE_2D, tex_buffer_id );
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB, 512, 512, 0, gl::RGBA, gl::FLOAT, src_terrain.as_ptr() as *const CVoid);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
    }
}


static mut delay_counter : i32 = 0;
static mut play_pos : usize = 0;
static mut camera_speed : f32 = 1.0;

fn update_world( now: f32 ) {
    
    unsafe{
        delay_counter = *SEQUENCE.get_unchecked( play_pos*2+0 ) as i32*60;
        let arg : u32 = ((*SEQUENCE.get_unchecked( play_pos*2+1 )) & 0x0fff ) as u32;
        let mode : u16 = (*SEQUENCE.get_unchecked( play_pos*2+1 )) & 0xf000;

        super::log!( "Camera", arg as f32, camera_mode as f32);
        if mode == MODE_CAM_PAN {
            setup_camera( arg, camera_mode as u8 );
        } else if mode == MODE_CAM_SPEED {
            camera_speed = arg as f32;
        } else {
            sphere_delta = now;
            sphere_scale = arg as f32;
        }
        play_pos += 1;
    }
}

static mut cam_count : u32 = 1918;          // (1753 0 )

fn setup_random_camera( ) {
    let seed : u32;
    unsafe{ 
        cam_count += 1;
        setup_camera( cam_count, 0);
        camera_mode = 0;

    }   
}


fn setup_camera( seed : u32, mode : u8) {
    unsafe{ super::log!( "Setup Camera: ", mode as f32, seed as f32 ); }

    let mut crng : random::Rng = random::Rng{seed: core::num::Wrapping(9231249+seed)};
    unsafe{ super::log!( "Setup Camera: ", 2.0 ); }
    unsafe{
        super::log!( "Setup Camera: ", 11.0 );
        set_r3( &mut global_spheres[ CAMERA_POS_IDX ], &mut crng, 512f32, 512f32, 512f32, 0.0);
        global_spheres[ CAMERA_POS_IDX ][ 1 ] = (*src_terrain.get_unchecked( r3_pos ))*60.0-2.1+crng.next_f32()*5.0;
        super::log!( "Setup Camera: ", 12.0 );
        set_r3( &mut global_spheres[ CAMERA_ROT_IDX ], &mut crng, FP_1_54, 3.15, FP_0_05, 0.5 );
        set_r3( &mut camera_velocity, &mut crng, FP_0_20, FP_0_05, FP_0_20, 0.5);
        set_r3( &mut camera_rot_speed, &mut crng, 0.002, 0.001, 0.001, 0.5 );
    }
    unsafe{ super::log!( "Setup Camera: ", 3.0 ); }
    
}

const MODE_CAM_PAN   : u16 = 0x1000; 
const MODE_CAM_PIVOT : u16 = 0x3000; 
const MODE_CAM_SPEED : u16 = 0x4000; 
const MODE_SPHERE_SCALE : u16 = 0x5000; 

static SEQUENCE : &[u16] = &[
//     1200,   MODE_CAM_PAN | 1612,
// Slow pan in
28,   MODE_CAM_PAN | 786 ,
// Quick camera flashes
2,    MODE_CAM_PAN | 1223 ,
2,    MODE_CAM_PAN | 1239 ,
2,   MODE_CAM_PAN | 2157,  // join slow upshot 
// Hold on up side wall
4,    MODE_CAM_PAN | 945 ,


// Pan forward    /// find better
4,   MODE_CAM_PAN | 2290, // forward wtith accel
1,    MODE_CAM_SPEED | 12 ,
7,    MODE_CAM_SPEED | 1 ,

// up again and release the spheres
1,   MODE_CAM_PAN | 1849,
23,   MODE_SPHERE_SCALE | 48,

// lock down the spheres again
0,   MODE_SPHERE_SCALE | 1,
3,   MODE_CAM_PAN | 2102,  // spin down
1,  MODE_CAM_SPEED | 12 ,
4,  MODE_CAM_SPEED | 1 ,

4,   MODE_CAM_PAN | 2156,  // dunk down
0,   MODE_SPHERE_SCALE | 48,
22,   MODE_CAM_PAN | 2118,  //**
16,   MODE_CAM_PAN | 1011,

];

pub fn frame( now : f32 ) -> () {
    set_sphere_positions(now);

    unsafe {
        if delay_counter <= 0 {
            update_world( now );
            global_spheres[ CAMERA_CUT_INFO ][ 1 ] = 0f32;

        }
        delay_counter -= 1;
        global_spheres[ CAMERA_CUT_INFO ][ 1 ] += 1f32;
    }

    unsafe{
        // let mut dst:x86::__m128 = core::arch::x86::_mm_load_ps(global_spheres[ CAMERA_ROT_IDX ].as_mut_ptr());
        // let mut src:x86::__m128 = core::arch::x86::_mm_load_ps(camera_rot_speed.as_mut_ptr());
        // dst = core::arch::x86::_mm_add_ps( dst, src);
        // core::arch::x86::_mm_store_ss( (&mut global_spheres[ CAMERA_ROT_IDX ]).as_mut_ptr(), dst );
        global_spheres[ CAMERA_ROT_IDX ][ 0 ] += camera_rot_speed[ 0 ]*camera_speed;
        global_spheres[ CAMERA_ROT_IDX ][ 1 ] += camera_rot_speed[ 1 ]*camera_speed;
        global_spheres[ CAMERA_ROT_IDX ][ 2 ] += camera_rot_speed[ 2 ]*camera_speed;
            // dst = core::arch::x86::_mm_load_ps(global_spheres[ CAMERA_POS_IDX ].as_mut_ptr());
            // src = core::arch::x86::_mm_load_ps(camera_velocity.as_mut_ptr());
            // dst = core::arch::x86::_mm_add_ps( dst, src);
            // core::arch::x86::_mm_store_ss( (&mut global_spheres[ CAMERA_POS_IDX ]).as_mut_ptr(), dst );
        global_spheres[ CAMERA_POS_IDX ][ 0 ] += camera_velocity[ 0 ]*camera_speed;
        global_spheres[ CAMERA_POS_IDX ][ 1 ] += camera_velocity[ 1 ]*camera_speed;
        global_spheres[ CAMERA_POS_IDX ][ 2 ] += camera_velocity[ 2 ]*camera_speed;

        global_spheres[ CAMERA_CUT_INFO ][ 0 ] = delay_counter as f32;
        global_spheres[ CAMERA_CUT_INFO ][ 2 ] = now;
    }

    unsafe{
        gl::UseProgram(shader_prog);
        let shperes_loc : i32 = gl::GetUniformLocation(shader_prog, "sp\0".as_ptr());
        gl::Uniform4fv(shperes_loc, (num_spheres+sphere_extras) as i32 * 2, transmute::<_,*const gl::GLfloat>( global_spheres.as_ptr() ) );
        gl::Recti( -1, -1, 1, 1 );
    }
}