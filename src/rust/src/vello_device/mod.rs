#[cfg(not(target_os = "macos"))]
mod default;

#[cfg(not(target_os = "macos"))]
pub use default::VelloGraphicsDevice;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::VelloGraphicsDevice;

mod with_server;

use vellogd_shared::{
    ffi::R_GE_gcontext,
    protocol::{FillParams, Request, Response, StrokeParams},
};
pub use with_server::VelloGraphicsDeviceWithServer;

fn xy_to_path(x: &[f64], y: &[f64], close: bool) -> kurbo::BezPath {
    let mut path = kurbo::BezPath::new();

    let x_iter = x.iter();
    let y_iter = y.iter();
    let mut points = x_iter.zip(y_iter);
    if let Some(first) = points.next() {
        path.move_to(kurbo::Point::new(*first.0, *first.1));
    } else {
        return path;
    }

    for (x, y) in points {
        path.line_to(kurbo::Point::new(*x, *y));
    }

    if close {
        path.close_path();
    }

    path
}

pub trait WindowController {
    fn send_event(&self, event: Request) -> savvy::Result<()>;
    fn recv_response(&self) -> savvy::Result<Response>;

    fn get_window_sizes(&self) -> savvy::Result<(u32, u32)> {
        self.send_event(Request::GetWindowSizes)?;
        match self.recv_response()? {
            Response::WindowSizes { width, height } => Ok((width, height)),
            _ => Err("Unexpected result".into()),
        }
    }

    fn request_new_window(&self) -> savvy::Result<()> {
        self.send_event(Request::NewWindow)
    }

    fn request_close_window(&self) -> savvy::Result<()> {
        self.send_event(Request::CloseWindow)
    }

    fn request_new_page(&self) -> savvy::Result<()> {
        self.send_event(Request::NewPage)
    }

    fn request_circle(
        &mut self,
        center: (f64, f64),
        r: f64,
        gc: R_GE_gcontext,
    ) -> savvy::Result<()> {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawCircle {
                center: center.into(),
                radius: r,
                fill_params,
                stroke_params,
            })
        } else {
            Ok(())
        }
    }

    fn request_line(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        gc: R_GE_gcontext,
    ) -> savvy::Result<()> {
        if let Some(stroke_params) = StrokeParams::from_gc(gc) {
            self.send_event(Request::DrawLine {
                p0: from.into(),
                p1: to.into(),
                stroke_params,
            })
        } else {
            Ok(())
        }
    }

    fn request_polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext) -> savvy::Result<()> {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawPolygon {
                path: xy_to_path(x, y, true),
                fill_params,
                stroke_params,
            })
        } else {
            Ok(())
        }
    }

    fn request_polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext) -> savvy::Result<()> {
        let stroke_params = StrokeParams::from_gc(gc);
        if let Some(stroke_params) = stroke_params {
            self.send_event(Request::DrawPolyline {
                path: xy_to_path(x, y, true),
                stroke_params,
            })
        } else {
            Ok(())
        }
    }

    fn request_rect(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        gc: R_GE_gcontext,
    ) -> savvy::Result<()> {
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawRect {
                p0: from.into(),
                p1: to.into(),
                fill_params,
                stroke_params,
            })
        } else {
            Ok(())
        }
    }

    fn request_text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
    ) -> savvy::Result<()> {
        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = peniko::Color::rgba8(r, g, b, a);
        let family = unsafe {
            std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawText {
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
        } else {
            Ok(())
        }
    }
}
