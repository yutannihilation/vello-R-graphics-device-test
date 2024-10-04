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
            let request = tonic::Request::new(DrawCircleRequest {
                cx: 0.0,
                cy: 0.0,
                radius: 100.0,
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
