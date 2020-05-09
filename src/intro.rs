use super::math_util;
use super::gl;
use super::gl_util;
use super::random;

use gl::CVoid;
use core::mem::{size_of,transmute};
use core::ops::{Add,Sub,Mul};

pub const CAMERA_POS_IDX : usize = 80*2;
pub const CAMERA_ROT_IDX : usize = 80*2+1;

pub const num_spheres : usize = 80;
pub const sphere_extras : usize = 2;

static mut shader_prog : gl::GLuint = 0;
static mut vertex_array_id : gl::GLuint = 0;

static mut rng : random::Rng = random::Rng{seed: core::num::Wrapping(21431249)};

static mut global_spheres: [ [ f32; 4]; (num_spheres+sphere_extras)*2] = [ [ 0f32; 4]; (num_spheres+sphere_extras)*2 ];  

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
//static mut world_pos : [ f32;3] = [ 0.0; 3];
static mut rotating_camera : bool  = false;
//static mut world_rot : [ f32;3] = [ 0.0; 3];

//static mut camera_velocity : [ f32; 3] = [ 0.0; 3];
static mut camera_velocity : [ f32; 4] = [ 0.0; 4];
static mut camera_rot_speed : [ f32; 4] = [ 0.0; 4];

static mut pivot_cam_centre : [ f32; 3] = [ 0.0; 3];
static mut pivot_cam_dist : [ f32; 3] = [ 0.0; 3];
static mut pivot_cam_angle : [ f32; 3] = [ 0.0; 3];

static mut camera_mode : u32 = 0;

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
    unsafe{ super::log!( "Camera: ", global_spheres[ CAMERA_POS_IDX ][ 0 ], global_spheres[ CAMERA_POS_IDX ][ 1 ], global_spheres[ CAMERA_POS_IDX ][ 2 ]); }
}

fn set_r3( dest : &mut[ f32 ; 4 ], crng : &mut random::Rng, a: f32, b: f32, c: f32 ) {
    dest[ 0 ] = (crng.next_f32()-0.5)*a;
    dest[ 1 ] = (crng.next_f32()-0.5)*b;
    dest[ 2 ] = (crng.next_f32()-0.5)*c;
}

fn random_pos( rng_terrain: &mut random::Rng ) -> (f32, f32,usize) {
    let x = rng_terrain.next_f32();
    let y = rng_terrain.next_f32();
    let pos = (((y*512f32) as usize *512)+(x*512f32) as usize)*4;
    ( x, y, pos )
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

    // collect the values from f64 into u8 in a separate vec


    unsafe{
        super::log!( "Build terrain!");
        let mut rng_terrain : random::Rng = random::Rng{seed: core::num::Wrapping(9231249)};
        let mut lumps : [(f32,f32,f32);50] = [(0.0,0.0,0.0);50];
        let num_lumps = 50;

        for nl in 0..num_lumps {
            let x = rng_terrain.next_f32();
            let y = rng_terrain.next_f32();
            let f = rng_terrain.next_f32();
            lumps[ nl ] = (x,y,f);
        }

        for i in 0..1_000_000 {
            let (x,y,pos) = random_pos( &mut rng_terrain );
            let mut charge = 0.0;
            for nl in 0..num_lumps {
                let lmp = lumps.get(nl).unwrap();
                let dist = (x-lmp.0)*(x-lmp.0) + (y-lmp.1)*(y-lmp.1);
                charge += lmp.2*0.0001/dist;
            }
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
                let (x,y,pos) = random_pos( &mut rng_terrain );
                if src_terrain[ pos ] > 0.3f32 {
//                    set_v4( &mut spheres[ idx*2 ], x*512f32, src_terrain[ pos ]*60.0-12.1, y*512f32, 18.0f32);
                    spheres[ idx*2 ][ 0 ] = x*512f32;
                    spheres[ idx*2 ][ 1 ] = src_terrain[ pos ]*60.0-12.1;
                    spheres[ idx*2 ][ 2 ] = y*512f32;
                    spheres[ idx*2 ][ 3 ] = 18.0f32;//0.915*0.915;    //25.3f32;            // 5^2
                    spheres[ idx*2+1 ][ 0 ] = 0.02f32;
                    spheres[ idx*2+1 ][ 1 ] = 0.02f32;
                    spheres[ idx*2+1 ][ 2 ] = 0.02f32;
                    spheres[ idx*2+1 ][ 3 ] = 1.317f32;
                    break;
                }
            }
        }
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

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
    }
}


