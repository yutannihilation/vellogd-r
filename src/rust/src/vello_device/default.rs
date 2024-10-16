use super::WindowController;
use crate::graphics::DeviceDriver;
use vellogd_shared::ffi::DevDesc;
use vellogd_shared::ffi::R_GE_gcontext;
use vellogd_shared::protocol::Request;
use vellogd_shared::protocol::Response;
use vellogd_shared::text_layouter::TextLayouter;
use vellogd_shared::text_layouter::TextMetric;
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
        match self.request_new_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to activate: {e}"),
        }
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        match self.request_circle(center, r, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw circle: {e}"),
        }
    }

    // TODO
    // fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn close(&mut self, _: DevDesc) {
        match self.request_close_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to close window: {e}"),
        }
    }

    // TODO
    // fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        match self.request_line(from, to, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw line: {e}"),
        }
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        self.get_char_metric(c, gc)
    }

    // TODO
    // fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, _: R_GE_gcontext, _: DevDesc) {
        match self.request_new_page() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to create a new page: {e}"),
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        match self.request_polygon(x, y, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw polygon: {e}"),
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        match self.request_polyline(x, y, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw polyline: {e}"),
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        match self.request_rect(from, to, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw rect: {e}"),
        }
    }

    // TODO
    // fn path(
    //     &mut self,
    //     x: &[f64],
    //     y: &[f64],
    //     nper: &[i32],
    //     winding: bool,
    //     gc: R_GE_gcontext,
    //     dd: DevDesc,
    // ) {
    // }

    // TODO
    // fn raster<T: AsRef<[u32]>>(
    //     &mut self,
    //     raster: crate::graphics::Raster<T>,
    //     pos: (f64, f64),
    //     size: (f64, f64),
    //     angle: f64,
    //     interpolate: bool,
    //     gc: R_GE_gcontext,
    //     _: DevDesc,
    // ) {
    // }

    // TODO
    // fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
    //     unsafe { R_NilValue }
    // }

    fn size(&mut self, _: DevDesc) -> (f64, f64, f64, f64) {
        let sizes = self.get_window_sizes().unwrap_or((0, 0));
        (0.0, sizes.0 as _, 0.0, sizes.1 as _)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, _: DevDesc) -> f64 {
        self.get_text_width(text, gc)
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
        match self.request_text(pos, text, angle, hadj, gc) {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to draw text: {e}"),
        }
    }

    // TODO
    // fn on_exit(&mut self, _: DevDesc) {}

    // TODO
    // fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
    //     true
    // }

    // TODO
    // fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
    //     0
    // }

    // TODO
    // fn locator(&mut self, x: *mut f64, y: *mut f64, _: DevDesc) -> bool {
    //     true
    // }

    // TODO
    // fn eventHelper(&mut self, _: DevDesc, code: i32) {}
}
