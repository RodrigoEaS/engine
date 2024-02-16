use ash::vk;
use cgmath::Matrix4;

use crate::device::GraphicDevice;

use super::create_buffer;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub(crate) model: Matrix4<f32>,
    pub(crate) view: Matrix4<f32>,
    pub(crate) proj: Matrix4<f32>,
}

pub struct UniformBuffer {
    pub(crate) buffers: Vec<vk::Buffer>,
    pub(crate) memory: Vec<vk::DeviceMemory>
}

impl UniformBuffer {
    pub fn new(device: &GraphicDevice, swapchain_image_count: usize) -> Self {
        let buffer_size = std::mem::size_of::<UniformBufferObject>();

        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];

        for _ in 0..swapchain_image_count {
            let (uniform_buffer, uniform_buffer_memory) = create_buffer(
                &device.logical,
                buffer_size as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &device.memory_properties,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }
        
        Self {
            buffers: uniform_buffers,
            memory: uniform_buffers_memory
        }
    }

    pub(crate) fn destroy(&self, device: &GraphicDevice) {
        unsafe {
            for i in 0..self.buffers.len() {
                device.logical
                    .destroy_buffer(self.buffers[i], None);
                device.logical
                    .free_memory(self.memory[i], None);
            }
        }
    }
}