// The code related to vello is based on
//
// - the example code on linbender/vello (examples/simple/main.rs).
// - the example code on linbender/parley (examples/vello_editor/src/main.rs).

mod wgpu_util;

use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, LazyLock, Mutex,
    },
};

use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    AaConfig, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes},
};

use crate::{
    protocol::{AppResponseRelay, FillParams, GlyphParams, Request, Response, StrokeParams},
    text_layouter::{fontface_to_weight_and_style, TextLayouter},
};

pub struct ActiveRenderState<'a> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'a>,
    window: Arc<Window>,
}

pub enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    Suspended(Option<Arc<Window>>),
}

#[derive(Debug)]
pub enum FillPattern {
    Gradient(peniko::Gradient),
    Tiling, // TODO
}

#[derive(Clone)]
pub struct SceneDrawer {
    inner: Arc<Mutex<Scene>>,
    // This is a bit tricky. Scene doesn't need to know the window size, but,
    // since R requires a flipped Y-axis, SceneDrawer needs to know how to flip,
    // at least.
    //
    // One more tricky thing is that, this cannot be specified as the transform
    // of the layer. The positions definitely need to be flipped, but, the drawn
    // items (e.g. glyph) are not.
    y_transform: Arc<Mutex<vello::kurbo::Affine>>,
    window_height: Arc<AtomicU32>,

    active_pattern: Arc<Mutex<Option<FillPattern>>>,

    needs_redraw: Arc<AtomicBool>,
}

