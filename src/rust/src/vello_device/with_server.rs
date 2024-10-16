use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use vellogd_shared::{
    ffi::{DevDesc, R_GE_gcontext},
    protocol::{Request, Response},
    text_layouter::{TextLayouter, TextMetric},
};

use crate::graphics::DeviceDriver;

use super::WindowController;

pub struct VelloGraphicsDeviceWithServer {
    filename: String,
    layout: parley::Layout<peniko::Brush>,
    process: Option<std::process::Child>,
    tx: IpcSender<Request>,
    rx: IpcReceiver<Response>,
}

impl VelloGraphicsDeviceWithServer {
    pub(crate) fn new(filename: &str, server: Option<&str>) -> savvy::Result<Self> {
        // server -> controller
        let (rx_server, rx_server_name) = IpcOneShotServer::<Response>::new().unwrap();

        let server_process = if let Some(server_bin) = server {
            // spawn a server process
            let res = std::process::Command::new(server_bin)
                .arg(rx_server_name)
                // .stdout(std::process::Stdio::piped())
                .spawn();

            match res {
                Ok(c) => {
                    savvy::r_eprintln!("Server runs at PID {}", c.id());
                    Some(c)
                }
                Err(e) => {
                    let msg = format!("failed to spawn the process: {e}");
                    return Err(savvy::Error::new(&msg));
                }
            }
        } else {
            savvy::r_eprintln!("rx_server_name: {rx_server_name}");
            None
        };

        // establish connections of both direction
        let (tx, rx) = match rx_server.accept() {
            Ok((rx, Response::Connect { server_name })) => {
                savvy::r_eprint!("Connecting to {server_name}...");
                let tx: IpcSender<Request> = IpcSender::connect(server_name).unwrap();
                tx.send(Request::ConnectionReady).unwrap();
                (tx, rx)
            }
            Ok((_, data)) => panic!("got unexpected data: {data:?}"),
            Err(e) => panic!("failed to accept connection: {e}"),
        };
        savvy::r_eprintln!("connected!");

        Ok(Self {
            filename: filename.into(),
            layout: parley::Layout::new(),
            process: server_process,
            tx,
            rx,
        })
    }
}

impl WindowController for VelloGraphicsDeviceWithServer {
    fn send_event(&self, event: vellogd_shared::protocol::Request) -> savvy::Result<()> {
        self.tx.send(event).map_err(|e| e.to_string().into())
    }

    fn recv_response(&self) -> savvy::Result<vellogd_shared::protocol::Response> {
        self.rx.recv().map_err(|e| e.to_string().into())
    }
}

impl TextLayouter for VelloGraphicsDeviceWithServer {
    fn layout_mut(&mut self) -> &mut parley::Layout<peniko::Brush> {
        &mut self.layout
    }

    fn layout_ref(&self) -> &parley::Layout<peniko::Brush> {
        &self.layout
    }
}

impl DeviceDriver for VelloGraphicsDeviceWithServer {
    fn activate(&mut self, _: DevDesc) {
        self.request_new_window().unwrap();
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        self.request_circle(center, r, gc).unwrap();
    }

    // TODO
    // fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn close(&mut self, _: DevDesc) {
        self.request_close_window().unwrap();
    }

    // TODO
    // fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        self.request_line(from, to, gc).unwrap();
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        self.get_char_metric(c, gc)
    }

    // TODO
    // fn mode(&mut self, mode: i32, _: DevDesc) {}

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
        self.request_text(pos, text, angle, hadj, gc).unwrap();
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
