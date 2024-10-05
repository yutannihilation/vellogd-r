use vellogd_protocol::graphics_device_client::GraphicsDeviceClient;
use vellogd_protocol::*;

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
    Clear {},

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

    #[command()]
    Line {
        #[arg()]
        x0: f64,
        #[arg()]
        y0: f64,
        #[arg()]
        x1: f64,
        #[arg()]
        y1: f64,
        #[arg(long, short, default_value_t = 8.0)]
        width: f64,
        #[arg(long, short, default_value = "000")]
        color: String,
    },

    #[command()]
    Lines {
        #[arg()]
        pos: Vec<f64>,
        #[arg(long, short, default_value_t = 8.0)]
        width: f64,
        #[arg(long, short, default_value = "000")]
        color: String,
    },

    #[command()]
    Polygon {
        #[arg()]
        pos: Vec<f64>,
        #[arg(long, short, default_value_t = 8.0)]
        width: f64,
        #[arg(long, short, default_value = "999")]
        fill: String,
        #[arg(long, short, default_value = "000")]
        color: String,
    },

    #[command()]
    Text {
        #[arg()]
        x: f64,
        #[arg()]
        y: f64,
        #[arg(default_value = "🌶")] // to test emoji (cannot input from Powershell)
        text: String,
        #[arg(long, short, default_value = "000")]
        color: String,
        #[arg(long, short, default_value_t = 50.0)]
        size: f32,
        #[arg(long, default_value_t = 1.0)]
        lineheight: f32,
        #[arg(long, default_value_t = 1)]
        face: u32,
        #[arg(long, default_value = "Arial")]
        family: String,
        /// Angle in degree (translated to radian internally)
        #[arg(long, default_value_t = 0.0)]
        angle: f32,
        #[arg(long, default_value_t = 0.0)]
        hadj: f32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("{args:?}");

    let mut client = GraphicsDeviceClient::connect("http://[::1]:50051").await?;

    let response = match args.command {
        Commands::Close {} => client.close_window(Empty {}).await,
        Commands::Clear {} => client.new_page(Empty {}).await,

        Commands::Circle {
            cx,
            cy,
            radius,
            width,
            fill,
            color,
        } => {
            let fill_color = hex_color_to_u32(fill);
            let fill_color = if fill_color != 0 {
                Some(fill_color)
            } else {
                None
            };

            let stroke_color = hex_color_to_u32(color);
            let stroke_params = if stroke_color != 0 {
                Some(StrokeParameters {
                    color: stroke_color,
                    width,
                    linetype: 1,
                    join: 1,
                    miter_limit: 1.0,
                    cap: 1,
                })
            } else {
                None
            };

            let request = tonic::Request::new(DrawCircleRequest {
                cx,
                cy,
                radius,
                fill_color,
                stroke_params,
            });
            client.draw_circle(request).await
        }

        Commands::Line {
            x0,
            y0,
            x1,
            y1,
            width,
            color,
        } => {
            let color = hex_color_to_u32(color);
            let stroke_params = if color != 0 {
                Some(StrokeParameters {
                    color,
                    width,
                    linetype: 1,
                    join: 1,
                    miter_limit: 1.0,
                    cap: 1,
                })
            } else {
                None
            };

            let request = tonic::Request::new(DrawLineRequest {
                x0,
                y0,
                x1,
                y1,
                stroke_params,
            });
            client.draw_line(request).await
        }

        Commands::Lines { pos, width, color } => {
            if pos.len() < 4 || pos.len() % 2 != 0 {
                panic!("invalid number of arguments; the locations need to be pairs of X and Y");
            }

            let mut x = Vec::new();
            let mut y = Vec::new();

            for p in pos.chunks_exact(2) {
                x.push(p[0]);
                y.push(p[1]);
            }

            let color = hex_color_to_u32(color);
            let stroke_params = if color != 0 {
                Some(StrokeParameters {
                    color,
                    width,
                    linetype: 1,
                    join: 1,
                    miter_limit: 1.0,
                    cap: 1,
                })
            } else {
                None
            };

            let request = tonic::Request::new(DrawPolylineRequest {
                x,
                y,
                stroke_params,
            });
            client.draw_polyline(request).await
        }

        Commands::Polygon {
            pos,
            width,
            fill,
            color,
        } => {
            if pos.len() < 6 || pos.len() % 2 != 0 {
                panic!("invalid number of arguments; the locations need to be pairs of X and Y");
            }

            let mut x = Vec::new();
            let mut y = Vec::new();

            for p in pos.chunks_exact(2) {
                x.push(p[0]);
                y.push(p[1]);
            }

            let fill_color = hex_color_to_u32(fill);
            let fill_color = if fill_color != 0 {
                Some(fill_color)
            } else {
                None
            };

            let color = hex_color_to_u32(color);
            let stroke_params = if color != 0 {
                Some(StrokeParameters {
                    color,
                    width,
                    linetype: 1,
                    join: 1,
                    miter_limit: 1.0,
                    cap: 1,
                })
            } else {
                None
            };

            let request = tonic::Request::new(DrawPolygonRequest {
                x,
                y,
                fill_color,
                stroke_params,
            });
            client.draw_polygon(request).await
        }

        Commands::Text {
            x,
            y,
            text,
            color,
            size,
            lineheight,
            face,
            family,
            angle,
            hadj,
        } => {
            let color = hex_color_to_u32(color);
            let request = tonic::Request::new(DrawTextRequest {
                x,
                y,
                text,
                color,
                size,
                lineheight,
                face,
                family,
                angle: angle.to_radians(),
                hadj,
            });
            client.draw_text(request).await
        }
    }?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
