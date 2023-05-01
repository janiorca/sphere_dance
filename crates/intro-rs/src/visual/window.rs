use crate::visual::gl;

#[cfg(feature = "logger")]
use core::mem::MaybeUninit;

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

#[cfg(feature = "fullscreen")]
use winapi::um::wingdi::{
    DEVMODEA,
};

use winapi::shared::minwindef::{
    LRESULT,
    LPARAM,
    LPVOID,
    WPARAM,
    UINT,
};

#[cfg(feature = "fullscreen")]
use winapi::shared::minwindef::{
    HINSTANCE,
};

use winapi::shared::windef::{
    HDC,
    HGLRC,
    HWND,
    HMENU,
};

#[cfg(not(feature = "fullscreen"))]
use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, GetDC, PostQuitMessage};

#[cfg(not(feature = "fullscreen"))]
use winapi::um::winuser::{RegisterClassA, WNDCLASSA, CS_OWNDC, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, WS_OVERLAPPEDWINDOW};

#[cfg(feature = "logger")]
use winapi::um::winuser::{
    PeekMessageA,
    DispatchMessageA,
    TranslateMessage,
    PM_REMOVE,
    MB_ICONERROR,
    MessageBoxA
};

#[cfg(feature = "fullscreen")]
use winapi::um::winuser::{
    WS_MAXIMIZE,
    WS_POPUP,
};

use winapi::um::winuser::{
    WS_VISIBLE,
};
use crate::intro::Command;

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
            let mut dev_mode: DEVMODEA = core::mem::zeroed();
            dev_mode.dmSize = core::mem::size_of::<DEVMODEA>() as u16;
            dev_mode.dmFields = winapi::um::wingdi::DM_BITSPERPEL | winapi::um::wingdi::DM_PELSWIDTH | winapi::um::wingdi::DM_PELSHEIGHT;
            dev_mode.dmBitsPerPel = 32;
            dev_mode.dmPelsWidth  = 1920;
            dev_mode.dmPelsHeight = 1080;
            if winapi::um::winuser::ChangeDisplaySettingsA(&mut dev_mode, winapi::um::winuser::CDS_FULLSCREEN)!= winapi::um::winuser::DISP_CHANGE_SUCCESSFUL {
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
                CW_USEDEFAULT, CW_USEDEFAULT, 1920, 1080,	// size and position
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


        // make the system font the device context's selected font
        winapi::um::wingdi::SelectObject (h_dc, winapi::um::wingdi::GetStockObject (winapi::um::wingdi::SYSTEM_FONT as i32));

        // create the bitmap display lists
        winapi::um::wingdi::wglUseFontBitmapsA (h_dc, 0, 255, 1000);

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
        let msg = MaybeUninit::uninit();
        let mut msg = msg.assume_init();
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

pub struct Window {
    _window: HWND,
    hdc: HDC
}

impl Window {
    pub fn new() -> Self {
        let ( _window, hdc ) = create_window();
        Self { _window, hdc }
    }

    pub fn manage(&self) -> Option<Command> {
        #[cfg(feature = "logger")]
        {
            if !handle_message( _window ) {
                return Some(Command::Exit);
            }
        }
        None
    }

    pub fn present(&self) {
        unsafe {
            SwapBuffers(self.hdc);
        }
    }
}