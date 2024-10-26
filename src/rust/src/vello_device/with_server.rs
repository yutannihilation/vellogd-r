use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use vellogd_shared::{
    ffi::{DevDesc, R_GE_gcontext},
    protocol::{FillParams, Request, Response, StrokeParams},
    text_layouter::{TextLayouter, TextMetric},
};

use crate::{add_tracing_point, graphics::DeviceDriver};

use super::{xy_to_path, xy_to_path_with_hole, WindowController};

pub struct VelloGraphicsDeviceWithServer {
    filename: String,
    layout: parley::Layout<peniko::Brush>,
    process: Option<std::process::Child>,
    tx: IpcSender<Request>,
    rx: IpcReceiver<Response>,
}

impl Drop for VelloGraphicsDeviceWithServer {
    fn drop(&mut self) {
        if let Some(c) = self.process.as_mut() {
            c.kill().unwrap();
        }
    }
}

impl VelloGraphicsDeviceWithServer {
    pub(crate) fn new(
        filename: &str,
        server: Option<&str>,
        width: f64,
        height: f64,
    ) -> savvy::Result<Self> {
        // server -> controller
        let (rx_server, rx_server_name) = IpcOneShotServer::<Response>::new().unwrap();

        let server_process = if let Some(server_bin) = server {
            // spawn a server process
            let res = std::process::Command::new(server_bin)
                .args([
                    rx_server_name,
                    (width as u32).to_string(),
                    (height as u32).to_string(),
                ])
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
        add_tracing_point!();

        self.request_new_window().unwrap();
    }

    fn close(&mut self, _: DevDesc) {
        add_tracing_point!();

        self.request_close_window().unwrap();
    }

    // TODO
    // fn deactivate(&mut self, _: DevDesc) {}

    // TODO
    // fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        self.request_set_base_color(gc.fill).unwrap();
        self.request_new_page().unwrap();
    }

    // TODO
    // fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawCircle {
                center: center.into(),
                radius: r,
                fill_params,
                stroke_params,
            })
            .unwrap();
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        if let Some(stroke_params) = StrokeParams::from_gc(gc) {
            self.send_event(Request::DrawLine {
                p0: from.into(),
                p1: to.into(),
                stroke_params,
            })
            .unwrap();
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawPolygon {
                path: xy_to_path(x, y, true),
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
        _: DevDesc,
    ) {
        let fill_params = FillParams::from_gc_with_flag(gc, winding);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawPolygon {
                path: xy_to_path_with_hole(x, y, nper),
                fill_params,
                stroke_params,
            })
            .unwrap();
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let stroke_params = StrokeParams::from_gc(gc);
        if let Some(stroke_params) = stroke_params {
            self.send_event(Request::DrawPolyline {
                path: xy_to_path(x, y, false),
                stroke_params,
            })
            .unwrap();
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = FillParams::from_gc(gc);
        let stroke_params = StrokeParams::from_gc(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            self.send_event(Request::DrawRect {
                p0: from.into(),
                p1: to.into(),
                fill_params,
                stroke_params,
            })
            .unwrap();
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
                family,
                face: gc.fontface,
                angle: angle as _,
                hadj: hadj as _,
            })
            .unwrap();
        }
    }

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

    fn size(&mut self, width: &mut f64, height: &mut f64, _: DevDesc) {
        add_tracing_point!();

        // TODO: cache result? (for example, for 1 second)

        let sizes = self.get_window_sizes().unwrap_or((0, 0));
        *width = sizes.0 as f64;
        *height = sizes.1 as f64;
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
