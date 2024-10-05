// Copied from https://github.com/extendr/extendr/blob/master/extendr-api/src/graphics/

mod ffi;

mod device_descriptor;
mod device_driver;

pub use device_descriptor::DeviceDescriptor;

pub use device_driver::{ClippingStrategy, DeviceDriver};
use savvy::Sexp;

use ffi::*;

pub use ffi::{DevDesc, R_GE_gcontext, R_NilValue};

pub struct Context {
    context: R_GE_gcontext,
    xscale: (f64, f64),
    yscale: (f64, f64),
    offset: (f64, f64),
    scalar: f64,
}

pub struct Device {
    inner: pGEDevDesc,
}

pub struct Pattern {
    inner: Sexp,
}

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

impl Device {
    pub(crate) fn inner(&self) -> pGEDevDesc {
        self.inner
    }

    // pub(crate) fn asref(&self) -> &GEDevDesc {
    //     unsafe { &*self.inner }
    // }

    // pub(crate) fn dev(&self) -> &DevDesc {
    //     unsafe { &*self.asref().dev }
    // }
}

#[derive(PartialEq, Debug, Clone)]
pub enum LineEnd {
    Round,
    Butt,
    Square,
}

#[derive(PartialEq, Debug, Clone)]
pub enum LineJoin {
    Round,
    Mitre,
    Bevel,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Unit {
    Device,
    Normalized,
    Inches,
    CM,
}

impl From<LineEnd> for R_GE_lineend {
    fn from(value: LineEnd) -> Self {
        match value {
            LineEnd::Round => R_GE_lineend_GE_ROUND_CAP,
            LineEnd::Butt => R_GE_lineend_GE_BUTT_CAP,
            LineEnd::Square => R_GE_lineend_GE_SQUARE_CAP,
        }
    }
}

impl From<LineJoin> for R_GE_linejoin {
    fn from(value: LineJoin) -> Self {
        match value {
            LineJoin::Round => R_GE_linejoin_GE_ROUND_JOIN,
            LineJoin::Mitre => R_GE_linejoin_GE_MITRE_JOIN,
            LineJoin::Bevel => R_GE_linejoin_GE_BEVEL_JOIN,
        }
    }
}

fn unit_to_ge(unit: Unit) -> GEUnit {
    match unit {
        Unit::Device => GEUnit_GE_DEVICE,
        Unit::Normalized => GEUnit_GE_NDC,
        Unit::Inches => GEUnit_GE_INCHES,
        Unit::CM => GEUnit_GE_CM,
    }
}

#[allow(non_snake_case)]
impl Device {
    // /// Get the current device.
    // pub fn current() -> Result<Device> {
    //     // At present we can't trap an R error from a function
    //     // that does not return a SEXP.
    //     unsafe {
    //         Ok(Device {
    //             inner: GEcurrentDevice(),
    //         })
    //     }
    // }

    // /// Enable device rendering.
    // pub fn mode_on(&self) -> Result<()> {
    //     unsafe {
    //         if Rf_NoDevices() != 0 {
    //             Err(Error::NoGraphicsDevices(r!(())))
    //         } else {
    //             GEMode(1, self.inner());
    //             Ok(())
    //         }
    //     }
    // }

    // /// Disable device rendering and flush.
    // pub fn mode_off(&self) -> Result<()> {
    //     unsafe {
    //         if Rf_NoDevices() != 0 {
    //             Err(Error::NoGraphicsDevices(r!(())))
    //         } else {
    //             GEMode(0, self.inner());
    //             Ok(())
    //         }
    //     }
    // }

    // /// Get the device number for this device.
    // pub fn device_number(&self) -> i32 {
    //     unsafe { GEdeviceNumber(self.inner()) }
    // }

    // /// Get a device by number.
    // pub fn get_device(number: i32) -> Result<Device> {
    //     unsafe {
    //         if number < 0 || number >= Rf_NumDevices() {
    //             Err(Error::NoGraphicsDevices(r!(())))
    //         } else {
    //             Ok(Device {
    //                 inner: GEgetDevice(number),
    //             })
    //         }
    //     }
    // }

    /// Convert device coordinates into a specified unit.
    /// This is usually done by the API.
    pub fn from_device_coords(&self, value: (f64, f64), from: Unit) -> (f64, f64) {
        let from = unit_to_ge(from);
        unsafe {
            (
                GEfromDeviceX(value.0, from, self.inner()),
                GEfromDeviceY(value.1, from, self.inner()),
            )
        }
    }

