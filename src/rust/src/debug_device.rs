use crate::graphics::DeviceDriver;

use vellogd_shared::ffi::DevDesc;
use vellogd_shared::ffi::R_GE_gcontext;
use vellogd_shared::ffi::R_NilValue;
use vellogd_shared::text_layouter::TextMetric;

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

pub struct DebugGraphicsDevice {}

impl DeviceDriver for DebugGraphicsDevice {
    fn activate(&mut self, _: DevDesc) {
        savvy::r_eprintln!("[activate]");
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!(
            "[circle] center: {center:?} r: {r}, fill params: {{ {} }}, line params: {{ {} }}",
            fill_related_params(gc),
            line_related_params(gc)
        );
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {
        savvy::r_eprintln!("[clip] from: {from:?}, to: {to:?}");
    }

    fn close(&mut self, _: DevDesc) {
        savvy::r_eprintln!("[close]");
    }

    fn deactivate(&mut self, _: DevDesc) {
        savvy::r_eprintln!("[deactivate]");
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!(
            "[line] from: {from:?}, to: {to:?}, line params: {{ {} }}",
            line_related_params(gc)
        );
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        savvy::r_eprintln!(
            "[char_metric] c: {c:?}, text params: {{ {} }}",
            text_related_params(gc)
        );

        TextMetric {
            ascent: 0.0,
            descent: 0.0,
            width: 0.0,
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {
        savvy::r_eprintln!("[mode] mode: {mode}");
    }

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!("[new_page]");
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!("[polygon]");
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!("[polyline]");
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        savvy::r_eprintln!("[rect]");
    }

    fn path(
        &mut self,
        x: &[f64],
        y: &[f64],
        nper: &[i32],
        winding: bool,
        gc: R_GE_gcontext,
        dd: DevDesc,
    ) {
        savvy::r_eprintln!("[path] nper: {nper:?}");
    }

    fn raster<T: AsRef<[u32]>>(
        &mut self,
        raster: crate::graphics::Raster<T>,
        pos: (f64, f64),
        size: (f64, f64),
        angle: f64,
        interpolate: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        savvy::r_eprintln!("[raster]");
    }

    fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
        savvy::r_eprintln!("[capture]");

        unsafe { R_NilValue }
    }

    fn size(&mut self, dd: DevDesc) -> (f64, f64, f64, f64) {
        savvy::r_eprintln!("[size]");

        (dd.left, dd.right, dd.bottom, dd.top)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, dd: DevDesc) -> f64 {
        savvy::r_eprintln!("[text_width]");

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
        savvy::r_eprintln!("[text] text: {text}");
    }

    fn on_exit(&mut self, _: DevDesc) {
        savvy::r_eprintln!("[on_exit]");
    }

    fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
        savvy::r_eprintln!("[new_frame_confirm]");

        true
    }

    fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
        savvy::r_eprintln!("[holdflush] level: {level}");

        0
    }

    fn locator(&mut self, x: *mut f64, y: *mut f64, _: DevDesc) -> bool {
        savvy::r_eprintln!("[locator]");

        true
    }

    fn eventHelper(&mut self, _: DevDesc, code: i32) {
        savvy::r_eprintln!("[eventHelper] code {code}");
    }
}
