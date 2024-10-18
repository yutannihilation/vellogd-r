use std::sync::atomic::Ordering;

use super::xy_to_path;
use super::WindowController;
use crate::add_tracing_point;
use crate::graphics::DeviceDriver;
use vellogd_shared::ffi::DevDesc;
use vellogd_shared::ffi::R_GE_gcontext;
use vellogd_shared::protocol::FillParams;
use vellogd_shared::protocol::Request;
use vellogd_shared::protocol::Response;
use vellogd_shared::protocol::StrokeParams;
use vellogd_shared::text_layouter::TextLayouter;
use vellogd_shared::text_layouter::TextMetric;
use vellogd_shared::winit_app::VELLO_APP_PROXY;

pub struct VelloGraphicsDevice {
    filename: String,
    layout: parley::Layout<peniko::Brush>,
}

impl VelloGraphicsDevice {
    pub(crate) fn new(filename: &str, width: f64, height: f64) -> savvy::Result<Self> {
        VELLO_APP_PROXY.set_size(width as u32, height as u32);
        Ok(Self {
            filename: filename.into(),
            layout: parley::Layout::new(),
        })
    }
}

impl WindowController for VelloGraphicsDevice {
    fn send_event(&self, event: Request) -> savvy::Result<()> {
        VELLO_APP_PROXY
            .tx
            .send_event(event)
            .map_err(|e| format!("Failed to send event {e:?}").into())
    }

    fn recv_response(&self) -> savvy::Result<Response> {
        VELLO_APP_PROXY
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
        add_tracing_point!();

        match self.request_new_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to activate: {e}"),
        }
    }

    fn close(&mut self, _: DevDesc) {
        add_tracing_point!();

        match self.request_close_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to close window: {e}"),
        }
    }

    // TODO
    // fn deactivate(&mut self, _: DevDesc) {}

    // TODO
    // fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        VELLO_APP_PROXY.set_base_color(gc.fill);
        match self.request_new_page() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to create a new page: {e}"),
        }
    }

    // TODO
    // fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_circle(center.into(), r, fill_params, stroke_params);
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        if let Some(stroke_params) = StrokeParams::from_gc(gc) {
            VELLO_APP_PROXY
                .scene
                .draw_line(from.into(), to.into(), stroke_params);
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_polygon(xy_to_path(x, y, true), fill_params, stroke_params);
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let stroke_params = StrokeParams::from_gc(gc);
        if let Some(stroke_params) = stroke_params {
            VELLO_APP_PROXY
                .scene
                .draw_polyline(xy_to_path(x, y, false), stroke_params);
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_rect(from.into(), to.into(), fill_params, stroke_params);
        }
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
        add_tracing_point!();

        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = peniko::Color::rgba8(r, g, b, a);
        // TODO
        // let family = unsafe {
        //     std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
        //         .to_str()
        //         .unwrap_or("Arial")
        // }
        // .to_string();
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            let size = (gc.cex * gc.ps) as f32;
            let lineheight = gc.lineheight as f32;
            self.build_layout(text, size, lineheight);

            let layout_width = self.layout.width() as f64;
            let window_height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;

            let transform = vello::kurbo::Affine::translate((-(layout_width * hadj), 0.0))
                .then_rotate(-angle.to_radians())
                .then_translate((pos.0, window_height - pos.1).into()); // Y-axis is flipped

            for line in self.layout.lines() {
                let vadj = line.metrics().ascent * 0.5;
                for item in line.items() {
                    // ignore inline box
                    let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                        continue;
                    };

                    // TODO: do not lock per glyph
                    VELLO_APP_PROXY
                        .scene
                        .draw_glyph(glyph_run, color, transform, vadj);
                }
            }
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
        add_tracing_point!();

        let width = VELLO_APP_PROXY.width.load(Ordering::Relaxed) as f64;
        let height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;

        (0.0, width, 0.0, height)
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        add_tracing_point!();

        self.get_char_metric(c, gc)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, _: DevDesc) -> f64 {
        add_tracing_point!();

        self.get_text_width(text, gc)
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
