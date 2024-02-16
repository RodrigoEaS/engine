use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use std::{
    ffi::{c_void, CString},
    path::Path,
    ptr,
};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{
    device::GraphicDevice, model::Model, renderer::{
        buffers::{
            commandbuffer::{create_command_buffers, create_command_pool, destroy_command_pool, free_command_buffers}, framebuffer::{self, destroy_framebuffers}, indexbuffer::IndexBuffer, uniformbuffer::{UniformBuffer, UniformBufferObject}, vertexbuffer::VertexBuffer
        }, color_image::ColorImage, debug_object::DebugObjects, depth_image::DepthImage, descriptorset::{
            create_descriptor_pool, create_descriptor_set_layout, create_descriptor_sets, destroy_descriptor_pool, destroy_descriptor_set_layout,
        }, pass::{create_render_pass, destroy_render_pass}, pipeline::GraphicPipeline, required_extension_names, surface::Surface, swapchain::SwapChain, sync_object::{SyncObjects, MAX_FRAMES_IN_FLIGHT}, vk_to_string, vulkan_debug_utils_callback, ValidationInfo
    }, texture::{check_mipmap_support, Texture}
};

pub const NAME: &str = "Rail";
pub const SIZE: (u32, u32) = (800, 600);

pub(crate) const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub struct App {
    instance: ash::Instance,

    window: Window,
    surface: Surface,

    debug_objects: DebugObjects,

    device: GraphicDevice,

    swapchain: SwapChain,

    render_pass: vk::RenderPass,

    ubo_layout: vk::DescriptorSetLayout,

    framebuffers: Vec<vk::Framebuffer>,

    pipeline: GraphicPipeline,

    color_image: ColorImage,
    depth_image: DepthImage,
    msaa_samples: vk::SampleCountFlags,
    texture: Texture,

    model: Model,
    vertexbuffer: VertexBuffer,
    indexbuffer: IndexBuffer,

    uniform_transform: UniformBufferObject,
    uniformbuffer: UniformBuffer,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    sync_objects: SyncObjects,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let entry = ash::Entry::linked();
        let instance = Self::create_instance(&entry);

        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(SIZE.0, SIZE.1))
            .with_title(NAME)
            .build(event_loop)
            .unwrap();
        
        let surface = Surface::new(&entry, &instance, &window);

        let debug_objects = DebugObjects::new(&entry, &instance);
       
        let device = GraphicDevice::new(&instance, &surface);
        
        let msaa_samples = Self::get_max_usable_sample_count(&instance, device.physical);
        
        let swapchain = SwapChain::new(&instance, &device, &window, &surface);
        
        let render_pass = create_render_pass(&instance, &device, &swapchain, msaa_samples);
        
        let ubo_layout = create_descriptor_set_layout(&device);
        
        let command_pool = create_command_pool(&device);
        
        let pipeline = GraphicPipeline::new(
            &device, &render_pass, &swapchain, &ubo_layout, msaa_samples
        );

        let color_image = ColorImage::new(&device, &swapchain, msaa_samples);
        let depth_image = DepthImage::new(
            &instance,
            &device,
            &swapchain.extent,
            msaa_samples,
        );

        let framebuffers = framebuffer::create_framebuffers(
            &device,
            &render_pass,
            depth_image.image_view,
            color_image.image_view,
            &swapchain,
        );

        let model = Model::from_obj(Path::new("rail.obj"));
        
        check_mipmap_support(&instance, device.physical);
        let texture = Texture::new(&device, &command_pool, Path::new("rail.png"));
        
        let vertexbuffer = VertexBuffer::new(&device, &command_pool, &model.vertices);
        let indexbuffer = IndexBuffer::new(&device, &command_pool, &model.indices);
        
        let uniform_transform = UniformBufferObject {
            model: Matrix4::from_angle_z(Deg(90.0)),
            view: Matrix4::look_at_rh(
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let mut proj = cgmath::perspective(
                    Deg(45.0),
                    swapchain.extent.width as f32
                        / swapchain.extent.height as f32,
                    0.1,
                    10.0,
                );
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
        };
        let uniformbuffer = UniformBuffer::new(&device, swapchain.images.len());
       
        let descriptor_pool = create_descriptor_pool(
            &device.logical, swapchain.images.len()
        );
        let descriptor_sets = create_descriptor_sets(
            &device.logical,
            &descriptor_pool,
            &ubo_layout,
            &uniformbuffer.buffers,
            swapchain.images.len(),
            &texture,
        );
        
        let command_buffers = create_command_buffers(
            &device,
            &command_pool,
            &pipeline,
            &framebuffers,
            &render_pass,
            &swapchain,
            &vertexbuffer,
            &indexbuffer,
            &descriptor_sets,
            model.indices.len() as u32,
        );
        
        let sync_objects = SyncObjects::new(&device);

        Self { 
            instance,

            window, 
            surface,

            debug_objects,

            device, 

            swapchain, 

            render_pass,

            ubo_layout, 

            framebuffers, 

            pipeline, 

            color_image, 
            depth_image, 
            msaa_samples, 
            texture, 

            model, 
            vertexbuffer, 
            indexbuffer, 

            uniform_transform, 
            uniformbuffer,

            descriptor_pool, 
            descriptor_sets,

            command_pool, 
            command_buffers, 

            sync_objects, 
            current_frame: 0,

            is_framebuffer_resized: false 
        }
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        if VALIDATION.is_enable && Self::check_validation_layer_support(entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_application_name: NAME.as_ptr() as *const i8,
            application_version: vk::make_api_version(1, 0, 0, 0),
            p_engine_name: "Rail Engine".as_ptr() as *const i8,
            engine_version: vk::make_api_version(1, 0, 0, 0),
            api_version: vk::API_VERSION_1_0,
            ..Default::default()
        };

        let debug_utils_create_info = populate_debug_messenger_create_info();

        let extension_names = required_extension_names();

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &info,
            pp_enabled_layer_names: if VALIDATION.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if VALIDATION.is_enable {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            ..Default::default()
        };

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        // if support validation layer, then return true

        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties!");

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");
            return false;
        } else {
            println!("Instance Available Layers: ");
            for layer in layer_properties.iter() {
                let layer_name = vk_to_string(&layer.layer_name);
                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in VALIDATION.required_validation_layers.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = vk_to_string(&layer_property.layer_name);
                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if is_layer_found == false {
                return false;
            }
        }

        true
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

    pub fn draw_frame(&mut self, delta_time: f32) {
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

        self.update_uniform_buffer(image_index as usize, delta_time);

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
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device.logical
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device.logical
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
            self.swapchain.loader
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

    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
        self.uniform_transform.model =
            Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time)
                * self.uniform_transform.model;

        let ubos = [self.uniform_transform.clone()];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =
                self.device.logical
                    .map_memory(
                        self.uniformbuffer.memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device.logical
                .unmap_memory(self.uniformbuffer.memory[current_image]);
        }
    }

    fn recreate_swapchain(&mut self) {
        self.device.wait_device_idle();

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(&self.instance, &self.device, &self.window, &self.surface);
        self.render_pass = create_render_pass(
            &self.instance,
            &self.device,
            &self.swapchain,
            self.msaa_samples,
        );
        self.pipeline = GraphicPipeline::new(
            &self.device,
            &self.render_pass,
            &self.swapchain,
            &self.ubo_layout,
            self.msaa_samples,
        );
        self.color_image = ColorImage::new(&self.device, &self.swapchain, self.msaa_samples);
        self.depth_image = DepthImage::new(
            &self.instance,
            &self.device,
            &self.swapchain.extent,
            self.msaa_samples,
        );
        self.framebuffers = framebuffer::create_framebuffers(
            &self.device,
            &self.render_pass,
            self.depth_image.image_view,
            self.color_image.image_view,
            &self.swapchain,
        );
        self.command_buffers = create_command_buffers(
            &self.device,
            &self.command_pool,
            &self.pipeline,
            &self.framebuffers,
            &self.render_pass,
            &self.swapchain,
            &self.vertexbuffer,
            &self.indexbuffer,
            &self.descriptor_sets,
            self.model.indices.len() as u32,
        );
    }

    fn cleanup_swapchain(&self) {
        self.depth_image.destroy(&self.device);

        self.color_image.destroy(&self.device);

        free_command_buffers(
            &self.device,
            self.command_pool,
            &self.command_buffers,
        );

        destroy_framebuffers(&self.device, &self.framebuffers);

        self.pipeline.destroy(&self.device);
        
        destroy_render_pass(&self.device, &self.render_pass);
        
        self.swapchain.destroy(&self.device);
    }

    pub(crate) fn wait_device_idle(&self) {
        self.device.wait_device_idle();
    }

    pub(crate) fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }

    pub(crate) fn window_ref(&self) -> &Window {
        &self.window
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

impl Drop for App {
    fn drop(&mut self) {
        self.wait_device_idle();

        self.sync_objects.destroy(&self.device);

        self.cleanup_swapchain();

        destroy_descriptor_pool(&self.device, &self.descriptor_pool);

        self.uniformbuffer.destroy(&self.device);

        self.indexbuffer.destroy(&self.device);

        self.vertexbuffer.destroy(&self.device);

        self.texture.destroy(&self.device);

        destroy_descriptor_set_layout(&self.device, &self.ubo_layout);

        destroy_command_pool(&self.device, &self.command_pool);

        self.device.destroy();
        
        self.surface.destroy();

        self.debug_objects.destroy();
        
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
