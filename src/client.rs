use graphics_device::graphics_device_client::GraphicsDeviceClient;
use graphics_device::{DrawCircleRequest, Empty, ResizeWindowRequest, SetBackgroundRequest};

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
            let cx: u32 = std::env::args()
                .nth(2)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100);
            let cy: u32 = std::env::args()
                .nth(3)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100);

            let a = (cx + cy) << 1;
            let r = (cx + cy) << 2;
            let g = (cx + cy) << 3;
            let b = (cx + cy) << 4;
            let color = (r << 24) | (g << 16) | (b << 8) | a as u32;

            let request = tonic::Request::new(DrawCircleRequest {
                cx: cx as _,
                cy: cy as _,
                radius: 100.0,
                color,
            });
            client.draw_circle(request).await
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
