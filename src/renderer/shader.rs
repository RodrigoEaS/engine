use std::{fs::File, io::Read, path::Path, ptr};

use ash::vk;

use crate::core::device::GraphicDevice;

pub struct Shader {
    pub(super) module: vk::ShaderModule
}

impl Shader {
    pub fn from_spv(shader_path: &Path, device: &GraphicDevice) -> Self {
        let spv_file = File::open(shader_path)
            .expect(&format!("Failed to find spv file at {:?}", shader_path));
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: bytes_code.len(),
            p_code: bytes_code.as_ptr() as *const u32,
        };

        let module = unsafe {
            device.logical
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module!")
        };

        Self {
            module
        }
    }
}