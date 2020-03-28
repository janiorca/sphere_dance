use super::math_util;
use super::gl;
use super::gl_util;
use super::random;

use gl::CVoid;
use core::mem::{size_of,transmute};
use core::ops::{Add,Sub,Mul};

pub const num_spheres : usize = 80;

static mut shader_prog : gl::GLuint = 0;
static mut vertex_array_id : gl::GLuint = 0;

static mut rng : random::Rng = random::Rng{seed: core::num::Wrapping(21431249)};

static mut global_spheres: [ [ [ f32; 4]; num_spheres*2]; 3 ] = [ [ [ 0f32; 4]; num_spheres*2 ]; 3 ];  

const SPHERES_CURRENT : usize = 0;
const SPHERES_TARGET : usize = 1;
const SPHERES_INTEPOLATOR : usize = 2;

static mut global_locals: [ f32; 16 ] = [ 0f32; 16 ];  

struct SphereData {
    target_pos : Vec3,
    sphere_class : u32,

}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vec3{
    x : f32, 
    y : f32, 
    z : f32, 
}

impl Vec3 {
    fn new( x : f32, y : f32, z : f32 ) -> Vec3 {
        Vec3{ x,y,z }   
    }
    fn squared_length( &self ) -> f32 {
        return self.x*self.x + self.y*self.y + self.z*self.z;
    }
}

impl Sub for Vec3{
    type Output = Vec3;

