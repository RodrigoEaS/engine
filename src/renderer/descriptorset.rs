use ash::vk;
use std::{ptr, rc::Rc};

use crate::{core::device::GraphicDevice, texture::Texture};

use super::buffers::uniformbuffer::UniformBufferObject;

pub struct DescriptorPool {
    device: Rc<GraphicDevice>,

    pub(crate) pool: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(device: Rc<GraphicDevice>, swapchain_images_size: usize) -> Self {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                // transform descriptor pool
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain_images_size as u32,
            },
            vk::DescriptorPoolSize {
                // sampler descriptor pool
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swapchain_images_size as u32,
            },
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: swapchain_images_size as u32,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        let descriptor_pool = unsafe {
            device
                .logical
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        };

        Self {
            device,
            pool: descriptor_pool,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical.destroy_descriptor_pool(self.pool, None);
        }
    }
}

pub struct DescriptorSets {
    device: Rc<GraphicDevice>,

    pub(crate) sets: Vec<vk::DescriptorSet>,
}

impl DescriptorSets {
    pub fn new(
        device: Rc<GraphicDevice>,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_set_layout: &vk::DescriptorSetLayout,
        uniforms_buffers: &Vec<vk::Buffer>,
        swapchain_images_size: usize,
        texture: &Texture,
    ) -> Self {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(*descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: *descriptor_pool,
            descriptor_set_count: swapchain_images_size as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device.logical
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniforms_buffers[i],
                offset: 0,
                range: ::std::mem::size_of::<UniformBufferObject>() as u64,
            }];

            let descriptor_image_infos = [vk::DescriptorImageInfo {
                sampler: texture.sampler,
                image_view: texture.image_view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            }];

            let descriptor_write_sets = [
                vk::WriteDescriptorSet {
                    // transform uniform
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descritptor_set,
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info: ptr::null(),
                    p_buffer_info: descriptor_buffer_infos.as_ptr(),
                    p_texel_buffer_view: ptr::null(),
                },
                vk::WriteDescriptorSet {
                    // sampler uniform
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descritptor_set,
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    p_image_info: descriptor_image_infos.as_ptr(),
                    p_buffer_info: ptr::null(),
                    p_texel_buffer_view: ptr::null(),
                },
            ];

            unsafe {
                device.logical.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        Self {
            device,

            sets: descriptor_sets,
        }
    }

    pub(crate) fn bind(&self, i: usize, command_buffer: vk::CommandBuffer, layout: vk::PipelineLayout) {
        let descriptor_sets_to_bind = [self.sets[i]];

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
}

pub struct DescriptorSetLayout {
    device: Rc<GraphicDevice>,

    pub(crate) layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: Rc<GraphicDevice>) -> Self {
        let ubo_layout_bindings = [
            vk::DescriptorSetLayoutBinding {
                // transform uniform
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: ptr::null(),
            },
            vk::DescriptorSetLayoutBinding {
                // sampler uniform
                binding: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: ptr::null(),
            },
        ];

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: ubo_layout_bindings.len() as u32,
            p_bindings: ubo_layout_bindings.as_ptr(),
        };

        let ubo_layout = unsafe {
            device
                .logical
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")
        };

        Self {
            device,
            layout: ubo_layout,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device
                .logical
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
