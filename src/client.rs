use graphics_device::graphics_device_client::GraphicsDeviceClient;
use graphics_device::{
    DrawCircleRequest, DrawLineRequest, Empty, ResizeWindowRequest, SetBackgroundRequest,
};

pub mod graphics_device {
    tonic::include_proto!("graphics_device");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GraphicsDeviceClient::connect("http://[::1]:50051").await?;

    let subcommand = std::env::args().nth(1).unwrap_or_default();
    println!("{subcommand}");
    let response = match subcommand.as_str() {
        "close" => {
            let request = tonic::Request::new(Empty {});
            client.close_window(request).await
        }
        "bg" => {
            let color = std::env::args().nth(2).unwrap_or_default();
            let request = tonic::Request::new(SetBackgroundRequest {
                color: color.parse().unwrap_or(1),
            });
            client.set_background(request).await
        }
        "circle" => {
            let cx: f64 = std::env::args()
                .nth(2)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);
            let cy: f64 = std::env::args()
                .nth(3)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);

            let vello::peniko::Color { r, g, b, a } = vello::peniko::Color::PURPLE;
            let fill_color = u32::from_ne_bytes([r, g, b, a]);
            let stroke_color = u32::from_ne_bytes([r, 0, 0, a]);
            let stroke_width = 10.0;

            let request = tonic::Request::new(DrawCircleRequest {
                cx,
                cy,
                radius: 100.0,
                fill_color,
                stroke_color,
                stroke_width,
            });
            client.draw_circle(request).await
        }
        "line" => {
            let x0: f64 = std::env::args()
                .nth(2)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);
            let y0: f64 = std::env::args()
                .nth(3)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);
            let x1 = x0 + 100.0;
            let y1 = y0 + 100.0;

            let vello::peniko::Color { r, g, b, a } = vello::peniko::Color::PURPLE;
            let stroke_color = u32::from_ne_bytes([r, g, b, a]);
            let stroke_width = 15.0;

            let request = tonic::Request::new(DrawLineRequest {
                x0,
                y0,
                x1,
                y1,
                stroke_color,
                stroke_width,
            });
            client.draw_line(request).await
        }
        _ => {
            let request = tonic::Request::new(ResizeWindowRequest {
                height: 100,
                width: 100,
            });
            client.resize_window(request).await
        }
    }?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
