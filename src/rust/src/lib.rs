mod graphics;
use graphics::ClippingStrategy;
use graphics::DevDesc;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use graphics::R_GE_gcontext;
use graphics::R_NilValue;
use savvy::savvy;

pub struct VelloGraphicsDevice {}

impl VelloGraphicsDevice {
    pub fn new(filename: &str, width: u32, height: u32) -> Self {
        Self {}
    }
}

#[cfg(debug_assertions)]
fn fill_related_params(gc: R_GE_gcontext) -> String {
    format!("fill: {:08x}", gc.fill)
}

#[cfg(debug_assertions)]
fn line_related_params(gc: R_GE_gcontext) -> String {
    format!(
        "color: {:08x}, linewidth: {}, line type: {},  cap: {}, join: {}, mitre: {}",
        gc.col, gc.lwd, gc.lty, gc.lend, gc.ljoin, gc.lmitre
    )
}

#[cfg(debug_assertions)]
fn text_related_params(gc: R_GE_gcontext) -> String {
    format!("fill: {:08x}", gc.fill)
}

impl DeviceDriver for VelloGraphicsDevice {
    const USE_RASTER: bool = true;

    const USE_CAPTURE: bool = true;

    const USE_LOCATOR: bool = true;

    const USE_PLOT_HISTORY: bool = false;

    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::DeviceAndEngine;

    const ACCEPT_UTF8_TEXT: bool = true;

    fn activate(&mut self, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[activate]");
        }
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!(
                "[circle] center: {center:?} r: {r}, fill params: {{ {} }}, line params: {{ {} }}",
                fill_related_params(gc),
                line_related_params(gc)
            );
        }
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[clip] from: {from:?}, to: {to:?}");
        }
    }

    fn close(&mut self, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[close]");
        }
    }

    fn deactivate(&mut self, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[deactivate]");
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!(
                "[line] from: {from:?}, to: {to:?}, line params: {{ {} }}",
                line_related_params(gc)
            );
        }
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> graphics::TextMetric {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!(
                "[char_metric] c: {c:?}, text params: {{ {} }}",
                text_related_params(gc)
            );
        }

        graphics::TextMetric {
            ascent: 0.0,
            descent: 0.0,
            width: 0.0,
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[mode] mode: {mode}");
        }
    }

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[new_page]");
        }
    }

    fn polygon<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        if cfg!(debug_assertions) {
            savvy::r_eprintln!("[polygon]");
        }
    }

    fn polyline<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {}

    fn path<T: IntoIterator<Item = impl IntoIterator<Item = (f64, f64)>>>(
        &mut self,
        coords: T,
        winding: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
    }

    fn raster<T: AsRef<[u32]>>(
        &mut self,
        raster: graphics::Raster<T>,
        pos: (f64, f64),
        size: (f64, f64),
        angle: f64,
        interpolate: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
    }

    fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
        unsafe { R_NilValue }
    }

    fn size(&mut self, dd: DevDesc) -> (f64, f64, f64, f64) {
        (dd.left, dd.right, dd.bottom, dd.top)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, dd: DevDesc) -> f64 {
        text.chars()
            .map(|c| self.char_metric(c, gc, dd).width)
            .sum()
    }

    fn text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
    }

    fn on_exit(&mut self, _: DevDesc) {}

    fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
        true
    }

    fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
        0
    }

    fn locator(&mut self, x: *mut f64, y: *mut f64, _: DevDesc) -> bool {
        true
    }

    fn eventHelper(&mut self, _: DevDesc, code: i32) {}
}

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
