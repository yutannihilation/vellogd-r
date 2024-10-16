mod graphics;
mod vello_device;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use savvy::savvy;
use vello_device::VelloGraphicsDeviceWithServer;
use vellogd_shared::protocol::Request;
use vellogd_shared::protocol::Response;

use vello_device::VelloGraphicsDevice;

#[cfg(debug_assertions)]
mod debug_device;

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
}

#[savvy]
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDevice::new(filename)?;

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd")?;

    Ok(())
}

#[savvy]
fn vellogd_with_server_impl(
    filename: &str,
    width: f64,
    height: f64,
    server: Option<&str>,
) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDeviceWithServer::new(filename, server)?;

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDeviceWithServer>(device_descriptor, "vellogd")?;

    Ok(())
}

#[savvy]
fn debuggd() -> savvy::Result<()> {
    debuggd_inner();
    Ok(())
}

#[cfg(debug_assertions)]
fn debuggd_inner() {
    let device_driver = debug_device::DebugGraphicsDevice {};

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(480.0, 480.0);

    device_driver
        .create_device::<debug_device::DebugGraphicsDevice>(device_descriptor, "debug")
        .unwrap();
}

#[cfg(not(debug_assertions))]
fn debuggd_inner() {}
