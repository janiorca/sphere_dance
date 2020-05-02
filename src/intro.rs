use super::math_util;
use super::gl;
use super::gl_util;
use super::random;

use gl::CVoid;
use core::mem::{size_of,transmute};
use core::ops::{Add,Sub,Mul};

pub const num_spheres : usize = 80;
pub const sphere_extras : usize = 2;

static mut shader_prog : gl::GLuint = 0;
static mut vertex_array_id : gl::GLuint = 0;

static mut rng : random::Rng = random::Rng{seed: core::num::Wrapping(21431249)};

static mut global_spheres: [ [ [ f32; 4]; (num_spheres+sphere_extras)*2]; 3 ] = [ [ [ 0f32; 4]; (num_spheres+sphere_extras)*2 ]; 3 ];  

const SPHERES_CURRENT : usize = 0;
const SPHERES_TARGET : usize = 1;
const SPHERES_INTEPOLATOR : usize = 2;

static mut global_locals: [ f32; 16 ] = [ 0f32; 16 ];  

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

// fn randomize( pixels: &mut [ f32; 512*512 ], rng_terrain : &mut random::Rng, strength: f32 ) {
//     for p in 0..512*512 {
//         pixels[ p ] += ( rng_terrain.next_f32()*strength );
//     }
// }

fn smooth( pixels: &mut [ f32; 512*512*4 ]) {
    for y in 0..511 {
        for x in 0..511{
            let mut val  = pixels[ (512*y+x)*4 ];
            val += pixels[ (512*y+x+1)*4 ];
            val += pixels[ (512*(y+1)+x)*4 ];
            val += pixels[ (512*(y+1)+x+1)*4 ];
            pixels[ (512*y+x)*4 ] = val / 4.0;
        }
    }
}

//static mut gpixels : [ u8; 512*512*4 ] = [ 0; 512*512*4 ];      // large ones are OK as long as they are 0. Otherwise crinkler chokes
static mut src_terrain  : [ f32; 512*512*4 ] = [ 0.0; 512*512*4 ];
static mut tex_buffer_id : gl::GLuint = 0;

#[cfg(feature = "logger")]
static mut glbl_shader_code : [ u8;25000] = [0; 25000];

static mut old_x : i32 = 0;
static mut old_y : i32 = 0;
static mut moving_camera : bool  = false;
static mut world_pos : [ f32;3] = [ 0.0; 3];
static mut rotating_camera : bool  = false;
static mut world_rot : [ f32;3] = [ 0.0; 3];

static mut camera_velocity : [ f32; 3] = [ 0.0; 3];
static mut camera_rot_speed : [ f32; 3] = [ 0.0; 3];

static mut pivot_cam_centre : [ f32; 3] = [ 0.0; 3];
static mut pivot_cam_dist : [ f32; 3] = [ 0.0; 3];
static mut pivot_cam_angle : [ f32; 3] = [ 0.0; 3];

static mut camera_mode : u32 = 0;

