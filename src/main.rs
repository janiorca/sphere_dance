#![feature(asm)]
#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#![feature(core_intrinsics)]

#[cfg(windows)] extern crate winapi;

mod shaders;
mod math_util;
mod gl;
mod gl_util;
mod intro;
mod random;

use core::mem::MaybeUninit;
use core::panic::PanicInfo;
    
use winapi::um::wingdi::{
    ChoosePixelFormat,
    SwapBuffers,
    wglMakeCurrent,
    wglCreateContext,
    SetPixelFormat,

    PFD_TYPE_RGBA,
    PFD_DOUBLEBUFFER,
    PFD_SUPPORT_OPENGL,
    PFD_DRAW_TO_WINDOW,
    PIXELFORMATDESCRIPTOR
};

use winapi::shared::minwindef::{
    LRESULT,
    LPARAM,
    LPVOID,
    WPARAM,
    UINT,
};

use winapi::shared::windef::{
    HDC,
    HGLRC,
    HWND,
    HMENU,
    HICON,
    HBRUSH,
};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    CreateWindowExA,
    DefWindowProcA,
    DispatchMessageA,
    GetDC,
    PostQuitMessage,
    RegisterClassA,
    TranslateMessage,
    PeekMessageA,
    MessageBoxA,

    MB_ICONERROR,
    MSG,
    WNDCLASSA,
    CS_OWNDC,
    CS_HREDRAW,
    CS_VREDRAW,
    CW_USEDEFAULT,
    PM_REMOVE, 
    WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};

use winapi::um::winnt::{
    FILE_ATTRIBUTE_NORMAL,
    FILE_APPEND_DATA,
    GENERIC_READ,
    GENERIC_WRITE
};

use winapi::um::fileapi::{
    OPEN_ALWAYS,
    OPEN_EXISTING,
    CREATE_ALWAYS,
    WriteFile,
    ReadFile,
    CreateFileA,
};

use winapi::um::handleapi::CloseHandle;

#[cfg(not(feature = "logger"))]
pub unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {

    match msg {
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => { return DefWindowProcA(hwnd, msg, w_param, l_param); }
    }
    return 0;
}

#[cfg(feature = "logger")]
pub unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {

    match msg {
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        },
        winapi::um::winuser::WM_MOUSEMOVE => {
            let x_pos = ( ( l_param as u32 ) & 0x0000ffff) as i32;
            let y_pos = ((( l_param as u32 ) & 0xffff0000)>>16) as i32;
            let ctrl : bool = ( w_param & winapi::um::winuser::MK_CONTROL ) != 0;
            intro::set_pos(x_pos, y_pos, ctrl);
        },
        winapi::um::winuser::WM_LBUTTONDOWN => {
            let x_pos = ( ( l_param as u32 ) & 0x0000ffff) as i32;
            let y_pos = ((( l_param as u32 ) & 0xffff0000)>>16) as i32;
            intro::lbutton_down(x_pos,y_pos);
        },
        winapi::um::winuser::WM_LBUTTONUP => {
            intro::lbutton_up();
        }
        winapi::um::winuser::WM_RBUTTONDOWN => {
            let x_pos = ( ( l_param as u32 ) & 0x0000ffff) as i32;
            let y_pos = ((( l_param as u32 ) & 0xffff0000)>>16) as i32;
            intro::rbutton_down(x_pos,y_pos);
        },
        winapi::um::winuser::WM_RBUTTONUP => {
            intro::rbutton_up();
        }
        _ => { return DefWindowProcA(hwnd, msg, w_param, l_param); }
    }
    return 0;
}

fn show_error( message : *const i8 ) {
    unsafe{
        MessageBoxA(0 as HWND, message, "Window::create\0".as_ptr() as *const i8, MB_ICONERROR);
    }
}

