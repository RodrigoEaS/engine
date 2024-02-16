use std::ptr;

use ash::vk;

use super::{indexbuffer::IndexBuffer, vertexbuffer::VertexBuffer};
use crate::{
    device::GraphicDevice,
    renderer::{pipeline::GraphicPipeline, swapchain::SwapChain}
};

pub fn create_command_pool(device: &GraphicDevice) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: device.family_indices.graphics_family.unwrap(),
    };

    unsafe {
        device.logical
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

pub fn create_command_buffers(
    device: &GraphicDevice,
    command_pool: &vk::CommandPool,
    graphics_pipeline: &GraphicPipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: &vk::RenderPass,
    swapchain: &SwapChain,
    vertexbuffer: &VertexBuffer,
    indexbuffer: &IndexBuffer,
    descriptor_sets: &Vec<vk::DescriptorSet>,
    index_count: u32,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool: *command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device.logical
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            device.logical
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [
            vk::ClearValue {
                // clear value for color buffer
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                // clear value for depth buffer
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: *render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            device.logical.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.logical.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.pipeline,
            );

            let vertex_buffers = [vertexbuffer.buffer];
            let offsets = [0_u64];
            let descriptor_sets_to_bind = [descriptor_sets[i]];

            device.logical.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
            device.logical.cmd_bind_index_buffer(
                command_buffer,
                indexbuffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
            device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );

            device.logical
                .cmd_draw_indexed(command_buffer, index_count, 1, 0, 0, 0);

            device.logical.cmd_end_render_pass(command_buffer);

            device.logical
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    command_buffers
}

pub(crate) fn destroy_command_pool(
    device: &GraphicDevice,
    command_pool: &vk::CommandPool
) {
    unsafe {
        device.logical
            .destroy_command_pool(*command_pool, None);
    }
}

pub(crate) fn free_command_buffers(
    device: &GraphicDevice,
    command_pool: vk::CommandPool,
    command_buffers: &Vec<vk::CommandBuffer>,
) {
    unsafe {
        device.logical.free_command_buffers(
            command_pool,
            command_buffers
        );
    }
}
