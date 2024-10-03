use graphics_device::graphics_device_client::GraphicsDeviceClient;
use graphics_device::{Empty, ResizeWindowRequest};

pub mod graphics_device {
    tonic::include_proto!("graphics_device");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GraphicsDeviceClient::connect("http://[::1]:50051").await?;

    let args = std::env::args().nth(1).unwrap_or_default();
    println!("{args}");
    let response = match args.as_str() {
        "close" => {
            let request = tonic::Request::new(Empty {});
            client.close_window(request).await
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
