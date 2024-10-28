use std::os::raw::c_uint;

use crate::add_tracing_point;
use crate::graphics::DeviceDriver;

use vellogd_shared::ffi::*;
use vellogd_shared::text_layouter::TextMetric;

#[cfg(debug_assertions)]
fn fill_related_params(gc: R_GE_gcontext) -> String {
    let pattern_fill = unsafe {
        if gc.patternFill != R_NilValue {
            (*INTEGER(gc.patternFill)).to_string()
        } else {
            "NULL".to_string()
        }
    };
    format!("fill: {:08x}, patternFill: {pattern_fill}", gc.fill)
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
    let family = unsafe {
        std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
            .to_str()
            .unwrap_or("(empty)")
    }
    .to_string();
    format!("color: {:08x}, family: {family}", gc.col)
}

pub struct DebugGraphicsDevice {
    n_clip: i32,
    n_pattern: i32,
}

impl DebugGraphicsDevice {
    pub fn new() -> Self {
        Self {
            n_clip: 0,
            n_pattern: 0,
        }
    }
}

fn take3(x: &[f64]) -> String {
    if x.len() < 3 {
        return format!("{x:?}");
    }

    let x = x
        .iter()
        .take(3)
        .map(|x| format!("{x:.1}"))
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
        savvy::r_eprintln!("[new_page] fill params: {{ {} }}", fill_related_params(gc));
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[polygon] x: {} y: {} fill params: {{ {} }}, line params: {{ {} }}",
            take3(x),
            take3(y),
            fill_related_params(gc),
            line_related_params(gc),
        );
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[polyline] x: {} y: {},  line params: {{ {} }}",
            take3(x),
            take3(y),
            line_related_params(gc),
        );
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[rect] from: {from:?} to: {to:?} fill params: {{ {} }}, line params: {{ {} }}",
            fill_related_params(gc),
            line_related_params(gc),
        );
    }

    fn path(
        &mut self,
        _x: &[f64],
        _y: &[f64],
        nper: &[i32],
        _winding: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();
        savvy::r_eprintln!(
            "[path] nper: {nper:?} fill params: {{ {} }}, line params: {{ {} }}",
            fill_related_params(gc),
            line_related_params(gc),
        );
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

    fn glyph(
        &mut self,
        glyphs: &[u32],
        x: &[f64],
        y: &[f64],
        fontfile: &str,
        index: i32,
        family: &str,
        weight: f64,
        style: i32,
        angle: f64,
        size: f64,
        colour: c_uint,
    ) {
        add_tracing_point!();

        let fontfile = std::fs::canonicalize(fontfile);
        savvy::r_eprintln!(
            "[glyph]
  glyphs: {glyphs:?}
  x: {x:?}
  y: {y:?}
  fontfile: {fontfile:?}
  index: {index}
  family: {family}
  weight: {weight}
  style: {style}
  angle: {angle}
  size: {size}
  colour: {colour}
"
        );
    }

    fn set_pattern(&mut self, pattern: SEXP, _: DevDesc) -> SEXP {
        unsafe {
            if pattern == R_NilValue {
                return Rf_ScalarInteger(-1);
            }
        }

        match unsafe { R_GE_patternType(pattern) } {
            1 => unsafe {
                let x1 = R_GE_linearGradientX1(pattern);
                let y1 = R_GE_linearGradientY1(pattern);
                let x2 = R_GE_linearGradientX2(pattern);
                let y2 = R_GE_linearGradientY2(pattern);
                let extend = R_GE_linearGradientExtend(pattern);
                savvy::r_eprintln!(
                    "[setPattern] linearGradient
  from: ({x1}, {y1})
  to:   ({x2}, {y2})
  extend: {extend}"
                );

                let num_stops = R_GE_linearGradientNumStops(pattern);
                savvy::r_eprintln!("  stops:");

                for i in 0..num_stops {
                    let stop = R_GE_linearGradientStop(pattern, i);
                    let color = R_GE_linearGradientColour(pattern, i);
                    savvy::r_eprintln!("    {i}: {stop},{color:08x}");
                }
            },
            2 => unsafe {
                let cx1 = R_GE_radialGradientCX1(pattern);
                let cy1 = R_GE_radialGradientCY1(pattern);
                let r1 = R_GE_radialGradientR1(pattern);
                let cx2 = R_GE_radialGradientCX2(pattern);
                let cy2 = R_GE_radialGradientCY2(pattern);
                let r2 = R_GE_radialGradientR2(pattern);
                let extend = R_GE_radialGradientExtend(pattern);
                savvy::r_eprintln!(
                    "[setPattern] radialGradient
  from: ({cx1}, {cy1}), r: {r1}
  to:   ({cx2}, {cy2}), r: {r2}
  extend: {extend}"
                );

                let num_stops = R_GE_radialGradientNumStops(pattern);
                savvy::r_eprintln!("  stops:");

                for i in 0..num_stops {
                    let stop = R_GE_radialGradientStop(pattern, i);
                    let color = R_GE_radialGradientColour(pattern, i);
                    savvy::r_eprintln!("    {i}: {stop},{color:08x}");
                }
            },
            3 => unsafe {
                let x = R_GE_tilingPatternX(pattern);
                let y = R_GE_tilingPatternY(pattern);
                let w = R_GE_tilingPatternWidth(pattern);
                let h = R_GE_tilingPatternHeight(pattern);
                let extend = R_GE_tilingPatternExtend(pattern);
                savvy::r_eprintln!(
                    "[setPattern] tilingPattern
  pos:  ({x}, {y})
  size: ({w}, {h})
  extend: {extend}

=== pattern function ==============
"
                );

                let fun = R_GE_tilingPatternFunction(pattern);
                let call = Rf_protect(Rf_lang1(fun));
                Rf_eval(call, R_GlobalEnv);
                Rf_unprotect(1);

                savvy::r_eprintln!("=== pattern function end ==========")
            },
            _ => {}
        }

        unsafe {
            let pattern_id = self.n_pattern;
            self.n_pattern += 1;
            Rf_ScalarInteger(pattern_id)
        }
    }

    fn release_pattern(&mut self, ref_: SEXP, _: DevDesc) {
        savvy::r_eprintln!("[releasePattern]");

        unsafe {
            if ref_ != R_NilValue {
                savvy::r_eprintln!("  index: {}", *INTEGER(ref_));
            }
        }
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
