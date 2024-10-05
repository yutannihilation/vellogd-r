// The code related to vello is based on
//
// - the example code on linbender/vello (examples/simple/main.rs).
// - the example code on linbender/parley (examples/vello_editor/src/main.rs).

mod utils;

use std::{num::NonZeroUsize, sync::Arc};

use utils::u32_to_color;
use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    AaConfig, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
};

use tonic::{transport::Server, Request, Response, Status};

use vellogd_protocol::graphics_device_server::{GraphicsDevice, GraphicsDeviceServer};
use vellogd_protocol::*;

#[derive(Debug)]
struct VelloGraphicsDevice {
    event_loop_proxy: EventLoopProxy<UserEvent>,
}

impl VelloGraphicsDevice {
    fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { event_loop_proxy }
    }
}

#[tonic::async_trait]
impl GraphicsDevice for VelloGraphicsDevice {
    async fn close_window(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        self.event_loop_proxy
            .send_event(UserEvent::CloseWindow)
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn new_page(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        self.event_loop_proxy
            .send_event(UserEvent::NewPage)
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
            stroke_params,
        } = request.into_inner();

        let fill_params = fill_color.map(FillParams::from_request);
        let stroke_params = stroke_params.map(StrokeParams::from_request);

        self.event_loop_proxy
            .send_event(UserEvent::DrawCircle {
                center: vello::kurbo::Point::new(cx, cy),
                radius,
                fill_params,
                stroke_params,
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
            stroke_params,
        } = request.into_inner();

        let stroke_params = stroke_params.ok_or_else(|| {
            Status::new(
                tonic::Code::InvalidArgument,
                "stroke_params must be specified",
            )
        })?;

        let stroke_params = StrokeParams::from_request(stroke_params);

        self.event_loop_proxy
            .send_event(UserEvent::DrawLine {
                p0: vello::kurbo::Point::new(x0, y0),
                p1: vello::kurbo::Point::new(x1, y1),
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_polyline(
        &self,
        request: Request<DrawPolylineRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let DrawPolylineRequest {
            x,
            y,
            stroke_params,
        } = request.into_inner();

        let stroke_params = stroke_params.ok_or_else(|| {
            Status::new(
                tonic::Code::InvalidArgument,
                "stroke_params must be specified",
            )
        })?;

        let path = utils::xy_to_path(x, y, false);

        let stroke_params = StrokeParams::from_request(stroke_params);

        self.event_loop_proxy
            .send_event(UserEvent::DrawPolyline {
                path,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_polygon(
        &self,
        request: Request<DrawPolygonRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let DrawPolygonRequest {
            x,
            y,
            fill_color,
            stroke_params,
        } = request.into_inner();

        let fill_params = fill_color.map(FillParams::from_request);
        let stroke_params = stroke_params.map(StrokeParams::from_request);
        let path = utils::xy_to_path(x, y, true);

        self.event_loop_proxy
            .send_event(UserEvent::DrawPolygon {
                path,
                fill_params,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_text(
        &self,
        request: Request<DrawTextRequest>,
    ) -> Result<Response<Empty>, Status> {
        println!("{:?}", request);

        let DrawTextRequest {
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
        } = request.into_inner();

        self.event_loop_proxy
            .send_event(UserEvent::DrawText {
                pos: vello::kurbo::Point::new(x, y),
                text,
                color: u32_to_color(color),
                size,
                lineheight,
                family,
                angle,
                hadj,
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
    background_color: Color,
    font_ctx: parley::FontContext,
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
            UserEvent::CloseWindow => {
                event_loop.exit();
            }
            UserEvent::NewPage => {
                self.scene.reset();
                render_state.window.request_redraw();
            }
            UserEvent::DrawCircle {
                center,
                radius,
                fill_params,
                stroke_params,
            } => {
                let circle = vello::kurbo::Circle::new(center, radius);

                if let Some(fill_params) = fill_params {
                    self.scene.fill(
                        vello::peniko::Fill::NonZero,
                        vello::kurbo::Affine::IDENTITY,
                        fill_params.color,
                        None,
                        &circle,
                    );
                }

                if let Some(stroke_params) = stroke_params {
                    self.scene.stroke(
                        &stroke_params.stroke,
                        vello::kurbo::Affine::IDENTITY,
                        stroke_params.color,
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
                stroke_params,
            } => {
                let line = vello::kurbo::Line::new(p0, p1);

                self.scene.stroke(
                    &stroke_params.stroke,
                    vello::kurbo::Affine::IDENTITY,
                    stroke_params.color,
                    None,
                    &line,
                );

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
            UserEvent::DrawPolyline {
                path,
                stroke_params,
            } => {
                self.scene.stroke(
                    &stroke_params.stroke,
                    vello::kurbo::Affine::IDENTITY,
                    stroke_params.color,
                    None,
                    &path,
                );

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
            UserEvent::DrawPolygon {
                path,
                fill_params,
                stroke_params,
            } => {
                if let Some(fill_params) = fill_params {
                    self.scene.fill(
                        vello::peniko::Fill::NonZero,
                        vello::kurbo::Affine::IDENTITY,
                        fill_params.color,
                        None,
                        &path,
                    );
                }

                if let Some(stroke_params) = stroke_params {
                    self.scene.stroke(
                        &stroke_params.stroke,
                        vello::kurbo::Affine::IDENTITY,
                        stroke_params.color,
                        None,
                        &path,
                    );
                }

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
            UserEvent::DrawText {
                pos,
                text,
                color,
                size,
                lineheight,
                family,
                angle,
                hadj,
            } => {
                // Note: parley is probably a little bit overkill, but it seems
                // this is the only interface.
                let mut layout_ctx: parley::LayoutContext<vello::peniko::Brush> =
                    parley::LayoutContext::new();
                let mut layout_builder =
                    layout_ctx.ranged_builder(&mut self.font_ctx, text.as_str(), 1.0); // TODO: should scale be configurable?
                layout_builder.push_default(&parley::StyleProperty::FontSize(size));
                layout_builder.push_default(&parley::StyleProperty::LineHeight(lineheight));
                layout_builder.push_default(&parley::StyleProperty::FontStack(
                    parley::FontStack::Source("system-iu"), // TODO: specify family
                ));
                // TODO: use build_into() to reuse a Layout?
                let mut layout = layout_builder.build(text.as_str());
                layout.break_all_lines(None); // It seems this is mandatory, otherwise no text is drawn. Why?
                layout.align(None, parley::Alignment::Start);

                let width = layout.width();
                let transform = vello::kurbo::Affine::translate((-(width * hadj) as f64, 0.0))
                    .then_rotate(-angle as f64)
                    .then_translate((pos.x, pos.y).into());

                for line in layout.lines() {
                    let vadj = line.metrics().ascent * 0.5;
                    for item in line.items() {
                        // ignore inline box
                        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };

                        let mut x = glyph_run.offset();
                        let y = glyph_run.baseline() - vadj;
                        let run = glyph_run.run();

                        let font = run.font();
                        let font_size = run.font_size();

                        // TODO:  It seems this is to handle italic. Is this necessary?
                        //
                        // https://github.com/linebender/parley/blob/be9e9ab3fc3fe92b3887048d5123c963cffac3d5/examples/vello_editor/src/text.rs#L364-L366
                        // https://docs.rs/kurbo/latest/kurbo/struct.Affine.html#method.skew
                        //
                        // let glyph_xform = run.synthesis().skew().map(|angle| {
                        //     vello::kurbo::Affine::skew(angle.to_radians().tan() as f64, 0.0)
                        // });

                        let coords = run
                            .normalized_coords()
                            .iter()
                            .map(|coord| {
                                vello::skrifa::instance::NormalizedCoord::from_bits(*coord)
                            })
                            .collect::<Vec<_>>();

                        // TODO: vello and parley uses different versions of font
                        let font = {
                            let raw = font.clone().data.into_raw_parts();
                            let data = vello::peniko::Blob::from_raw_parts(raw.0, raw.1);
                            vello::peniko::Font::new(data, font.index)
                        };

                        self.scene
                            .draw_glyphs(&font)
                            .brush(color)
                            .transform(transform)
                            .font_size(font_size)
                            .normalized_coords(&coords)
                            .draw(
                                vello::peniko::Fill::NonZero,
                                glyph_run.glyphs().map(|g| {
                                    let gx = x + g.x;
                                    let gy = y - g.y;
                                    x += g.advance;
                                    vello::Glyph {
                                        id: g.id as _,
                                        x: gx,
                                        y: gy,
                                    }
                                }),
                            );
                    }
                }

                // TODO: set a flag and redraw lazily
                render_state.window.request_redraw();
            }
        };
    }
}

#[derive(Debug, Clone)]
struct FillParams {
    color: vello::peniko::Color,
}

#[derive(Debug, Clone)]
struct StrokeParams {
    color: vello::peniko::Color,
    stroke: vello::kurbo::Stroke,
}

#[derive(Debug, Clone)]
enum UserEvent {
    CloseWindow,
    NewPage,
    DrawCircle {
        center: vello::kurbo::Point,
        radius: f64,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawLine {
        p0: vello::kurbo::Point,
        p1: vello::kurbo::Point,
        stroke_params: StrokeParams,
    },
    DrawPolyline {
        path: vello::kurbo::BezPath,
        stroke_params: StrokeParams,
    },
    DrawPolygon {
        path: vello::kurbo::BezPath,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawText {
        pos: vello::kurbo::Point,
        text: String,
        color: vello::peniko::Color,
        size: f32,
        lineheight: f32,
        // TODO
        // face
        family: String,
        angle: f32,
        hadj: f32,
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
        font_ctx: parley::FontContext::new(),
    };

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();

    let addr = "[::1]:50051".parse()?;
    let greeter = VelloGraphicsDevice::new(event_loop_proxy);

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
