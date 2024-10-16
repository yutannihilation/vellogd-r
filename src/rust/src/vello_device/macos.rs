use crate::graphics::DeviceDriver;

pub struct VelloGraphicsDevice {}

impl VelloGraphicsDevice {
    pub(crate) fn new(filename: &str) -> savvy::Result<Self> {
        Err("This method is not supported on macOS".into())
    }
}

impl DeviceDriver for VelloGraphicsDevice {
    fn create_device<T: DeviceDriver>(
        self,
        device_descriptor: crate::graphics::DeviceDescriptor,
        device_name: &'static str,
    ) -> savvy::Result<()> {
        Err("This method is not supported on macOS".into())
    }
}
