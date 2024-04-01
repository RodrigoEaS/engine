use std::{ptr, rc::Rc};

use ash::vk;

use crate::core::device::GraphicDevice;

use super::commandpool::CommandPool;

pub struct Buffer {
    device: Rc<GraphicDevice>,
    
    pub(crate) buffer: vk::Buffer,
    pub(crate) memory: vk::DeviceMemory
}

impl Buffer {
    pub fn new(
        device: Rc<GraphicDevice>, 
        size: u64, 
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags
    ) -> Self {
        let buffer_create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };
    
        let buffer = unsafe {
            device.logical
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create Buffer")
        };
    
        let mem_requirements = unsafe { device.logical.get_buffer_memory_requirements(buffer) };
        let memory_type = find_memory_type(
            mem_requirements.memory_type_bits,
            memory_properties,
            &device.memory_properties,
        );
    
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: memory_type,
        };
    
        let buffer_memory = unsafe {
            device.logical
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate vertex buffer memory!")
        };
    
        unsafe {
            device.logical
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer");
        }

        Self {
            device,
            buffer,
            memory: buffer_memory,
        }
    }
    
    pub fn staging(device: Rc<GraphicDevice>, size: u64) -> Self {
        Self::new(
            device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        )
    }

    pub fn vertex(device: Rc<GraphicDevice>, size: u64) -> Self {
        Self::new(
            device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }

    pub fn index(device: Rc<GraphicDevice>, size: u64) -> Self {
        Self::new(
            device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }

    pub fn uniform(device: Rc<GraphicDevice>, size: u64) -> Self {
        Self::new(
            device, 
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
    }

    pub fn storage(device: Rc<GraphicDevice>, size: u64) -> Self {
        Self::new(
            device, 
            size, 
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
    }

    pub(crate) fn map<T>(&self, data: &[T], size: vk::DeviceSize) {
        unsafe {
            let data_ptr = self.device.logical
                .map_memory(
                    self.memory,
                    0,
                    size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut T;
    
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
            self.device.logical.unmap_memory(self.memory);
        }
    }

    pub(crate) fn copy(&self, src: &Buffer, command_pool: &CommandPool, size: u64) {
        let command_buffer = command_pool.begin_single_time_command();

        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        unsafe {
            self.device.logical
                .cmd_copy_buffer(command_buffer, src.buffer, self.buffer, &copy_regions);
        }

        command_pool.end_single_time_command(command_buffer);
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

pub(crate) fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0
            && memory_type.property_flags.contains(required_properties)
        {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}