#[cfg(feature = "logger")]
pub fn set_pos( x: i32, y: i32, ctrl : bool ) {
    unsafe{
        if moving_camera {
            if ctrl{
                world_pos[ 1 ] += ( y-old_y) as f32 / 32.0;
            } else {
                world_pos[ 0 ] += ( x-old_x) as f32 / 32.0;
                world_pos[ 2 ] += ( y-old_y) as f32 / 32.0;
            }
        } else if rotating_camera {
            world_rot[ 0 ] += ( y-old_y) as f32 / 1024.0;
            world_rot[ 1 ] += ( x-old_x) as f32 / 1024.0;
//            world_rot[ 2 ] += ( y-old_y) as f32 / 32.0;
    
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
    unsafe{ super::log!( "Camera: ", world_pos[ 0 ], world_pos[ 1 ], world_pos[ 2 ]); }
}


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
    
    let spheres : &mut[ [ [ f32; 4]; (num_spheres+sphere_extras)*2]; 3 ];  
    unsafe{
        spheres  = &mut global_spheres;
    }
    
    let vtx_shader : u32;
    let frag_shader : u32;
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

    // collect the values from f64 into u8 in a separate vec

    unsafe{
        let mut rng_terrain : random::Rng = random::Rng{seed: core::num::Wrapping(9231249)};
        let mut lumps : [(f32,f32,f32);50] = [(0.0,0.0,0.0);50];
        let num_lumps = 50;

        for nl in 0..num_lumps {
            let x = rng_terrain.next_f32();
            let y = rng_terrain.next_f32();
            let f = rng_terrain.next_f32();
            lumps[ nl ] = (x,y,f);
        }

//        filled_spheres = 
        for i in 0..1_000_000 {
            let x = rng_terrain.next_f32();
            let y = rng_terrain.next_f32();
            let mut charge = 0.0;
            for nl in 0..num_lumps {
                let lmp = lumps.get(nl).unwrap();
                let dist = (x-lmp.0)*(x-lmp.0) + (y-lmp.1)*(y-lmp.1);
                charge += lmp.2*0.0001/dist;
            }
            let pos = (((y*512f32) as usize *512)+(x*512f32) as usize)*4;
            src_terrain[ pos ] += charge;
            if src_terrain[ pos ] > 1.0  {
                src_terrain[ pos ] = 1.0
            }
        }
        smooth( &mut src_terrain); 
        let x : u32 = 256 + 130;
        let y : u32 = 256 + 191;
        let pos = ((y*512)+(x))*4+1;
        super::log!( "!!!!");
        super::log!( "", src_terrain[ (pos-1) as usize] );
        super::log!( "!!!!");
        src_terrain[ pos as usize] = 1.0f32;//500.0f32;

        for idx in 0..num_spheres {
            loop{
                let x = rng_terrain.next_f32();
                let y = rng_terrain.next_f32();
                // let pos = (((y*512f32) as usize *512)+(x*512f32) as usize)*4;
                // let dist = x*x+y*y;

                let pos = (((y*512f32) as usize *512)+(x*512f32) as usize)*4;
                if src_terrain[ pos ] > 0.3f32 {
                    spheres[ SPHERES_CURRENT][ idx*2 ][ 0 ] = (x-0.5)*512f32;
                    spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = src_terrain[ pos ]*60.0-12.1;
                    spheres[ SPHERES_CURRENT][ idx*2 ][ 2 ] = (y-0.5)*512f32;
                    spheres[ SPHERES_CURRENT][ idx*2 ][ 3 ] = 18.0f32;//0.915*0.915;    //25.3f32;            // 5^2
                    spheres[ SPHERES_CURRENT][ idx*2+1 ][ 0 ] = 0.02f32;
                    spheres[ SPHERES_CURRENT][ idx*2+1 ][ 1 ] = 0.02f32;
                    spheres[ SPHERES_CURRENT][ idx*2+1 ][ 2 ] = 0.02f32;
                    spheres[ SPHERES_CURRENT][ idx*2+1 ][ 3 ] = 1.317f32;
                    break;
                }
            }
            
            // let fidx : f32  = idx as f32;
            // let fidx2 : f32  = fidx*fidx;
            // let adjustedTime = 0.0;
            // spheres[ SPHERES_CURRENT][ idx*2 ][ 0 ] = 0.0f32;
            // if idx > 0{
            //     spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 1000.0f32 + idx as f32;
            // } else {
            //     spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 1.0;
            // }
            // spheres[ SPHERES_CURRENT][ idx*2 ][ 2 ] = 0.0f32;
            // spheres[ SPHERES_CURRENT][ idx*2 ][ 3 ] = 0.15*0.15;    //25.3f32;            // 5^2
            // spheres[ SPHERES_CURRENT][ idx*2+1 ][ 0 ] = 0.02f32;
            // spheres[ SPHERES_CURRENT][ idx*2+1 ][ 1 ] = 0.02f32;
            // spheres[ SPHERES_CURRENT][ idx*2+1 ][ 2 ] = 0.02f32;
            // spheres[ SPHERES_CURRENT][ idx*2+1 ][ 3 ] = 0.15*0.15;    //1.317f32;
        }

        // for y in 0..512  {
        //     for x in 0..512{
        //         gpixels[ (y*512+x)*4 as usize ] = (src_terrain[ (y*512+x) as usize ]*255.0) as u8;
        //     }
        // }

        // let mut scl = 128.0;
        // for itr in 0..7 {
        //     randomize( &mut src_terrain, &mut rng, scl ); 
        //     for i in 0..itr{
        //         smooth( &mut src_terrain); 
        //     }
        //     scl /= 3.0;
        // }

        // for y in 0..512  {
        //     for x in 0..512{
        //         let fx = ( 256.0 - x as f32 ) / 256f32;
        //         let fy = ( 256.0 - y as f32 ) / 256f32;
        //         let mut dist : f32 =  ((fx*fx)+ (fy*fy));//.sqrt();
        //         dist = ( 1.0 / ( 0.28 + dist ) ).max( 2.4);
        //         gpixels[ (y*512+x)*4 as usize ] = ( src_terrain[ (y*512+x) as usize ] * (1.0+dist) ) as u8;
        //     }
        // }
        // gpixels[ (256*512+272) as usize ] = 255;
        // gpixels[ (255*512+272) as usize ] = 255;
        // gpixels[ (260*512+272) as usize ] = 220;
        // gpixels[ (259*512+272) as usize ] = 220;
        // gpixels[ (260*512+271) as usize ] = 220;
        // gpixels[ (259*512+271) as usize ] = 220;
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
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB, 512, 512, 0, gl::RGBA, gl::FLOAT, src_terrain.as_ptr() as *const CVoid);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    }

    // for idx in 0..num_spheres {
    //     let fidx : f32  = idx as f32;
    //     let fidx2 : f32  = fidx*fidx;
    //     let adjustedTime = 0.0;
    //     spheres[ SPHERES_CURRENT][ idx*2 ][ 0 ] = 0.0f32;
    //     if idx > 0{
    //         spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 1000.0f32 + idx as f32;
    //     } else {
    //         spheres[ SPHERES_CURRENT][ idx*2 ][ 1 ] = 1.0;
    //     }
    //     spheres[ SPHERES_CURRENT][ idx*2 ][ 2 ] = 0.0f32;
    //     spheres[ SPHERES_CURRENT][ idx*2 ][ 3 ] = 0.15*0.15;    //25.3f32;            // 5^2
    //     spheres[ SPHERES_CURRENT][ idx*2+1 ][ 0 ] = 0.02f32;
    //     spheres[ SPHERES_CURRENT][ idx*2+1 ][ 1 ] = 0.02f32;
    //     spheres[ SPHERES_CURRENT][ idx*2+1 ][ 2 ] = 0.02f32;
    //     spheres[ SPHERES_CURRENT][ idx*2+1 ][ 3 ] = 0.15*0.15;    //1.317f32;
    // }
//    frame( 960, 0.0f32, false );
//    frame( 1020, false );
    unsafe{
        camera_velocity = [ 0.01, 0.001, 0.0];
        camera_rot_speed = [ 0.0, 0.001, 0.00];
    }
    setup_random_camera();
}

