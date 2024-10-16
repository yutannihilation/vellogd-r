use std::ffi::CStr;

use crate::graphics::DeviceDriver;
use crate::graphics::TextMetric;
use crate::WindowController;
use vellogd_shared::ffi::DevDesc;
use vellogd_shared::ffi::R_GE_gcontext;
use vellogd_shared::ffi::R_NilValue;
use vellogd_shared::protocol::FillParams;
use vellogd_shared::protocol::StrokeParams;
use vellogd_shared::protocol::UserEvent;
use vellogd_shared::protocol::UserResponse;
use vellogd_shared::winit_app::build_layout_into;
use vellogd_shared::winit_app::EVENT_LOOP;

pub struct VelloGraphicsDevice {
    filename: String,
    layout: parley::Layout<vello::peniko::Brush>,
}

impl VelloGraphicsDevice {
    pub(crate) fn new(filename: &str) -> savvy::Result<Self> {
        Ok(Self {
            filename: filename.into(),
            layout: parley::Layout::new(),
        })
    }
}

impl WindowController for VelloGraphicsDevice {
    fn send_event(&self, event: UserEvent) -> savvy::Result<()> {
        EVENT_LOOP
            .event_loop
            .send_event(event)
            .map_err(|e| format!("Failed to send event {e:?}").into())
    }

    fn recv_response(&self) -> savvy::Result<UserResponse> {
        EVENT_LOOP
            .rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|e| e.to_string().into())
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
    fn activate(&mut self, _: DevDesc) {
        self.send_event(UserEvent::NewWindow).unwrap();
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(UserEvent::DrawCircle {
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
        self.send_event(UserEvent::CloseWindow).unwrap();
    }

    fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        if let Some(stroke_params) = StrokeParams::from_gc(gc) {
            self.send_event(UserEvent::DrawLine {
                p0: from.into(),
                p1: to.into(),
                stroke_params,
            })
            .unwrap();
        }
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        // TODO
        let _family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = gc.cex * gc.ps;
        build_layout_into(
            &mut self.layout,
            c.to_string(),
            size as _,
            gc.lineheight as _,
        );
        let line = self.layout.lines().next();
        match line {
            Some(line) => {
                let metrics = line.metrics();
                TextMetric {
                    ascent: metrics.ascent as _,
                    descent: metrics.descent as _,
                    width: self.layout.width() as _, // TOOD: should this be run.metrics().width of the first char?
                }
            }
            None => TextMetric {
                ascent: 0.0,
                descent: 0.0,
                width: 0.0,
            },
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        self.send_event(UserEvent::NewPage).unwrap();
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(UserEvent::DrawPolygon {
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
            self.send_event(UserEvent::DrawPolyline {
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
            self.send_event(UserEvent::DrawRect {
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
        raster: crate::graphics::Raster<T>,
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
        let sizes = self.get_window_sizes().unwrap_or((0, 0));
        (0.0, sizes.0 as _, 0.0, sizes.1 as _)
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
        build_layout_into(&mut self.layout, text, size as _, gc.lineheight as _);
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
            self.send_event(UserEvent::DrawText {
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
