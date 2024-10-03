use graphics_device::graphics_device_client::GraphicsDeviceClient;
use graphics_device::ResizeWindowRequest;

pub mod graphics_device {
    tonic::include_proto!("graphics_device");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GraphicsDeviceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ResizeWindowRequest {
        height: 100,
        width: 100,
    });

    let response = client.resize_window(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
