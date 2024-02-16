use ash::extensions::khr::Win32Surface;
use ash::vk;
use std::ops::Deref;
use std::os::raw::c_void;
use std::ptr;
use winapi::um::libloaderapi::GetModuleHandleW;
use winit::window::Window;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub struct Surface {
    pub(crate) loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &Window) -> Self {
        let surface = unsafe {
            Self::create_surface(entry, instance, &window).expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Self {
            loader: surface_loader,
            surface
        }
    }

    unsafe fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => panic!("not running on Windows"),
        };
        let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance,
            hwnd: hwnd as *const c_void,
        };
        let win32_surface_loader = Win32Surface::new(entry, instance);
        win32_surface_loader.create_win32_surface(&win32_create_info, None)
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }
    }
}

impl Deref for Surface {
    type Target = vk::SurfaceKHR;

    fn deref(&self) -> &Self::Target {
        &self.surface
    }
}