    /// Convert a specified unit coordinates into device coordinates.
    /// This is usually done by the API.
    pub fn to_device_coords(&self, value: (f64, f64), to: Unit) -> (f64, f64) {
        if to == Unit::Device {
            value
        } else {
            let to = unit_to_ge(to);
            unsafe {
                (
                    GEtoDeviceX(value.0, to, self.inner()),
                    GEtoDeviceY(value.1, to, self.inner()),
                )
            }
        }
    }

    /// Convert device width/height coordinates into a specified unit.
    /// This is usually done by the API.
    pub fn from_device_wh(&self, value: (f64, f64), from: Unit) -> (f64, f64) {
        let from = unit_to_ge(from);
        unsafe {
            (
                GEfromDeviceWidth(value.0, from, self.inner()),
                GEfromDeviceHeight(value.1, from, self.inner()),
            )
        }
    }

    /// Convert a specified unit width/height coordinates into device coordinates.
    /// This is usually done by the API.
    pub fn to_device_wh(&self, value: (f64, f64), to: Unit) -> (f64, f64) {
        let to = unit_to_ge(to);
        unsafe {
            (
                GEtoDeviceWidth(value.0, to, self.inner()),
                GEtoDeviceHeight(value.1, to, self.inner()),
            )
        }
    }

    // /// Start a new page. The page color can be set in advance.
    // pub fn new_page(&self, gc: &Context) {
    //     unsafe { GENewPage(gc.context(), self.inner()) }
    // }

    // /// Change the clip rectangle.
    // pub fn clip(&self, from: (f64, f64), to: (f64, f64), gc: &Context) {
    //     let from = gc.t(from);
    //     let to = gc.t(to);
    //     unsafe { GESetClip(from.0, from.1, to.0, to.1, self.inner()) }
    // }

    // /// Draw a stroked line. gc.color() is the stroke color.
    // pub fn line(&self, from: (f64, f64), to: (f64, f64), gc: &Context) {
    //     let from = gc.t(from);
    //     let to = gc.t(to);
    //     unsafe { GELine(from.0, from.1, to.0, to.1, gc.context(), self.inner()) }
    // }

    // /// Draw a stroked/filled polyline. gc.color() is the stroke color.
    // /// The input is anything yielding (x,y) coordinate pairs.
    // /// Polylines are not closed.
    // pub fn polyline<T: IntoIterator<Item = (f64, f64)>>(&self, coords: T, gc: &Context) {
    //     let (mut x, mut y): (Vec<_>, Vec<_>) = coords.into_iter().map(|xy| gc.t(xy)).unzip();
    //     let xptr = x.as_mut_slice().as_mut_ptr();
    //     let yptr = y.as_mut_slice().as_mut_ptr();
    //     unsafe {
    //         GEPolyline(
    //             x.len() as std::os::raw::c_int,
    //             xptr,
    //             yptr,
    //             gc.context(),
    //             self.inner(),
    //         )
    //     }
    // }

    // /// Draw a stroked/filled polygon. gc.color() is the stroke color.
    // /// The input is anything yielding (x,y) coordinate pairs.
    // /// Polygons are closed.
    // pub fn polygon<T: IntoIterator<Item = (f64, f64)>>(&self, coords: T, gc: &Context) {
    //     let (mut x, mut y): (Vec<_>, Vec<_>) = coords.into_iter().map(|xy| gc.t(xy)).unzip();
    //     let xptr = x.as_mut_slice().as_mut_ptr();
    //     let yptr = y.as_mut_slice().as_mut_ptr();
    //     unsafe {
    //         GEPolygon(
    //             x.len() as std::os::raw::c_int,
    //             xptr,
    //             yptr,
    //             gc.context(),
    //             self.inner(),
    //         )
    //     }
    // }

    // // /// Return a list of (x, y) points generated from a spline.
    // // /// The iterator returns ((x, y), s) where s is -1 to 1.
    // // pub fn xspline<T: Iterator<Item = ((f64, f64), f64)> + Clone>(
    // //     &self,
    // //     coords: T,
    // //     open: bool,
    // //     rep_ends: bool,
    // //     draw: bool,
    // //     gc: &Context,
    // // ) -> Robj {
    // //     let (mut x, mut y): (Vec<_>, Vec<_>) = coords
    // //         .clone()
    // //         .map(|(xy, _s)| gc.t(xy))
    // //         .unzip();
    // //     let mut s: Vec<_> = coords.map(|(_xy, s)| s).collect();
    // //     let xptr = x.as_mut_slice().as_mut_ptr();
    // //     let yptr = y.as_mut_slice().as_mut_ptr();
    // //     let sptr = s.as_mut_slice().as_mut_ptr();
    // //     unsafe {
    // //         new_owned(GEXspline(
    // //             x.len() as std::os::raw::c_int,
    // //             xptr,
    // //             yptr,
    // //             sptr,
    // //             if open { 1 } else { 0 },
    // //             if rep_ends { 1 } else { 0 },
    // //             if draw { 1 } else { 0 },
    // //             gc.context(),
    // //             self.inner(),
    // //         ))
    // //     }
    // // }

