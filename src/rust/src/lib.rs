mod graphics;
mod winit_app;

use std::ffi::CStr;

use graphics::ClippingStrategy;
use graphics::DevDesc;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use graphics::R_GE_gcontext;
use graphics::R_NilValue;
use savvy::savvy;
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit_app::VelloApp;

#[cfg(debug_assertions)]
mod debug_device;

pub struct VelloGraphicsDevice {
    filename: String,
    event_loop: winit::event_loop::EventLoopProxy<UserEvent>,
    layout: parley::Layout<vello::peniko::Brush>,
}

impl VelloGraphicsDevice {
    pub(crate) fn new(
        filename: &str,
        event_loop: winit::event_loop::EventLoopProxy<UserEvent>,
    ) -> Self {
        Self {
            filename: filename.into(),
            event_loop,
            layout: parley::Layout::new(),
        }
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
    RedrawWindow,
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
    DrawRect {
        p0: vello::kurbo::Point,
        p1: vello::kurbo::Point,
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

impl StrokeParams {
    pub fn from_gc(gc: R_GE_gcontext) -> Option<Self> {
        if gc.col == 0 || gc.lty == -1 {
            return None;
        }

        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = vello::peniko::Color::rgba8(r, g, b, a);

        let width = gc.lwd;

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let join = match gc.ljoin {
            1 => vello::kurbo::Join::Round,
            2 => vello::kurbo::Join::Miter,
            3 => vello::kurbo::Join::Bevel,
            v => panic!("invalid join value: {v}"),
        };
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let cap = match gc.lend {
            1 => vello::kurbo::Cap::Round,
            2 => vello::kurbo::Cap::Butt,
            3 => vello::kurbo::Cap::Square,
            v => panic!("invalid cap value: {v}"),
        };

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L413C1-L419C50
        //
        // Based on these implementations
        //
        // https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/modules/X11/devX11.c#L1224
        // https://github.com/r-lib/ragg/blob/6e8bfd1264dfaa36aa6f92592e13a1169986e7b9/src/AggDevice.h#L195C8-L205
        let dash_pattern: Vec<f64> = match gc.lty {
            -1 => vec![], // LTY_BLANK;
            0 => vec![],  // LTY_SOLID;
            lty => {
                let ptn_bytes = lty.to_ne_bytes();
                let mut ptn = Vec::new();
                for b in ptn_bytes {
                    let dash = b & 0b00001111;
                    let gap = (b & 0b11110000) >> 4;

                    if dash == 0 {
                        break;
                    }

                    ptn.push(dash as f64 * width);
                    ptn.push(gap as f64 * width);
                }
                ptn
            }
        };

        Some(Self {
            color,
            stroke: vello::kurbo::Stroke {
                width,
                join,
                miter_limit: gc.lmitre,
                start_cap: cap,
                end_cap: cap,
                dash_pattern: dash_pattern.into(),
                dash_offset: 0.0,
            },
        })
    }
}

impl FillParams {
    pub fn from_gc(gc: R_GE_gcontext) -> Option<Self> {
        if gc.fill == 0 {
            return None;
        }
        let [r, g, b, a] = gc.fill.to_ne_bytes();
        let color = vello::peniko::Color::rgba8(r, g, b, a);
        Some(Self { color })
    }
}

fn xy_to_path(x: &[f64], y: &[f64], close: bool) -> vello::kurbo::BezPath {
    let mut path = vello::kurbo::BezPath::new();

    let x_iter = x.iter();
    let y_iter = y.iter();
    let mut points = x_iter.zip(y_iter);
    if let Some(first) = points.next() {
        path.move_to(vello::kurbo::Point::new(*first.0, *first.1));
    } else {
        return path;
    }

    for (x, y) in points {
        path.line_to(vello::kurbo::Point::new(*x, *y));
    }

    if close {
        path.close_path();
    }

    path
}

impl DeviceDriver for VelloGraphicsDevice {
    const USE_RASTER: bool = true;

    const USE_CAPTURE: bool = true;

    const USE_LOCATOR: bool = true;

    const USE_PLOT_HISTORY: bool = false;

    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    const ACCEPT_UTF8_TEXT: bool = true;

    fn activate(&mut self, _: DevDesc) {}

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.event_loop
                .send_event(UserEvent::DrawCircle {
                    center: center.into(),
                    radius: r,
                    fill_params,
                    stroke_params,
                })
                .unwrap();
        }
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn close(&mut self, _: DevDesc) {
        self.event_loop.send_event(UserEvent::CloseWindow).unwrap();
    }

    fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        if let Some(stroke_params) = StrokeParams::from_gc(gc) {
            self.event_loop
                .send_event(UserEvent::DrawLine {
                    p0: from.into(),
                    p1: to.into(),
                    stroke_params,
                })
                .unwrap();
        }
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> graphics::TextMetric {
        // TODO
        let _family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = gc.cex * gc.ps;
        winit_app::build_layout_into(
            &mut self.layout,
            c.to_string(),
            size as _,
            gc.lineheight as _,
        );
        let line = self.layout.lines().next();
        match line {
            Some(line) => {
                let metrics = line.metrics();
                graphics::TextMetric {
                    ascent: metrics.ascent as _,
                    descent: metrics.descent as _,
                    width: self.layout.width() as _, // TOOD: should this be run.metrics().width of the first char?
                }
            }
            None => graphics::TextMetric {
                ascent: 0.0,
                descent: 0.0,
                width: 0.0,
            },
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        self.event_loop.send_event(UserEvent::NewPage).unwrap();
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.event_loop
                .send_event(UserEvent::DrawPolygon {
                    path: xy_to_path(x, y, true),
                    fill_params,
                    stroke_params,
                })
                .unwrap();
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        let stroke_params = StrokeParams::from_gc(gc);
        if let Some(stroke_params) = stroke_params {
            self.event_loop
                .send_event(UserEvent::DrawPolyline {
                    path: xy_to_path(x, y, true),
                    stroke_params,
                })
                .unwrap();
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.event_loop
                .send_event(UserEvent::DrawRect {
                    p0: from.into(),
                    p1: to.into(),
                    fill_params,
                    stroke_params,
                })
                .unwrap();
        }
    }

    fn path(
        &mut self,
        x: &[f64],
        y: &[f64],
        nper: &[i32],
        winding: bool,
        gc: R_GE_gcontext,
        dd: DevDesc,
    ) {
    }

    fn raster<T: AsRef<[u32]>>(
        &mut self,
        raster: graphics::Raster<T>,
        pos: (f64, f64),
        size: (f64, f64),
        angle: f64,
        interpolate: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
    }

    fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
        unsafe { R_NilValue }
    }

    fn size(&mut self, dd: DevDesc) -> (f64, f64, f64, f64) {
        (dd.left, dd.right, dd.bottom, dd.top)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, dd: DevDesc) -> f64 {
        // TODO
        let family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        // TODO
        let _family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = gc.cex * gc.ps;
        winit_app::build_layout_into(&mut self.layout, text, size as _, gc.lineheight as _);
        self.layout.width() as _
    }

    fn text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = vello::peniko::Color::rgba8(r, g, b, a);
        let family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.event_loop
                .send_event(UserEvent::DrawText {
                    pos: pos.into(),
                    text: text.into(),
                    color,
                    size: (gc.cex * gc.ps) as _,
                    lineheight: gc.lineheight as _,
                    // face: gc.fontface as _,
                    family,
                    angle: angle.to_radians() as _,
                    hadj: hadj as _,
                })
                .unwrap();
        }
    }

