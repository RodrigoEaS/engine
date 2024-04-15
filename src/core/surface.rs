use ash::extensions::khr::Win32Surface;
use ash::vk;
use cgmath::Vector2;
use std::os::raw::c_void;
use std::ptr;
use windows::{
    core::*, Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*},
};

use crate::app::App;

pub struct Win32Window {
    pub(crate) hwnd: HWND,
    pub(crate) instance: HMODULE,

    pub(crate) size: Vector2<u32>
}

impl Win32Window {
    pub fn new() -> Win32Window {
        unsafe {
            let instance = GetModuleHandleA(None).unwrap();
            debug_assert!(instance.0 != 0);

            let window_class = s!("window");
            
            let wc = WNDCLASSA {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hInstance: instance.into(),
                lpszClassName: window_class,

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };
            
            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            let window = CreateWindowExA(
                WS_EX_APPWINDOW,
                window_class,
                s!("This is a sample window"),
                WS_THICKFRAME | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1080,
                720,
                None,
                None,
                instance,
                None,
            );

            ShowWindow(window, SW_SHOW);

            Self {
                hwnd:window,
                instance,

                size: Vector2 { x: 1080, y: 720 }
            }
        }
    }

    pub fn update(&self, app: &mut App) -> bool {
        unsafe {
            let mut msg = MSG::default();
    
            if PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).into() {
                _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
    
                match msg.message {
                    WM_QUIT => {
                        return false;
                    }
                    WM_KEYDOWN => {
                        app.input.register(msg.wParam.0 as u8);
                        return true
                    }
                    WM_KEYUP => {
                        app.input.register(0);
                        return true
                    }
                    WM_SIZE => {
                        return true
                    }
                    _ => return true,
                } 
            }

            true
        }
    }
}


extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_CREATE => {
            println!("Window created");
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => {
            unsafe { DefWindowProcA(window, message, wparam, lparam) }
        }
    }
}

pub struct Surface {
    pub(crate) loader: ash::extensions::khr::Surface,
    pub(crate) surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &Win32Window) -> Self {
        let surface = Self::create_surface(entry, instance, &window);
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Self {
            loader: surface_loader,
            surface
        }
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Win32Window,
    ) -> vk::SurfaceKHR {
        let hwnd = window.hwnd.0;
        let hinstance = window.instance.0;
        
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance: hinstance as *const c_void,
            hwnd: hwnd as *const c_void,
        };
        let win32_surface_loader = Win32Surface::new(entry, instance);

        unsafe {
            win32_surface_loader.create_win32_surface(
                &win32_create_info, None
            ).expect("")
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }
    }
}
