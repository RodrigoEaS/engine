use std::rc::Rc;

use ash::vk;
use memoffset::offset_of;

use crate::core::device::GraphicDevice;

use super::{copy_buffer, create_buffer};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, tex_coord) as u32,
            },
        ]
    }
}

pub struct VertexBuffer {
    device: Rc<GraphicDevice>,
    
    pub(crate) buffer: vk::Buffer,
    pub(crate) memory: vk::DeviceMemory
} 

impl VertexBuffer {
    pub fn new (device: Rc<GraphicDevice>, command_pool: &vk::CommandPool, data: &[Vertex]) -> Self {
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
                .expect("Failed to Map Memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

            device.logical.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            &device.logical,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device.memory_properties,
        );

        copy_buffer(
            &device.logical,
            device.graphics_queue,
            *command_pool,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        unsafe {
            device.logical.destroy_buffer(staging_buffer, None);
            device.logical.free_memory(staging_buffer_memory, None);
        };

        Self {
            device,
            buffer: vertex_buffer,
            memory: vertex_buffer_memory,
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