    fn on_exit(&mut self, _: DevDesc) {}

    fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
        true
    }

    fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
        0
    }

    fn locator(&mut self, x: *mut f64, y: *mut f64, _: DevDesc) -> bool {
        true
    }

    fn eventHelper(&mut self, _: DevDesc, code: i32) {}
}

const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

#[savvy]
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let h = std::thread::spawn(move || {
        let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
            .unwrap();
        let proxy = event_loop.create_proxy();
        sender.send(proxy).unwrap();

        let mut app = VelloApp::new(width as _, height as _);

        // this blocks until event_loop exits
        event_loop.run_app(&mut app).unwrap();
    });

    let event_loop = receiver.recv().unwrap();
    let event_loop_for_refresh = event_loop.clone();

    std::thread::spawn(move || loop {
        event_loop_for_refresh
            .send_event(UserEvent::RedrawWindow)
            .unwrap();
        std::thread::sleep(REFRESH_INTERVAL);
    });

    let device_driver = VelloGraphicsDevice::new(filename, event_loop);

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd");

    Ok(())
}

#[savvy]
fn debuggd() -> savvy::Result<()> {
    debuggd_inner();
    Ok(())
}

#[cfg(debug_assertions)]
fn debuggd_inner() {
    let device_driver = debug_device::DebugDevice {};

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(480.0, 480.0);

    device_driver.create_device::<debug_device::DebugDevice>(device_descriptor, "debug");
}

#[cfg(not(debug_assertions))]
fn debuggd_inner() {}