// const CONST_BEAT : u8 = 140;

// const INSTRUCTION_PTR : u8  = 0;
// const DELAY_COUNTER : u8  = 1;
// const X_ADJUST : u8  = 2;
// const Y_ADJUST : u8  = 3;
// const Z_ADJUST : u8  = 4;
// const SIZE_ADJUST : u8  = 5;

// const R_ADJUST : u8  = 2;
// const G_ADJUST : u8  = 3;
// const B_ADJUST : u8  = 4;
// const REF_ADJUST : u8  = 5;

// const LOOP_POS : u8  = 6;
// const REPULSION : u8 = 7;
// const REPULSE_MAX : u8 = 8;

// const CLASS_DIVIDER : u8 = 9;

// const ZERO : u8 = 0;
// const ONE : u8 = 1;
// const POINT2 : u8 = 2;
// const CREATE_OFFSET : u8 = 3;
// const REPULSION1 : u8 = 4;
// const REPULSION2 : u8 = 5;
// const REPULSE_MAX1 : u8 = 6; 
// const REPULSE_MAX2 : u8 = 7; 
// const LOW_RG : u8 = 8;
// const HI_B : u8 = 9;
// const REFRACT_LO : u8 = 10;
// const MINI_SIZE_SQRD : u8 = 11;
// const MID_SIZE_SQRD : u8 = 12;
// const BIG_SIZE_SQRD : u8 = 13;

// static global_consts : &[f32] = &[ 
//     0.0f32,     //ZERO
//     1.0f32,     //ONE
//     0.2f32,     //_POINT2
//     0.00011f32,   //CREATE_OFFSET
//     0.004f32,    // REPULSION1
//     0.004f32,     // REPULSION2
//     1.01f32,    // REPULSE_MAX1
//     4.02f32,    // REPULSE_MAX2

//     0.02f32,     // LOW_RG
//     0.952f32,    // HI_B
//     1.09131f32,  // REFRACT_LO
//     0.1*0.1,    //    9.0f32,     // MINI_SIZE_SQRD
//     0.15*0.15,      // 49.0f32,     // MID_SIZE_SQRD
//     0.25*0.25   // 136.0f32     // BIG_SIZE_SQRD
// ];

// const SetLocal : u8 = 1;                // dest idx, src idx
// const SetLocalI : u8 = 2;               // dest idx, value (u8, converted to f32) 
// const CopyAdjust : u8 = 3;              // dest idx, src idx, count
// const CopyRNDAdjust : u8 = 4;              // dest idx, src idx, count
// const SetV : u8 = 5;



