use std::ffi::CStr;

use super::WindowController;
use crate::graphics::DeviceDriver;
use crate::graphics::TextMetric;
use vellogd_shared::ffi::DevDesc;
use vellogd_shared::ffi::R_GE_gcontext;
use vellogd_shared::ffi::R_NilValue;
use vellogd_shared::protocol::Request;
use vellogd_shared::protocol::Response;
use vellogd_shared::text_layouter::TextLayouter;
use vellogd_shared::winit_app::EVENT_LOOP;

pub struct VelloGraphicsDevice {
    filename: String,
    layout: parley::Layout<peniko::Brush>,
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
    fn send_event(&self, event: Request) -> savvy::Result<()> {
        EVENT_LOOP
            .event_loop
            .send_event(event)
            .map_err(|e| format!("Failed to send event {e:?}").into())
    }

    fn recv_response(&self) -> savvy::Result<Response> {
        EVENT_LOOP
            .rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|e| e.to_string().into())
    }
}

impl TextLayouter for VelloGraphicsDevice {
    fn layout_mut(&mut self) -> &mut parley::Layout<peniko::Brush> {
        &mut self.layout
    }

    fn layout_ref(&self) -> &parley::Layout<peniko::Brush> {
        &self.layout
    }
}

impl DeviceDriver for VelloGraphicsDevice {
    fn activate(&mut self, _: DevDesc) {
        self.request_new_window().unwrap();
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        self.request_circle(center, r, gc).unwrap();
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn close(&mut self, _: DevDesc) {
        self.request_close_window().unwrap();
    }

    fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        self.request_line(from, to, gc).unwrap();
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
        self.build_layout(c.to_string(), size as _, gc.lineheight as _);
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

    fn new_page(&mut self, _: R_GE_gcontext, _: DevDesc) {
        self.request_new_page().unwrap();
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        self.request_polygon(x, y, gc).unwrap();
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        self.request_polyline(x, y, gc).unwrap();
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        self.request_rect(from, to, gc).unwrap();
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
        self.build_layout(text, size as _, gc.lineheight as _);
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
        self.request_text(pos, text, angle, hadj, gc).unwrap();
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
