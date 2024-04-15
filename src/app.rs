use cgmath::{Vector2, Vector3};

use crate::{core::{camera::Camera, input::InputManager, surface::Win32Window, time::Fps}, renderer::Renderer};

pub const NAME: &str = "Rail";

pub struct App {
    camera: Camera,
    pub(crate) input: InputManager
}

impl App {
    fn new() -> Self {
        let camera = Camera::new(Vector2::new(1080, 720));
        let input = InputManager::new();

        Self {
            camera,
            input
        }
    }
    
    fn run(mut self, renderer: &mut Renderer, window: Win32Window) {
        let mut tick_counter = Fps::new();

        renderer.record();

        let speed = 3.0;

        loop {
            if window.update(&mut self) == false {
                renderer.device.wait_idle();
                break;
            }
            
            match self.input.input {
                //-z
                /*W*/87 => {
                    self.camera.position += Vector3 { 
                        x: 0.0, y: 0.0, z: -1.0 
                    } * speed * tick_counter.delta_time();
                },
                //+z
                /*S*/83 => {
                    self.camera.position += Vector3 { 
                        x: 0.0, y: 0.0, z: 1.0 
                    } * speed * tick_counter.delta_time();
                },
                //-x
                /*A*/65 => {
                    self.camera.position += Vector3 { 
                        x: 1.0, y: 0.0, z: 0.0 
                    } * speed * tick_counter.delta_time();
                },
                //+x
                    /*D*/68 => {
                    self.camera.position += Vector3 { 
                        x: -1.0, y: 0.0, z: 0.0 
                    } * speed * tick_counter.delta_time();
                },
                //+y
                    /*J*/74 => {
                    self.camera.position += Vector3 { 
                        x: 0.0, y: 1.0, z: 0.0 
                    } * speed * tick_counter.delta_time();
                },
                //-y
                    /*K*/75 => {
                    self.camera.position += Vector3 { 
                        x: 0.0, y: -1.0, z: 0.0 
                    } * speed * tick_counter.delta_time();
                },
                _ => ()
            }

            renderer.draw(&window, &self.camera);

            tick_counter.tick_frame();
        }
    }
}

pub fn run_rail() {
    let app = App::new();

    let window = Win32Window::new();

    let mut renderer = Renderer::new(&window);

    app.run(&mut renderer, window);

    renderer.destroy();
}