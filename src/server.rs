use std::{error::Error, sync::mpsc, time::Duration};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use tonic::{transport::Server, Request, Response, Status};

use graphics_device::graphics_device_server::{GraphicsDevice, GraphicsDeviceServer};
use graphics_device::{Empty, ResizeWindowRequest};

pub mod graphics_device {
    tonic::include_proto!("graphics_device");
}

#[derive(Debug)]
struct MyGraphicsDevice {
    event_loop_proxy: EventLoopProxy<UserEvent>,
}

impl MyGraphicsDevice {
    fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { event_loop_proxy }
    }
}

#[tonic::async_trait]
impl GraphicsDevice for MyGraphicsDevice {
    async fn resize_window(
        &self,
        request: Request<ResizeWindowRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("Got a request: {:?}", request);

        let ResizeWindowRequest { width, height } = request.get_ref();
        self.event_loop_proxy
            .send_event(UserEvent::ResizeWindow {
                height: *height,
                width: *width,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }
}

#[derive(Default)]
struct VelloApp {
    idx: usize,
    window_id: Option<WindowId>,
    window: Option<Window>,
}

impl ApplicationHandler<UserEvent> for VelloApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attr = Window::default_attributes()
            .with_title("test")
            .with_inner_size(winit::dpi::LogicalSize::new(400.0, 400.0));
        let window = event_loop.create_window(attr).unwrap();
        self.window_id = Some(window.id());
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if event == WindowEvent::Destroyed && self.window_id == Some(window_id) {
            self.window_id = None;
            event_loop.exit();
            return;
        }

        let window = match self.window.as_mut() {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                // TODO
                // fill::cleanup_window(window);
                self.window = None;
            }
            WindowEvent::RedrawRequested => {
                // TODO
                // fill::fill_window(window);
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ResizeWindow { height, width } => {
                if let Some(window) = self.window.as_mut() {
                    let sizes = window.inner_size();
                    // TODO: handle error
                    let _res = window.request_inner_size(winit::dpi::LogicalSize::new(
                        (sizes.width as i32 + height) as u32,
                        (sizes.height as i32 + width) as u32,
                    ));
                }
            }
            UserEvent::WakeUp => {}
        };
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum UserEvent {
    WakeUp,
    ResizeWindow { height: i32, width: i32 },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = VelloApp {
        idx: 1,
        ..Default::default()
    };
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();

    let addr = "[::1]:50051".parse()?;
    let greeter = MyGraphicsDevice::new(event_loop_proxy);

    tokio::spawn(async move {
        // TODO: propagate error via EventLoopProxy
        let _res = Server::builder()
            .add_service(GraphicsDeviceServer::new(greeter))
            .serve(addr)
            .await;
    });

    event_loop.run_app(&mut app)?;

    Ok(())
}
