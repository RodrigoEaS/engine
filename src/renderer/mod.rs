pub(crate) mod buffers;
pub(crate) mod color_image;
pub(crate) mod debug_object;
pub(crate) mod depth_image;
pub(crate) mod descriptorset;
pub(crate) mod commandpool;
pub(crate) mod pipeline;
pub(crate) mod shader;
pub(crate) mod swapchain;
pub(crate) mod render_pass;
mod sync_object;

use ash::{
    extensions::{ext, khr},
    vk,
};
use cgmath::{Matrix4, SquareMatrix};

use core::ffi::{c_char, c_void, CStr};
use std::{path::Path, ptr, rc::Rc};
use winit::window::Window;

use crate::{
    core::{camera::Camera, device::GraphicDevice, game_object::{GameObject, Transform}, surface::Surface},
    mesh::Mesh,
    texture::{check_mipmap_support, Texture},
};

use self::{
    buffers::uniformbuffer::{UniformBuffer, UniformBufferObject}, color_image::ColorImage, commandpool::CommandPool, debug_object::DebugObjects, depth_image::DepthImage, descriptorset::{
        DescriptorPool, DescriptorSetLayout, DescriptorSets
    }, pipeline::GraphicPipeline, render_pass::RenderPass, swapchain::SwapChain, sync_object::{SyncObjects, MAX_FRAMES_IN_FLIGHT}
};

pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        khr::Surface::name().as_ptr(),
        khr::Win32Surface::name().as_ptr(),
        ext::DebugUtils::name().as_ptr(),
    ]
}

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub(crate) const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}

pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub struct Renderer {
    msaa_samples: vk::SampleCountFlags,

    device: Rc<GraphicDevice>,
    instance: Rc<ash::Instance>,

    window: Rc<Window>,
    surface: Rc<Surface>,

    debug_objects: DebugObjects,

    swapchain: SwapChain,

    depth_image: DepthImage,
    color_image: ColorImage,

    render_pass: RenderPass,

    ubo_layout: DescriptorSetLayout,

    pipeline: GraphicPipeline,

    texture: Texture,

    object1: GameObject,
    object2: GameObject,

    global_ubo: UniformBufferObject,
    uniformbuffer: UniformBuffer,

    descriptor_pool: DescriptorPool,
    descriptor_sets: DescriptorSets,

    command_pool: CommandPool,

    sync_objects: SyncObjects,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl Renderer {
    pub fn new(
        device: Rc<GraphicDevice>,
        entry: ash::Entry,
        instance: Rc<ash::Instance>,
        window: Rc<Window>,
        surface: Rc<Surface>,
    ) -> Self {
        let msaa_samples = Self::get_max_usable_sample_count(&instance, device.physical);
        
        let debug_objects = DebugObjects::new(&entry, &instance);

        let mut swapchain = SwapChain::new(
            &instance, device.clone(), &window, &surface
        );

        let color_image = ColorImage::new(
            device.clone(), &swapchain.format, &swapchain.extent, msaa_samples
        );
        let depth_image = DepthImage::new(
            &instance, device.clone(), &swapchain.extent, msaa_samples
        );

        let render_pass = RenderPass::new(
            &instance, device.clone(), &swapchain.format, msaa_samples
        );

        swapchain.create_framebuffer(
            &render_pass.pass, 
            depth_image.image_view, 
            color_image.image_view
        );

        let ubo_layout = DescriptorSetLayout::new(device.clone());

        let mut command_pool = CommandPool::new(device.clone());

        let pipeline = GraphicPipeline::new(
            device.clone(), 
            &render_pass.pass, 
            &swapchain, 
            &ubo_layout.layout, 
            msaa_samples
        );

        let mesh = Mesh::from_obj(
            device.clone(), 
            &command_pool.pool, 
            Path::new("Rail.obj")
        );

        let mut object1 = GameObject::new(mesh.clone());
        object1.position.x = -2.0;

        let object2 = GameObject::new(mesh.clone());

        check_mipmap_support(&instance, device.physical);
        let texture = Texture::new(device.clone(), &command_pool, Path::new("Rail.png"));

        let global_ubo = UniformBufferObject {
            model: object1.transform(),
            view: Matrix4::identity(),
            proj: Matrix4::identity()
        };

        let uniformbuffer = UniformBuffer::new(device.clone(), swapchain.images.len());

        let descriptor_pool = DescriptorPool::new(device.clone(), swapchain.images.len());
        let descriptor_sets = DescriptorSets::new(
            device.clone(),
            &descriptor_pool.pool,
            &ubo_layout.layout,
            &uniformbuffer.buffers,
            swapchain.images.len(),
            &texture,
        );

        let sync_objects = SyncObjects::new(device.clone());

        command_pool.allocate_buffers(&swapchain.framebuffers);

        Self {
            msaa_samples,

            device,
            instance,

            window,
            surface,

            debug_objects,

            swapchain,

            depth_image,
            color_image,

            render_pass,

            ubo_layout,

            pipeline,

            texture,

            object1,
            object2,

            global_ubo,
            uniformbuffer,

            descriptor_pool,
            descriptor_sets,

            command_pool,

            sync_objects,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn get_max_usable_sample_count(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk::SampleCountFlags {
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
    
        let count = std::cmp::min(
            physical_device_properties
                .limits
                .framebuffer_color_sample_counts,
            physical_device_properties
                .limits
                .framebuffer_depth_sample_counts,
        );
    
        if count.contains(vk::SampleCountFlags::TYPE_64) {
            return vk::SampleCountFlags::TYPE_64;
        }
        if count.contains(vk::SampleCountFlags::TYPE_32) {
            return vk::SampleCountFlags::TYPE_32;
        }
        if count.contains(vk::SampleCountFlags::TYPE_16) {
            return vk::SampleCountFlags::TYPE_16;
        }
        if count.contains(vk::SampleCountFlags::TYPE_8) {
            return vk::SampleCountFlags::TYPE_8;
        }
        if count.contains(vk::SampleCountFlags::TYPE_4) {
            return vk::SampleCountFlags::TYPE_4;
        }
        if count.contains(vk::SampleCountFlags::TYPE_2) {
            return vk::SampleCountFlags::TYPE_2;
        }
    
        vk::SampleCountFlags::TYPE_1
    }

    pub(crate) fn record_command_buffers(&mut self) {
        for (i, &command_buffer) in self.command_pool.buffers.iter().enumerate() {
            self.command_pool.begin_command_buffer(command_buffer);

            self.render_pass.begin(
                command_buffer, 
                self.swapchain.extent, 
                self.swapchain.framebuffers[i]
            );

            self.descriptor_sets.bind(i, command_buffer, self.pipeline.layout);

            self.pipeline.bind(command_buffer);

            self.object1.mesh.bind(command_buffer);

            self.object1.mesh.draw(command_buffer);

            self.render_pass.end(command_buffer);

            self.command_pool.end_command_buffer(command_buffer);
        }
    }

    pub(crate) fn draw(&mut self, camera: &Camera) {
        let wait_fences = [self.sync_objects.in_flight_fences[self.current_frame]];

        unsafe {
            self.device.logical
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain.loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        self.update_uniform_buffer(image_index as usize, camera);

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: self.command_pool.get_buffer(image_index as usize),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .logical
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .logical
                .queue_submit(
                    self.device.graphics_queue,
                    &submit_infos,
                    self.sync_objects.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.device.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
    
    pub(crate) fn cleanup_swapchain(&self) {
        self.depth_image.destroy();
        self.color_image.destroy();

        self.command_pool.free_buffers();

        self.swapchain.destroy_framebuffers();

        self.pipeline.destroy();

        self.render_pass.destroy();

        self.swapchain.destroy();
    }

    fn recreate_swapchain(&mut self) {
        self.device.wait_idle();

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(
            &self.instance, self.device.clone(), &self.window, &self.surface,
        );
        self.render_pass = RenderPass::new(
            &self.instance,
            self.device.clone(),
            &self.swapchain.format,
            self.msaa_samples,
        );
        self.pipeline = GraphicPipeline::new(
            self.device.clone(),
            &self.render_pass.pass,
            &self.swapchain,
            &self.ubo_layout.layout,
            self.msaa_samples,
        );
        self.color_image = ColorImage::new(
            self.device.clone(), 
            &self.swapchain.format,
            &self.swapchain.extent, 
            self.msaa_samples
        );
        self.depth_image = DepthImage::new(
            &self.instance,
            self.device.clone(),
            &self.swapchain.extent,
            self.msaa_samples,
        );

        self.swapchain.create_framebuffer(
            &self.render_pass.pass, 
            self.depth_image.image_view, 
            self.color_image.image_view
        );

        self.command_pool.allocate_buffers(&self.swapchain.framebuffers);

        self.record_command_buffers();
    }
    
    fn update_uniform_buffer(&mut self, current_image: usize, camera: &Camera) {
        self.global_ubo.view = camera.get_view();
        self.global_ubo.proj = camera.get_projection();
        
        let ubos = [self.global_ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self.device.logical.map_memory(
                self.uniformbuffer.memory[current_image],
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device.logical.unmap_memory(self.uniformbuffer.memory[current_image]);
        }
    }
    
    pub(crate) fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }

    pub fn destroy(&self) {
        self.device.wait_idle();

        self.sync_objects.destroy();

        self.cleanup_swapchain();

        self.descriptor_pool.destroy();

        self.uniformbuffer.destroy();

        self.object1.mesh.destroy();

        self.texture.destroy();

        self.ubo_layout.destroy();

        self.command_pool.destroy();

        self.debug_objects.destroy();
    }
}

pub(crate) fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}
