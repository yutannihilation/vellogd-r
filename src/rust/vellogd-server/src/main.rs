use std::sync::Arc;

use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use vellogd_shared::{
    protocol::{Request, Response},
    winit_app::{create_event_loop, SceneDrawer, VelloApp},
};

// TODO: make this configurable
const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

fn handle_event(scene_drawer: &mut SceneDrawer, event: Request) {
    match event {
        Request::DrawCircle {
            center,
            radius,
            fill_params,
            stroke_params,
        } => {
            scene_drawer.draw_circle(center, radius, fill_params, stroke_params);
        }
        Request::DrawLine {
            p0,
            p1,
            stroke_params,
        } => {
            scene_drawer.draw_line(p0, p1, stroke_params);
        }
        Request::DrawPolygon {
            path,
            fill_params,
            stroke_params,
        } => {
            scene_drawer.draw_polygon(path, fill_params, stroke_params);
        }
        Request::DrawPolyline {
            path,
            stroke_params,
        } => {
            scene_drawer.draw_polyline(path, stroke_params);
        }
        Request::DrawRect {
            p0,
            p1,
            fill_params,
            stroke_params,
        } => {
            scene_drawer.draw_rect(p0, p1, fill_params, stroke_params);
        }
        Request::DrawText {
            pos,
            text,
            color,
            size,
            lineheight,
            family,
            angle,
            hadj,
        } => {
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

fn main() {
    let tx_server_name = std::env::args().nth(1).unwrap();

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

    let scene = SceneDrawer::new();
    let mut scene_drawer = scene.clone();

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
            | Request::DrawText { .. } => handle_event(&mut scene_drawer, event),
            _ => proxy.send_event(event).unwrap(),
        }
    });

    // TODO: supply width and height
    let width = Arc::new(480.into());
    let height = Arc::new(480.into());
    let mut app = VelloApp::new(width, height, tx, scene);
    event_loop.run_app(&mut app).unwrap();
}
