// The code related to vello is based on
//
// - the example code on linbender/vello (examples/simple/main.rs).
// - the example code on linbender/parley (examples/vello_editor/src/main.rs).

use std::{
    num::NonZeroUsize,
    sync::{Arc, LazyLock, Mutex},
};

use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    AaConfig, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

use crate::UserEvent;

pub struct ActiveRenderState<'a> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'a>,
    window: Arc<Window>,
}

pub enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended(Option<Arc<Window>>),
}

pub struct VelloApp<'a> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    state: RenderState<'a>,
    scene: Scene,
    background_color: Color,

    // Since R's graphics device is left-bottom origin, the Y value needs to be
    // flipped
    y_transform: vello::kurbo::Affine,

    window_title: String,
    width: f32,
    height: f32,
    needs_redraw: bool,
}

impl<'a> VelloApp<'a> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended(None),
            scene: Scene::new(),
            background_color: Color::WHITE_SMOKE,
            y_transform: calc_y_translate(height),
            window_title: "vellogd".to_string(),
            width,
            height,
            needs_redraw: true,
        }
    }
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

#[cfg(target_os = "windows")]
fn add_platform_specific_window_attributes(attrs: WindowAttributes) -> WindowAttributes {
    use winit::platform::windows::CornerPreference;
    use winit::platform::windows::WindowAttributesExtWindows;

    // square corner
    attrs.with_corner_preference(CornerPreference::DoNotRound)
}

#[cfg(target_os = "macos")]
fn add_platform_specific_window_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs
}

#[cfg(target_os = "linux")]
fn add_platform_specific_window_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs
}

impl<'a> ApplicationHandler<UserEvent> for VelloApp<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };
        let window = cached_window.take().unwrap_or_else(|| {
            let attrs_basic = Window::default_attributes()
                .with_title(&self.window_title)
                .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
            let attrs = add_platform_specific_window_attributes(attrs_basic);

            Arc::new(
                event_loop
                    .create_window(attrs)
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
                self.width = size.width as _;
                self.height = size.height as _;
                self.y_transform = calc_y_translate(self.width);
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

                    // surface is up-to-date
                    self.needs_redraw = false;
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
            UserEvent::RedrawWindow => {
                if self.needs_redraw {
                    render_state.window.request_redraw();
                }
            }
            UserEvent::CloseWindow => {
                event_loop.exit();
            }
            UserEvent::NewPage => {
                self.scene.reset();
                self.needs_redraw = true;
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
                        self.y_transform,
                        fill_params.color,
                        None,
                        &circle,
                    );
                }

                if let Some(stroke_params) = stroke_params {
                    self.scene.stroke(
                        &stroke_params.stroke,
                        self.y_transform,
                        stroke_params.color,
                        None,
                        &circle,
                    );
                }

                self.needs_redraw = true;
            }
            UserEvent::DrawLine {
                p0,
                p1,
                stroke_params,
            } => {
                let line = vello::kurbo::Line::new(p0, p1);

                self.scene.stroke(
                    &stroke_params.stroke,
                    self.y_transform,
                    stroke_params.color,
                    None,
                    &line,
                );

                self.needs_redraw = true;
            }
            UserEvent::DrawPolyline {
                path,
                stroke_params,
            } => {
                self.scene.stroke(
                    &stroke_params.stroke,
                    self.y_transform,
                    stroke_params.color,
                    None,
                    &path,
                );

                self.needs_redraw = true;
            }
            UserEvent::DrawPolygon {
                path,
                fill_params,
                stroke_params,
            } => {
                if let Some(fill_params) = fill_params {
                    self.scene.fill(
                        vello::peniko::Fill::NonZero,
                        self.y_transform,
                        fill_params.color,
                        None,
                        &path,
                    );
                }

                if let Some(stroke_params) = stroke_params {
                    self.scene.stroke(
                        &stroke_params.stroke,
                        self.y_transform,
                        stroke_params.color,
                        None,
                        &path,
                    );
                }

                self.needs_redraw = true;
            }

            UserEvent::DrawRect {
                p0,
                p1,
                fill_params,
                stroke_params,
            } => {
                let rect = vello::kurbo::Rect::new(p0.x, p0.y, p1.x, p1.y);
                if let Some(fill_params) = fill_params {
                    self.scene.fill(
                        vello::peniko::Fill::NonZero,
                        self.y_transform,
                        fill_params.color,
                        None,
                        &rect,
                    );
                }

                if let Some(stroke_params) = stroke_params {
                    self.scene.stroke(
                        &stroke_params.stroke,
                        self.y_transform,
                        stroke_params.color,
                        None,
                        &rect,
                    );
                }

                self.needs_redraw = true;
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
                let layout = build_layout(text, size, lineheight);

                let width = layout.width();
                let transform = vello::kurbo::Affine::translate((-(width * hadj) as f64, 0.0))
                    .then_rotate(-angle as f64)
                    .then_translate((pos.x, self.height as f64 - pos.y).into()); // Y-axis is flipped

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
                                        y: -gy, // Y-axis is flipped
                                    }
                                }),
                            );
                    }
                }

                self.needs_redraw = true;
            }
        };
    }
}

static FONT_CTX: LazyLock<Mutex<parley::FontContext>> =
    LazyLock::new(|| Mutex::new(parley::FontContext::new()));

pub fn build_layout(
    text: impl AsRef<str>,
    // TODO
    // family: String,
    // face: i32,
    size: f32,
    lineheight: f32,
) -> parley::Layout<vello::peniko::Brush> {
    let text = text.as_ref();
    let mut font_ctx = FONT_CTX.lock().unwrap();
    // Note: parley is probably a little bit overkill, but it seems
    // this is the only interface.
    let mut layout_ctx: parley::LayoutContext<vello::peniko::Brush> = parley::LayoutContext::new();
    let mut layout_builder = layout_ctx.ranged_builder(&mut font_ctx, text, 1.0);
    // TODO: should scale be configurable?
    layout_builder.push_default(&parley::StyleProperty::FontSize(size));
    layout_builder.push_default(&parley::StyleProperty::LineHeight(lineheight));
    layout_builder.push_default(&parley::StyleProperty::FontStack(
        parley::FontStack::Source("system-iu"), // TODO: specify family
    ));
    // TODO: use build_into() to reuse a Layout?
    let mut layout = layout_builder.build(text);
    layout.break_all_lines(None);
    // It seems this is mandatory, otherwise no text is drawn. Why?
    layout.align(None, parley::Alignment::Start);
    layout
}

pub fn calc_y_translate(h: f32) -> vello::kurbo::Affine {
    vello::kurbo::Affine::FLIP_Y.then_translate(vello::kurbo::Vec2 { x: 0.0, y: h as _ })
}
