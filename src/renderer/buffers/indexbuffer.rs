use std::rc::Rc;

use ash::vk;

use crate::core::device::GraphicDevice;

use super::{copy_buffer, create_buffer};

pub struct IndexBuffer {
    device: Rc<GraphicDevice>,
    
    pub(crate) buffer: vk::Buffer,
    pub(crate) memory: vk::DeviceMemory,
}

impl IndexBuffer {
    pub fn new(
        device: Rc<GraphicDevice>,
        command_pool: &vk::CommandPool,
        data: &[u32],
    ) -> Self {
        let buffer_size = std::mem::size_of_val(data) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            &device.logical,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device.memory_properties,
        );

        unsafe {
            let data_ptr = device.logical
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            device.logical.unmap_memory(staging_buffer_memory);
        }

        let (index_buffer, index_buffer_memory) = create_buffer(
            &device.logical,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device.memory_properties,
        );

        copy_buffer(
            &device.logical,
            device.graphics_queue,
            *command_pool,
            staging_buffer,
            index_buffer,
            buffer_size,
        );

        unsafe {
            device.logical.destroy_buffer(staging_buffer, None);
            device.logical.free_memory(staging_buffer_memory, None);
        };

        Self {
            device,
            buffer: index_buffer,
            memory: index_buffer_memory,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical
                .destroy_buffer(self.buffer, None);
            self.device.logical
                .free_memory(self.memory, None);
        }
    }
}
