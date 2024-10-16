use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use vellogd_shared::protocol::{UserEvent, UserResponse};

use crate::{graphics::DeviceDriver, WindowController};

pub struct VelloGraphicsDeviceWithServer {
    filename: String,
    layout: parley::Layout<vello::peniko::Brush>,
    process: Option<std::process::Child>,
    tx: IpcSender<UserEvent>,
    rx: IpcReceiver<UserResponse>,
}

impl VelloGraphicsDeviceWithServer {
    pub(crate) fn new(filename: &str, server: Option<&str>) -> savvy::Result<Self> {
        // server -> controller
        let (rx_server, rx_server_name) = IpcOneShotServer::<UserResponse>::new().unwrap();

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
            Ok((rx, UserResponse::Connect { server_name })) => {
                savvy::r_eprint!("Connecting to {server_name}...");
                let tx: IpcSender<UserEvent> = IpcSender::connect(server_name).unwrap();
                tx.send(UserEvent::ConnectionReady).unwrap();
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
    fn send_event(&self, event: vellogd_shared::protocol::UserEvent) -> savvy::Result<()> {
        self.tx.send(event).map_err(|e| e.to_string().into())
    }

    fn recv_response(&self) -> savvy::Result<vellogd_shared::protocol::UserResponse> {
        self.rx.recv().map_err(|e| e.to_string().into())
    }
}

impl DeviceDriver for VelloGraphicsDeviceWithServer {}