    // /// Draw a stroked/filled circle.
    // /// gc.color() is the stroke color.
    // /// gc.fill() is the fill color.
    // pub fn circle(&self, center: (f64, f64), radius: f64, gc: &Context) {
    //     let center = gc.t(center);
    //     let radius = gc.ts(radius);
    //     unsafe { GECircle(center.0, center.1, radius, gc.context(), self.inner()) }
    // }

    // /// Draw a stroked/filled axis-aligned rectangle.
    // /// gc.color() is the stroke color.
    // /// gc.fill() is the fill color.
    // pub fn rect(&self, from: (f64, f64), to: (f64, f64), gc: &Context) {
    //     let from = gc.t(from);
    //     let to = gc.t(to);
    //     unsafe { GERect(from.0, from.1, to.0, to.1, gc.context(), self.inner()) }
    // }

    // /// Draw a path with multiple segments.
    // /// gc.color() is the stroke color.
    // /// gc.fill() is the fill color.
    // /// The input is an interator of iterators yielding (x,y) pairs.
    // pub fn path<T: IntoIterator<Item = impl IntoIterator<Item = (f64, f64)>>>(
    //     &self,
    //     coords: T,
    //     winding: bool,
    //     gc: &Context,
    // ) {
    //     let mut x = Vec::new();
    //     let mut y = Vec::new();
    //     let mut nper: Vec<std::os::raw::c_int> = Vec::new();
    //     let coords = coords.into_iter();
    //     for segment in coords {
    //         let mut n = 0;
    //         for xy in segment {
    //             let xy = gc.t(xy);
    //             x.push(xy.0);
    //             y.push(xy.1);
    //             n += 1;
    //         }
    //         nper.push(n);
    //     }

    //     let xptr = x.as_mut_slice().as_mut_ptr();
    //     let yptr = y.as_mut_slice().as_mut_ptr();
    //     let nperptr = nper.as_mut_slice().as_mut_ptr();
    //     unsafe {
    //         GEPath(
    //             xptr,
    //             yptr,
    //             nper.len() as std::os::raw::c_int,
    //             nperptr,
    //             winding.into(),
    //             gc.context(),
    //             self.inner(),
    //         )
    //     }
    // }

    // /// Screen capture. Returns an integer matrix representing pixels if it is able.
    // pub fn capture(&self) -> Robj {
    //     unsafe { Robj::from_sexp(GECap(self.inner())) }
    // }

    // /// Draw a bitmap.
    // pub fn raster<T: AsRef<[u32]>>(
    //     &self,
    //     raster: Raster<T>,
    //     pos: (f64, f64),
    //     size: (f64, f64),
    //     angle: f64,
    //     interpolate: bool,
    //     gc: &Context,
    // ) {
    //     let (x, y) = gc.t(pos);
    //     let (width, height) = gc.trel(size);
    //     let w = raster.width;
    //     let pixels = raster.pixels.as_ref();
    //     let h = pixels.len() / w;
    //     unsafe {
    //         let raster = pixels.as_ptr() as *mut u32;
    //         let w = w as i32;
    //         let h = h as i32;
    //         let interpolate = interpolate.into();
    //         GERaster(
    //             raster,
    //             w,
    //             h,
    //             x,
    //             y,
    //             width,
    //             height,
    //             angle,
    //             interpolate,
    //             gc.context(),
    //             self.inner(),
    //         )
    //     };
    // }

    // /// Draw a text string starting at pos.
    // /// TODO: do we need to convert units?
    // pub fn text<T: AsRef<str>>(
    //     &self,
    //     pos: (f64, f64),
    //     text: T,
    //     center: (f64, f64),
    //     rot: f64,
    //     gc: &Context,
    // ) {
    //     unsafe {
    //         let (x, y) = gc.t(pos);
    //         let (xc, yc) = gc.trel(center);
    //         let text = std::ffi::CString::new(text.as_ref()).unwrap();
    //         let enc = cetype_t::CE_UTF8;
    //         GEText(
    //             x,
    //             y,
    //             text.as_ptr(),
    //             enc,
    //             xc,
    //             yc,
    //             rot,
    //             gc.context(),
    //             self.inner(),
    //         );
    //     }
    // }

