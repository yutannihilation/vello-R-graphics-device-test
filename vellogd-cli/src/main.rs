use vellogd_protocol::graphics_device_client::GraphicsDeviceClient;
use vellogd_protocol::*;

const MEDIUM_PURPLE: u32 = u32::from_ne_bytes([147, 112, 219, 255]);
const PALE_GREEN: u32 = u32::from_ne_bytes([152, 251, 152, 255]);

use clap::{Parser, Subcommand};

fn hex_color_to_u32<T: AsRef<str>>(x: T) -> u32 {
    let x = x.as_ref();
    let x_parsed = u32::from_str_radix(x, 16).unwrap();

    match x.len() {
        4 => {
            let a = x_parsed & 0xf;
            let b = (x_parsed >> 4) & 0xf;
            let g = (x_parsed >> 8) & 0xf;
            let r = (x_parsed >> 12) & 0xf;
            r + (r << 4) + (g << 8) + (g << 12) + (b << 16) + (b << 20) + (a << 24) + (a << 28)
        }
        3 => {
            let b = x_parsed & 0xf;
            let g = (x_parsed >> 4) & 0xf;
            let r = (x_parsed >> 8) & 0xf;
            r + (r << 4) + (g << 8) + (g << 12) + (b << 16) + (b << 20) + 0xff000000_u32
        }
        _ => panic!("invalid color format"),
    }
}

/// A CLI to debug vellogd-server
#[derive(Debug, Parser)] // requires `derive` feature
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    Close {},

    #[command()]
    Circle {
        #[arg()]
        cx: f64,
        #[arg()]
        cy: f64,
        #[arg(long, short, default_value_t = 50.0)]
        radius: f64,
        #[arg(long, short, default_value_t = 8.0)]
        width: f64,
        #[arg(long, short, default_value = "999")]
        fill: String,
        #[arg(long, short, default_value = "000")]
        color: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("{args:?}");

    let mut client = GraphicsDeviceClient::connect("http://[::1]:50051").await?;

    let response = match args.command {
        Commands::Close {} => {
            let request = tonic::Request::new(Empty {});
            client.close_window(request).await
        }

        Commands::Circle {
            cx,
            cy,
            radius,
            width,
            fill,
            color,
        } => {
            let fill_color = hex_color_to_u32(fill);
            let stroke_color = hex_color_to_u32(color);
            let stroke_params = StrokeParameters {
                color: stroke_color,
                width,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            };

            let request = tonic::Request::new(DrawCircleRequest {
                cx,
                cy,
                radius,
                fill_color: Some(fill_color),
                stroke_params: Some(stroke_params),
            });
            client.draw_circle(request).await
        }
    }?;

    println!("RESPONSE={:?}", response);

    return Ok(());

    let subcommand = std::env::args().nth(1).unwrap_or_default();
    println!("{subcommand}");
    let response_ = match subcommand.as_str() {
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
