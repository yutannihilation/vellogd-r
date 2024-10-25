use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc, Mutex,
};

use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use vellogd_shared::{
    protocol::{Request, Response},
    winit_app::{calc_y_translate, create_event_loop, SceneDrawer, VelloApp},
};

// TODO: make this configurable
const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

struct SceneRequestHandler {
    pub scene: SceneDrawer,
}

impl SceneRequestHandler {
    fn handle_event(&self, event: Request) {
        match event {
            Request::DrawCircle {
                center,
                radius,
                fill_params,
                stroke_params,
            } => {
                self.scene
                    .draw_circle(center, radius, fill_params, stroke_params);
            }
            Request::DrawLine {
                p0,
                p1,
                stroke_params,
            } => {
                self.scene.draw_line(p0, p1, stroke_params);
            }
            Request::DrawPolygon {
                path,
                fill_params,
                stroke_params,
            } => {
                self.scene.draw_polygon(path, fill_params, stroke_params);
            }
            Request::DrawPolyline {
                path,
                stroke_params,
            } => {
                self.scene.draw_polyline(path, stroke_params);
            }
            Request::DrawRect {
                p0,
                p1,
                fill_params,
                stroke_params,
            } => {
                self.scene.draw_rect(p0, p1, fill_params, stroke_params);
            }
            Request::DrawText { .. } => {
                // TODO: where to store layout?

                // self.build_layout(text, size, lineheight);

                // let width = self.layout.width() as f64;
                // let transform = vello::kurbo::Affine::translate((-(width * hadj), 0.0))
                //     .then_rotate(-angle.to_radians())
                //     .then_translate((pos.0, self.height - pos.1).into()); // Y-axis is flipped

                // for line in self.layout.lines() {
                //     let vadj = line.metrics().ascent * 0.5;
                //     for item in line.items() {
                //         // ignore inline box
                //         let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                //             continue;
                //         };

                //         // TODO: do not lock per glyph
                //         scene_drawer.draw_glyph(glyph_run, color, transform, vadj);
                //     }
                // }
            }
            _ => {}
        }
    }
}

// expect `vellogd-server SERVER_NAME [WIDTH [HEIGHT]]`
fn parse_args() -> (String, u32, u32) {
    let tx_server_name = std::env::args().nth(1).unwrap();

    let width = match std::env::args().nth(2).map(|x| x.parse::<u32>()) {
        Some(Ok(w)) => w,
        _ => 480,
    };

    let height = match std::env::args().nth(3).map(|x| x.parse::<u32>()) {
        Some(Ok(w)) => w,
        _ => width,
    };

    (tx_server_name, width, height)
}

fn main() {
    let (tx_server_name, width, height) = parse_args();

    let width = Arc::new(AtomicU32::new(width));

    let y_transform = Arc::new(Mutex::new(calc_y_translate(height as f32)));
    let height = Arc::new(AtomicU32::new(height));

    // First, connect from server to client
    let tx: IpcSender<Response> = IpcSender::connect(tx_server_name).unwrap();
    // Then, create a connection of the opposite direction
    let (rx_server, rx_server_name) = IpcOneShotServer::<Request>::new().unwrap();
    // Tell the server name to the client
    tx.send(Response::Connect {
        server_name: rx_server_name,
    })
    .unwrap();
    // Wait for the client is ready
    let rx: IpcReceiver<Request> = match rx_server.accept() {
        Ok((rx, Request::ConnectionReady)) => rx,
        Ok((_, data)) => panic!("got unexpected data: {data:?}"),
        Err(e) => panic!("failed to accept connection: {e}"),
    };

    let event_loop = create_event_loop(false);
    let proxy = event_loop.create_proxy();

    let proxy_for_refresh = proxy.clone();
    // TODO: stop refreshing when no window
    std::thread::spawn(move || loop {
        proxy_for_refresh.send_event(Request::RedrawWindow).unwrap();
        std::thread::sleep(REFRESH_INTERVAL);
    });

    let needs_redraw = Arc::new(AtomicBool::new(false));
    let scene = SceneDrawer::new(y_transform.clone(), height.clone(), needs_redraw.clone());

    let request_handler = SceneRequestHandler {
        scene: scene.clone(),
    };

    // Since the main thread will be occupied by event_loop, the server needs to
    // run in a spawned thread. rx waits for the event and forward it to
    // event_loop via proxy.
    std::thread::spawn(move || loop {
        let event = rx.recv().unwrap();
        match event {
            Request::DrawCircle { .. }
            | Request::DrawLine { .. }
            | Request::DrawPolygon { .. }
            | Request::DrawPolyline { .. }
            | Request::DrawRect { .. }
            | Request::DrawText { .. } => request_handler.handle_event(event),
            _ => proxy.send_event(event).unwrap(),
        }
    });

    // This is not used for server; the base color is set via Request::SetBaseColor
    let base_color = Arc::new(AtomicU32::new(0));

    let mut app = VelloApp::new(
        width,
        height,
        y_transform,
        tx,
        scene,
        needs_redraw,
        base_color,
    );
    event_loop.run_app(&mut app).unwrap();
}