    // /// Draw a special symbol centered on pos.
    // /// See <https://stat.ethz.ch/R-manual/R-devel/library/graphics/html/points.html>
    // pub fn symbol(&self, pos: (f64, f64), symbol: i32, size: f64, gc: &Context) {
    //     unsafe {
    //         let (x, y) = gc.t(pos);
    //         GESymbol(x, y, symbol, gc.ts(size), gc.context(), self.inner());
    //     }
    // }

    // /// Get the metrics for a single unicode codepoint.
    // pub fn char_metric(&self, c: char, gc: &Context) -> TextMetric {
    //     unsafe {
    //         let mut res = TextMetric {
    //             ascent: 0.0,
    //             descent: 0.0,
    //             width: 0.0,
    //         };
    //         GEMetricInfo(
    //             c as i32,
    //             gc.context(),
    //             &mut res.ascent as *mut f64,
    //             &mut res.descent as *mut f64,
    //             &mut res.width as *mut f64,
    //             self.inner(),
    //         );
    //         gc.tmetric(res)
    //     }
    // }

    // /// Get the width of a unicode string.
    // pub fn text_width<T: AsRef<str>>(&self, text: T, gc: &Context) -> f64 {
    //     let text = std::ffi::CString::new(text.as_ref()).unwrap();
    //     let enc = cetype_t::CE_UTF8;
    //     unsafe { gc.its(GEStrWidth(text.as_ptr(), enc, gc.context(), self.inner())) }
    // }

    // /// Get the height of a unicode string.
    // pub fn text_height<T: AsRef<str>>(&self, text: T, gc: &Context) -> f64 {
    //     let text = std::ffi::CString::new(text.as_ref()).unwrap();
    //     let enc = cetype_t::CE_UTF8;
    //     unsafe { gc.its(GEStrHeight(text.as_ptr(), enc, gc.context(), self.inner())) }
    // }

    // /// Get the metrics for a unicode string.
    // pub fn text_metric<T: AsRef<str>>(&self, text: T, gc: &Context) -> TextMetric {
    //     let text = std::ffi::CString::new(text.as_ref()).unwrap();
    //     let enc = cetype_t::CE_UTF8;
    //     unsafe {
    //         let mut res = TextMetric {
    //             ascent: 0.0,
    //             descent: 0.0,
    //             width: 0.0,
    //         };
    //         GEStrMetric(
    //             text.as_ptr(),
    //             enc,
    //             gc.context(),
    //             &mut res.ascent as *mut f64,
    //             &mut res.descent as *mut f64,
    //             &mut res.width as *mut f64,
    //             self.inner(),
    //         );
    //         gc.tmetric(res)
    //     }
    // }

    // /// Get the width of a mathematical expression.
    // pub fn math_text_width(&self, expr: &Robj, gc: &Context) -> f64 {
    //     unsafe { gc.its(GEExpressionWidth(expr.get(), gc.context(), self.inner())) }
    // }

    // /// Get the height of a mathematical expression.
    // pub fn math_text_height(&self, expr: &Robj, gc: &Context) -> f64 {
    //     unsafe { gc.its(GEExpressionHeight(expr.get(), gc.context(), self.inner())) }
    // }

    // /// Get the metrics for a mathematical expression.
    // pub fn math_text_metric(&self, expr: &Robj, gc: &Context) -> TextMetric {
    //     unsafe {
    //         let mut res = TextMetric {
    //             ascent: 0.0,
    //             descent: 0.0,
    //             width: 0.0,
    //         };
    //         GEExpressionMetric(
    //             expr.get(),
    //             gc.context(),
    //             &mut res.ascent as *mut f64,
    //             &mut res.descent as *mut f64,
    //             &mut res.width as *mut f64,
    //             self.inner(),
    //         );
    //         gc.tmetric(res)
    //     }
    // }

    // /// Draw a mathematical expression.
    // pub fn math_text(
    //     &self,
    //     expr: &Robj,
    //     pos: (f64, f64),
    //     center: (f64, f64),
    //     rot: f64,
    //     gc: &Context,
    // ) {
    //     unsafe {
    //         let (x, y) = gc.t(pos);
    //         let (xc, yc) = gc.trel(center);
    //         GEMathText(x, y, expr.get(), xc, yc, rot, gc.context(), self.inner());
    //     }
    // }
}