// static global_program : &[u8] = &[
//     // data ptr at 0
//     // vec at 0

//     SetLocalI,          CLASS_DIVIDER, 80,
//     SetLocal,           REPULSE_MAX, REPULSE_MAX1,
//     SetLocal,           REPULSION, REPULSION1,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocal,           X_ADJUST, CREATE_OFFSET,
//     SetLocal,           Y_ADJUST, CREATE_OFFSET,
//     SetLocal,           Z_ADJUST, CREATE_OFFSET,
//     CopyRNDAdjust,      2,0,1,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      4,0,2,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      8,0,4,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      16,0,8,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      32,0,16,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      64,0,32,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     CopyRNDAdjust,      128,0,16,               // to  location 1, from location 0, total of 1 copies
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     // Push out everything
//     SetLocal,           REPULSE_MAX, REPULSE_MAX2,      // Maybe push camera out at this point
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,


// //    SetLocal,           R_ADJUST, LOW_RG,
// //    SetLocal,           G_ADJUST, LOW_RG,
// //    SetLocal,           B_ADJUST, HI_B,
// //    SetLocal,           REF_ADJUST, REFRACT_LO,
// //    Set,                21,70,               // Set all colors from sphere 10 ( 20*2+1) onwards ( other 70 )

//     // Create the blue sphere colors   ( DO WE need an inetrpolatorr for them ¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬¬``!!!!!! )
//     SetV,               SPHERES_TARGET as u8, 21,70, LOW_RG, LOW_RG, HI_B, REFRACT_LO,
//     SetV,               SPHERES_INTEPOLATOR as u8, 21,70, POINT2, POINT2, POINT2, POINT2,
//     SetLocalI,          CLASS_DIVIDER, 10,          // seggregate into different class
//     // Shrink the blue spheres
//     SetV,               SPHERES_TARGET as u8, 20,70, ZERO, ZERO,ZERO, MINI_SIZE_SQRD,
//     SetV,               SPHERES_INTEPOLATOR as u8, 20,70, ZERO, ZERO, ZERO, POINT2,
//     // And pukll everything back in
//     SetLocal,           REPULSE_MAX, REPULSE_MAX1,      // Maybe push camera out at this point
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     SetV,               SPHERES_INTEPOLATOR as u8, 0,10, POINT2, POINT2, POINT2, POINT2,
//     SetV,               SPHERES_TARGET as u8, 0,10, LOW_RG, LOW_RG, HI_B, MID_SIZE_SQRD,

//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     // SetLocal,           SIZE_ADJUST,MINI_SIZE_SQRD,
//     // Set,                20,70,               // Set all size from sphere 10 ( 20*2+1) onwards ( other 70 )
//     // SetLocalI,          DELAY_COUNTER, CONST_BEAT,
    

// //    SetLocal,           SIZE_ADJUST,BIG_SIZE_SQRD,
// //    Set,                0,1,               // Set all size from sphere 10 ( 20*2+1) onwards ( other 70 )
//     // Move and grow the central sphere
//     SetV,                 SPHERES_TARGET as u8, 0,1, CREATE_OFFSET, MINI_SIZE_SQRD, CREATE_OFFSET, BIG_SIZE_SQRD,
//     SetV,                 SPHERES_INTEPOLATOR as u8, 0,1, LOW_RG, LOW_RG, LOW_RG, LOW_RG,

//     // Set the position targert of 0
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,

//     // Set the class divider ( below limit class 0, otherwise 1)


//     // start with shadow vars for color that is interpolated to constantly
//     // each sphere to have shadow target vars + own repulse _ attraction 
//     // Set color and size of some with copy adjust
//     // Set size of all spheres 

//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
//     SetLocalI,          DELAY_COUNTER, CONST_BEAT,
// ];

// fn executeCommand( program : &[ u8], locals : &mut [f32], spheres : &mut [ [ [ f32; 4]; (num_spheres+sphere_extras)*2]; 3 ], consts : &[f32] ){
//     let ip : usize = locals[ INSTRUCTION_PTR as usize ] as usize;
//     let instruction = program[ ip ];
//     match instruction{
//         SetLocal => {
//             let dst_offset = program[ ip+ 1] as usize;
//             let src_offset = program[ ip + 2 ] as usize;
//             locals[ dst_offset ] = consts[ src_offset ];
//             locals[ INSTRUCTION_PTR as usize ] += 3.0f32;
//         },
//         SetLocalI => {
//             let dst_offset = program[ ip+ 1] as usize;
//             let src : f32 = program[ ip + 2 ] as f32;
//             locals[ dst_offset ] = src;
//             locals[ INSTRUCTION_PTR as usize ] += 3.0f32;

