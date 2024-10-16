mod graphics;
mod shared;
mod vello_device;

#[cfg(not(target_os = "macos"))]
mod winit_app;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use savvy::savvy;
use shared::UserEvent;
use shared::UserResponse;

use vello_device::VelloGraphicsDevice;

#[cfg(debug_assertions)]
mod debug_device;

pub trait WindowController {
    fn send_event(&self, event: UserEvent) -> savvy::Result<()>;
    fn recv_response(&self) -> savvy::Result<UserResponse>;
    fn get_window_sizes(&self) -> savvy::Result<(u32, u32)> {
        self.send_event(UserEvent::GetWindowSizes)?;
        match self.recv_response()? {
            UserResponse::WindowSizes { width, height } => Ok((width, height)),
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
fn debuggd() -> savvy::Result<()> {
    debuggd_inner();
    Ok(())
}

#[cfg(debug_assertions)]
fn debuggd_inner() {
    let device_driver = debug_device::DebugGraphicsDevice {};

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(480.0, 480.0);

    device_driver.create_device::<debug_device::DebugGraphicsDevice>(device_descriptor, "debug");
}

#[cfg(not(debug_assertions))]
fn debuggd_inner() {}
