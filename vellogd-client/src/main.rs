use vellogd_protocol::graphics_device_client::GraphicsDeviceClient;
use vellogd_protocol::*;

const MEDIUM_PURPLE: u32 = u32::from_ne_bytes([147, 112, 219, 255]);
const PALE_GREEN: u32 = u32::from_ne_bytes([152, 251, 152, 255]);

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

            let fill_color = MEDIUM_PURPLE;

            let stroke_color = PALE_GREEN;
            let stroke_width = 10.0;
            let stroke_params = StrokeParameters {
                color: stroke_color,
                width: stroke_width,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            };

            let request = tonic::Request::new(DrawCircleRequest {
                cx,
                cy,
                radius: 100.0,
                fill_color: Some(fill_color),
                stroke_params: Some(stroke_params),
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

            let color = MEDIUM_PURPLE;
            let width = 15.0;
            let stroke_params = StrokeParameters {
                color,
                width,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            };

            let request = tonic::Request::new(DrawLineRequest {
                x0,
                y0,
                x1,
                y1,
                stroke_params: Some(stroke_params),
            });
            client.draw_line(request).await
        }
        "lines" => {
            let color = MEDIUM_PURPLE;
            let width = 15.0;
            let stroke_params = StrokeParameters {
                color,
                width,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            };

            let request = tonic::Request::new(DrawPolylineRequest {
                x: vec![100.0, 300.0, 500.0],
                y: vec![100.0, 500.0, 300.0],
                stroke_params: Some(stroke_params),
            });
            client.draw_polyline(request).await
        }
        "polygon" => {
            let color = MEDIUM_PURPLE;
            let width = 15.0;
            let stroke_params = StrokeParameters {
                color,
                width,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            };
            let request = tonic::Request::new(DrawPolygonRequest {
                x: vec![100.0, 300.0, 500.0],
                y: vec![100.0, 500.0, 300.0],
                fill_color: Some(PALE_GREEN),
                stroke_params: Some(stroke_params),
            });
            client.draw_polygon(request).await
        }
        "text" => {
            let x: f64 = std::env::args()
                .nth(2)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);
            let y: f64 = std::env::args()
                .nth(3)
                .unwrap_or_default()
                .parse()
                .unwrap_or(100.0);
            let text = std::env::args().nth(4).unwrap_or("🥷".into());

            let color = u32::from_ne_bytes([147, 112, 219, 255]);
            let request = tonic::Request::new(DrawTextRequest {
                x,
                y,
                text,
                color,
                size: 100.0,
                lineheight: 1.2,
                face: 1,
                family: "Arial".into(),
                angle: 30.0_f32.to_radians(),
                hadj: 0.0,
            });
            client.draw_text(request).await
        }
        _ => client.new_page(Empty {}).await,
    }?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