static mut delay_counter : u32 = 0;
static mut play_pos : usize = 0;

fn update_world() {
    
    unsafe{
        camera_mode = sequence[ play_pos] as u32;
        delay_counter = sequence[ play_pos+1] as u32*60;
            let seed : u32 = sequence[ play_pos+2].into() ;
            super::log!( "Camera", seed as f32, camera_mode as f32);
            setup_camera( seed, camera_mode as u8 );
        play_pos += 3;
    }
}

static mut cam_count : u32 = 1100;          // (1753 0 )

fn setup_random_camera( ) {
    let seed : u32;
    unsafe{ 
        cam_count += 1;
        setup_camera( cam_count, 1);
        camera_mode = 1;

    }   
}

fn setup_camera( seed : u32, mode : u8) {
    unsafe{ super::log!( "Setup Camera: ", mode as f32, seed as f32 ); }

    let mut crng : random::Rng = random::Rng{seed: core::num::Wrapping(9231249+seed)};

    let (x,y,pos) = random_pos( &mut crng );

    unsafe{
        //if mode == 0 {
            global_spheres[ CAMERA_POS_IDX ][ 0 ] = x*512f32;
            global_spheres[ CAMERA_POS_IDX ][ 1 ] = src_terrain[ pos ]*60.0-2.1+crng.next_f32()*5.0;
            global_spheres[ CAMERA_POS_IDX ][ 2 ] = y*512f32;
            set_r3( &mut global_spheres[ CAMERA_ROT_IDX ], &mut crng, 1.54, 3.15, 0.05 );
            set_r3( &mut camera_velocity, &mut crng, 0.2, 0.05, 0.2);
            set_r3( &mut camera_rot_speed, &mut crng, 0.002, 0.001, 0.001 );
        //} 
        if mode == 1 {
            let scale = crng.next_f32()*10f32;
            global_spheres[ CAMERA_ROT_IDX ][ 0 ]  =  (crng.next_f32()-0.5)*1.54;
    
            camera_velocity[ 0 ] *= 0.01;
            camera_rot_speed[ 1 ] *= 5.0f32;

            pivot_cam_dist[ 0 ] = (1.1f32-crng.next_f32())*scale;
            pivot_cam_angle[1] = global_spheres[ CAMERA_ROT_IDX ][ 1 ];
            pivot_cam_centre[ 0 ] = 130.5 + 256.0; 
            pivot_cam_centre[ 1 ] = 25.8 + crng.next_f32()*0.4f32*scale;
            pivot_cam_centre[ 2 ] = 191.5 + 256.0;
        }
    }
    
}

//random camera centre 130.5525, 27.7635, 191.6042

