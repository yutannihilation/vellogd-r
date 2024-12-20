#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

// redefine necessary symbols. If this gets too many, probably I should use
// savvy-ffi

pub type SEXP = *mut c_void;
pub type R_xlen_t = usize;
pub type SEXPTYPE = c_uint;

pub const INTSXP: SEXPTYPE = 13;

extern "C" {
    pub static mut R_NilValue: SEXP;
    pub static mut R_GlobalEnv: SEXP;
    pub static mut R_EmptyEnv: SEXP;

    pub fn Rf_xlength(arg1: SEXP) -> R_xlen_t;

    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP;

    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    pub fn Rf_unprotect(arg1: c_int);
    pub fn Rf_allocVector(arg1: SEXPTYPE, arg2: R_xlen_t) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut c_int;
    pub fn Rf_ScalarInteger(arg1: c_int) -> SEXP;
    pub static mut R_NaInt: c_int;

    pub fn Rf_lang1(arg1: SEXP) -> SEXP;
    pub fn Rf_eval(arg1: SEXP, arg2: SEXP) -> SEXP;
}

// TODO: do not include GE version
pub const R_GE_version: u32 = 16;

extern "C" {
    pub fn R_GE_checkVersionOrDie(version: c_int);
    pub fn R_CheckDeviceAvailable();
}

pub const LTY_BLANK: i32 = -1;
pub const LTY_SOLID: i32 = 0;
pub const LTY_DASHED: i32 = 68;
pub const LTY_DOTTED: i32 = 49;
pub const LTY_DOTDASH: i32 = 13361;
pub const LTY_LONGDASH: i32 = 55;
pub const LTY_TWODASH: i32 = 9762;

pub const Rboolean_FALSE: Rboolean = 0;
pub const Rboolean_TRUE: Rboolean = 1;
pub type Rboolean = c_int;

pub const GEUnit_GE_DEVICE: GEUnit = 0;
pub const GEUnit_GE_NDC: GEUnit = 1;
pub const GEUnit_GE_INCHES: GEUnit = 2;
pub const GEUnit_GE_CM: GEUnit = 3;
pub type GEUnit = c_int;

pub const GEevent_GE_InitState: GEevent = 0;
pub const GEevent_GE_FinaliseState: GEevent = 1;
pub const GEevent_GE_SaveState: GEevent = 2;
pub const GEevent_GE_RestoreState: GEevent = 6;
pub const GEevent_GE_CopyState: GEevent = 3;
pub const GEevent_GE_SaveSnapshotState: GEevent = 4;
pub const GEevent_GE_RestoreSnapshotState: GEevent = 5;
pub const GEevent_GE_CheckPlot: GEevent = 7;
pub const GEevent_GE_ScalePS: GEevent = 8;
pub type GEevent = c_int;

pub const R_GE_lineend_GE_ROUND_CAP: R_GE_lineend = 1;
pub const R_GE_lineend_GE_BUTT_CAP: R_GE_lineend = 2;
pub const R_GE_lineend_GE_SQUARE_CAP: R_GE_lineend = 3;
pub type R_GE_lineend = c_int;

pub const R_GE_linejoin_GE_ROUND_JOIN: R_GE_linejoin = 1;
pub const R_GE_linejoin_GE_MITRE_JOIN: R_GE_linejoin = 2;
pub const R_GE_linejoin_GE_BEVEL_JOIN: R_GE_linejoin = 3;
pub type R_GE_linejoin = c_int;

// capabilities
pub const R_GE_capability_semiTransparency: R_xlen_t = 0;
pub const R_GE_capability_transparentBackground: R_xlen_t = 1;
pub const R_GE_capability_rasterImage: R_xlen_t = 2;
pub const R_GE_capability_capture: R_xlen_t = 3;
pub const R_GE_capability_locator: R_xlen_t = 4;
pub const R_GE_capability_events: R_xlen_t = 5;
pub const R_GE_capability_patterns: R_xlen_t = 6;
pub const R_GE_capability_clippingPaths: R_xlen_t = 7;
pub const R_GE_capability_masks: R_xlen_t = 8;
pub const R_GE_capability_compositing: R_xlen_t = 9;
pub const R_GE_capability_transformations: R_xlen_t = 10;
pub const R_GE_capability_paths: R_xlen_t = 11;
pub const R_GE_capability_glyphs: R_xlen_t = 12;

