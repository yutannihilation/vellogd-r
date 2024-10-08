// Copied from https://github.com/extendr/extendr/blob/master/extendr-api/src/graphics/

mod ffi;

mod device_descriptor;
mod device_driver;

pub use device_descriptor::DeviceDescriptor;

pub use device_driver::{ClippingStrategy, DeviceDriver};

use ffi::*;

pub use ffi::{DevDesc, R_GE_gcontext, R_NilValue};

pub struct TextMetric {
    pub ascent: f64,
    pub descent: f64,
    pub width: f64,
}

/// A row-major array of pixels. One pixel is 32-bit, whose each byte represents
/// alpha, blue, green, and red in the order.
#[derive(Clone, Debug, PartialEq)]
pub struct Raster<P: AsRef<[u32]>> {
    pub pixels: P,
    pub width: usize,
}
