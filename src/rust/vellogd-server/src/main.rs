use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use vellogd_shared::{
    protocol::{Request, Response},
    winit_app::{create_event_loop, VelloApp},
};

// TODO: make this configurable
const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16); // = 60fps

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
    let rx = match rx_server.accept() {
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

    // Since the main thread will be occupied by event_loop, the server needs to
    // run in a spawned thread. rx waits for the event and forward it to
    // event_loop via proxy.
    std::thread::spawn(move || loop {
        let event = rx.recv().unwrap();
        proxy.send_event(event).unwrap();
    });

    // TODO: supply width and height
    let mut app = VelloApp::new(480.0 as _, 480.0 as _, tx);
    event_loop.run_app(&mut app).unwrap();
}
