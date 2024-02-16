use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}};

use crate::{app::App, fps::Fps};

pub struct AppWindow {
    pub event_loop: EventLoop<()>,
}

impl AppWindow {
    pub fn new() -> Self {
        // init window stuff
        let event_loop = EventLoop::new().unwrap();

        Self { event_loop }
    }

    pub fn run(self, mut app: App) {
        let mut tick_counter = Fps::new();

        self.event_loop.set_control_flow(ControlFlow::Poll);

        let _ = self.event_loop.run(move |event, control_flow| {

            match event {
                | Event::AboutToWait => {
                    app.window_ref().request_redraw();
                },
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::RedrawRequested => {
                            app.draw_frame(tick_counter.delta_time());
                            
                            print!("FPS: {}\r", tick_counter.fps());

                            tick_counter.tick_frame();
                        },
                        | WindowEvent::CloseRequested => {
                            app.wait_device_idle();
                            control_flow.exit()
                        },
                        | WindowEvent::Resized(_) => {
                            app.wait_device_idle();
                            //FIX THIS LATER 
                            app.resize_framebuffer();
                        },
                        | _ => {},
                    }
                },
                _ => (),
            }
        });
    }
}