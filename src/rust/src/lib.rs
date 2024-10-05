mod graphics;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use savvy::savvy;

pub struct VelloGraphicsDevice {}

impl VelloGraphicsDevice {
    pub fn new(filename: &str, width: u32, height: u32) -> Self {
        Self {}
    }
}

impl DeviceDriver for VelloGraphicsDevice {}

#[savvy]
fn vellogd(filename: &str, width: i32, height: i32) -> savvy::Result<()> {
    // Typically, 72 points per inch
    let width_pt = width * 72;
    let height_pt = height * 72;

    let device_driver = VelloGraphicsDevice::new(filename, width_pt as _, height_pt as _);

    let device_descriptor =
        DeviceDescriptor::new().device_size(0.0, width_pt as _, 0.0, height_pt as _);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd");

    Ok(())
}
