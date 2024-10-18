// The code related to vello is based on
//
// - the example code on linbender/vello (examples/simple/main.rs).
// - the example code on linbender/parley (examples/vello_editor/src/main.rs).

use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU32, Ordering},
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
    protocol::{AppResponseRelay, FillParams, Request, Response, StrokeParams},
    text_layouter::TextLayouter,
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

pub struct SceneWithFlag {
    pub scene: Scene,
    pub needs_redraw: bool,
}

impl SceneWithFlag {
    fn new() -> Self {
        Self {
            scene: Scene::new(),
            needs_redraw: false,
        }
    }
}

#[derive(Clone)]
pub struct SceneDrawer {
    inner: Arc<Mutex<SceneWithFlag>>,
    y_transform: vello::kurbo::Affine,
}

impl SceneDrawer {
    pub fn new(height: f32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SceneWithFlag::new())),
            y_transform: calc_y_translate(height),
        }
    }

    pub fn set_y_translate(&mut self, height: f32) {
        self.y_transform = calc_y_translate(height);
    }

    pub fn reset(&mut self) {
        self.inner.lock().unwrap().scene.reset();
    }

    pub fn scene(&self) -> std::sync::MutexGuard<'_, SceneWithFlag> {
        self.inner.lock().unwrap()
    }

    pub fn draw_circle(
        &self,
        center: kurbo::Point,
        radius: f64,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        let circle = vello::kurbo::Circle::new(center, radius);
        let scene_with_flag = &mut self.inner.lock().unwrap();

        if let Some(fill_params) = fill_params {
            scene_with_flag.scene.fill(
                peniko::Fill::NonZero,
                self.y_transform,
                fill_params.color,
                None,
                &circle,
            );
        }

        if let Some(stroke_params) = stroke_params {
            scene_with_flag.scene.stroke(
                &stroke_params.stroke,
                self.y_transform,
                stroke_params.color,
                None,
                &circle,
            );
        }

        scene_with_flag.needs_redraw = true;
    }

    pub fn draw_line(&self, p0: kurbo::Point, p1: kurbo::Point, stroke_params: StrokeParams) {
        let line = vello::kurbo::Line::new(p0, p1);
        let scene_with_flag = &mut self.inner.lock().unwrap();

        scene_with_flag.scene.stroke(
            &stroke_params.stroke,
            self.y_transform,
            stroke_params.color,
            None,
            &line,
        );

        scene_with_flag.needs_redraw = true;
    }

    pub fn draw_polyline(&self, path: kurbo::BezPath, stroke_params: StrokeParams) {
        let scene_with_flag = &mut self.inner.lock().unwrap();
        scene_with_flag.scene.stroke(
            &stroke_params.stroke,
            self.y_transform,
            stroke_params.color,
            None,
            &path,
        );

        scene_with_flag.needs_redraw = true;
    }

    pub fn draw_polygon(
        &self,
        path: kurbo::BezPath,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        let scene_with_flag = &mut self.inner.lock().unwrap();
        if let Some(fill_params) = fill_params {
            scene_with_flag.scene.fill(
                peniko::Fill::NonZero,
                self.y_transform,
                fill_params.color,
                None,
                &path,
            );
        }

        if let Some(stroke_params) = stroke_params {
            scene_with_flag.scene.stroke(
                &stroke_params.stroke,
                self.y_transform,
                stroke_params.color,
                None,
                &path,
            );
        }

        scene_with_flag.needs_redraw = true;
    }

    pub fn draw_rect(
        &self,
        p0: kurbo::Point,
        p1: kurbo::Point,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    ) {
        let rect = vello::kurbo::Rect::new(p0.x, p0.y, p1.x, p1.y);
        let scene_with_flag = &mut self.inner.lock().unwrap();
        if let Some(fill_params) = fill_params {
            scene_with_flag.scene.fill(
                peniko::Fill::NonZero,
                self.y_transform,
                fill_params.color,
                None,
                &rect,
            );
        }

        if let Some(stroke_params) = stroke_params {
            scene_with_flag.scene.stroke(
                &stroke_params.stroke,
                self.y_transform,
                stroke_params.color,
                None,
                &rect,
            );
        }

        scene_with_flag.needs_redraw = true;
    }

    pub fn draw_glyph(
        &self,
        glyph_run: parley::GlyphRun<peniko::Brush>,
        color: peniko::Color,
        transform: kurbo::Affine,
        vadj: f32,
    ) {
        let scene_with_flag = &mut self.inner.lock().unwrap();

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
            .map(|coord| vello::skrifa::instance::NormalizedCoord::from_bits(*coord))
            .collect::<Vec<_>>();

        scene_with_flag
            .scene
            .draw_glyphs(font)
            .brush(color)
            .transform(transform)
            .font_size(font_size)
            .normalized_coords(&coords)
            .draw(
                peniko::Fill::NonZero,
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

        scene_with_flag.needs_redraw = true;
    }
}

