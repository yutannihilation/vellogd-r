#[cfg(not(target_os = "macos"))]
mod default;

#[cfg(not(target_os = "macos"))]
pub use default::VelloGraphicsDevice;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::VelloGraphicsDevice;

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
}
