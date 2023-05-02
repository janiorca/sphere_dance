#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ptr::null_mut;
use winapi::um::libloaderapi::LoadLibraryA;
use winapi::um::libloaderapi::GetProcAddress;
use winapi::um::wingdi::wglGetProcAddress;
use winapi::um::winnt::LPCSTR;
use winapi::shared::minwindef::HMODULE;

#[macro_export]
macro_rules! count_functions {
    (fn $head_name:ident($($head_arg:ident: $head_typ:ty),+) -> $head_ret:ty, $(fn $tail_name:ident($($tail_arg:ident: $tail_typ:ty),+) -> $tail_ret:ty),+) => {
        1usize + $crate::count_functions!($(fn $tail_name($($tail_arg: $tail_typ),+) -> $tail_ret),+)
    };
    (fn $head_name:ident($($head_arg:ident: $head_typ:ty),+) -> $head_ret:ty) => {
        1usize
    };
}

#[macro_export]
macro_rules! function_definition {
    ($index:expr, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty) => {
        pub fn $name($($arg: $typ),+) -> $ret {
            unsafe {
                core::mem::transmute::<_, extern "system" fn($($typ),+) -> $ret>(*$functions.get_unchecked($index))($($arg),+)
            }
        }
    }
}

#[macro_export]
macro_rules! function_definitions {
    (@step $idx:expr, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty, $(fn $tail_name:ident($($tail_arg:ident: $tail_typ:ty),+) -> $tail_ret:ty),+) => {
        $crate::function_definition!($idx, $functions, fn $name($($arg: $typ),+) -> $ret);
        $crate::function_definitions!(@step $idx + 1usize, $functions, $(fn $tail_name($($tail_arg: $tail_typ),+) -> $tail_ret),+);
    };
    (@step $idx:expr, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty) => {
        $crate::function_definition!($idx, $functions, fn $name($($arg: $typ),+) -> $ret);
    };
    ($functions:ident, $(fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty),+) => {
        $crate::function_definitions!(@step 0usize, $functions, $(fn $name($($arg: $typ),+) -> $ret),+);
    }
}

#[macro_export]
macro_rules! function_initialization {
    ($index:expr, $library:ident, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty) => {
        unsafe {
            load(&mut $functions, $index, $library, concat!(stringify!($name),"\0").as_ptr() as *const i8)
        }
    }
}

#[macro_export]
macro_rules! function_initializations {
    (@step $idx:expr, $library:ident, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty, $(fn $tail_name:ident($($tail_arg:ident: $tail_typ:ty),+) -> $tail_ret:ty),+) => {
        $crate::function_initialization!($idx, $library, $functions, fn $name($($arg: $typ),+) -> $ret);
        $crate::function_initializations!(@step $idx + 1, $library, $functions, $(fn $tail_name($($tail_arg: $tail_typ),+) -> $tail_ret),+)
    };
    (@step $idx:expr, $library:ident, $functions:ident, fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty) => {
        $crate::function_initialization!($idx, $library, $functions, fn $name($($arg: $typ),+) -> $ret);
    };
    ($library:ident, $functions:ident, $(fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty),+) => {
        $crate::function_initializations!(@step 0, $library, $functions, $(fn $name($($arg: $typ),+) -> $ret),+)
    }
}

#[macro_export]
macro_rules! load_functions {
    ($(fn $name:ident($($arg:ident: $typ:ty),+) -> $ret:ty),+) => {
        use $crate::gl::*;
        const FUNCTIONS_AMOUNT: usize = $crate::count_functions!($(fn $name($($arg: $typ),+) -> $ret),+);
        static mut FUNCTIONS: [usize; FUNCTIONS_AMOUNT] = [0; FUNCTIONS_AMOUNT];

        $crate::function_definitions!(FUNCTIONS, $(fn $name($($arg: $typ),+) -> $ret),+);

        pub fn initialize_functions() {
            let library = load_library();
            $crate::function_initializations!(library, FUNCTIONS, $(fn $name($($arg: $typ),+) -> $ret),+);
        }
    }
}

pub mod internal {
    // TODO: Size optimization. This is duplicated but take less than 10bytes.
    load_functions! {
        fn wglSwapIntervalEXT(interval: GLint) -> GLuint
    }
}

pub enum CVoid {}

pub type GLboolean = u8;
pub type GLchar = u8;
pub type GLfloat = f32;
pub type GLenum = u32;
pub type GLint = i32;
pub type GLuint = u32;
pub type GLsizei = i32;
pub type GLsizeiptr = isize;

pub const FALSE: GLboolean = 0;
pub const TRIANGLES: GLenum = 0x0004;
pub const TRIANGLE_STRIP: GLenum = 0x0005;
pub const TEXTURE_2D: GLenum = 0x0DE1;
pub const UNSIGNED_BYTE: GLenum = 0x1401;
pub const FLOAT: GLenum = 0x1406;
pub const COLOR: GLenum = 0x1800;
pub const RGB: GLenum = 0x1907;
pub const RGBA: GLenum = 0x1908;
pub const NEAREST: GLenum = 0x2600;
pub const TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const TEXTURE_WRAP_S: GLenum = 0x2802;
pub const TEXTURE_WRAP_T: GLenum = 0x2803;
pub const REPEAT: GLenum = 0x2901;
pub const CLAMP_TO_EDGE: GLenum = 0x812F;

pub const COLOR_BUFFER_BIT: GLenum = 16384;

pub const TEXTURE0: GLenum = 0x84C0;
pub const FRAGMENT_SHADER: GLenum = 0x8B30;
pub const VERTEX_SHADER: GLenum = 0x8B31;
pub const COMPILE_STATUS: GLenum = 0x8B81;
pub const LINK_STATUS: GLenum = 0x8B82;
pub const ARRAY_BUFFER: GLenum = 0x8892;
pub const STATIC_DRAW: GLenum = 0x88E4;

pub fn load_library() -> HMODULE {
    unsafe { LoadLibraryA( "Opengl32.dll\0".as_ptr() as *const i8) }
}

pub fn load(functions: &mut [usize], index: u16, handle: HMODULE, name: LPCSTR) {
    unsafe {
        let mut prc = wglGetProcAddress(name);
        if prc == null_mut() {
            prc = GetProcAddress(handle, name);
        }
        *functions.get_unchecked_mut(index as usize) = prc as usize;
    }
}