// style
pub const R_GE_text_style_normal: u32 = 1;
pub const R_GE_text_style_italic: u32 = 2;
pub const R_GE_text_style_oblique: u32 = 3;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct R_GE_gcontext {
    pub col: c_uint,  // uint (u32) is easiler to use on Rust's side
    pub fill: c_uint, // uint (u32) is easiler to use on Rust's side
    pub gamma: f64,
    pub lwd: f64,
    pub lty: c_int,
    pub lend: R_GE_lineend,
    pub ljoin: R_GE_linejoin,
    pub lmitre: f64,
    pub cex: f64,
    pub ps: f64,
    pub lineheight: f64,
    pub fontface: c_int,
    pub fontfamily: [c_char; 201usize],
    pub patternFill: SEXP,
}

pub type pGEcontext = *mut R_GE_gcontext;
pub type DevDesc = _DevDesc;
pub type pDevDesc = *mut DevDesc;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DevDesc {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
    pub top: f64,
    pub clipLeft: f64,
    pub clipRight: f64,
    pub clipBottom: f64,
    pub clipTop: f64,
    pub xCharOffset: f64,
    pub yCharOffset: f64,
    pub yLineBias: f64,
    pub ipr: [f64; 2usize],
    pub cra: [f64; 2usize],
    pub gamma: f64,
    pub canClip: Rboolean,
    pub canChangeGamma: Rboolean,
    pub canHAdj: c_int,
    pub startps: f64,
    pub startcol: c_uint,  // uint (u32) is easiler to use on Rust's side
    pub startfill: c_uint, // uint (u32) is easiler to use on Rust's side
    pub startlty: c_int,
    pub startfont: c_int,
    pub startgamma: f64,
    pub deviceSpecific: *mut ::std::os::raw::c_void,
    pub displayListOn: Rboolean,
    pub canGenMouseDown: Rboolean,
    pub canGenMouseMove: Rboolean,
    pub canGenMouseUp: Rboolean,
    pub canGenKeybd: Rboolean,
    pub canGenIdle: Rboolean,
    pub gettingEvent: Rboolean,
    pub activate: Option<unsafe extern "C" fn(arg1: pDevDesc)>,
    pub circle: Option<unsafe extern "C" fn(x: f64, y: f64, r: f64, gc: pGEcontext, dd: pDevDesc)>,
    pub clip: Option<unsafe extern "C" fn(x0: f64, x1: f64, y0: f64, y1: f64, dd: pDevDesc)>,
    pub close: Option<unsafe extern "C" fn(dd: pDevDesc)>,
    pub deactivate: Option<unsafe extern "C" fn(arg1: pDevDesc)>,
    pub locator: Option<unsafe extern "C" fn(x: *mut f64, y: *mut f64, dd: pDevDesc) -> Rboolean>,
    pub line: Option<
        unsafe extern "C" fn(x1: f64, y1: f64, x2: f64, y2: f64, gc: pGEcontext, dd: pDevDesc),
    >,
    pub metricInfo: Option<
        unsafe extern "C" fn(
            c: c_int,
            gc: pGEcontext,
            ascent: *mut f64,
            descent: *mut f64,
            width: *mut f64,
            dd: pDevDesc,
        ),
    >,
    pub mode: Option<unsafe extern "C" fn(mode: c_int, dd: pDevDesc)>,
    pub newPage: Option<unsafe extern "C" fn(gc: pGEcontext, dd: pDevDesc)>,
    pub polygon: Option<
        unsafe extern "C" fn(n: c_int, x: *mut f64, y: *mut f64, gc: pGEcontext, dd: pDevDesc),
    >,
    pub polyline: Option<
        unsafe extern "C" fn(n: c_int, x: *mut f64, y: *mut f64, gc: pGEcontext, dd: pDevDesc),
    >,
    pub rect: Option<
        unsafe extern "C" fn(x0: f64, y0: f64, x1: f64, y1: f64, gc: pGEcontext, dd: pDevDesc),
    >,
    pub path: Option<
        unsafe extern "C" fn(
            x: *mut f64,
            y: *mut f64,
            npoly: c_int,
            nper: *mut c_int,
            winding: Rboolean,
            gc: pGEcontext,
            dd: pDevDesc,
        ),
    >,
    pub raster: Option<
        unsafe extern "C" fn(
            raster: *mut c_uint,
            w: c_uint,
            h: c_uint,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
            rot: f64,
            interpolate: Rboolean,
            gc: pGEcontext,
            dd: pDevDesc,
        ),
    >,
    pub cap: Option<unsafe extern "C" fn(dd: pDevDesc) -> SEXP>,
    pub size: Option<
        unsafe extern "C" fn(
            left: *mut f64,
            right: *mut f64,
            bottom: *mut f64,
            top: *mut f64,
            dd: pDevDesc,
        ),
    >,
    pub strWidth:
        Option<unsafe extern "C" fn(str_: *const c_char, gc: pGEcontext, dd: pDevDesc) -> f64>,
    pub text: Option<
        unsafe extern "C" fn(
            x: f64,
            y: f64,
            str_: *const c_char,
            rot: f64,
            hadj: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ),
    >,
    pub onExit: Option<unsafe extern "C" fn(dd: pDevDesc)>,
    pub getEvent: Option<unsafe extern "C" fn(arg1: SEXP, arg2: *const c_char) -> SEXP>,
    pub newFrameConfirm: Option<unsafe extern "C" fn(dd: pDevDesc) -> Rboolean>,
    pub hasTextUTF8: Rboolean,
    pub textUTF8: Option<
        unsafe extern "C" fn(
            x: f64,
            y: f64,
            str_: *const c_char,
            rot: f64,
            hadj: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ),
    >,
    pub strWidthUTF8:
        Option<unsafe extern "C" fn(str_: *const c_char, gc: pGEcontext, dd: pDevDesc) -> f64>,
    pub wantSymbolUTF8: Rboolean,
    pub useRotatedTextInContour: Rboolean,
    pub eventEnv: SEXP,
    pub eventHelper: Option<unsafe extern "C" fn(dd: pDevDesc, code: c_int)>,
    pub holdflush: Option<unsafe extern "C" fn(dd: pDevDesc, level: c_int) -> c_int>,
    pub haveTransparency: c_int,
    pub haveTransparentBg: c_int,
    pub haveRaster: c_int,
    pub haveCapture: c_int,
    pub haveLocator: c_int,
    pub setPattern: Option<unsafe extern "C" fn(pattern: SEXP, dd: pDevDesc) -> SEXP>,
    pub releasePattern: Option<unsafe extern "C" fn(ref_: SEXP, dd: pDevDesc)>,
    pub setClipPath: Option<unsafe extern "C" fn(path: SEXP, ref_: SEXP, dd: pDevDesc) -> SEXP>,
    pub releaseClipPath: Option<unsafe extern "C" fn(ref_: SEXP, dd: pDevDesc)>,
    pub setMask: Option<unsafe extern "C" fn(path: SEXP, ref_: SEXP, dd: pDevDesc) -> SEXP>,
    pub releaseMask: Option<unsafe extern "C" fn(ref_: SEXP, dd: pDevDesc)>,
    pub deviceVersion: c_int,
    pub deviceClip: Rboolean,
    pub defineGroup: Option<
        unsafe extern "C" fn(source: SEXP, op: c_int, destination: SEXP, dd: pDevDesc) -> SEXP,
    >,
    pub useGroup: Option<unsafe extern "C" fn(ref_: SEXP, trans: SEXP, dd: pDevDesc)>,
    pub releaseGroup: Option<unsafe extern "C" fn(ref_: SEXP, dd: pDevDesc)>,
    pub stroke: Option<unsafe extern "C" fn(path: SEXP, gc: pGEcontext, dd: pDevDesc)>,
    pub fill: Option<unsafe extern "C" fn(path: SEXP, rule: c_int, gc: pGEcontext, dd: pDevDesc)>,
    pub fillStroke:
        Option<unsafe extern "C" fn(path: SEXP, rule: c_int, gc: pGEcontext, dd: pDevDesc)>,
    pub capabilities: Option<unsafe extern "C" fn(cap: SEXP) -> SEXP>,
    pub glyph: Option<
        unsafe extern "C" fn(
            n: c_int,
            glyphs: *mut c_uint,
            x: *mut f64,
            y: *mut f64,
            font: SEXP,
            size: f64,
            colour: c_uint,
            rot: f64,
            dd: pDevDesc,
        ),
    >,
    pub reserved: [c_char; 64usize],
}