// Create window function 
// https://mariuszbartosik.com/opengl-4-x-initialization-in-windows-without-a-framework/
fn create_window( ) -> ( HWND, HDC ) {
    unsafe {
        let hinstance = GetModuleHandleA( 0 as *const i8 );
        let wnd_class = WNDCLASSA {
            style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,     
            lpfnWndProc : Some( window_proc ),
            hInstance : hinstance,							// The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName : "MyClass\0".as_ptr() as *const i8,
            cbClsExtra : 0,									
            cbWndExtra : 0,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        RegisterClassA( &wnd_class );

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        let h_wnd = CreateWindowExA(
            0,
            //WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,                     // dwExStyle 
            "MyClass\0".as_ptr() as *const i8,		                // class we registered.
            "GLWIN\0".as_ptr() as *const i8,						// title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
            CW_USEDEFAULT, CW_USEDEFAULT, 1280, 720,	// size and position
//            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,	// size and position
            0 as HWND,               	// hWndParent
            0 as HMENU,					// hMenu
            hinstance,                  // hInstance
            0 as LPVOID );				// lpParam
        
        let h_dc : HDC = GetDC(h_wnd);        // Device Context            

        let mut pfd : PIXELFORMATDESCRIPTOR = core::mem::zeroed();
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cAlphaBits = 8;
        pfd.cDepthBits = 32;
         
        let pf_id : i32 = ChoosePixelFormat(h_dc, &pfd );
        if pf_id == 0 {
            show_error( "ChoosePixelFormat() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }

        if SetPixelFormat(h_dc, pf_id, &pfd) == 0  {
            show_error( "SetPixelFormat() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }

        let gl_context : HGLRC = wglCreateContext(h_dc);    // Rendering Contex
        if gl_context == 0 as HGLRC {
            show_error( "wglCreateContext() failed.\0".as_ptr() as *const i8 );
            return ( 0 as HWND, h_dc ) ;
        }
         
        if wglMakeCurrent(h_dc, gl_context) == 0 {
            show_error( "wglMakeCurrent() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }
        gl::init();
        gl::wglSwapIntervalEXT(1);
        ( h_wnd, h_dc )
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message( _window : HWND ) -> bool {
    unsafe {
       let mut msg : MSG = MaybeUninit::uninit().assume_init();
        loop{
            if PeekMessageA( &mut msg,0 as HWND,0,0,PM_REMOVE) == 0 {
                return true;
            }
            if msg.message == winapi::um::winuser::WM_QUIT {
                return false;
            }
            TranslateMessage( &msg  );
            DispatchMessageA( &msg  );
        }
    }
}


#[panic_handler]
#[no_mangle]
pub extern fn panic( _info: &PanicInfo ) -> ! { loop {} }

#[no_mangle]
pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = c as u8;
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        i += 1;
    }
    dest
}

#[cfg(feature = "logger")]
pub unsafe fn log( message : &str ) {
    let name = "dbg_out.txt\0";
    let mut out = 0;

    let hFile = CreateFileA( name.as_ptr() as *const i8, FILE_APPEND_DATA, 0, 
                0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, 
                0 as *mut winapi::ctypes::c_void );
    WriteFile( hFile, message.as_ptr() as *const winapi::ctypes::c_void, message.len() as u32, &mut out, 
                0 as *mut winapi::um::minwinbase::OVERLAPPED );
    CloseHandle( hFile );
}

#[cfg(feature = "logger")]
pub unsafe fn read_file( file_name : &str, dst : &mut [u8] ) {
    let name = "dbg_out.txt\0";
    let mut out = 0;

    log( "Creating file for reading\n");
    let hFile = CreateFileA( file_name.as_ptr() as *const i8, GENERIC_READ, 0, 
                0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 
                0 as *mut winapi::ctypes::c_void );
    log( "Reading...\n");
    ReadFile( hFile, dst.as_mut_ptr() as *mut winapi::ctypes::c_void, dst.len() as u32, &mut out, 
                0 as *mut winapi::um::minwinbase::OVERLAPPED );
    log( "Close handle...\n");
    CloseHandle( hFile );
}


#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let ( window, hdc ) = create_window(  );

    unsafe{
        log("Started\n");
        log("Line2\n");
    }

    intro::prepare();
    let mut time : f32 = 0.0;
    unsafe{ log("Entering loop\n"); };

    loop {
        if !handle_message( window ) {
            break;
        }
        intro::frame( 1, time, true );
        unsafe{ SwapBuffers(hdc); }
        time += 1.0 / 60.0f32;            
    }
    unsafe{
        // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
        winapi::um::processthreadsapi::ExitProcess(0);
    }
}

// Compiling with no_std seems to require the following symbol to be set if there is any floating point code anywhere in the code
#[no_mangle]
pub static _fltused : i32 = 1;
