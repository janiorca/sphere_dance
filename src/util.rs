#[cfg(feature = "logger")]
use winapi::um::winnt::{
    FILE_ATTRIBUTE_NORMAL,
    FILE_APPEND_DATA,
    GENERIC_READ,
};

#[cfg(feature = "logger")]
use winapi::um::{
    fileapi::{
        OPEN_ALWAYS,
        OPEN_EXISTING,
        WriteFile,
        ReadFile,
        CreateFileA,
    },
    handleapi::CloseHandle
};

#[cfg(feature = "logger")]
#[macro_export]
macro_rules! log {
    ($text:expr) => { unsafe { crate::util::log0($text); } };
    ($text:expr, $val:expr) => { crate::util::log1($text,$val); };
    ($text:expr, $val1:expr, $val2:expr) => { crate::util::log2($text,$val1,$val2); };
    ($text:expr, $val1:expr, $val2:expr, $val3:expr) => { crate::util::log3($text,$val1,$val2,$val3); };
}

#[cfg(not(feature = "logger"))]
#[macro_export]
macro_rules! log {
    ($text:expr) => { };
    ($text:expr, $val:expr) => {};
    ($text:expr, $val1:expr, $val2:expr) => {};
    ($text:expr, $val1:expr, $val2:expr, $val3:expr) => {};
}

#[cfg(feature = "logger")]
pub unsafe fn log0( message : &str ) {
    let name = "dbg_out.txt\0";
    let mut out = 0;

    let h_file = CreateFileA(name.as_ptr() as *const i8, FILE_APPEND_DATA, 0,
                             0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL,
                             0 as *mut winapi::ctypes::c_void );
    WriteFile(h_file, message.as_ptr() as *const winapi::ctypes::c_void, message.len() as u32, &mut out,
              0 as *mut winapi::um::minwinbase::OVERLAPPED );
    CloseHandle(h_file);
}

#[cfg(feature = "logger")]
pub fn get_c_string_length( buffer: &[u8]) -> usize {
    let mut buffer_text_len = 0;
    while buffer_text_len < buffer.len() {
        if buffer[buffer_text_len] == 0 {
            break;
        }
        buffer_text_len += 1
    }
    return buffer_text_len;
}

#[cfg(feature = "logger")]
pub fn f32_to_text( dest: &mut[u8], value: f32, comma: bool ) -> usize {
    let int_part = value as u32;
    let frac_part = ((value - int_part as f32)*10000f32 ) as u32; 
    unsafe{ winapi::um::winuser::wsprintfA( dest.as_mut_ptr() as * mut i8, "%d.%.4d\0".as_ptr() as * const i8, int_part, frac_part); }
    if comma {
        let length = get_c_string_length(dest);
        dest[ length ] = ',' as u8;
        dest[ length+1 ] = ' ' as u8;
        return length+2;
    }
    return get_c_string_length( &dest );
}

#[cfg(feature = "logger")]
pub unsafe fn log1( _message : &str, value: f32 ) {
    let mut buffer : [ u8; 256 ] = [ 0;256 ];
    let mut length : usize = 0;
    length += f32_to_text( &mut buffer, value, false );
    buffer[ length ] = '\n' as u8;
    let buffer_text_len = get_c_string_length(&buffer);
    log0( core::str::from_utf8_unchecked(&buffer[ 0 .. buffer_text_len ]));
}

#[cfg(feature = "logger")]
pub unsafe fn log2( _message : &str, value1: f32, value2: f32 ) {
    let mut buffer : [ u8; 256 ] = [ 0;256 ];
    let mut length : usize = 0;
    length += f32_to_text( &mut buffer, value1, true );
    length += f32_to_text( &mut buffer[length..], value2, false );
    buffer[ length ] = '\n' as u8;
    let buffer_text_len = get_c_string_length(&buffer);
    log0( core::str::from_utf8_unchecked(&buffer[ 0 .. buffer_text_len ]));
}

#[cfg(feature = "logger")]
pub unsafe fn log3( _message : &str, value1: f32, value2: f32, value3: f32 ) {
    let mut buffer : [ u8; 256 ] = [ 0;256 ];
    let mut length : usize = 0;
    length += f32_to_text( &mut buffer, value1, true );
    length += f32_to_text( &mut buffer[length..], value2, true );
    length += f32_to_text( &mut buffer[length..], value3, false );
    buffer[ length ] = '\n' as u8;
    let buffer_text_len = get_c_string_length(&buffer);
    log0( core::str::from_utf8_unchecked(&buffer[ 0 .. buffer_text_len ]));
}

#[cfg(feature = "logger")]
pub unsafe fn read_file( file_name : &str, dst : &mut [u8] ) {
    let mut out = 0;

    log!( "Creating file for reading\n");
    let h_file = CreateFileA(file_name.as_ptr() as *const i8, GENERIC_READ, 0,
                             0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
                             0 as *mut winapi::ctypes::c_void );
    log!( "Reading...\n");
    ReadFile(h_file, dst.as_mut_ptr() as *mut winapi::ctypes::c_void, dst.len() as u32, &mut out,
             0 as *mut winapi::um::minwinbase::OVERLAPPED );
    log!( "Close handle...\n");
    CloseHandle(h_file);
}
