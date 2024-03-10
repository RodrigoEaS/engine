use std::{ptr, rc::Rc};

use ash::vk::{self, Framebuffer};

use crate::core::device::GraphicDevice;

pub struct CommandPool {
    device: Rc<GraphicDevice>,

    pub(crate) pool: vk::CommandPool,
    pub(crate) buffers: Vec<vk::CommandBuffer>
}

impl CommandPool {
    pub fn new(device: Rc<GraphicDevice>) -> Self {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: device.family_indices.graphics_family.unwrap(),
        };

        let command_pool = unsafe {
            device
                .logical
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        };

        Self {device, pool: command_pool, buffers: Vec::new()}
    }
    
    pub(crate) fn allocate_buffers(&mut self, framebuffers: &Vec<Framebuffer>) {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool: self.pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };
    
        let command_buffers = unsafe {
            self.device.logical
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        self.buffers = command_buffers;
    }

    pub(crate) fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            self.device.logical
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }
    }

    pub(crate) fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.logical
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }
    /*
    pub(crate) fn create_buffers(
        &mut self,
        graphics_pipeline: &GraphicPipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: &vk::RenderPass,
        swapchain: &SwapChain,
        vertexbuffer: &VertexBuffer,
        indexbuffer: &IndexBuffer,
        descriptor_sets: &Vec<vk::DescriptorSet>,
        index_count: u32,
    ) {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool: self.pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };
    
        let command_buffers = unsafe {
            self.device.logical
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
                self.device.logical
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
                self.device.logical.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                self.device.logical.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.pipeline,
                );
    
                let vertex_buffers = [vertexbuffer.buffer];
                let offsets = [0_u64];
                let descriptor_sets_to_bind = [descriptor_sets[i]];
    
                self.device.logical.cmd_bind_vertex_buffers(
                    command_buffer, 
                    0, 
                    &vertex_buffers, 
                    &offsets
                );
                self.device.logical.cmd_bind_index_buffer(
                    command_buffer,
                    indexbuffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                self.device.logical.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.layout,
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );
    
                self.device.logical.cmd_draw_indexed(
                    command_buffer, 
                    index_count, 
                    1, 
                    0, 
                    0, 
                    0
                );
    
                self.device.logical.cmd_end_render_pass(command_buffer);
    
                self.device.logical
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        self.buffers = command_buffers;
    }
    */
    pub(crate) fn get_buffer(&self, i: usize) -> *const vk::CommandBuffer {
        &self.buffers[i] as *const vk::CommandBuffer
    }

    pub(crate) fn begin_single_time_command(&self) -> vk::CommandBuffer {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: self.pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffer = unsafe {
            self.device.logical
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
            self.device.logical
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        command_buffer
    }

    pub(crate) fn end_single_time_command(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.logical
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
            self.device.logical
                .queue_submit(self.device.graphics_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            self.device.logical
                .queue_wait_idle(self.device.graphics_queue)
                .expect("Failed to wait Queue idle!");
            self.device.logical.free_command_buffers(self.pool, &buffers_to_submit);
        }
    }

    pub(crate) fn free_buffers(&self) {
        unsafe {
            self.device.logical
                .free_command_buffers(self.pool, &self.buffers);
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical.destroy_command_pool(self.pool, None);
        }
    }
}