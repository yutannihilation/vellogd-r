mod graphics;
mod vello_device;

use savvy::savvy;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use vello_device::VelloGraphicsDevice;
use vello_device::VelloGraphicsDeviceWithServer;

#[cfg(debug_assertions)]
mod debug_device;

#[savvy]
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDevice::new(filename, height)?;

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
