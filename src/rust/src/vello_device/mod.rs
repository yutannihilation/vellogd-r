#[cfg(feature = "winit")]
mod default;

#[cfg(feature = "winit")]
pub use default::VelloGraphicsDevice;

#[cfg(not(feature = "winit"))]
mod no_winit;
#[cfg(not(feature = "winit"))]
pub use no_winit::VelloGraphicsDevice;

mod with_server;

use vellogd_shared::protocol::{Request, Response};
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

fn xy_to_path_with_hole(x: &[f64], y: &[f64], nper: &[i32]) -> kurbo::BezPath {
    let mut path = kurbo::BezPath::new();

    let x_iter = x.iter();
    let y_iter = y.iter();
    let mut points = x_iter.zip(y_iter);

    for n in nper {
        if let Some(first) = points.next() {
            path.move_to(kurbo::Point::new(*first.0, *first.1));
        } else {
            break;
        }

        let n_rest = *n as usize - 1;
        for _ in 0..n_rest {
            if let Some((x, y)) = points.next() {
                path.line_to(kurbo::Point::new(*x, *y));
            } else {
                break;
            }
        }
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

    fn request_set_base_color(&self, color: u32) -> savvy::Result<()> {
        self.send_event(Request::SetBaseColor { color })
    }

    fn request_save_as_png<T: ToString>(&self, filename: T) -> savvy::Result<()> {
        self.send_event(Request::SaveAsPng {
            filename: filename.to_string(),
        })
    }

    fn request_register_tile(
        &self,
        x_offset: f32,
        y_offset: f32,
        extend: peniko::Extend,
    ) -> savvy::Result<()> {
        self.send_event(Request::SaveAsTile {
            x_offset,
            y_offset,
            extend,
        })
    }
}
