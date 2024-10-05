#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_void};

use savvy::ffi::SEXP;
extern "C" {
    pub static mut R_NilValue: SEXP;
}

// TODO: do not include GE version
pub const R_GE_version: u32 = 16;
pub const R_GE_definitions: u32 = 13;

extern "C" {
    pub fn R_GE_checkVersionOrDie(version: c_int);
    pub fn R_CheckDeviceAvailable();

    pub static mut R_GlobalEnv: SEXP;
    pub static mut R_EmptyEnv: SEXP;
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct R_GE_gcontext {
    pub col: c_int,
    pub fill: c_int,
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
    pub startcol: c_int,
    pub startfill: c_int,
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
            raster: *mut ::std::os::raw::c_uint,
            w: c_int,
            h: c_int,
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
            glyphs: *mut c_int,
            x: *mut f64,
            y: *mut f64,
            font: SEXP,
            size: f64,
            colour: c_int,
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
}
