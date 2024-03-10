use std::{ffi::{c_void, CString}, ptr, rc::Rc};

use ash::vk;
use cgmath::Vector3;
use winit::{dpi::LogicalSize, event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::Key, window::{Window, WindowBuilder}};

use crate::{core::{camera::Camera, device::GraphicDevice, surface::Surface, time::Fps}, renderer::{populate_debug_messenger_create_info, required_extension_names, vk_to_string, Renderer, VALIDATION}};

pub const NAME: &str = "Rail";

pub struct App {
    instance: Rc<ash::Instance>,
    device: Rc<GraphicDevice>,

    window: Rc<Window>,
    surface: Rc<Surface>,

    renderer: Renderer,

    camera: Camera
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let entry = ash::Entry::linked();
        let instance = Rc::new(Self::create_instance(&entry));

        let window = Rc::new(WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800, 600))
            .with_title(NAME)
            .build(event_loop)
            .unwrap());
        
        let surface = Rc::new(Surface::new(&entry, &instance, &window));

        let device = Rc::new(GraphicDevice::new(&instance, &surface));

        let renderer = Renderer::new(
            device.clone(), 
            entry, 
            instance.clone(), 
            window.clone(), 
            surface.clone()
        );

        let camera = Camera::new(
            (window.inner_size().width as f32, window.inner_size().height as f32)
        );

        Self {
            instance,
            device,

            window,
            surface,

            renderer,

            camera
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

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut tick_counter = Fps::new();

        event_loop.set_control_flow(ControlFlow::Poll);

        self.renderer.record_command_buffers();

        let _ = event_loop.run(move |event, control_flow| {
            match event {
                | Event::AboutToWait => {
                    self.window.request_redraw();
                },
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::RedrawRequested => {
                            if self.window.inner_size().width != 0 && self.window.inner_size().height != 0 {
                                self.renderer.draw(&self.camera);
                            } 

                            self.window.set_title(&format!("{} - {}", NAME, tick_counter.fps()));

                            tick_counter.tick_frame();
                        },
                        | WindowEvent::CloseRequested => {
                            self.device.wait_idle();
                            control_flow.exit()
                        },
                        | WindowEvent::Resized(_) => {
                            self.device.wait_idle();
                            self.renderer.resize_framebuffer();
                        },
                        | WindowEvent::KeyboardInput {event, ..} => {
                            match event.logical_key {
                                Key::Character(c) => {
                                    match c.as_str() {
                                        //-z
                                        "w" => {
                                            self.camera.position += Vector3 { 
                                                x: 0.0, y: 0.0, z: -1.0 
                                            } * tick_counter.delta_time();
                                        },
                                        //+z
                                        "s" => {
                                            self.camera.position += Vector3 { 
                                                x: 0.0, y: 0.0, z: 1.0 
                                            } * tick_counter.delta_time();
                                        },
                                        //-x
                                        "a" => {
                                            self.camera.position += Vector3 { 
                                                x: 1.0, y: 0.0, z: 0.0 
                                            } * tick_counter.delta_time();
                                        },
                                        //+x
                                        "d" => {
                                            self.camera.position += Vector3 { 
                                                x: -1.0, y: 0.0, z: 0.0 
                                            } * tick_counter.delta_time();
                                        },
                                        //+y
                                        "j" => {
                                            self.camera.position += Vector3 { 
                                                x: 0.0, y: 1.0, z: 0.0 
                                            } * tick_counter.delta_time();
                                        },
                                        //-y
                                        "k" => {
                                            self.camera.position += Vector3 { 
                                                x: 0.0, y: -1.0, z: 0.0 
                                            } * tick_counter.delta_time();
                                        },

                                        _ => {}

                                    }
                                },
                                _ => {}
                            }
                        },
                        | _ => {},
                    }
                },
                _ => (),
            }
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.renderer.destroy();

        self.device.destroy();
        
        self.surface.destroy();

        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}