//         },  
// //         Set => {
// //             let dst_offset = program[ ip+ 1] as usize;
// //             let count = program[ ip + 2 ] as usize;
// //             for i in 0..count {
// //                 for j in 0..4 {
// // //                    spheres[ dst_offset+i*2 ][ j ] = locals[ j + X_ADJUST as usize ]; 
// //                     unsafe{
// //                         global_spheres_target[ dst_offset+i*2 ][ j ] = locals[ j + X_ADJUST as usize ]; 

// //                     }
// //                 }
// //             }
// //             locals[ INSTRUCTION_PTR as usize ] += 3.0f32;
// //         }
//         SetV => {
//             let dst_layer = program[ ip+ 1] as usize;
//             let dst_offset = program[ ip+ 2] as usize;
//             let count = program[ ip + 3 ] as usize;
//             for i in 0..count {
//                 for j in 0..4 {
//                     spheres[ dst_layer ][ dst_offset+i*2 ][ j ] = consts[ program[ ip+4+j ] as usize ]; 
//                 }
//             }
//             locals[ INSTRUCTION_PTR as usize ] += 8.0f32;
//         }
//         CopyAdjust => {
//             let dst_offset = program[ ip+ 1] as usize;
//             let src_offset = program[ ip + 2 ] as usize;
//             let count = program[ ip + 3 ] as usize;
//             for i in 0..count {
//                 for j in 0..4 {
//                     spheres[SPHERES_CURRENT][ dst_offset+i*2 ][ j ] = spheres[SPHERES_CURRENT][ src_offset+i*2 ][ j ] + locals[ j + X_ADJUST as usize ]; 
//                 }
//             }
//             locals[ INSTRUCTION_PTR as usize ] += 4.0f32;
//         },
//         CopyRNDAdjust => {
//             let dst_offset = program[ ip+ 1] as usize;
//             let src_offset = program[ ip + 2 ] as usize;
//             let count = program[ ip + 3 ] as usize;
//             for i in 0..count {
//                 for j in 0..4 {
//                     unsafe{
//                         spheres[SPHERES_CURRENT][ dst_offset+i*2 ][ j ] = spheres[SPHERES_CURRENT][ src_offset+i*2 ][ j ] + locals[ j + X_ADJUST as usize ]*rng.next_f32(); 
//                     }
//                 }
//             }
//             locals[ INSTRUCTION_PTR as usize ] += 4.0f32;
//         },
//         _ => {
//             return;
//         }
//     }
// }

// pub fn clamp( x : f32, min_value: f32, max_value: f32 ) -> f32 {
//     return x.max( min_value).min( max_value );
// }

// pub fn smooth_Step( x: f32, min_value: f32, max_value: f32 ) -> f32 {
//     let val : f32 = clamp((x - min_value) / (max_value - min_value), 0.0, 1.0); 
//     return val * val * (3.0f32 - 2.0f32 * val);
// }

// calc sphere positions


static mut delay_counter : u32 = 0;
static mut play_pos : usize = 0;

fn update_world() {
    
    unsafe{
        camera_mode = sequence[ play_pos] as u32;
        delay_counter = sequence[ play_pos+1].into() ;
//        if camera_mode == 0 {
            let seed : u32 = sequence[ play_pos+2].into() ;
            setup_camera( seed, camera_mode as u8 );
//        }
        play_pos += 3;
    }
}

static mut cam_count : u32 = 0;

fn setup_random_camera( ) {
    let seed : u32;
    unsafe{ 
        cam_count += 1;
        setup_camera( cam_count, 1);
    }   
}

