// The code related to vello is based on the example code on vello's repo
// (examples/simple/main.rs).

use std::{num::NonZeroUsize, sync::Arc};

use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    AaConfig, DebugLayers, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
};

use tonic::{transport::Server, Request, Response, Status};

use graphics_device::graphics_device_server::{GraphicsDevice, GraphicsDeviceServer};
use graphics_device::{
    DrawCircleRequest, DrawLineRequest, Empty, ResizeWindowRequest, SetBackgroundRequest,
};

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
        println!("{:?}", request);

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

    async fn close_window(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        self.event_loop_proxy
            .send_event(UserEvent::CloseWindow)
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn set_background(
        &self,
        request: Request<SetBackgroundRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let SetBackgroundRequest { color } = request.get_ref();

        let color = match color {
            1 => Color::WHITE,
            2 => Color::RED,
            3 => Color::BLUE,
            4 => Color::GREEN,
            _ => Color::BLACK,
        };
        self.event_loop_proxy
            .send_event(UserEvent::SetBackground { color })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }
    async fn draw_circle(
        &self,
        request: Request<DrawCircleRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let DrawCircleRequest {
            cx,
            cy,
            radius,
            fill_color,
            stroke_color,
            stroke_width,
        } = request.get_ref();

        self.event_loop_proxy
            .send_event(UserEvent::DrawCircle {
                center: vello::kurbo::Point::new(*cx, *cy),
                radius: *radius,
                fill_color: *fill_color,
                stroke_color: *stroke_color,
                stroke_width: *stroke_width,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_line(
        &self,
        request: Request<DrawLineRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let DrawLineRequest {
            x0,
            y0,
            x1,
            y1,
            stroke_color,
            stroke_width,
        } = request.get_ref();

        self.event_loop_proxy
            .send_event(UserEvent::DrawLine {
                p0: vello::kurbo::Point::new(*x0, *y0),
                p1: vello::kurbo::Point::new(*x1, *y1),
                stroke_color: *stroke_color,
                stroke_width: *stroke_width,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }
}

pub struct ActiveRenderState<'a> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'a>,
    window: Arc<Window>,
}

enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended(Option<Arc<Window>>),
}

struct VelloApp<'a> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    state: RenderState<'a>,
    scene: Scene,

    background_color: Color, // TODO
}

impl<'a> ApplicationHandler<UserEvent> for VelloApp<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };
        let window = cached_window.take().unwrap_or_else(|| {
            let attr = Window::default_attributes()
                .with_title("test")
                .with_inner_size(winit::dpi::LogicalSize::new(600.0, 600.0));
            Arc::new(
                event_loop
                    .create_window(attr)
                    .expect("failed to create window"),
            )
        });

        let size = window.inner_size();
        let surface = pollster::block_on(self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            vello::wgpu::PresentMode::AutoVsync,
        ))
        .expect("failed to create surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.state = RenderState::Active(ActiveRenderState { window, surface });
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let RenderState::Active(state) = &self.state {
            self.state = RenderState::Suspended(Some(state.window.clone()));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let render_state = match &mut self.state {
            RenderState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                // TODO: can this always be executed immediately?
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                self.context
                    .resize_surface(&mut render_state.surface, size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                // self.scene.reset();

                let surface = &render_state.surface;
                let width = surface.config.width;
                let height = surface.config.height;

                let device_handle = &self.context.devices[surface.dev_id];

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                if let Some(renderer) = self.renderers[surface.dev_id].as_mut() {
                    renderer
                        .render_to_surface(
                            &device_handle.device,
                            &device_handle.queue,
                            &self.scene,
                            &surface_texture,
                            &vello::RenderParams {
                                base_color: self.background_color,
                                width,
                                height,
                                antialiasing_method: AaConfig::Msaa16,
                                debug: DebugLayers::none(),
                            },
                        )
                        .expect("failed to render");
                }

                surface_texture.present();
                device_handle.device.poll(vello::wgpu::Maintain::Poll);
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };

        match event {
            UserEvent::ResizeWindow { height, width } => {
                let sizes = render_state.window.inner_size();
                // TODO: handle error
                let _res = render_state
                    .window
                    .request_inner_size(winit::dpi::LogicalSize::new(
                        (sizes.width as i32 + height) as u32,
                        (sizes.height as i32 + width) as u32,
                    ));
            }
            UserEvent::CloseWindow => {
                event_loop.exit();
            }
            UserEvent::SetBackground { color } => {
                self.background_color = color;
                render_state.window.request_redraw();
            }
            UserEvent::DrawCircle {
                center,
                radius,
                fill_color,
                stroke_color,
                stroke_width,
            } => {
                let circle = vello::kurbo::Circle::new(center, radius);

                if fill_color != 0 {
                    let [r, g, b, a] = fill_color.to_ne_bytes();
                    self.scene.fill(
                        vello::peniko::Fill::NonZero,
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::rgba8(r, g, b, a),
                        None,
                        &circle,
                    );
                }

                if stroke_color != 0 && stroke_width > 0.0 {
                    let [r, g, b, a] = stroke_color.to_ne_bytes();
                    self.scene.stroke(
                        &vello::kurbo::Stroke::new(stroke_width),
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::rgba8(r, g, b, a),
                        None,
                        &circle,
                    );
                }

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
            UserEvent::DrawLine {
                p0,
                p1,
                stroke_color,
                stroke_width,
            } => {
                let line = vello::kurbo::Line::new(p0, p1);

                if stroke_color != 0 && stroke_width > 0.0 {
                    let [r, g, b, a] = stroke_color.to_ne_bytes();
                    self.scene.stroke(
                        &vello::kurbo::Stroke::new(stroke_width),
                        vello::kurbo::Affine::IDENTITY,
                        vello::peniko::Color::rgba8(r, g, b, a),
                        None,
                        &line,
                    );
                }

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
        };
    }
}

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    ResizeWindow {
        height: i32,
        width: i32,
    },
    CloseWindow,
    SetBackground {
        color: Color,
    },
    DrawCircle {
        center: vello::kurbo::Point,
        radius: f64,
        fill_color: u32,
        stroke_color: u32,
        stroke_width: f64,
    },
    DrawLine {
        p0: vello::kurbo::Point,
        p1: vello::kurbo::Point,
        stroke_color: u32,
        stroke_width: f64,
    },
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Couldn't create renderer")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = VelloApp {
        context: RenderContext::new(),
        renderers: vec![],
        state: RenderState::Suspended(None),
        scene: Scene::new(),
        background_color: Color::WHITE_SMOKE,
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
