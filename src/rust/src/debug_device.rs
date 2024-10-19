use crate::add_tracing_point;
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

pub struct DebugGraphicsDevice {
    n_clip: i32,
}

impl DebugGraphicsDevice {
    pub fn new() -> Self {
        Self { n_clip: 0 }
    }
}

fn take3<T: std::fmt::Debug>(x: &[T]) -> String {
    if x.len() < 3 {
        return format!("{x:?}");
    }

    let x = x
        .iter()
        .take(3)
        .map(|x| format!("{x:?}"))
        .collect::<Vec<String>>()
        .join(", ");

    format!("[{x}, ...]")
}

impl DeviceDriver for DebugGraphicsDevice {
    fn activate(&mut self, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[activate]");
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[circle] center: {center:?} r: {r}, fill params: {{ {} }}, line params: {{ {} }}",
            fill_related_params(gc),
            line_related_params(gc)
        );
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), dd: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[clip] from: {from:?}, to: {to:?}");
        if from.0 <= 0.0 && from.1 <= 0.0 && to.0 >= dd.right && to.1 >= dd.top {
            self.n_clip = (self.n_clip - 1).max(0);
            savvy::r_eprintln!("[clip] pop (n_clip: {})", self.n_clip);
        } else {
            self.n_clip += 1;
            savvy::r_eprintln!("[clip] push (n_clip: {})", self.n_clip);
        }
    }

    fn close(&mut self, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[close]");
    }

    fn deactivate(&mut self, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[deactivate]");
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[line] from: {from:?}, to: {to:?}, line params: {{ {} }}",
            line_related_params(gc)
        );
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        add_tracing_point!();
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
        add_tracing_point!();
        savvy::r_eprintln!("[mode] mode: {mode}");
    }

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        let fill = gc.fill;
        savvy::r_eprintln!("[new_page] fill: {fill:#08x}");
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], _: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[polygon] x: {} y: {}", take3(x), take3(y));
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], _: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[polyline] x: {} y: {}", take3(x), take3(y));
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), _: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[rect] from: {from:?} to: {to:?}");
    }

    fn path(
        &mut self,
        _x: &[f64],
        _y: &[f64],
        nper: &[i32],
        _winding: bool,
        _gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();
        savvy::r_eprintln!("[path] nper: {nper:?}");
    }

    fn raster(
        &mut self,
        _raster: &[u8],
        pixels: (u32, u32),
        pos: (f64, f64), // bottom left corner
        size: (f64, f64),
        _angle: f64,
        _interpolate: bool, // ?
        _gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();

        savvy::r_eprintln!(
            "[raster] 
        pixels: {pixels:?}
        pos: {pos:?} 
        size: {size:?}"
        );
    }

    fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
        add_tracing_point!();
        savvy::r_eprintln!("[capture]");

        unsafe { R_NilValue }
    }

    fn size(&mut self, width: &mut f64, height: &mut f64, dd: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[size]");

        *width = dd.right;
        *height = dd.top;
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, dd: DevDesc) -> f64 {
        add_tracing_point!();
        savvy::r_eprintln!("[text_width]");

        text.chars()
            .map(|c| self.char_metric(c, gc, dd).width)
            .sum()
    }

    fn text(
        &mut self,
        _pos: (f64, f64),
        text: &str,
        _angle: f64,
        _hadj: f64,
        _: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();
        savvy::r_eprintln!("[text] text: {text}");
    }

    fn on_exit(&mut self, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!("[on_exit]");
    }

    fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
        add_tracing_point!();
        savvy::r_eprintln!("[new_frame_confirm]");

        true
    }

    fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
        add_tracing_point!();
        savvy::r_eprintln!("[holdflush] level: {level}");

        0
    }

    fn locator(&mut self, _x: *mut f64, _y: *mut f64, _: DevDesc) -> bool {
        add_tracing_point!();
        savvy::r_eprintln!("[locator]");

        true
    }

    fn eventHelper(&mut self, _: DevDesc, code: i32) {
        add_tracing_point!();
        savvy::r_eprintln!("[eventHelper] code {code}");
    }
}