    fn sub( self, other : Vec3 ) -> Vec3 {
        Vec3{ x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl Add for Vec3{
    type Output = Vec3;

    fn add( self, other : Vec3 ) -> Vec3 {
        Vec3{ x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl Mul<f32> for Vec3{
    type Output = Vec3;

    fn mul( self, other : f32 ) -> Vec3 {
        Vec3{ x: self.x*other, y: self.y*other, z: self.z*other }
    }
}

fn randomize( pixels: &mut [ f32; 512*512 ], rng_terrain : &mut random::Rng, strength: f32 ) {
    for p in 0..512*512 {
        pixels[ p ] += ( rng_terrain.next_f32()*strength );
    }
}

fn smooth( pixels: &mut [ f32; 512*512 ]) {
    for y in 0..511 {
        for x in 0..511{
            let mut val  = pixels[ 512*y+x ];
            val += pixels[ 512*y+x+1 ];
            val += pixels[ 512*(y+1)+x ];
            val += pixels[ 512*(y+1)+x+1 ];
            pixels[ 512*y+x ] = (val / 4.0 );
        }
    }
}

static mut gpixels : [ u8; 512*512*4 ] = [ 125; 512*512*4 ];
static mut src_terrain  : [ f32; 512*512 ] = [ 0.0; 512*512 ];
static mut tex_buffer_id : gl::GLuint = 0;

pub fn prepare() -> () {
    let mut error_message : [i8;100] = [ 0; 100];
     let vtx_shader_src : &'static str = "#version 330 core
    layout (location = 0) in vec3 Pos;
    void main()
    {
     gl_Position = vec4(Pos, 1.0);
    }\0";

    let vtx_coords : [ [ gl::GLfloat; 3 ]; 4 ] = [
        [ -1.0, -1.0, 0.0 ],
        [ 1.0, -1.0, 0.0 ],
        [ -1.0,  1.0, 0.0 ],
        [ 1.0,  1.0, 0.0 ],
     ];
    
    let spheres : &mut[ [ [ f32; 4]; num_spheres*2]; 3 ];  
    unsafe{
        spheres  = &mut global_spheres;
    }
    
    let vtx_shader = match gl_util::shader_from_source( vtx_shader_src, gl::VERTEX_SHADER, &mut error_message ) {
        Some( shader ) => shader,
        None => { super::show_error( error_message.as_ptr()  ); 0 }
    };

    let frag_shader  = match gl_util::shader_from_source( super::shaders::frag_shader_src, gl::FRAGMENT_SHADER,  &mut error_message ) {
        Some( shader ) => shader,
        None => { super::show_error( error_message.as_ptr() ); 0 }
    };

    unsafe{
        shader_prog = match gl_util::program_from_shaders(vtx_shader, frag_shader, &mut error_message ) {
            Some( prog ) => prog,
            None => { super::show_error( error_message.as_ptr() ); 0 }
        };
    }

    // collect the values from f64 into u8 in a separate vec
    let mut rng_terrain : random::Rng = random::Rng{seed: core::num::Wrapping(9231249)};

    unsafe{
        let mut scl = 128.0;
        for itr in 0..7 {
            randomize( &mut src_terrain, &mut rng_terrain, scl ); 
            for i in 0..itr{
                smooth( &mut src_terrain); 
            }
            scl /= 3.0;
        }

        for y in 0..512  {
            for x in 0..512{
                let fx = ( 256.0 - x as f32 ) / 256f32;
                let fy = ( 256.0 - y as f32 ) / 256f32;
                let mut dist : f32 =  (fx*fx)+ (fy*fy);//.sqrt();
                dist = ( 1.0 / ( 0.28 + dist ) ).max( 2.4);
                gpixels[ (y*512+x)*4 as usize ] = ( src_terrain[ (y*512+x) as usize ] * (1.0+dist) ) as u8;
                gpixels[ (y*512+x)*4+1 as usize ] = ( src_terrain[ (y*512+x) as usize ] * (1.0+dist) ) as u8;
                gpixels[ (y*512+x)*4+2 as usize ] = ( src_terrain[ (y*512+x) as usize ] * (1.0+dist) ) as u8;
                gpixels[ (y*512+x)*4+3 as usize ] = ( src_terrain[ (y*512+x) as usize ] * (1.0+dist) ) as u8;
            }
        }
        gpixels[ (260*512+272)*4 as usize ] = 220;
        gpixels[ (259*512+272)*4 as usize ] = 220;
        gpixels[ (260*512+271)*4 as usize ] = 220;
        gpixels[ (259*512+271)*4 as usize ] = 220;
    }



    let mut vertex_buffer_id : gl::GLuint = 0;
    unsafe{
        // Generate 1 buffer, put the resulting identifier in vertexbuffer
        gl::GenBuffers(1, &mut vertex_buffer_id);
        // one vertex array to hold the vertex and its attributes
        gl::GenVertexArrays(1, &mut vertex_array_id );
        gl::BindVertexArray(vertex_array_id);
        // bind the buffer and load the vertices
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
        gl::BufferData( gl::ARRAY_BUFFER, size_of::<gl::GLfloat>() as isize * 3 * 4, vtx_coords.as_ptr() as *const gl::CVoid, gl::STATIC_DRAW);
        // enable and define vertex attributes 
        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer( 0,  3, gl::FLOAT, gl::FALSE, 3 * size_of::<gl::GLfloat>() as gl::GLint, 0 as *const CVoid );    

        // Create the map texture
        gl::GenTextures( 1, &mut tex_buffer_id );
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture( gl::TEXTURE_2D, tex_buffer_id );
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB, 512, 512, 0, gl::RGBA, gl::UNSIGNED_BYTE, gpixels.as_ptr() as *const CVoid);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    }

    for idx in 0..num_spheres {
        let fidx : f32  = idx as f32;
        let fidx2 : f32  = fidx*fidx;
        let adjustedTime = 0.0;
        spheres[ SPHERES_CURRENT][ idx*2 ][ 0 ] = 0.0f32;
        if idx > 0{
            spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 1000.0f32 + idx as f32;
        } else {
//            spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = -2.0f32 + 52.5;
            spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 0.0;
        }
        spheres[ SPHERES_CURRENT][ idx*2 ][ 2 ] = 0.0f32;
        spheres[ SPHERES_CURRENT][ idx*2 ][ 3 ] = 0.15*0.15;    //25.3f32;            // 5^2
        spheres[ SPHERES_CURRENT][ idx*2+1 ][ 0 ] = 0.02f32;
        spheres[ SPHERES_CURRENT][ idx*2+1 ][ 1 ] = 0.02f32;
        spheres[ SPHERES_CURRENT][ idx*2+1 ][ 2 ] = 0.02f32;
        spheres[ SPHERES_CURRENT][ idx*2+1 ][ 3 ] = 0.15*0.15;    //1.317f32;
//        spheres[ idx ] = spos + vec3( 0.0, 52.5, 0.0 );

        // spheres[ SPHERES_TARGET][ idx*2 ][ 3 ] = 25.3f32;
        // spheres[ SPHERES_TARGET][ idx*2+1 ][ 0 ] = 0.02f32;
        // spheres[ SPHERES_TARGET][ idx*2+1 ][ 1 ] = 0.02f32;
        // spheres[ SPHERES_TARGET][ idx*2+1 ][ 2 ] = 0.02f32;
        // spheres[ SPHERES_TARGET][ idx*2+1 ][ 3 ] = 1.317f32;
    }
    frame( 960, 0.0f32, false );
//    frame( 1020, false );
}

const CONST_BEAT : u8 = 140;

const INSTRUCTION_PTR : u8  = 0;
const DELAY_COUNTER : u8  = 1;
const X_ADJUST : u8  = 2;
const Y_ADJUST : u8  = 3;
const Z_ADJUST : u8  = 4;
const SIZE_ADJUST : u8  = 5;

const R_ADJUST : u8  = 2;
const G_ADJUST : u8  = 3;
const B_ADJUST : u8  = 4;
const REF_ADJUST : u8  = 5;

const LOOP_POS : u8  = 6;
const REPULSION : u8 = 7;
const REPULSE_MAX : u8 = 8;

const CLASS_DIVIDER : u8 = 9;

const ZERO : u8 = 0;
const ONE : u8 = 1;
const POINT2 : u8 = 2;
const CREATE_OFFSET : u8 = 3;
const REPULSION1 : u8 = 4;
const REPULSION2 : u8 = 5;
const REPULSE_MAX1 : u8 = 6; 
const REPULSE_MAX2 : u8 = 7; 
const LOW_RG : u8 = 8;
const HI_B : u8 = 9;
const REFRACT_LO : u8 = 10;
const MINI_SIZE_SQRD : u8 = 11;
const MID_SIZE_SQRD : u8 = 12;
const BIG_SIZE_SQRD : u8 = 13;

static global_consts : &[f32] = &[ 
    0.0f32,     //ZERO
    1.0f32,     //ONE
    0.2f32,     //_POINT2
    0.00011f32,   //CREATE_OFFSET
    0.004f32,    // REPULSION1
    0.004f32,     // REPULSION2
    1.01f32,    // REPULSE_MAX1
    4.02f32,    // REPULSE_MAX2

    0.02f32,     // LOW_RG
    0.952f32,    // HI_B
    1.09131f32,  // REFRACT_LO
    0.1*0.1,    //    9.0f32,     // MINI_SIZE_SQRD
    0.15*0.15,      // 49.0f32,     // MID_SIZE_SQRD
    0.25*0.25   // 136.0f32     // BIG_SIZE_SQRD
];

const SetLocal : u8 = 1;                // dest idx, src idx
const SetLocalI : u8 = 2;               // dest idx, value (u8, converted to f32) 
const CopyAdjust : u8 = 3;              // dest idx, src idx, count
const CopyRNDAdjust : u8 = 4;              // dest idx, src idx, count
const SetV : u8 = 5;


static global_program : &[u8] = &[
    // data ptr at 0
    // vec at 0

    SetLocalI,          CLASS_DIVIDER, 80,
    SetLocal,           REPULSE_MAX, REPULSE_MAX1,
    SetLocal,           REPULSION, REPULSION1,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocal,           X_ADJUST, CREATE_OFFSET,
    SetLocal,           Y_ADJUST, CREATE_OFFSET,
    SetLocal,           Z_ADJUST, CREATE_OFFSET,
    CopyRNDAdjust,      2,0,1,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      4,0,2,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      8,0,4,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      16,0,8,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      32,0,16,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      64,0,32,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    CopyRNDAdjust,      128,0,16,               // to  location 1, from location 0, total of 1 copies
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    // Push out everything
    SetLocal,           REPULSE_MAX, REPULSE_MAX2,      // Maybe push camera out at this point
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,


//    SetLocal,           R_ADJUST, LOW_RG,
//    SetLocal,           G_ADJUST, LOW_RG,
//    SetLocal,           B_ADJUST, HI_B,
//    SetLocal,           REF_ADJUST, REFRACT_LO,
//    Set,                21,70,               // Set all colors from sphere 10 ( 20*2+1) onwards ( other 70 )

    // Create the blue sphere colors   ( DO WE need an inetrpolatorr for them ¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬``!!!!!! )
    SetV,               SPHERES_TARGET as u8, 21,70, LOW_RG, LOW_RG, HI_B, REFRACT_LO,
    SetV,               SPHERES_INTEPOLATOR as u8, 21,70, POINT2, POINT2, POINT2, POINT2,
    SetLocalI,          CLASS_DIVIDER, 10,          // seggregate into different class
    // Shrink the blue spheres
    SetV,               SPHERES_TARGET as u8, 20,70, ZERO, ZERO,ZERO, MINI_SIZE_SQRD,
    SetV,               SPHERES_INTEPOLATOR as u8, 20,70, ZERO, ZERO, ZERO, POINT2,
    // And pukll everything back in
    SetLocal,           REPULSE_MAX, REPULSE_MAX1,      // Maybe push camera out at this point
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    SetV,               SPHERES_INTEPOLATOR as u8, 0,10, POINT2, POINT2, POINT2, POINT2,
    SetV,               SPHERES_TARGET as u8, 0,10, LOW_RG, LOW_RG, HI_B, MID_SIZE_SQRD,

    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    // SetLocal,           SIZE_ADJUST,MINI_SIZE_SQRD,
    // Set,                20,70,               // Set all size from sphere 10 ( 20*2+1) onwards ( other 70 )
    // SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    

//    SetLocal,           SIZE_ADJUST,BIG_SIZE_SQRD,
//    Set,                0,1,               // Set all size from sphere 10 ( 20*2+1) onwards ( other 70 )
    // Move and grow the central sphere
    SetV,                 SPHERES_TARGET as u8, 0,1, CREATE_OFFSET, MINI_SIZE_SQRD, CREATE_OFFSET, BIG_SIZE_SQRD,
    SetV,                 SPHERES_INTEPOLATOR as u8, 0,1, LOW_RG, LOW_RG, LOW_RG, LOW_RG,

    // Set the position targert of 0
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,

    // Set the class divider ( below limit class 0, otherwise 1)


    // start with shadow vars for color that is interpolated to constantly
    // each sphere to have shadow target vars + own repulse _ attraction 
    // Set color and size of some with copy adjust
    // Set size of all spheres 

    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    SetLocalI,          DELAY_COUNTER, CONST_BEAT,
];

fn executeCommand( program : &[ u8], locals : &mut [f32], spheres : &mut [ [ [ f32; 4]; num_spheres*2]; 3 ], consts : &[f32] ){
    let ip : usize = locals[ INSTRUCTION_PTR as usize ] as usize;
    let instruction = program[ ip ];
    match instruction{
        SetLocal => {
            let dst_offset = program[ ip+ 1] as usize;
            let src_offset = program[ ip + 2 ] as usize;
            locals[ dst_offset ] = consts[ src_offset ];
            locals[ INSTRUCTION_PTR as usize ] += 3.0f32;
        },
        SetLocalI => {
            let dst_offset = program[ ip+ 1] as usize;
            let src : f32 = program[ ip + 2 ] as f32;
            locals[ dst_offset ] = src;
            locals[ INSTRUCTION_PTR as usize ] += 3.0f32;

        },  
//         Set => {
//             let dst_offset = program[ ip+ 1] as usize;
//             let count = program[ ip + 2 ] as usize;
//             for i in 0..count {
//                 for j in 0..4 {
// //                    spheres[ dst_offset+i*2 ][ j ] = locals[ j + X_ADJUST as usize ]; 
//                     unsafe{
//                         global_spheres_target[ dst_offset+i*2 ][ j ] = locals[ j + X_ADJUST as usize ]; 

//                     }
//                 }
//             }
//             locals[ INSTRUCTION_PTR as usize ] += 3.0f32;
//         }
        SetV => {
            let dst_layer = program[ ip+ 1] as usize;
            let dst_offset = program[ ip+ 2] as usize;
            let count = program[ ip + 3 ] as usize;
            for i in 0..count {
                for j in 0..4 {
                    spheres[ dst_layer ][ dst_offset+i*2 ][ j ] = consts[ program[ ip+4+j ] as usize ]; 
                }
            }
            locals[ INSTRUCTION_PTR as usize ] += 8.0f32;
        }
        CopyAdjust => {
            let dst_offset = program[ ip+ 1] as usize;
            let src_offset = program[ ip + 2 ] as usize;
            let count = program[ ip + 3 ] as usize;
            for i in 0..count {
                for j in 0..4 {
                    spheres[SPHERES_CURRENT][ dst_offset+i*2 ][ j ] = spheres[SPHERES_CURRENT][ src_offset+i*2 ][ j ] + locals[ j + X_ADJUST as usize ]; 
                }
            }
            locals[ INSTRUCTION_PTR as usize ] += 4.0f32;
        },
        CopyRNDAdjust => {
            let dst_offset = program[ ip+ 1] as usize;
            let src_offset = program[ ip + 2 ] as usize;
            let count = program[ ip + 3 ] as usize;
            for i in 0..count {
                for j in 0..4 {
                    unsafe{
                        spheres[SPHERES_CURRENT][ dst_offset+i*2 ][ j ] = spheres[SPHERES_CURRENT][ src_offset+i*2 ][ j ] + locals[ j + X_ADJUST as usize ]*rng.next_f32(); 
                    }
                }
            }
            locals[ INSTRUCTION_PTR as usize ] += 4.0f32;
        },
        _ => {
            return;
        }
    }
}

pub fn clamp( x : f32, min_value: f32, max_value: f32 ) -> f32 {
    return x.max( min_value).min( max_value );
}

pub fn smooth_Step( x: f32, min_value: f32, max_value: f32 ) -> f32 {
    let val : f32 = clamp((x - min_value) / (max_value - min_value), 0.0, 1.0); 
    return val * val * (3.0f32 - 2.0f32 * val);
}

// calc sphere positions
pub fn frame( ticks : u32, now : f32, render_frame : bool ) -> () {
    let spheres : &mut[ [ [ f32; 4]; num_spheres*2]; 3 ];  
    let locals : &mut [ f32; 16 ];
    unsafe{
        spheres  = &mut global_spheres;
        locals   = &mut global_locals;
    }

    for tick in 0..ticks {

        loop{
            if locals[ DELAY_COUNTER as usize ] > 0.0 {
                break;
            }
            executeCommand( global_program, locals, spheres, global_consts );
        }
        locals[ DELAY_COUNTER as usize ] -= 1.0f32;


        let t : f32 = now*2.0;
//        let mut adjustedTime : f32  = (30.0*t - 45.0*math_util::cos(t) + math_util::cos(3.0f32*t) - 9.0* math_util::sin(2.0f32*t))/96.0;
        let mut adjustedTime : f32  = t;//(30.0*t - 45.0*math_util::cos(t) + math_util::cos(3.0f32*t) - 9.0* math_util::sin(2.0f32*t))/96.0;
        // adjustedTime += time*0.1;


        for idx in 0..locals[ CLASS_DIVIDER as usize ] as usize {
            let fidx : f32  = idx as f32;

            let fidx2 : f32  = fidx*fidx;
            spheres[SPHERES_TARGET][ idx*2 ][ 0 ] = math_util::sin( fidx*0.41212f32 + adjustedTime * 0.722f32 + 0.423f32 + fidx2*0.324f32) * 0.70f32;
            spheres[SPHERES_TARGET][ idx*2 ][ 1 ] = math_util::sin( fidx*0.3312f32 + adjustedTime * 0.32f32 + 0.23f32+ fidx2*1.54f32 ) * 0.70f32;
            spheres[SPHERES_TARGET][ idx*2 ][ 2 ] = math_util::sin( fidx*0.2912f32 + adjustedTime * 0.125f32 + 1.3f32 + fidx2*1.1f32 ) * 0.70f32;
//            spheres[SPHERES_TARGET][ idx*2 ][ 3 ] = 100f32;
            // if idx >= 10 {
            //     spheres[ idx*2 ][ 3 ] = 4.1;
            //     spheres[ idx*2+1 ][ 0 ] = 0.02f32;
            //     spheres[ idx*2+1 ][ 1 ] = 0.02f32;
            //     spheres[ idx*2+1 ][ 2 ] = 0.952f32;
            //     spheres[ idx*2+1 ][ 3 ] = 1.09131f32;
            // } else {
            //     spheres[ idx*2 ][ 3 ] = 25.3f32;
            //     spheres[ idx*2+1 ][ 0 ] = 0.02f32;
            //     spheres[ idx*2+1 ][ 1 ] = 0.02f32;
            //     spheres[ idx*2+1 ][ 2 ] = 0.02f32;
            //     spheres[ idx*2+1 ][ 3 ] = 1.317f32;
            // }
        }
        
        for idx in 0..num_spheres {
            let mut pos = Vec3::new( spheres[SPHERES_CURRENT][ idx*2 ][ 0 ], spheres[SPHERES_CURRENT][ idx*2 ][ 1 ], spheres[SPHERES_CURRENT][ idx*2 ][ 2 ] );
            for effect_idx in 0..num_spheres { 
                if effect_idx == idx {
                    continue;
                }
                let effect_pos = Vec3::new( spheres[SPHERES_CURRENT][ effect_idx*2 ][ 0 ], spheres[SPHERES_CURRENT][ effect_idx*2 ][ 1 ], spheres[SPHERES_CURRENT][ effect_idx*2 ][ 2 ] );
                let mut effect_dir = pos - effect_pos;
                let sql : f32;
                unsafe{
                    sql = core::intrinsics::sqrtf32(effect_dir.squared_length());
                } 
                effect_dir = effect_dir.mul(1.0/sql);
                let mut repulse_mid : f32;
                unsafe{ 
                    repulse_mid =  core::intrinsics::sqrtf32( spheres[SPHERES_CURRENT][ idx*2 ][ 3 ] ) + core::intrinsics::sqrtf32( spheres[SPHERES_CURRENT][ effect_idx*2 ][ 3 ] );
                }
                let mut strength =  locals[ REPULSION as usize ]  * ( 1.0f32 - smooth_Step(sql, repulse_mid*0.8f32, repulse_mid*1.2f32*locals[ REPULSE_MAX as usize ] ) );
                if  idx >= locals[ CLASS_DIVIDER as usize ] as usize  {
                    if sql > repulse_mid  {
                        if effect_idx < locals[ CLASS_DIVIDER as usize ] as usize {
                            strength -= ( 0.004f32 / sql ).min( 0.01);
                        }
                    }
                }
                pos = pos + effect_dir * strength;
            }
            spheres[SPHERES_CURRENT][ idx*2 ][ 0 ] = pos.x;
            spheres[SPHERES_CURRENT][ idx*2 ][ 1 ] = pos.y;
            spheres[SPHERES_CURRENT][ idx*2 ][ 2 ] = pos.z;
        }
        // // Interpolate sphere attributes
        for idx in 0..num_spheres*2 {
            for sub_idx in 0..4 {
                spheres[SPHERES_CURRENT][ idx ][ sub_idx ] = spheres[SPHERES_CURRENT][ idx ][ sub_idx ]*(1.0f32-spheres[SPHERES_INTEPOLATOR][ idx ][ sub_idx ]) + 
                                                                        spheres[SPHERES_TARGET][ idx ][ sub_idx ]*spheres[SPHERES_INTEPOLATOR][ idx ][ sub_idx ];
            }
        }
    }
    if render_frame {
        unsafe{
            let rgba = &[ 0.4f32, 1.0, 0.9, 0.0 ];
            gl::ClearBufferfv(gl::COLOR, 0, rgba as *const _ );  

            gl::UseProgram(shader_prog);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture( gl::TEXTURE_2D, tex_buffer_id );
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB, 512, 512, 0, gl::RGBA, gl::UNSIGNED_BYTE, gpixels.as_ptr() as *const CVoid);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32 );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32 );
    
            let time_loc : i32 = gl::GetUniformLocation(shader_prog, "iTime\0".as_ptr());
    //        let time_loc : i32 = gl::GetUniformLocation(shader_prog, "e\0".as_ptr());
            gl::Uniform1f(time_loc, now );          //time

//            let shperes_loc : i32 = gl::GetUniformLocation(shader_prog, "d\0".as_ptr());
            let shperes_loc : i32 = gl::GetUniformLocation(shader_prog, "spheres\0".as_ptr());
            gl::Uniform4fv(shperes_loc, num_spheres as i32 * 2, transmute::<_,*const gl::GLfloat>( spheres.as_ptr() ) );

            gl::BindVertexArray(vertex_array_id);
            gl::DrawArrays( gl::TRIANGLE_STRIP, 0, 4 );
        }
    }
}