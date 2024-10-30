use crate::graphics::DeviceDriver;

use savvy::savvy_err;

pub struct VelloGraphicsDevice {}

impl VelloGraphicsDevice {
    pub(crate) fn new(filename: &str, _width: f64, _height: f64) -> savvy::Result<Self> {
        Err(savvy_err!("This method is not supported on macOS"))
    }
}

impl DeviceDriver for VelloGraphicsDevice {
    fn create_device<T: DeviceDriver>(
        self,
        device_descriptor: crate::graphics::DeviceDescriptor,
        device_name: &'static str,
    ) -> savvy::Result<()> {
        Err(savvy_err!("This method is not supported on macOS"))
    }
}