pub type GEDevDesc = _GEDevDesc;
pub type GEcallback =
    Option<unsafe extern "C" fn(arg1: GEevent, arg2: *mut GEDevDesc, arg3: SEXP) -> SEXP>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GESystemDesc {
    pub systemSpecific: *mut c_void,
    pub callback: GEcallback,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _GEDevDesc {
    pub dev: pDevDesc,
    pub displayListOn: Rboolean,
    pub displayList: SEXP,
    pub DLlastElt: SEXP,
    pub savedSnapshot: SEXP,
    pub dirty: Rboolean,
    pub recordGraphics: Rboolean,
    pub gesd: [*mut GESystemDesc; 24usize],
    pub ask: Rboolean,
    pub appending: Rboolean,
}
pub type pGEDevDesc = *mut GEDevDesc;

// pattern
pub const R_GE_linearGradientPattern: u32 = 1;
pub const R_GE_radialGradientPattern: u32 = 2;
pub const R_GE_tilingPattern: u32 = 3;

pub const R_GE_patternExtendPad: u32 = 1;
pub const R_GE_patternExtendRepeat: u32 = 2;
pub const R_GE_patternExtendReflect: u32 = 3;
pub const R_GE_patternExtendNone: u32 = 4;

extern "C" {
    pub fn GEfromDeviceX(value: f64, to: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEtoDeviceX(value: f64, from: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEfromDeviceY(value: f64, to: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEtoDeviceY(value: f64, from: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEfromDeviceWidth(value: f64, to: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEtoDeviceWidth(value: f64, from: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEfromDeviceHeight(value: f64, to: GEUnit, dd: pGEDevDesc) -> f64;
    pub fn GEtoDeviceHeight(value: f64, from: GEUnit, dd: pGEDevDesc) -> f64;

    pub fn GEcreateDevDesc(dev: pDevDesc) -> pGEDevDesc;
    pub fn GEinitDisplayList(dd: pGEDevDesc);
    pub fn GEaddDevice2(arg1: pGEDevDesc, arg2: *const c_char);

    // pattern
    pub fn R_GE_patternType(pattern: SEXP) -> c_int;

    // linear gradient
    pub fn R_GE_linearGradientX1(pattern: SEXP) -> f64;
    pub fn R_GE_linearGradientY1(pattern: SEXP) -> f64;
    pub fn R_GE_linearGradientX2(pattern: SEXP) -> f64;
    pub fn R_GE_linearGradientY2(pattern: SEXP) -> f64;
    pub fn R_GE_linearGradientNumStops(pattern: SEXP) -> c_int;
    pub fn R_GE_linearGradientStop(pattern: SEXP, i: c_int) -> f64;
    pub fn R_GE_linearGradientColour(pattern: SEXP, i: c_int) -> c_uint;
    pub fn R_GE_linearGradientExtend(pattern: SEXP) -> c_int;

    // radial gradient
    pub fn R_GE_radialGradientCX1(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientCY1(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientR1(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientCX2(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientCY2(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientR2(pattern: SEXP) -> f64;
    pub fn R_GE_radialGradientNumStops(pattern: SEXP) -> c_int;
    pub fn R_GE_radialGradientStop(pattern: SEXP, i: c_int) -> f64;
    pub fn R_GE_radialGradientColour(pattern: SEXP, i: c_int) -> c_uint;
    pub fn R_GE_radialGradientExtend(pattern: SEXP) -> c_int;

    // tiling
    pub fn R_GE_tilingPatternFunction(pattern: SEXP) -> SEXP;
    pub fn R_GE_tilingPatternX(pattern: SEXP) -> f64;
    pub fn R_GE_tilingPatternY(pattern: SEXP) -> f64;
    pub fn R_GE_tilingPatternWidth(pattern: SEXP) -> f64;
    pub fn R_GE_tilingPatternHeight(pattern: SEXP) -> f64;
    pub fn R_GE_tilingPatternExtend(pattern: SEXP) -> c_int;

    // glyph
    pub fn R_GE_glyphFontFile(glyphFont: SEXP) -> *const c_char;
    pub fn R_GE_glyphFontIndex(glyphFont: SEXP) -> c_int;
    pub fn R_GE_glyphFontFamily(glyphFont: SEXP) -> *const c_char;
    pub fn R_GE_glyphFontWeight(glyphFont: SEXP) -> f64;
    pub fn R_GE_glyphFontStyle(glyphFont: SEXP) -> c_int;
}