static sequence : &[u16] = &[
// CONFIRMED SEQUENCE
// close shots of water - glimpses of land
0,          2,       64,         //water wobbles
0,          2,       434,         //water wobbles
0,          4,       65,         //water wobbles
0,          2,       798,         //water wobbles
0,          2,       436,         //water wobbles
0,          6,       1187,         //need better pan up from water shot   18

// forward shots
0,          3,       317, 
0,          3,       298, 
0,          5,       1649,       // low forward beach shot
0,          4,       909, 
0,          14,       724,         // long turning shot
0,          3,       1453,       // forward tuning shot nice
0,          3,       1007,       // nice color foward pan
0,          6,       723, 
0,          6,       1046,     // 33

// left
0,          4,       123, 
0,          8,       1299,        // animate sphere at this point. Could be longer

// up to pan around
0,          3,       1120,       // pan color angle forward
0,          8,       636,        // rising upo high from water, nice long shadow
0,          6,       613,        // nice high shot looking down
0,          6,       1006,       // nice high look down

// Pan into holo map
// far   
1, 4, 449,    
1, 4, 729,
1, 3, 398,

//    nearer
1, 3, 353,

// very close
1, 3, 942,
1, 5, 666,
1, 3, 345,
1, 3, 420,
1, 3, 495,
1, 3, 983,
1, 10, 741,

// Leave holo
1, 2, 1490,

// Final pull back
0,          4,       166,        // nice pull back
0,          4,       151,        // pan back over sphere
0,          2,       691,        // very nice backward pass 
0,          2,       1112,       // pan back water  
0,          20,       1261,       // big wide back pan color

];

pub fn frame( ticks : u32, now : f32, render_frame : bool ) -> () {
    // let spheres : &mut[ [ f32; 4]; (num_spheres+sphere_extras)*2];  
    // unsafe{
    //     spheres  = &mut global_spheres;
    // }

    for tick in 0..ticks {
        unsafe {
            if delay_counter == 0 {
                update_world( )
            }
            delay_counter -= 1;

        }

        unsafe{
            global_spheres[ CAMERA_ROT_IDX ][ 0 ] += camera_rot_speed[ 0 ];
            global_spheres[ CAMERA_ROT_IDX ][ 1 ] += camera_rot_speed[ 1 ];
            global_spheres[ CAMERA_ROT_IDX ][ 2 ] += camera_rot_speed[ 2 ];
            if camera_mode == 0 {
                global_spheres[ CAMERA_POS_IDX ][ 0 ] += camera_velocity[ 0 ];
                global_spheres[ CAMERA_POS_IDX ][ 1 ] += camera_velocity[ 1 ];
                global_spheres[ CAMERA_POS_IDX ][ 2 ] += camera_velocity[ 2 ];
    
            }  else if camera_mode == 1 {
    
                let angle = global_spheres[ CAMERA_ROT_IDX ][ 1 ] - 3.14f32 / 2.0f32; //pivot_cam_angle[1];
                global_spheres[ CAMERA_POS_IDX ][ 0 ] = pivot_cam_centre[ 0 ] + math_util::cos(angle )*pivot_cam_dist[ 0 ]*pivot_cam_dist[ 0 ];
                global_spheres[ CAMERA_POS_IDX ][ 1 ] = pivot_cam_centre[ 1 ];
                global_spheres[ CAMERA_POS_IDX ][ 2 ] = pivot_cam_centre[ 2 ]- math_util::sin(angle)*pivot_cam_dist[ 0 ]*pivot_cam_dist[ 0 ];
                pivot_cam_dist[ 0 ] += camera_velocity[ 0 ]*1.0f32;
            }
       }
      
    }
    if render_frame {
        unsafe{
            let rgba = &[ 0.4f32, 1.0, 0.9, 0.0 ];
            gl::ClearBufferfv(gl::COLOR, 0, rgba as *const _ );  

            gl::UseProgram(shader_prog);
    
//            let time_loc : i32 = gl::GetUniformLocation(shader_prog, "e\0".as_ptr());
            let time_loc : i32 = gl::GetUniformLocation(shader_prog, "iTime\0".as_ptr());
            gl::Uniform1f(time_loc, now );          //time

            let shperes_loc : i32 = gl::GetUniformLocation(shader_prog, "sp\0".as_ptr());
            gl::Uniform4fv(shperes_loc, (num_spheres+sphere_extras) as i32 * 2, transmute::<_,*const gl::GLfloat>( global_spheres.as_ptr() ) );

            gl::BindVertexArray(vertex_array_id);
            gl::DrawArrays( gl::TRIANGLE_STRIP, 0, 4 );
        }
    }
}