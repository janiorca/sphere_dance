#![feature(asm)]
#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#![feature(core_intrinsics)]

#[cfg(windows)] extern crate winapi;

mod shaders;
mod math_util;
mod gl;
pub mod gl_util;
pub mod util;
mod intro;
mod music;
mod random;

use core::mem::MaybeUninit;
use core::panic::PanicInfo;


use winapi::um::wingdi::{
    ChoosePixelFormat,
    SwapBuffers,
    wglMakeCurrent,
    wglCreateContext,
    SetPixelFormat,

    DEVMODEA,
    PFD_TYPE_RGBA,
    PFD_DOUBLEBUFFER,
    PFD_SUPPORT_OPENGL,
    PFD_DRAW_TO_WINDOW,
    PIXELFORMATDESCRIPTOR
};

use winapi::shared::minwindef::{
    HINSTANCE,
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
    WS_MAXIMIZE,
    WS_POPUP,
    WS_VISIBLE,
};


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

#[cfg(feature = "logger")]
fn show_error( message : *const i8 ) {
    unsafe{
        MessageBoxA(0 as HWND, message, "Window::create\0".as_ptr() as *const i8, MB_ICONERROR);
    }
}

fn create_window( ) -> ( HWND, HDC ) {
    unsafe {
        let h_wnd : HWND;

        #[cfg(feature = "fullscreen")]
        {
            let mut devMode : DEVMODEA = core::mem::zeroed();
            devMode.dmSize = core::mem::size_of::<DEVMODEA>() as u16;
            devMode.dmFields = winapi::um::wingdi::DM_BITSPERPEL | winapi::um::wingdi::DM_PELSWIDTH | winapi::um::wingdi::DM_PELSHEIGHT;
            devMode.dmBitsPerPel = 32;
            devMode.dmPelsWidth  = 1280;
            devMode.dmPelsHeight = 720;
            if winapi::um::winuser::ChangeDisplaySettingsA(&mut devMode, winapi::um::winuser::CDS_FULLSCREEN)!= winapi::um::winuser::DISP_CHANGE_SUCCESSFUL {
                return ( 0 as HWND, 0 as HDC ) ;
            }
            winapi::um::winuser::ShowCursor( 0 );            

            h_wnd = CreateWindowExA(
                0,
                "static\0".as_ptr() as *const i8,		                // class we registered.
                "GLWIN\0".as_ptr() as *const i8,						// title
                WS_POPUP | WS_VISIBLE | WS_MAXIMIZE, 0, 0, 0, 0,	// size and position
                0 as HWND,               	// hWndParent
                0 as HMENU,					// hMenu
                0 as HINSTANCE,             // hInstance
                0 as LPVOID );				// lpParam
        }

        #[cfg(not(feature = "fullscreen"))]
        {
            let hinstance = GetModuleHandleA( 0 as *const i8 );
            let mut wnd_class : WNDCLASSA = core::mem::zeroed();
            wnd_class.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
            wnd_class.lpfnWndProc = Some( window_proc );
            wnd_class.hInstance = hinstance;							// The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            wnd_class.lpszClassName = "MyClass\0".as_ptr() as *const i8;
            RegisterClassA( &wnd_class );
    
            h_wnd = CreateWindowExA(
                0,
                //WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,                     // dwExStyle 
                "MyClass\0".as_ptr() as *const i8,		                // class we registered.
                "GLWIN\0".as_ptr() as *const i8,						// title
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
                CW_USEDEFAULT, CW_USEDEFAULT, 1280, 720,	// size and position
                0 as HWND,               	// hWndParent
                0 as HMENU,					// hMenu
                hinstance,                  // hInstance
                0 as LPVOID );				// lpParam
        }
        let h_dc : HDC = GetDC(h_wnd);        // Device Context            

        let mut pfd : PIXELFORMATDESCRIPTOR = core::mem::zeroed();
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cAlphaBits = 8;
        pfd.cDepthBits = 32;

        #[cfg(feature = "logger")]
        {
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
        }

        #[cfg(not(feature = "logger"))]
        {
            let pf_id : i32 = ChoosePixelFormat(h_dc, &pfd );
            SetPixelFormat(h_dc, pf_id, &pfd);
            let gl_context : HGLRC = wglCreateContext(h_dc);    // Rendering Context
            wglMakeCurrent(h_dc, gl_context);
        }

        gl::init();
        gl::wglSwapIntervalEXT(1);
        ( h_wnd, h_dc )
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
#[cfg(feature = "logger")]
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

static waveFormat : winapi::shared::mmreg::WAVEFORMATEX = winapi::shared::mmreg::WAVEFORMATEX{ 
    wFormatTag : winapi::shared::mmreg::WAVE_FORMAT_IEEE_FLOAT, 
    nChannels : 1,
    nSamplesPerSec : 44100,
    nAvgBytesPerSec : 44100*4,
    nBlockAlign : 4,
    wBitsPerSample: 32,
    cbSize:0
 };

 static mut waveHeader : winapi::um::mmsystem::WAVEHDR = winapi::um::mmsystem::WAVEHDR{
    lpData: 0 as *mut i8,
    dwBufferLength: 44100*4*120,
    dwBytesRecorded: 0,
    dwUser: 0,
    dwFlags: 0,
    dwLoops: 0,
    lpNext: 0 as *mut winapi::um::mmsystem::WAVEHDR,
    reserved: 0,
};

static mut music_data : [f32;44100*120] = [ 0.0;44100*120];
#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let ( window, hdc ) = create_window(  );

    unsafe{ log!("Prepare\n"); };
    intro::prepare();

    let mut time : f32 = 0.0;

    unsafe{
        music::make_music( &mut music_data );
        waveHeader.lpData = music_data.as_mut_ptr() as *mut i8;
        let mut hWaveOut : winapi::um::mmsystem::HWAVEOUT = 0 as winapi::um::mmsystem::HWAVEOUT;
        winapi::um::mmeapi::waveOutOpen( &mut hWaveOut, winapi::um::mmsystem::WAVE_MAPPER, &waveFormat, 0, 0, winapi::um::mmsystem::CALLBACK_NULL);
        winapi::um::mmeapi::waveOutPrepareHeader(hWaveOut, &mut waveHeader, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32 );
        winapi::um::mmeapi::waveOutWrite(hWaveOut, &mut waveHeader, core::mem::size_of::<winapi::um::mmsystem::WAVEHDR>() as u32 );
    }

    unsafe{ log!("Entering loop\n"); };
    loop {
        #[cfg(feature = "logger")]
        {
            if !handle_message( window ) {
                break;
            }        
        }

        unsafe{
            if winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_ESCAPE) != 0 {
                break;
            }
        }

        intro::frame( time );
        unsafe{ SwapBuffers(hdc); }
        time += 1.0 / 60.0f32;  
        #[cfg(not(feature = "logger"))]
        if time > 120.0 {
            break;
        }          
    }

    unsafe{
        // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
        winapi::um::processthreadsapi::ExitProcess(0);
    }
}

// Compiling with no_std seems to require the following symbol to be set if there is any floating point code anywhere in the code
#[no_mangle]
pub static _fltused : i32 = 1;