fn setup_camera( seed : u32, mode : u8) {
//    super::log!( "Setup Camera ", mode, seed );
    unsafe{ super::log!( "Setup Camera: ", mode as f32, seed as f32 ); }

    let mut crng : random::Rng = random::Rng{seed: core::num::Wrapping(9231249+seed)};
    let x = crng.next_f32();
    let y = crng.next_f32();

    let pos = (((y*512f32) as usize *512)+(x*512f32) as usize)*4;
    unsafe{
        if mode == 0 {
            world_pos[ 0 ] = (x-0.5)*512f32;
            world_pos[ 1 ] = src_terrain[ pos ]*60.0-2.1+crng.next_f32()*5.0;
            world_pos[ 2 ] = (y-0.5)*512f32;
        
            world_rot[ 0 ]  =  (crng.next_f32()-0.5)*1.54;
            world_rot[ 1 ]  =  (crng.next_f32()-0.5)*3.15;
            world_rot[ 2 ]  =  (crng.next_f32()-0.5)*0.05;
        
            camera_velocity[ 0 ] = (crng.next_f32()-0.5)*0.2;
            camera_velocity[ 1 ] = (crng.next_f32()-0.5)*0.05;
            camera_velocity[ 2 ] = (crng.next_f32()-0.5)*0.2;
        
            camera_rot_speed[ 0 ] = (crng.next_f32()-0.5)*0.002;
            camera_rot_speed[ 1 ] = (crng.next_f32()-0.5)*0.001;
            camera_rot_speed[ 2 ] = (crng.next_f32()-0.5)*0.001;
    

            pivot_cam_angle[1] = world_rot[ 1 ];
            pivot_cam_centre[ 0 ] = 130.5;
            pivot_cam_centre[ 1 ] = 26.7;
            pivot_cam_centre[ 2 ] = 191.5;
        } 
        else if mode == 1 || mode == 2 {
            world_rot[ 0 ]  =  (crng.next_f32()-0.5)*1.54;
            world_rot[ 1 ]  =  (crng.next_f32()-0.5)*3.15;
            world_rot[ 2 ]  =  (crng.next_f32()-0.5)*0.05;

            camera_velocity[ 0 ] = (crng.next_f32()-0.5)*0.02;
            camera_velocity[ 1 ] = (crng.next_f32()-0.5)*0.005;
            camera_velocity[ 2 ] = (crng.next_f32()-0.5)*0.02;

            camera_rot_speed[ 0 ] = (crng.next_f32()-0.5)*0.002;
            camera_rot_speed[ 1 ] = (crng.next_f32()-0.5)*0.01;
            camera_rot_speed[ 2 ] = (crng.next_f32()-0.5)*0.001;
    
            pivot_cam_dist[ 0 ] = 8.0f32-crng.next_f32()*10.0f32;
            pivot_cam_dist[ 1 ] = crng.next_f32()*0.1f32;
            pivot_cam_dist[ 2 ] = crng.next_f32()*5.0f32;
    
            pivot_cam_angle[1] = world_rot[ 1 ];
    
            pivot_cam_centre[ 0 ] = 130.5;
            pivot_cam_centre[ 1 ] = 26.7;
            pivot_cam_centre[ 2 ] = 191.5;
            if mode == 2 {
                pivot_cam_centre[ 0 ] = (x-0.5)*512f32;
                pivot_cam_centre[ 1 ] = src_terrain[ pos ]*60.0-2.1+crng.next_f32()*5.0;
                pivot_cam_centre[ 2 ] = (y-0.5)*512f32;
            }
        }
//         world_pos[ 0 ] = (x-0.5)*512f32;
//         world_pos[ 1 ] = src_terrain[ pos ]*60.0-2.1+crng.next_f32()*5.0;
//         world_pos[ 2 ] = (y-0.5)*512f32;
    
// //        world_rot[ 0 ]  =  (crng.next_f32()-0.5)*1.54;
//         world_rot[ 1 ]  =  (crng.next_f32()-0.5)*3.15;
// //        world_rot[ 2 ]  =  (crng.next_f32()-0.5)*0.05;
    
//         camera_velocity[ 0 ] = (crng.next_f32()-0.5)*0.2;
//         camera_velocity[ 1 ] = (crng.next_f32()-0.5)*0.05;
//         camera_velocity[ 2 ] = (crng.next_f32()-0.5)*0.2;
    
    }
    
}

//random camera centre 130.5525, 27.7635, 191.6042

static sequence : &[u16] = &[
     // type,    delay,     rnd_offset,   
//     1,          65000,        0,

    //  2,          180,        87,     // up circle sun
    //  2,          180,        93,     // near spin
    //  2,          180,        138,     // up sun sphers near spin
    //  2,          180,        163,     // up sun sphers near spin
    //  2,          180,        171,     // high shot down
    //  2,          180,        190,     // near sphere

    1,          380,        33,     // pass back over  
    1,          380,        33,     // pass back over  
    1,          380,        33,     // pass back over  
    1,          380,        33,     // pass back over  
    1,          280,        2,     // far shot
     1,          280,        12,     // sun shot
     1,          280,        22,     // closer rot shot
     1,          280,        24,     // pass over ( clear shader error)
     1,          280,        33,     // pass back over  
     1,          280,        33,     // slow forward over pass
     

     0,          180,        24,
     0,          180,       44,
     0,          180,       51,
     0,          180,       53,
     0,          180,       58,
     0,          180,       62,         //departing shot
     
     0,          180,       64,         //water wobbles
     0,          180,       69,         //raising out
     0,          180,       83,         //pannign shot

     0,          180,       92,         //near past building

     0,          180,       108,         //another out of the water
     0,          180,       154,         //off sphere
     0,          180,       154,         //side pan dark
     0,          180,       166,         //side pan dark

     //     0,          0,          180,

];

