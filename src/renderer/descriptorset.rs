use std::rc::Rc;

use ash::vk;

use crate::core::device::GraphicDevice;

pub struct DescriptorPool {
    device: Rc<GraphicDevice>,
    
    pool: vk::DescriptorPool,
    pub(crate) sets: Vec<vk::DescriptorSet>
}

impl DescriptorPool {
    pub fn new(device: Rc<GraphicDevice>, pool_sizes: Vec<vk::DescriptorPoolSize>) -> Self {
        let descriptor_pool = {
            let pool_info = vk::DescriptorPoolCreateInfo {
                s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
                max_sets: 1,
                pool_size_count: pool_sizes.len() as u32,
                p_pool_sizes: pool_sizes.as_ptr(),
                ..Default::default()
            };
        
            unsafe {
                device.logical.create_descriptor_pool(&pool_info, None)
                    .expect("Failed to create descriptor pool")
            }
        };

        Self {
            device,
            pool: descriptor_pool,
            sets: Vec::new(),
        }
    }

    pub(crate) fn create_sets(&mut self, set_layout: vk::DescriptorSetLayout) {
        let allocation_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.pool,
            descriptor_set_count: 1,
            p_set_layouts: &set_layout,
            ..Default::default()
        };
        
        self.sets = unsafe {
            self.device.logical.allocate_descriptor_sets(&allocation_info)
                .expect("Failed to allocate descriptor sets")
        };
    }

    pub(crate) fn update_sets(&self, writes: Vec<vk::WriteDescriptorSet>) {
        unsafe { 
            self.device.logical.update_descriptor_sets(&writes, &[]) 
        };
    }

    pub(crate) fn bind(&self, command_buffer: vk::CommandBuffer, layout: vk::PipelineLayout) {
        let descriptor_sets_to_bind = [self.sets[0]];

        unsafe {
            self.device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical.destroy_descriptor_pool(self.pool, None)
        }
    }
}

pub struct DescriptorLayout {
    device: Rc<GraphicDevice>,
    
    pub(crate) layout: vk::DescriptorSetLayout,
}

impl DescriptorLayout {
    pub fn new(device: Rc<GraphicDevice>, layouts_bindings: Vec<vk::DescriptorSetLayoutBinding>) -> Self {
        let layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: layouts_bindings.len() as u32,
            p_bindings: layouts_bindings.as_ptr(),
            ..Default::default()
        };

        let set_layout = unsafe { 
            device.logical.create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        };

        Self {
            device,
            layout: set_layout,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical.destroy_descriptor_set_layout(self.layout, None)
        }
    }
}

pub enum DescriptorInfo {
    Buffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo)
}

impl DescriptorInfo {
    pub(crate) fn buffer(buffer: vk::Buffer) -> Self {
        Self::Buffer(
            vk::DescriptorBufferInfo {
                buffer,
                offset: 0,
                range: vk::WHOLE_SIZE,
            }
        )
    }

    pub(crate) fn image(sampler: vk::Sampler, view: vk::ImageView) -> Self {
        Self::Image(
            vk::DescriptorImageInfo {
                sampler,
                image_view: view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            }
        )
    }
}

pub(crate) fn descriptor_write(
    set: vk::DescriptorSet, 
    set_type: vk::DescriptorType,
    info: &DescriptorInfo,
    binding: u32,
    count: u32
) -> vk::WriteDescriptorSet {
    let mut write = vk::WriteDescriptorSet { 
        s_type: vk::StructureType::WRITE_DESCRIPTOR_SET, 
        dst_set: set, 
        dst_binding: binding, 
        descriptor_count: count, 
        descriptor_type: set_type, 
        ..Default::default()
    };
    
    match info {
        DescriptorInfo::Buffer(buffer) => {
            write.p_buffer_info = buffer
        },
        DescriptorInfo::Image(imagen) => {
            write.p_image_info = imagen
        },
    };

    write
}