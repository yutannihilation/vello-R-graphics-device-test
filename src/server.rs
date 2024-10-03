use std::{error::Error, sync::mpsc, time::Duration};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    platform::pump_events::EventLoopExtPumpEvents,
    window::{Window, WindowId},
};

use tonic::{transport::Server, Request, Response, Status};

use draw::greeter_server::{Greeter, GreeterServer};
use draw::{HelloReply, HelloRequest};

pub mod draw {
    tonic::include_proto!("draw");
}

#[derive(Debug, Default)]
struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

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
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum UserEvent {
    WakeUp,
}

// fn main() -> Result<(), Box<dyn Error>> {
//     let mut event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
//     // let (sender, receiver) = mpsc::channel();

//     let event_loop_proxy = event_loop.create_proxy();
//     // let sender_for_proxy = sender.clone();
//     std::thread::spawn(move || {
//         // let _ = sender.send(Action::Message);
//         let _ = event_loop_proxy.send_event(UserEvent::WakeUp);
//         std::thread::sleep(std::time::Duration::from_secs(1));
//     });

//     let mut app = VelloApp {
//         idx: 1,
//         ..Default::default()
//     };
//     event_loop.run_app_on_demand(&mut app)?;
//     event_loop.run_app_on_demand(&mut app)?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    let mut app = VelloApp {
        idx: 1,
        ..Default::default()
    };
    let mut event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