impl SceneDrawer {
    pub fn new(
        y_transform: Arc<Mutex<vello::kurbo::Affine>>,
        window_height: Arc<AtomicU32>,
        needs_redraw: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Scene::new())),
            y_transform,
            window_height,
            active_pattern: Arc::new(Mutex::new(None)),
            needs_redraw,
        }
    }

    pub fn reset(&mut self) {
        self.inner.lock().unwrap().reset();
    }

    pub fn scene(&self) -> std::sync::MutexGuard<'_, Scene> {
        self.inner.lock().unwrap()
    }

    fn draw_stroke_inner(
        &self,
        stroke: &kurbo::Stroke,
        color: peniko::Color,
        shape: &impl kurbo::Shape,
    ) {
        let scene = &mut self.inner.lock().unwrap();
        let y_transform = *self.y_transform.lock().unwrap();
        scene.stroke(stroke, y_transform, color, None, shape);
    }

    fn draw_fill_inner(
        &self,
        fill_rule: peniko::Fill,
        color: peniko::Color,
        shape: &impl kurbo::Shape,
    ) {
        let scene = &mut self.inner.lock().unwrap();
        let y_transform = *self.y_transform.lock().unwrap();
        let fill_pattern = self.active_pattern.lock().unwrap();
        let brush: peniko::BrushRef = match fill_pattern.as_ref() {
            Some(ptn) => match ptn {
                FillPattern::Gradient(gradient) => gradient.into(),
                FillPattern::Tiling => todo!(),
            },
            None => color.into(),
        };
        scene.fill(fill_rule, y_transform, brush, None, shape);
    }

    pub fn draw_circle(
        &self,
        center: kurbo::Point,
        radius: f64,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        let circle = vello::kurbo::Circle::new(center, radius);

        if let Some(fill_params) = fill_params {
            self.draw_fill_inner(peniko::Fill::NonZero, fill_params.color, &circle);
        }

        if let Some(stroke_params) = stroke_params {
            self.draw_stroke_inner(&stroke_params.stroke, stroke_params.color, &circle);
        }

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_line(&self, p0: kurbo::Point, p1: kurbo::Point, stroke_params: StrokeParams) {
        let line = vello::kurbo::Line::new(p0, p1);
        self.draw_stroke_inner(&stroke_params.stroke, stroke_params.color, &line);
        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_polyline(&self, path: kurbo::BezPath, stroke_params: StrokeParams) {
        self.draw_stroke_inner(&stroke_params.stroke, stroke_params.color, &path);
        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_polygon(
        &self,
        path: kurbo::BezPath,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        if let Some(fill_params) = fill_params {
            let style = if fill_params.use_nonzero_rule {
                peniko::Fill::NonZero
            } else {
                peniko::Fill::EvenOdd
            };
            self.draw_fill_inner(style, fill_params.color, &path);
        }

        if let Some(stroke_params) = stroke_params {
            self.draw_stroke_inner(&stroke_params.stroke, stroke_params.color, &path);
        }

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_rect(
        &self,
        p0: kurbo::Point,
        p1: kurbo::Point,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        let rect = vello::kurbo::Rect::new(p0.x, p0.y, p1.x, p1.y);

        if let Some(fill_params) = fill_params {
            self.draw_fill_inner(peniko::Fill::NonZero, fill_params.color, &rect);
        }

        if let Some(stroke_params) = stroke_params {
            self.draw_stroke_inner(&stroke_params.stroke, stroke_params.color, &rect);
        }

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_raster(
        &self,
        image: &peniko::Image,
        scale: (f64, f64),
        pos: kurbo::Vec2, // top left corner
        angle: f64,
        with_extended_edge: bool,
    ) {
        let transform = kurbo::Affine::scale_non_uniform(scale.0, scale.1)
            .then_translate(pos)
            .then_rotate(-angle.to_radians());
        let scene = &mut self.inner.lock().unwrap();

        let (brush_transform, width, height) = if with_extended_edge {
            // draw largely and clip the edge
            (
                Some(kurbo::Affine::translate((0.5, 0.5))),
                image.width as f64 - 1.0,
                image.height as f64 - 1.0,
            )
        } else {
            (None, image.width as f64, image.height as f64)
        };

        scene.fill(
            peniko::Fill::NonZero,
            transform,
            image,
            brush_transform,
            &kurbo::Rect::new(0.0, 0.0, width, height),
        );

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_glyph(
        &self,
        glyph_run: parley::GlyphRun<peniko::Brush>,
        color: peniko::Color,
        transform: kurbo::Affine,
    ) {
        let scene = &mut self.inner.lock().unwrap();

        let mut x = glyph_run.offset();
        let y = 0.0;
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
            .map(|coord| vello::skrifa::instance::NormalizedCoord::from_bits(*coord))
            .collect::<Vec<_>>();

        scene
            .draw_glyphs(font)
            .brush(color)
            .transform(transform)
            .font_size(font_size)
            .normalized_coords(&coords)
            .draw(
                peniko::Fill::NonZero,
                glyph_run.glyphs().map(|g| {
                    let gx = x + g.x;
                    let gy = y + g.y;
                    x += g.advance;
                    vello::Glyph {
                        id: g.id as _,
                        x: gx,
                        y: gy,
                    }
                }),
            );

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn draw_glyph_raw(
        &self,
        glyph_ids: &[u32],
        x: &[f64],
        y: &[f64],
        glyph_params: GlyphParams,
    ) {
        let scene = &mut self.inner.lock().unwrap();
        let window_height = self.window_height.load(Ordering::Relaxed) as f32;

        let glyphs = x
            .iter()
            .zip(y)
            .zip(glyph_ids)
            .map(|((x, y), id)| vello::Glyph {
                id: *id,
                x: *x as f32,
                y: window_height - *y as f32,
            });

        let transform = kurbo::Affine::rotate(-glyph_params.angle);

        let font = glyph_params.font().unwrap(); // TODO: handle error

        scene
            .draw_glyphs(&font)
            .brush(glyph_params.color)
            .transform(transform)
            .font_size(glyph_params.size)
            .draw(peniko::Fill::NonZero, glyphs);

        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn push_clip(&self, p0: kurbo::Point, p1: kurbo::Point) {
        let scene = &mut self.inner.lock().unwrap();
        let y_transform = *self.y_transform.lock().unwrap();

        // R's graphics device always replaces the clipping strategy (really?)
        scene.pop_layer();

        scene.push_layer(
            peniko::Mix::Clip,
            1.0,
            y_transform,
            &kurbo::Rect::new(p0.x, p0.y, p1.x, p1.y),
        );
    }

    pub fn pop_clip(&self) {
        let scene = &mut self.inner.lock().unwrap();
        scene.pop_layer();
    }

    pub fn set_pattern(&self, pattern: FillPattern) {
        self.active_pattern.lock().unwrap().replace(pattern);
    }

    pub fn release_pattern(&self) {
        self.active_pattern.lock().unwrap().take();
    }
}

// Note: I'm hoping to use no copy here. However, this raster might
//    be drawn after the raster() Graphics API call. There's no
//    guarantee that this still exists on R's memory at the time.
//    So, this needs to be kept on Rust's memory.
pub fn convert_to_image(
    raster: &[u8],
    width: usize,
    height: usize,
    alpha: u8,
    with_extended_edge: bool,
) -> peniko::Image {
    let (raster_owned, width, height) = if !with_extended_edge {
        (raster.to_vec(), width as u32, height as u32)
    } else {
        let extended_width = width + 1;
        let extended_height = height + 1;
        let mut raster_owned = Vec::with_capacity(extended_width * extended_height);
        for (i, row) in raster.chunks(width * 4).enumerate() {
            raster_owned.extend_from_slice(row);
            // copy the last pixel
            let last_pixel = &row[(width * 4 - 4)..(width * 4)];
            raster_owned.extend_from_slice(last_pixel);
            // fill the last line
            if i == height - 1 {
                raster_owned.extend_from_slice(row);
                raster_owned.extend_from_slice(last_pixel);
            }
        }
        (raster_owned, extended_width as u32, extended_height as u32)
    };

    let raster_blob = peniko::Blob::new(Arc::new(raster_owned));
    peniko::Image {
        data: raster_blob,
        format: peniko::Format::Rgba8,
        width,
        height,
        extend: peniko::Extend::Pad,
        alpha,
    }
}

pub struct VelloApp<'a, T: AppResponseRelay> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    state: RenderState<'a>,
    scene: SceneDrawer,
    needs_redraw: Arc<AtomicBool>,
    width: Arc<AtomicU32>,
    height: Arc<AtomicU32>,
    y_transform: Arc<Mutex<vello::kurbo::Affine>>,
    base_color: Arc<AtomicU32>,
    layout: parley::Layout<peniko::Brush>,
    tx: T,

    window_title: String,
}

impl<'a, T: AppResponseRelay> VelloApp<'a, T> {
    pub fn new(
        width: Arc<AtomicU32>,
        height: Arc<AtomicU32>,
        y_transform: Arc<Mutex<vello::kurbo::Affine>>,
        tx: T,
        scene: SceneDrawer,
        needs_redraw: Arc<AtomicBool>,
        base_color: Arc<AtomicU32>,
    ) -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended(None),
            scene,
            needs_redraw,
            width,
            height,
            y_transform,
            base_color,
            layout: parley::Layout::new(),
            tx,
            window_title: "vellogd".to_string(),
        }
    }

    pub fn set_size(&self, width: u32, height: u32) {
        self.width.store(width, Ordering::Relaxed);
        self.height.store(height, Ordering::Relaxed);
        *self.y_transform.lock().unwrap() = calc_y_translate(height as f32);
    }

    pub fn y_transform(&self) -> vello::kurbo::Affine {
        *self.y_transform.lock().unwrap()
    }

    pub fn create_new_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.scene.reset();

        // TODO: handle Active render state as well?
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        let width = self.width.load(Ordering::Relaxed) as f32;
        let height = self.height.load(Ordering::Relaxed) as f32;

        let window = cached_window.take().unwrap_or_else(|| {
            let this = &self;
            let attrs_basic = Window::default_attributes()
                .with_title(&this.window_title)
                .with_inner_size(winit::dpi::LogicalSize::new(width, height));
            let attrs = add_platform_specific_attributes(attrs_basic);

            let window = event_loop
                .create_window(attrs)
                .expect("failed to create window");
            window.focus_window();
            Arc::new(window)
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
fn add_platform_specific_attributes(attrs: WindowAttributes) -> WindowAttributes {
    use winit::platform::windows::WindowAttributesExtWindows;
    attrs.with_corner_preference(winit::platform::windows::CornerPreference::DoNotRound)
}

#[cfg(target_os = "linux")]
fn add_platform_specific_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs
}

#[cfg(target_os = "macos")]
fn add_platform_specific_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs
}

#[cfg(target_os = "windows")]
pub fn create_event_loop(any_thread: bool) -> EventLoop<Request> {
    use winit::platform::windows::EventLoopBuilderExtWindows;

    let event_loop = EventLoop::<Request>::with_user_event()
        .with_any_thread(any_thread)
        .build()
        .unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop
}

#[cfg(target_os = "linux")]
pub fn create_event_loop(any_thread: bool) -> EventLoop<Request> {
    use winit::platform::wayland::EventLoopBuilderExtWayland;

    let event_loop = EventLoop::<Request>::with_user_event()
        .with_any_thread(any_thread)
        .build()
        .unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop
}

#[cfg(target_os = "macos")]
pub fn create_event_loop(any_thread: bool) -> EventLoop<Request> {
    if any_thread {
        panic!("Not supported!");
    }
    let event_loop = EventLoop::<Request>::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop
}

impl<'a, T: AppResponseRelay> TextLayouter for VelloApp<'a, T> {
    fn layout_mut(&mut self) -> &mut parley::Layout<peniko::Brush> {
        &mut self.layout
    }

    fn layout_ref(&self) -> &parley::Layout<peniko::Brush> {
        &self.layout
    }
}

impl<'a, T: AppResponseRelay> ApplicationHandler<Request> for VelloApp<'a, T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_new_window(event_loop);
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let RenderState::Active(state) = &self.state {
            self.state = RenderState::Suspended(Some(state.window.clone()));
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let render_state = match &mut self.state {
            RenderState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                // Window is automatically closed when dropped, so just replacing it with Suspended is enough.
                self.state = RenderState::Suspended(None);
            }

            WindowEvent::Resized(size) => {
                // TODO: borrow checker doesn't allow self.set_size(), so inlined the code here.
                {
                    self.width.store(size.width, Ordering::Relaxed);
                    self.height.store(size.height, Ordering::Relaxed);
                    *self.y_transform.lock().unwrap() = calc_y_translate(size.height as f32);
                };

                self.context
                    .resize_surface(&mut render_state.surface, size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                let surface = &render_state.surface;
                let width = surface.config.width;
                let height = surface.config.height;

                let device_handle = &self.context.devices[surface.dev_id];

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                if let Some(renderer) = self.renderers[surface.dev_id].as_mut() {
                    let base_color = {
                        let [r, g, b, a] = self.base_color.load(Ordering::Relaxed).to_ne_bytes();
                        Color::rgba8(r, g, b, a)
                    };
                    renderer
                        .render_to_surface(
                            &device_handle.device,
                            &device_handle.queue,
                            &self.scene.scene(),
                            &surface_texture,
                            &vello::RenderParams {
                                base_color,
                                width,
                                height,
                                antialiasing_method: AaConfig::Msaa16,
                            },
                        )
                        .expect("failed to render");

                    // surface is now up-to-date!
                    self.needs_redraw.store(false, Ordering::Relaxed);
                }

                surface_texture.present();
                device_handle.device.poll(vello::wgpu::Maintain::Poll); // TODO: wait?
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Request) {
        if matches!(event, Request::NewWindow) {
            self.create_new_window(event_loop);
            return;
        }

        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            // TODO: this must NOT return if the event has return value.
            // incoming event must be consumed otherwise the UI freezes
            _ => return,
        };

        match event {
            Request::ConnectionReady => {
                unreachable!("This event should not be sent to app")
            }
            Request::NewWindow => {
                // TODO
            }
            Request::RedrawWindow => {
                if self.needs_redraw.load(Ordering::Relaxed) {
                    render_state.window.request_redraw();
                }
            }
            Request::CloseWindow => {
                self.state = RenderState::Suspended(None);
            }
            Request::NewPage => {
                self.scene.reset();
                self.needs_redraw.store(true, Ordering::Relaxed);
            }
            Request::GetWindowSizes => {
                let PhysicalSize { width, height } = render_state.window.inner_size();
                self.tx.respond(Response::WindowSizes { width, height });
            }
            Request::SetBaseColor { color } => self.base_color.store(color, Ordering::Relaxed),
            Request::DrawText {
                pos,
                text,
                color,
                size,
                lineheight,
                family,
                face,
                angle,
                hadj,
            } => {
                let (weight, style) = fontface_to_weight_and_style(face);
                self.build_layout(text, &family, weight, style, size, lineheight);

                let layout_width = self.layout.width();
                let window_height = self.height.load(Ordering::Relaxed) as f64;

                for line in self.layout.lines() {
                    let line_metrics = line.metrics();
                    let transform = vello::kurbo::Affine::translate((
                        -(layout_width * hadj) as f64,
                        (line_metrics.baseline - line_metrics.line_height) as f64, // TODO: is this correct?
                    ))
                    .then_rotate(-angle as f64)
                    .then_translate((pos.x, window_height - pos.y).into()); // Y-axis is flipped

                    for item in line.items() {
                        // ignore inline box
                        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };

                        self.scene.draw_glyph(glyph_run, color, transform);
                    }
                }

                self.needs_redraw.store(true, Ordering::Relaxed);
            }
            Request::SaveAsPng { filename } => {
                self.save_as_png(filename);
            }

            // ignore other events
            _ => {}
        };
    }
}

// Since R's graphics device is left-bottom origin, the Y value needs to be
// flipped
pub fn calc_y_translate(height: f32) -> vello::kurbo::Affine {
    vello::kurbo::Affine::new([1.0, 0., 0., -1.0, 0., height as _]) // = FLIP_Y.then_translate((0.0, height))
}

const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

// Hold the communication channel between VelloApp and the shared statuses.
pub struct VelloAppProxy {
    pub tx: EventLoopProxy<Request>,
    pub rx: std::sync::Mutex<std::sync::mpsc::Receiver<Response>>,

    pub scene: SceneDrawer,
    // Note: these fields are intentionally not bundled as a struct; if it's a
    // struct, it would need `Mutex`, but we want to read the values without
    // lock (probably doesn't affect much on the performance, though).
    pub width: Arc<AtomicU32>,
    pub height: Arc<AtomicU32>,
    y_transform: Arc<Mutex<vello::kurbo::Affine>>,
    base_color: Arc<AtomicU32>,

    // To be called by mode() API so that the device can stop rendering when it
    // is actively written.
    pub stop_rendering: Arc<AtomicBool>,
}

impl VelloAppProxy {
    pub fn set_size(&self, width: u32, height: u32) {
        self.width.store(width, Ordering::Relaxed);
        self.height.store(height, Ordering::Relaxed);
        *self.y_transform.lock().unwrap() = calc_y_translate(height as f32);
    }

    pub fn y_transform(&self) -> vello::kurbo::Affine {
        *self.y_transform.lock().unwrap()
    }

    pub fn set_base_color(&self, color: u32) {
        self.base_color.store(color, Ordering::Relaxed);
    }
}

pub static VELLO_APP_PROXY: LazyLock<VelloAppProxy> = LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let event_loop = create_event_loop(true);
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let (tx, rx) = std::sync::mpsc::channel::<Response>();

        let needs_redraw = Arc::new(AtomicBool::new(false));
        // Note: 0 is a dummy value and should be overwritten soon after the
        // creation. Ideally, VELLO_APP_PROXY should be OnceLock so that the
        // init function can initialize this with the actual sizes, but LazyLock
        // is far better at ergonomics; I want to avoid every-time Option
        // handling, but this might be a tradeoff...
        let width = Arc::new(AtomicU32::new(0));
        let height = Arc::new(AtomicU32::new(0));
        let y_transform = Arc::new(Mutex::new(calc_y_translate(0.0)));
        let base_color = Arc::new(AtomicU32::new(Color::WHITE_SMOKE.to_premul_u32()));

        let is_drawing = Arc::new(AtomicBool::new(false));

        let scene = SceneDrawer::new(y_transform.clone(), height.clone(), needs_redraw.clone());
        let proxy = VelloAppProxy {
            tx: event_loop.create_proxy(),
            rx: std::sync::Mutex::new(rx),
            scene: scene.clone(),
            width: width.clone(),
            height: height.clone(),
            y_transform: y_transform.clone(),
            base_color: base_color.clone(),
            stop_rendering: is_drawing.clone(),
        };
        sender.send(proxy).unwrap();

        let mut app = VelloApp::new(
            width,
            height,
            y_transform,
            tx,
            scene,
            needs_redraw,
            base_color,
        );

        // this blocks until event_loop exits
        event_loop.run_app(&mut app).unwrap();
    });

    let event_loop = receiver.recv().unwrap();
    let event_loop_for_refresh = event_loop.tx.clone();

    let stop_rendering = event_loop.stop_rendering.clone();

    // TODO: stop refreshing when no window
    std::thread::spawn(move || loop {
        // Skip refreshing the window if the R session is drawing into it.
        if !stop_rendering.load(Ordering::Relaxed) {
            event_loop_for_refresh
                .send_event(Request::RedrawWindow)
                .unwrap();
        }
        std::thread::sleep(REFRESH_INTERVAL);
    });

    event_loop
});