pub fn frame( ticks : u32, now : f32, render_frame : bool ) -> () {
    let spheres : &mut[ [ [ f32; 4]; (num_spheres+sphere_extras)*2]; 3 ];  
    let locals : &mut [ f32; 16 ];
    unsafe{
        spheres  = &mut global_spheres;
        locals   = &mut global_locals;
    }

    for tick in 0..ticks {
        unsafe {
            if delay_counter == 0 {
                update_world( )
            }
            delay_counter -= 1;

        }

        unsafe{
            if camera_mode == 0 {
                world_pos[ 0 ] += camera_velocity[ 0 ];
                world_pos[ 1 ] += camera_velocity[ 1 ];
                world_pos[ 2 ] += camera_velocity[ 2 ];
    
                world_rot[ 0 ] += camera_rot_speed[ 0 ];
                world_rot[ 1 ] += camera_rot_speed[ 1 ];
                world_rot[ 2 ] += camera_rot_speed[ 2 ];
            }  else if camera_mode == 1 || camera_mode == 2{
                world_rot[ 0 ] += camera_rot_speed[ 0 ];
                world_rot[ 1 ] += camera_rot_speed[ 1 ];
                world_rot[ 2 ] += camera_rot_speed[ 2 ];

                let angle = world_rot[ 1 ] - 3.14f32 / 2.0f32; //pivot_cam_angle[1];
                world_pos[ 0 ] = pivot_cam_centre[ 0 ] + math_util::cos(angle )*pivot_cam_dist[ 0 ];
                world_pos[ 1 ] = pivot_cam_centre[ 1 ];
                world_pos[ 2 ] = pivot_cam_centre[ 2 ]- math_util::sin(angle)*pivot_cam_dist[ 0 ];
                // // world_pos[ 2 ] += camera_velocity[ 2 ];
    
                pivot_cam_dist[ 0 ] += camera_velocity[ 0 ];
            }



//            let angle = world_rot[ 1 ] - 3.14f32 / 2.0f32; //pivot_cam_angle[1];
//            world_pos[ 0 ] = pivot_cam_centre[ 0 ] + math_util::cos(angle )*pivot_cam_dist[ 0 ];
 //           world_pos[ 1 ] = pivot_cam_centre[ 1 ];
 //           world_pos[ 2 ] = pivot_cam_centre[ 2 ]- math_util::sin(angle)*pivot_cam_dist[ 0 ];
            // // world_pos[ 2 ] += camera_velocity[ 2 ];

            // pivot_cam_centre = world_pos;
            // pivot_cam_dist[ 0 ] = crng.next_f32()*5.0f32;
            // pivot_cam_dist[ 1 ] = crng.next_f32()*0.1f32;
            // pivot_cam_dist[ 2 ] = crng.next_f32()*5.0f32;
    
            // pivot_cam_angle[1] = world_rot[ 1 ];
    


        }
        

        // loop{
        //     if locals[ DELAY_COUNTER as usize ] > 0.0 {
        //         break;
        //     }
        //     executeCommand( global_program, locals, spheres, global_consts );
        // }
        // locals[ DELAY_COUNTER as usize ] -= 1.0f32;


        // let t : f32 = now*2.0;
        // let mut adjustedTime : f32  = t;//(30.0*t - 45.0*math_util::cos(t) + math_util::cos(3.0f32*t) - 9.0* math_util::sin(2.0f32*t))/96.0;
        // // adjustedTime += time*0.1;

        // for idx in 0..locals[ CLASS_DIVIDER as usize ] as usize {
        //     let fidx : f32  = idx as f32;
        //     let fidx2 : f32  = fidx*fidx;
        //     spheres[SPHERES_TARGET][ idx*2 ][ 0 ] = math_util::sin( fidx*0.41212f32 + adjustedTime * 0.722f32 + 0.423f32 + fidx2*0.324f32) * 0.70f32;
        //     spheres[SPHERES_TARGET][ idx*2 ][ 1 ] = math_util::sin( fidx*0.3312f32 + adjustedTime * 0.32f32 + 0.23f32+ fidx2*1.54f32 ) * 0.70f32;
        //     spheres[SPHERES_TARGET][ idx*2 ][ 2 ] = math_util::sin( fidx*0.2912f32 + adjustedTime * 0.125f32 + 1.3f32 + fidx2*1.1f32 ) * 0.70f32;
        // }
        
        for idx in 0..num_spheres {
        //     let mut pos = Vec3::new( spheres[SPHERES_CURRENT][ idx*2 ][ 0 ], spheres[SPHERES_CURRENT][ idx*2 ][ 1 ], spheres[SPHERES_CURRENT][ idx*2 ][ 2 ] );
        //     for effect_idx in 0..num_spheres { 
        //         if effect_idx == idx {
        //             continue;
        //         }
        //         let effect_pos = Vec3::new( spheres[SPHERES_CURRENT][ effect_idx*2 ][ 0 ], spheres[SPHERES_CURRENT][ effect_idx*2 ][ 1 ], spheres[SPHERES_CURRENT][ effect_idx*2 ][ 2 ] );
        //         let mut effect_dir = pos - effect_pos;
        //         let sql : f32;
        //         unsafe{
        //             sql = core::intrinsics::sqrtf32(effect_dir.squared_length());
        //         } 
        //         effect_dir = effect_dir.mul(1.0/sql);
        //         let mut repulse_mid : f32;
        //         unsafe{ 
        //             repulse_mid =  core::intrinsics::sqrtf32( spheres[SPHERES_CURRENT][ idx*2 ][ 3 ] ) + core::intrinsics::sqrtf32( spheres[SPHERES_CURRENT][ effect_idx*2 ][ 3 ] );
        //         }
        //         let mut strength =  locals[ REPULSION as usize ]  * ( 1.0f32 - smooth_Step(sql, repulse_mid*0.8f32, repulse_mid*1.2f32*locals[ REPULSE_MAX as usize ] ) );
        //         if  idx >= locals[ CLASS_DIVIDER as usize ] as usize  {
        //             if sql > repulse_mid  {
        //                 if effect_idx < locals[ CLASS_DIVIDER as usize ] as usize {
        //                     strength -= ( 0.004f32 / sql ).min( 0.01);
        //                 }
        //             }
        //         }
        //         pos = pos + effect_dir * strength;
        //     }
            // spheres[SPHERES_CURRENT][ idx*2 ][ 0 ] = 0.0;//pos.x;
            // spheres[SPHERES_CURRENT][ idx*2 ][ 1 ] = 0.0;//pos.y;
            // spheres[SPHERES_CURRENT][ idx*2 ][ 2 ] = 0.0;//pos.z;
        }
        unsafe{
            spheres[SPHERES_CURRENT][ 80*2 ][ 0 ] = world_pos[0];
            spheres[SPHERES_CURRENT][ 80*2 ][ 1 ] = world_pos[1];
            spheres[SPHERES_CURRENT][ 80*2 ][ 2 ] = world_pos[2];

            spheres[SPHERES_CURRENT][ 80*2+1 ][ 0 ] = world_rot[0];
            spheres[SPHERES_CURRENT][ 80*2+1 ][ 1 ] = world_rot[1];
            spheres[SPHERES_CURRENT][ 80*2+1 ][ 2 ] = world_rot[2];

        }

        // // // Interpolate sphere attributes
        // for idx in 0..num_spheres*2 {
        //     for sub_idx in 0..4 {
        //         spheres[SPHERES_CURRENT][ idx ][ sub_idx ] = spheres[SPHERES_CURRENT][ idx ][ sub_idx ]*(1.0f32-spheres[SPHERES_INTEPOLATOR][ idx ][ sub_idx ]) + 
        //                                                                 spheres[SPHERES_TARGET][ idx ][ sub_idx ]*spheres[SPHERES_INTEPOLATOR][ idx ][ sub_idx ];
        //     }
        // }
    }
    if render_frame {
        unsafe{
            let rgba = &[ 0.4f32, 1.0, 0.9, 0.0 ];
            gl::ClearBufferfv(gl::COLOR, 0, rgba as *const _ );  

            gl::UseProgram(shader_prog);
    
            let time_loc : i32 = gl::GetUniformLocation(shader_prog, "iTime\0".as_ptr());
            gl::Uniform1f(time_loc, now );          //time

            let shperes_loc : i32 = gl::GetUniformLocation(shader_prog, "spheres\0".as_ptr());
            gl::Uniform4fv(shperes_loc, (num_spheres+sphere_extras) as i32 * 2, transmute::<_,*const gl::GLfloat>( spheres.as_ptr() ) );

            gl::BindVertexArray(vertex_array_id);
            gl::DrawArrays( gl::TRIANGLE_STRIP, 0, 4 );
        }
    }
}