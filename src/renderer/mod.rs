pub(crate) mod color_image;
pub(crate) mod debug_object;
pub(crate) mod depth_image;
pub(crate) mod descriptorset;
pub(crate) mod commandpool;
pub(crate) mod pipeline;
pub(crate) mod shader;
pub(crate) mod swapchain;
pub(crate) mod render_pass;
pub(crate) mod buffer;
mod sync_object;

use ash::{
    extensions::{ext, khr},
    vk,
};
use cgmath::{Matrix, Matrix4, SquareMatrix};

use core::ffi::{c_char, c_void, CStr};
use std::{ffi::CString, mem::{size_of, size_of_val}, path::Path, ptr, rc::Rc, slice};

use crate::{
    app::NAME, core::{camera::{Camera, ProjectionViewObject}, device::GraphicDevice, entity::{Entity, EntityJoin}, surface::{Surface, Win32Window}}, image::{check_mipmap_support, Image}, mesh::Mesh
};

use self::{
    buffer::Buffer, color_image::ColorImage, commandpool::CommandPool, debug_object::DebugObjects, depth_image::DepthImage, descriptorset::{descriptor_write, DescriptorInfo, DescriptorLayout, DescriptorPool}, pipeline::GraphicPipeline, render_pass::RenderPass, swapchain::SwapChain, sync_object::{SyncObjects, MAX_FRAMES_IN_FLIGHT}
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

pub fn size_of_array<T>(data: &[T]) -> usize {
    size_of::<T>() * data.len()
}

pub struct Renderer {
    msaa_samples: vk::SampleCountFlags,

    pub(crate) device: Rc<GraphicDevice>,
    instance: ash::Instance,

    surface: Surface,

    debug_objects: DebugObjects,

    swapchain: SwapChain,

    depth_image: DepthImage,
    color_image: ColorImage,

    render_pass: RenderPass,

    entities: EntityJoin,

    pipeline: GraphicPipeline,

    texture: Image,
    mesh: Mesh,

    texture2: Image,
    mesh2: Mesh,

    projection_view: ProjectionViewObject,
    uniform_buffer: Buffer,

    command_pool: CommandPool,

    set_layouts: Vec<DescriptorLayout>,
    descriptor_pool: DescriptorPool,

    sync_objects: SyncObjects,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl Renderer {
    pub fn new(window: &Win32Window) -> Self {
        let entry = ash::Entry::linked();
        let instance = Self::create_instance(&entry);
        
        let surface = Surface::new(&entry, &instance, &window);

        let device = Rc::new(GraphicDevice::new(&instance, &surface));
        
        check_mipmap_support(&instance, device.physical);

        let msaa_samples = Self::get_max_usable_sample_count(&instance, device.physical);
        
        let debug_objects = DebugObjects::new(&entry, &instance);

        let mut swapchain = SwapChain::new(
            &instance, device.clone(), window.size, &surface
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
        
        let mut command_pool = CommandPool::new(device.clone());

        let set_layouts = vec![
            DescriptorLayout::new(device.clone(), vec![
                vk::DescriptorSetLayoutBinding { 
                    binding: 0, 
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, 
                    descriptor_count: 1, 
                    stage_flags: vk::ShaderStageFlags::VERTEX, 
                    ..Default::default()
                }
            ]),
            DescriptorLayout::new(device.clone(), vec![
                vk::DescriptorSetLayoutBinding { 
                    binding: 0, 
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                    descriptor_count: 1, 
                    stage_flags: vk::ShaderStageFlags::FRAGMENT, 
                    ..Default::default()
                }
            ]),
            DescriptorLayout::new(device.clone(), vec![
                vk::DescriptorSetLayoutBinding { 
                    binding: 0, 
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                    descriptor_count: 1, 
                    stage_flags: vk::ShaderStageFlags::FRAGMENT, 
                    ..Default::default()
                }
            ])
        ];
        
        let texture = Image::new(
            device.clone(), 
            &command_pool, 
            Path::new("res/Rail.png")
        );
        let mesh = Mesh::from_obj(
            device.clone(), 
            &command_pool, 
            Path::new("res/Rail.obj")
        );

        let texture2 = Image::new(
            device.clone(), 
            &command_pool, 
            Path::new("res/Viking.png")
        );
        let mesh2 = Mesh::from_obj(
            device.clone(), 
            &command_pool, 
            Path::new("res/Viking.obj")
        );

        let object = Entity::new();
        let mut object2 = Entity::new();
        object2.position.x = -2.0;

        let mut entities = EntityJoin::new();
        entities.add(object);
        entities.add(object2);

        let pipeline = GraphicPipeline::new(
            device.clone(), 
            &render_pass.pass, 
            &swapchain, 
            {
                &set_layouts.iter().map(|x| -> vk::DescriptorSetLayout {
                        x.layout
                    }
                ).collect()
            }, 
            size_of_array(&entities.get_transforms()) as u32,
            msaa_samples
        );

        let projection_view = ProjectionViewObject {
            view: Matrix4::identity(),
            proj: Matrix4::identity()
        };
        let uniform_buffer = Buffer::uniform(device.clone(), size_of_val(&projection_view) as u64);

        let mut descriptor_pool = DescriptorPool::new(device.clone(), 
            vec![
                vk::DescriptorPoolSize { 
                    ty: vk::DescriptorType::UNIFORM_BUFFER, 
                    descriptor_count: 1 
                },
                vk::DescriptorPoolSize { 
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                    descriptor_count: 1 
                },
                vk::DescriptorPoolSize { 
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                    descriptor_count: 1 
                }
            ]
        );
        descriptor_pool.create_sets({
                &set_layouts.iter().map(|x| -> vk::DescriptorSetLayout {
                        x.layout
                    }
                ).collect()
            }
        );

        let descriptor_infos = vec![
            DescriptorInfo::buffer(uniform_buffer.buffer),
            DescriptorInfo::image(texture.sampler, texture.view),
            DescriptorInfo::image(texture2.sampler, texture2.view)
        ];
        let descriptor_writes = vec![
            descriptor_write(
                descriptor_pool.sets[0], 
                vk::DescriptorType::UNIFORM_BUFFER, 
                &descriptor_infos[0], 
                0, 
                1
            ),
            descriptor_write(
                descriptor_pool.sets[1], 
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                &descriptor_infos[1], 
                0, 
                1
            ),
            descriptor_write(
                descriptor_pool.sets[2], 
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 
                &descriptor_infos[2], 
                0, 
                1
            )
        ];

        descriptor_pool.update_sets(descriptor_writes);
            
        let sync_objects = SyncObjects::new(device.clone());

        command_pool.allocate_buffers(&swapchain.framebuffers);

        Self {
            msaa_samples,

            device,
            instance,

            surface,

            debug_objects,

            swapchain,

            depth_image,
            color_image,

            render_pass,

            entities,

            pipeline,

            texture,
            mesh,

            texture2,
            mesh2,

            projection_view,
            uniform_buffer,

            command_pool,

            set_layouts,
            descriptor_pool,

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

    pub(crate) fn record(&mut self) {
        for (i, &command_buffer) in self.command_pool.buffers.iter().enumerate() {
            self.command_pool.begin_command_buffer(command_buffer);

            self.render_pass.begin(
                command_buffer, 
                self.swapchain.extent, 
                self.swapchain.framebuffers[i]
            );

            self.pipeline.bind(command_buffer);

            {
                self.mesh.bind(command_buffer);

                self.descriptor_pool.bind(command_buffer, self.pipeline.layout, 0);

                unsafe { 
                    let model_bytes = slice::from_raw_parts(
                        self.entities.get_transforms()[0].as_ptr() as *const u8,
                        size_of::<Matrix4<f32>>()
                    );
                
                    self.device.logical.cmd_push_constants(
                        command_buffer, 
                        self.pipeline.layout, 
                        vk::ShaderStageFlags::VERTEX, 
                        0, 
                        model_bytes
                    ) 
                };
                self.mesh.draw(command_buffer, 1);
            }

            {
                self.mesh2.bind(command_buffer);

                self.descriptor_pool.bind(command_buffer, self.pipeline.layout, 1);

                unsafe { 
                    let model_bytes = slice::from_raw_parts(
                        self.entities.get_transforms()[1].as_ptr() as *const u8,
                        size_of::<Matrix4<f32>>()
                    );
                
                    self.device.logical.cmd_push_constants(
                        command_buffer, 
                        self.pipeline.layout, 
                        vk::ShaderStageFlags::VERTEX, 
                        0, 
                        model_bytes
                    ) 
                };
                self.mesh2.draw(command_buffer, 1);
            }
            
            self.render_pass.end(command_buffer);

            self.command_pool.end_command_buffer(command_buffer);
        }
    }

    pub(crate) fn draw(&mut self, window: &Win32Window, camera: &Camera) {
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
                        self.recreate_swapchain(window);
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        self.update_uniform_buffer(camera);

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
            self.recreate_swapchain(window);
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

    fn recreate_swapchain(&mut self, window: &Win32Window) {
        self.device.wait_idle();

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(
            &self.instance, 
            self.device.clone(), 
            window.size, 
            &self.surface
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
            {
                &self.set_layouts.iter().map(|x| -> vk::DescriptorSetLayout {
                        x.layout
                    }
                ).collect()
            },
            size_of_array(&self.entities.get_transforms()) as u32,
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

        self.record();
    }
    
    fn update_uniform_buffer(&mut self, camera: &Camera) {
        self.projection_view.view = camera.get_view();
        self.projection_view.proj = camera.get_projection();

        self.uniform_buffer.map(
            &[self.projection_view], 
            size_of_val(&self.projection_view) as u64
        );
    }
    
    pub(crate) fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }

    pub fn destroy(&self) {
        self.device.wait_idle();

        self.sync_objects.destroy();

        self.cleanup_swapchain();

        self.descriptor_pool.destroy();

        self.uniform_buffer.destroy();

        self.mesh.destroy();
        self.texture.destroy();

        self.mesh2.destroy();
        self.texture2.destroy();

        for layout in &self.set_layouts {
            layout.destroy();
        };

        self.command_pool.destroy();

        self.debug_objects.destroy();

        self.device.destroy();
        
        self.surface.destroy();

        unsafe { 
            self.instance.destroy_instance(None) 
        };
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
