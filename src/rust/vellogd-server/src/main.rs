use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use vellogd_shared::{
    protocol::{UserEvent, UserResponse},
    winit_app::{create_event_loop, VelloApp},
};

fn main() {
    let tx_server_name = std::env::args().nth(1).unwrap();

    // First, connect from server to client
    let tx: IpcSender<UserResponse> = IpcSender::connect(tx_server_name).unwrap();
    // Then, create a connection of the opposite direction
    let (rx_server, rx_server_name) = IpcOneShotServer::<UserEvent>::new().unwrap();
    // Tell the server name to the client
    tx.send(UserResponse::Connect {
        server_name: rx_server_name,
    })
    .unwrap();
    // Wait for the client is ready
    let rx = match rx_server.accept() {
        Ok((rx, UserEvent::ConnectionReady)) => rx,
        Ok((_, data)) => panic!("got unexpected data: {data:?}"),
        Err(e) => panic!("failed to accept connection: {e}"),
    };

    let event_loop = create_event_loop(false);
    let proxy = event_loop.create_proxy();

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