pub struct VelloApp<'a, T: AppResponseRelay> {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    state: RenderState<'a>,
    scene: SceneDrawer,
    width: Arc<AtomicU32>,
    height: Arc<AtomicU32>,
    background_color: Color, // TODO: probably always the same value
    layout: parley::Layout<peniko::Brush>,
    tx: T,

    window_title: String,
    needs_redraw: bool,
}

impl<'a, T: AppResponseRelay> VelloApp<'a, T> {
    pub fn new(width: Arc<AtomicU32>, height: Arc<AtomicU32>, tx: T, scene: SceneDrawer) -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended(None),
            scene,
            width,
            height,
            background_color: Color::WHITE_SMOKE,
            layout: parley::Layout::new(),
            tx,
            window_title: "vellogd".to_string(),
            needs_redraw: true,
        }
    }

    pub fn create_new_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.scene.reset();

        // TODO: handle Active render state as well?
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        let width = ACTIVE_WINDOW_SIZES.0.load(Ordering::Relaxed) as f32;
        let height = ACTIVE_WINDOW_SIZES.1.load(Ordering::Relaxed) as f32;

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
                ACTIVE_WINDOW_SIZES.0.store(size.width, Ordering::Relaxed);
                ACTIVE_WINDOW_SIZES.1.store(size.height, Ordering::Relaxed);

                // TODO: update y_transform

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
                    renderer
                        .render_to_surface(
                            &device_handle.device,
                            &device_handle.queue,
                            &self.scene.scene().scene, // TODO: looks a bit funny
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
                if let Ok(s) = self.scene.inner.try_lock() {
                    if s.needs_redraw {
                        render_state.window.request_redraw();
                    }
                }
            }
            Request::CloseWindow => {
                self.state = RenderState::Suspended(None);
            }
            Request::NewPage => {
                self.scene.reset();
                self.needs_redraw = true;
            }
            Request::GetWindowSizes => {
                let PhysicalSize { width, height } = render_state.window.inner_size();
                self.tx.respond(Response::WindowSizes { width, height });
            }

            Request::DrawText {
                pos,
                text,
                color,
                size,
                lineheight,
                family: _,
                angle,
                hadj,
            } => {
                self.build_layout(text, size, lineheight);

                let layout_width = self.layout.width();
                let window_height = ACTIVE_WINDOW_SIZES.1.load(Ordering::Relaxed) as f64;

                let transform =
                    vello::kurbo::Affine::translate((-(layout_width * hadj) as f64, 0.0))
                        .then_rotate(-angle as f64)
                        .then_translate((pos.x, window_height - pos.y).into()); // Y-axis is flipped

                for line in self.layout.lines() {
                    let vadj = line.metrics().ascent * 0.5;
                    for item in line.items() {
                        // ignore inline box
                        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                            continue;
                        };

                        self.scene.draw_glyph(glyph_run, color, transform, vadj);
                    }
                }

                self.needs_redraw = true;
            }

            // ignore other events
            _ => {}
        };
    }
}

// Since R's graphics device is left-bottom origin, the Y value needs to be
// flipped
pub fn calc_y_translate(height: f32) -> vello::kurbo::Affine {
    vello::kurbo::Affine::new([1.0, 0., 0., -1.0, 0., height as _])
}

const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

const DEFAULT_SIZE: u32 = 480;

pub static ACTIVE_WINDOW_SIZES: LazyLock<(AtomicU32, AtomicU32)> =
    LazyLock::new(|| (AtomicU32::new(DEFAULT_SIZE), AtomicU32::new(DEFAULT_SIZE)));

// Hold the communication channel between VelloApp and the shared statuses.
pub struct VelloAppProxy {
    pub tx: EventLoopProxy<Request>,
    pub rx: std::sync::Mutex<std::sync::mpsc::Receiver<Response>>,
    pub scene: SceneDrawer,
    pub width: Arc<AtomicU32>,
    pub height: Arc<AtomicU32>,
    // TODO
    // pub y_transform: Arc<Mutex<vello::kurbo::Affine>>,
}

// TODO: change this to OnceCell and create init function that takes width and height
pub static VELLO_APP_PROXY: LazyLock<VelloAppProxy> = LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let event_loop = create_event_loop(true);
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let (tx, rx) = std::sync::mpsc::channel::<Response>();

        let width = Arc::new(AtomicU32::new(0));
        let height = Arc::new(AtomicU32::new(0));

        // TODO: SceneDrawer should not hold height
        let scene = SceneDrawer::new(480.0);
        let proxy = VelloAppProxy {
            tx: event_loop.create_proxy(),
            rx: std::sync::Mutex::new(rx),
            scene: scene.clone(),
            width: width.clone(),
            height: height.clone(),
        };
        sender.send(proxy).unwrap();

        let mut app = VelloApp::new(width, height, tx, scene);

        // this blocks until event_loop exits
        event_loop.run_app(&mut app).unwrap();
    });

    let event_loop = receiver.recv().unwrap();
    let event_loop_for_refresh = event_loop.tx.clone();

    // TODO: stop refreshing when no window
    std::thread::spawn(move || loop {
        event_loop_for_refresh
            .send_event(Request::RedrawWindow)
            .unwrap();
        std::thread::sleep(REFRESH_INTERVAL);
    });

    event_loop
});
