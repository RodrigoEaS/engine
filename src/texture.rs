use std::{cmp::max, path::Path, ptr};

use ash::vk;

use crate::{device::GraphicDevice, renderer::buffers::{create_buffer, find_memory_type}};

pub const FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;

pub struct Texture {
    pub(crate) image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub(crate) sampler: vk::Sampler,
    pub(crate) memory: vk::DeviceMemory,
    mip_levels: u32
}

impl Texture {
    pub fn new(device: &GraphicDevice, command_pool: &vk::CommandPool, image_path: &Path) -> Self {
        let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_data = match &image_object {
            | image::DynamicImage::ImageLuma8(_)
            | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba8().into_raw(),
            | image::DynamicImage::ImageLumaA8(_)
            | image::DynamicImage::ImageRgba8(_) => image_object.to_rgba8().into_raw(),
            _ => panic!("unsupported file type")
        };
        let image_size =
            (::std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let mip_levels = ((::std::cmp::max(image_width, image_height) as f32)
            .log2()
            .floor() as u32)
            + 1;

        if image_size <= 0 {
            panic!("Failed to load texture image!")
        }

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            &device.logical,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device.memory_properties,
        );

        unsafe {
            let data_ptr = device.logical
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            device.logical.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = Self::create_image(
            &device.logical,
            image_width,
            image_height,
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device.memory_properties,
        );

        Self::transition_image_layout(
            &device.logical,
            *command_pool,
            device.graphics_queue,
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
        );

        Self::copy_buffer_to_image(
            &device.logical,
            *command_pool,
            device.graphics_queue,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
        );

        Self::generate_mipmaps(
            &device.logical,
            *command_pool,
            device.graphics_queue,
            texture_image,
            image_width,
            image_height,
            mip_levels,
        );

        unsafe {
            device.logical.destroy_buffer(staging_buffer, None);
            device.logical.free_memory(staging_buffer_memory, None);
        }

        let texture_image_view = 
            Self::create_texture_image_view(&device.logical, texture_image, 1);
        let texture_sampler = Self::create_texture_sampler(&device.logical, mip_levels);

        Self {
            image: texture_image,
            memory: texture_image_memory,
            image_view: texture_image_view,
            sampler: texture_sampler,
            mip_levels
        }
    }

    pub(crate) fn create_image(
        device: &ash::Device,
        width: u32,
        height: u32,
        mip_levels: u32,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        required_memory_properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties
    ) -> (vk::Image, vk::DeviceMemory) {
        let image_create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            mip_levels,
            array_layers: 1,
            samples: num_samples,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        };
    
        let texture_image = unsafe {
            device
                .create_image(&image_create_info, None)
                .expect("Failed to create Texture Image!")
        };
    
        let image_memory_requirement = unsafe { device.get_image_memory_requirements(texture_image) };
        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: image_memory_requirement.size,
            memory_type_index: find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
                device_memory_properties,
            ),
        };
    
        let texture_image_memory = unsafe {
            device
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };
    
        unsafe {
            device
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        }
    
        (texture_image, texture_image_memory)
    }

    fn transition_image_layout(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        image: vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        mip_levels: u32,
    ) {
        let command_buffer = Self::begin_single_time_command(device, command_pool);

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask =
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        Self::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn copy_buffer_to_image(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) {
        let command_buffer = Self::begin_single_time_command(device, command_pool);

        let buffer_image_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            buffer_offset: 0,
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];

        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &buffer_image_regions,
            );
        }

        Self::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn begin_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> vk::CommandBuffer {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }[0];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        command_buffer
    }

    fn end_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        let buffers_to_submit = [command_buffer];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: buffers_to_submit.as_ptr(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }];

        unsafe {
            device
                .queue_submit(submit_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            device
                .queue_wait_idle(submit_queue)
                .expect("Failed to wait Queue idle!");
            device.free_command_buffers(command_pool, &buffers_to_submit);
        }
    }

    pub(crate) fn create_image_view(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> vk::ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };
    
        unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        }
    }

    fn create_texture_image_view(device: &ash::Device, texture_image: vk::Image, mip_levels: u32) -> vk::ImageView {
        Self::create_image_view(
            device,
            texture_image,
            FORMAT,
            vk::ImageAspectFlags::COLOR,
            mip_levels,
        )
    }

    fn create_texture_sampler(device: &ash::Device, mip_levels: u32) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            min_lod: 0.0,
            max_lod: mip_levels as f32,
            mip_lod_bias: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            ..Default::default()
        };

        unsafe {
            device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }

    fn generate_mipmaps(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        image: vk::Image,
        tex_width: u32,
        tex_height: u32,
        mip_levels: u32,
    ) {
        let command_buffer = Self::begin_single_time_command(device, command_pool);

        let mut image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        let mut mip_width = tex_width as i32;
        let mut mip_height = tex_height as i32;

        for i in 1..mip_levels {
            image_barrier.subresource_range.base_mip_level = i - 1;
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }

            let blits = [vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1,
                    },
                ],
            }];

            unsafe {
                device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                );
            }

            image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }

            mip_width = max(mip_width / 2, 1);
            mip_height = max(mip_height / 2, 1);
        }

        image_barrier.subresource_range.base_mip_level = mip_levels - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        Self::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    pub(crate) fn destroy(&self, device: &GraphicDevice) {
        unsafe {
            device.logical
                .destroy_sampler(self.sampler, None);
            device.logical
                .destroy_image_view(self.image_view, None);
            device.logical
                .destroy_image(self.image, None);
            device.logical
                .free_memory(self.memory, None);
        }
    }
}

pub fn check_mipmap_support(
    instance: &ash::Instance,
    physcial_device: vk::PhysicalDevice,
) {
    let format_properties = unsafe {
        instance.get_physical_device_format_properties(physcial_device, FORMAT)
    };

    let is_sample_image_filter_linear_support = format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

    if is_sample_image_filter_linear_support == false {
        panic!("Texture Image format does not support linear blitting!")
    }
}