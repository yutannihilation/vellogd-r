use savvy::ffi::SEXP;
use std::slice;
use std::{ffi::CString, os::raw::c_uint};
use vellogd_shared::ffi::{
    R_GE_linearGradientPattern, R_GE_radialGradientPattern, R_GE_tilingPattern, R_NaInt,
};
use vellogd_shared::{
    ffi::{
        pDevDesc, pGEcontext, DevDesc, GEaddDevice2, GEcreateDevDesc, GEinitDisplayList,
        R_CheckDeviceAvailable, R_EmptyEnv, R_GE_capability_clippingPaths,
        R_GE_capability_compositing, R_GE_capability_glyphs, R_GE_capability_masks,
        R_GE_capability_paths, R_GE_capability_patterns, R_GE_capability_transformations,
        R_GE_checkVersionOrDie, R_GE_gcontext, R_GE_glyphFontFamily, R_GE_glyphFontFile,
        R_GE_glyphFontIndex, R_GE_glyphFontStyle, R_GE_glyphFontWeight, R_GE_version, R_NilValue,
        Rboolean, Rboolean_FALSE, Rboolean_TRUE, Rf_allocVector, Rf_protect, Rf_unprotect, INTEGER,
        INTSXP, SET_VECTOR_ELT,
    },
    text_layouter::TextMetric,
};

use super::device_descriptor::*;

/// A graphic device implementation.
///
/// # Safety
///
/// To implement these callback functions, extreme care is needed to avoid any
/// `panic!()` because it immediately crashes the R session. Usually, extendr
/// handles a panic gracefully, but there's no such protect on the callback
/// functions.
#[allow(non_snake_case, unused_variables, clippy::too_many_arguments)]
pub trait DeviceDriver: std::marker::Sized {
    /// A callback function to setup the device when the device is activated.
    fn activate(&mut self, dd: DevDesc) {}

    /// A callback function to draw a circle.
    ///
    /// The header file[^1] states:
    ///
    /// * The border of the circle should be drawn in the given `col` (i.e. `gc.col`).
    /// * The circle should be filled with the given `fill` (i.e. `gc.fill`) colour.
    /// * If `col` is `NA_INTEGER` then no border should be drawn.
    /// * If `fill` is `NA_INTEGER` then the circle should not be filled.
    ///
    /// [^1]: <https://github.com/wch/r-source/blob/9f284035b7e503aebe4a804579e9e80a541311bb/src/include/R_ext/GraphicsDevice.h#L205-L210>
    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function to clip.
    fn clip(&mut self, from: (f64, f64), to: (f64, f64), dd: DevDesc) {}

    /// A callback function to free device-specific resources when the device is
    /// killed. Note that, `self` MUST NOT be dropped within this function
    /// because the wrapper that extendr internally generates will do it.
    fn close(&mut self, dd: DevDesc) {}

    /// A callback function to clean up when the device is deactivated.
    fn deactivate(&mut self, dd: DevDesc) {}

    /// A callback function to draw a line.
    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function that returns the [TextMetric] (ascent, descent, and width) of the
    /// given character in device unit.
    ///
    /// The default implementation returns `(0, 0, 0)`, following the convention
    /// described in [the header file]:
    ///
    /// > If the device cannot provide metric information then it MUST return
    /// > 0.0 for ascent, descent, and width.
    ///
    /// [The header file]:
    ///     https://github.com/wch/r-source/blob/9bb47ca929c41a133786fa8fff7c70162bb75e50/src/include/R_ext/GraphicsDevice.h#L321-L322
    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, dd: DevDesc) -> TextMetric {
        TextMetric {
            ascent: 0.0,
            descent: 0.0,
            width: 0.0,
        }
    }

    /// A callback function called whenever the graphics engine starts
    /// drawing (mode=1) or stops drawing (mode=0).
    fn mode(&mut self, mode: i32, dd: DevDesc) {}

    /// A callback function called whenever a new plot requires a new page.
    fn new_page(&mut self, gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function to draw a polygon.
    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function to draw a polyline.
    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function to draw a rect.
    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, dd: DevDesc) {}

    /// A callback function to draw paths.
    ///
    /// `nper` contains number of points in each polygon. `winding` represents
    /// the filling rule; `true` means "nonzero", `false` means "evenodd" (c.f.
    /// <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/fill-rule>).
    fn path(
        &mut self,
        x: &[f64],
        y: &[f64],
        nper: &[i32],
        winding: bool,
        gc: R_GE_gcontext,
        dd: DevDesc,
    ) {
    }

    /// A callback function to draw a [Raster].
    ///
    /// `pos` gives the bottom-left corner. `angle` is the rotation in degrees,
    /// with positive rotation anticlockwise from the positive x-axis.
    /// `interpolate` is whether to apply the linear interpolation on the raster
    /// image.
    fn raster(
        &mut self,
        raster: &[u8],
        pixels: (u32, u32),
        pos: (f64, f64),
        size: (f64, f64),
        angle: f64,
        interpolate: bool,
        gc: R_GE_gcontext,
        dd: DevDesc,
    ) {
    }

    /// A callback function that captures and returns the current canvas.
    ///
    /// This is only meaningful for raster devices.
    fn capture(&mut self, dd: DevDesc) -> SEXP {
        unsafe { R_NilValue }
    }

    /// A callback function that returns the current device size in the format
    /// of `(left, right, bottom, top)` in points.
    ///
    /// - If the size of the graphic device won't change after creation, the
    ///   function can simply return the `left`, `right`, `bottom`, and `top` of
    ///   the `DevDesc` (the default implementation).
    /// - If the size can change, probably the actual size should be tracked in
    ///   the device-specific struct, i.e. `self`, and the function should refer
    ///   to the field (e.g., [`cbm_Size()` in the cairo device]).
    ///
    /// Note that, while this function is what is supposed to be called
    /// "whenever the device is resized," it's not automatically done by the
    /// graphic engine. [The header file] states:
    ///
    /// > This is not usually called directly by the graphics engine because the
    /// > detection of device resizes (e.g., a window resize) are usually
    /// > detected by device-specific code.
    ///
    /// [The header file]:
    ///     <https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L508-L527>
    /// [`cbm_Size()` in the cairo device]:
    ///     <https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/library/grDevices/src/cairo/cairoBM.c#L73-L83>
    fn size(&mut self, width: &mut f64, height: &mut f64, dd: DevDesc) {
        *width = dd.right;
        *height = dd.top;
    }

    /// A callback function that returns the width of the given string in the
    /// device units.
    ///
    /// The default implementation use `char_metric()` on each character in the
    /// text and sums the widths. This should be sufficient for most of the
    /// cases, but the developer can choose to implement this. The header
    /// file[^1] suggests the possible reasons:
    ///
    /// - for performance
    /// - to decide what to do when font metric information is not available
    ///
    /// [^1]: <https://github.com/wch/r-source/blob/9bb47ca929c41a133786fa8fff7c70162bb75e50/src/include/R_ext/GraphicsDevice.h#L67-L74>
    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, dd: DevDesc) -> f64 {
        text.chars()
            .map(|c| self.char_metric(c, gc, dd).width)
            .sum()
    }

    /// A callback function to draw a text.
    ///
    /// `angle` is the rotation in degrees, with positive rotation anticlockwise
    /// from the positive x-axis.
    fn text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
        dd: DevDesc,
    ) {
    }

    /// A callback function to draw a glyph.
    ///
    /// cf. https://www.stat.auckland.ac.nz/~paul/Reports/Typography/glyphs/glyphs.html
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
    }

    fn set_pattern(&mut self, pattern: SEXP, dd: DevDesc) -> SEXP {
        unsafe { R_NilValue }
    }

    fn release_pattern(&mut self, pattern: SEXP, dd: DevDesc) {}

    /// A callback function called when the user aborts some operation. It seems
    /// this is rarely implemented.
    fn on_exit(&mut self, dd: DevDesc) {}

    /// A callback function to confirm a new frame. It seems this is rarely
    /// implementad.
    fn new_frame_confirm(&mut self, dd: DevDesc) -> bool {
        true
    }

    /// A callback function to manage the "suspension level" of the device. R
    /// function `dev.hold()` is used to increase the level,  and `dev.flush()`
    /// to decrease it. When the level reaches zero, output is supposed to be
    /// flushed to the device. This is only meaningful for screen devices.
    fn holdflush(&mut self, dd: DevDesc, level: i32) -> i32 {
        0
    }

    /// A callback function that returns the coords of the event
    fn locator(&mut self, x: *mut f64, y: *mut f64, dd: DevDesc) -> bool {
        true
    }

    /// A callback function for X11_eventHelper.
    // TODO:
    // Argument `code` should, ideally, be of type c_int,
    // but compiler throws erors. It should be ok to use
    // i32 here.
    fn eventHelper(&mut self, dd: DevDesc, code: i32) {}

    /// cf. src/library/grDevices/src/devices.c in R's source code
    fn capabilities(cap: SEXP) -> SEXP {
        // patterns
        unsafe {
            let len = 3;
            let patterns = Rf_protect(Rf_allocVector(INTSXP, len));
            *INTEGER(patterns).offset(0) = R_GE_linearGradientPattern as i32;
            *INTEGER(patterns).offset(1) = R_GE_radialGradientPattern as i32;
            *INTEGER(patterns).offset(2) = R_GE_tilingPattern as i32;
            SET_VECTOR_ELT(cap, R_GE_capability_patterns, patterns);
            Rf_unprotect(1);
        }

        // clipping_paths
        unsafe {
            let clipping_paths = Rf_protect(Rf_allocVector(INTSXP, 1));
            *INTEGER(clipping_paths) = R_NaInt;
            SET_VECTOR_ELT(cap, R_GE_capability_clippingPaths, clipping_paths);
            Rf_unprotect(1);
        }

        // masks
        unsafe {
            let len = 1; // TODO: 2
            let masks = Rf_protect(Rf_allocVector(INTSXP, len));
            *INTEGER(masks).offset(0) = R_NaInt;
            // *INTEGER(masks).offset(1) = R_NaInt;
            SET_VECTOR_ELT(cap, R_GE_capability_masks, masks);
            Rf_unprotect(1);
        }

        // compositing
        unsafe {
            let len = 1; // TODO: 11
            let compositing = Rf_protect(Rf_allocVector(INTSXP, len));
            *INTEGER(compositing).offset(0) = R_NaInt;
            // *INTEGER(compositing).offset(1) = R_NaInt;
            // *INTEGER(compositing).offset(2) = R_NaInt;
            // *INTEGER(compositing).offset(3) = R_NaInt;
            // *INTEGER(compositing).offset(4) = R_NaInt;
            // *INTEGER(compositing).offset(5) = R_NaInt;
            // *INTEGER(compositing).offset(6) = R_NaInt;
            // *INTEGER(compositing).offset(7) = R_NaInt;
            // *INTEGER(compositing).offset(8) = R_NaInt;
            // *INTEGER(compositing).offset(9) = R_NaInt;
            // *INTEGER(compositing).offset(10) = R_NaInt;
            SET_VECTOR_ELT(cap, R_GE_capability_compositing, compositing);
            Rf_unprotect(1);
        }

        // transforms
        unsafe {
            let transforms = Rf_protect(Rf_allocVector(INTSXP, 1));
            *INTEGER(transforms) = 0;
            SET_VECTOR_ELT(cap, R_GE_capability_transformations, transforms);
            Rf_unprotect(1);
        }

        // paths
        unsafe {
            let paths = Rf_protect(Rf_allocVector(INTSXP, 1));
            *INTEGER(paths) = 0;
            SET_VECTOR_ELT(cap, R_GE_capability_paths, paths);
            Rf_unprotect(1);
        }

        // glyphs
        unsafe {
            let glyphs = Rf_protect(Rf_allocVector(INTSXP, 1));
            *INTEGER(glyphs) = 1;
            SET_VECTOR_ELT(cap, R_GE_capability_glyphs, glyphs);
            Rf_unprotect(1);
        }

        cap
    }

    /// Create a [Device].
    fn create_device<T: DeviceDriver>(
        self,
        device_descriptor: DeviceDescriptor,
        device_name: &'static str,
    ) -> savvy::Result<()> {
        #![allow(non_snake_case)]
        #![allow(unused_variables)]
        use std::os::raw::{c_char, c_int, c_uint};

        // The code here is a Rust interpretation of the C-version of example
        // code on the R Internals:
        //
        // https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Device-structures

        unsafe {
            // Check the API version
            R_GE_checkVersionOrDie(R_GE_version as _);

            // Check if there are too many devices
            R_CheckDeviceAvailable();
        }

        // Define wrapper functions. This is a bit boring, and frustrationg to
        // see `create_device()` bloats to such a massive function because of
        // this, but probably there's no other way to do this nicely...

        unsafe extern "C" fn device_driver_activate<T: DeviceDriver>(arg1: pDevDesc) {
            // Derefernce to the original struct without moving it. While this
            // is a dangerous operation, it should be safe as long as the data
            // lives only within this function.
            //
            // Note that, we bravely unwrap() here because deviceSpecific should
            // never be a null pointer, as we set it. If the pDevDesc got
            // currupted, it might happen, but we can do nothing in that weird
            // case anyway.
            let data = ((*arg1).deviceSpecific as *mut T).as_mut().unwrap();

            data.activate(*arg1);
        }

        unsafe extern "C" fn device_driver_circle<T: DeviceDriver>(
            x: f64,
            y: f64,
            r: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.circle((x, y), r, *gc, *dd);
        }

        unsafe extern "C" fn device_driver_clip<T: DeviceDriver>(
            x0: f64,
            x1: f64,
            y0: f64,
            y1: f64,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.clip((x0, y0), (x1, y1), *dd);
        }

        // Note: the close() wrapper is special. This function is responsible
        // for tearing down the DeviceDriver itself, which is always needed even
        // when no close callback is implemented.
        unsafe extern "C" fn device_driver_close<T: DeviceDriver>(dd: pDevDesc) {
            let dev_desc = *dd;
            let data_ptr = dev_desc.deviceSpecific as *mut T;
            // Convert back to a Rust struct to drop the resources on Rust's side.
            let mut data = Box::from_raw(data_ptr);

            data.close(dev_desc);
        }

        unsafe extern "C" fn device_driver_deactivate<T: DeviceDriver>(arg1: pDevDesc) {
            let mut data = ((*arg1).deviceSpecific as *mut T).read();
            data.deactivate(*arg1);
        }

        unsafe extern "C" fn device_driver_line<T: DeviceDriver>(
            x1: f64,
            y1: f64,
            x2: f64,
            y2: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.line((x1, y1), (x2, y2), *gc, *dd);
        }

        unsafe extern "C" fn device_driver_char_metric<T: DeviceDriver>(
            c: c_int,
            gc: pGEcontext,
            ascent: *mut f64,
            descent: *mut f64,
            width: *mut f64,
            dd: pDevDesc,
        ) {
            // Be aware that `c` can be a negative value if `hasTextUTF8` is
            // true, and we do set it true. The header file[^1] states:
            //
            // > the metricInfo entry point should accept negative values for
            // > 'c' and treat them as indicating Unicode points (as well as
            // > positive values in a MBCS locale).
            //
            // The negativity might be useful if the implementation treats ASCII
            // and non-ASCII characters differently, but I think it's rare. So,
            // we just use `c.abs()`.
            //
            // [^1]: https://github.com/wch/r-source/blob/9bb47ca929c41a133786fa8fff7c70162bb75e50/src/include/R_ext/GraphicsDevice.h#L615-L617
            if let Some(c) = std::char::from_u32(c.unsigned_abs()) {
                let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
                let metric_info = data.char_metric(c, *gc, *dd);
                *ascent = metric_info.ascent;
                *descent = metric_info.descent;
                *width = metric_info.width;
            }
        }

        unsafe extern "C" fn device_driver_mode<T: DeviceDriver>(mode: c_int, dd: pDevDesc) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.mode(mode as _, *dd);
        }

        unsafe extern "C" fn device_driver_new_page<T: DeviceDriver>(gc: pGEcontext, dd: pDevDesc) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.new_page(*gc, *dd);
        }

        unsafe extern "C" fn device_driver_polygon<T: DeviceDriver>(
            n: c_int,
            x: *mut f64,
            y: *mut f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let x = slice::from_raw_parts(x, n as _);
            let y = slice::from_raw_parts(y, n as _);

            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.polygon(x, y, *gc, *dd);
        }

        unsafe extern "C" fn device_driver_polyline<T: DeviceDriver>(
            n: c_int,
            x: *mut f64,
            y: *mut f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let x = slice::from_raw_parts(x, n as _);
            let y = slice::from_raw_parts(y, n as _);

            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.polyline(x, y, *gc, *dd);
        }

        unsafe extern "C" fn device_driver_rect<T: DeviceDriver>(
            x0: f64,
            y0: f64,
            x1: f64,
            y1: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.rect((x0, y0), (x1, y1), *gc, *dd);
        }

        unsafe extern "C" fn device_driver_path<T: DeviceDriver>(
            x: *mut f64,
            y: *mut f64,
            npoly: c_int,
            nper: *mut c_int,
            winding: Rboolean,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let nper = slice::from_raw_parts(nper, npoly as _);
            // TODO: This isn't very efficient as we need to iterate over nper at least twice.
            let n = nper.iter().sum::<i32>() as usize;
            let x = slice::from_raw_parts(x, n as _);
            let y = slice::from_raw_parts(y, n as _);

            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();

            // It seems `NA` is just treated as `true`. Probably it doesn't matter much here.
            // c.f. https://github.com/wch/r-source/blob/6b22b60126646714e0f25143ac679240be251dbe/src/library/grDevices/src/devPS.c#L4235
            let winding = winding != Rboolean_FALSE;

            data.path(x, y, nper, winding, *gc, *dd);
        }

        unsafe extern "C" fn device_driver_raster<T: DeviceDriver>(
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
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();

            // convert u32 to u8.
            let raster = unsafe {
                std::slice::from_raw_parts(
                    std::mem::transmute::<*mut c_uint, *mut u8>(raster),
                    (w * h * 4) as _, // u32 contains 4 u8s, so multiply by 4
                )
            };

            data.raster(
                raster,
                (w, h),
                (x, y),
                (width, height),
                rot,
                // It seems `NA` is just treated as `true`. Probably it doesn't matter much here.
                // c.f. https://github.com/wch/r-source/blob/6b22b60126646714e0f25143ac679240be251dbe/src/library/grDevices/src/devPS.c#L4062
                interpolate != Rboolean_FALSE,
                *gc,
                *dd,
            );
        }

        unsafe extern "C" fn device_driver_capture<T: DeviceDriver>(dd: pDevDesc) -> SEXP {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.capture(*dd)
        }

        unsafe extern "C" fn device_driver_size<T: DeviceDriver>(
            left: *mut f64,
            right: *mut f64,
            bottom: *mut f64,
            top: *mut f64,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            *left = 0.0;
            *bottom = 0.0;
            let right = right.as_mut().unwrap();
            let top = top.as_mut().unwrap();
            data.size(right, top, *dd);
        }

        unsafe extern "C" fn device_driver_text_width<T: DeviceDriver>(
            str: *const c_char,
            gc: pGEcontext,
            dd: pDevDesc,
        ) -> f64 {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            let cstr = std::ffi::CStr::from_ptr(str);

            // TODO: Should we do something when the str is not available?
            if let Ok(cstr) = cstr.to_str() {
                data.text_width(cstr, *gc, *dd)
            } else {
                0.0
            }
        }

        unsafe extern "C" fn device_driver_text<T: DeviceDriver>(
            x: f64,
            y: f64,
            str: *const c_char,
            rot: f64,
            hadj: f64,
            gc: pGEcontext,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            let cstr = std::ffi::CStr::from_ptr(str);

            // TODO: Should we do something when the str is not available?
            if let Ok(cstr) = cstr.to_str() {
                data.text((x, y), cstr, rot, hadj, *gc, *dd);
            }
        }

        unsafe extern "C" fn device_driver_on_exit<T: DeviceDriver>(dd: pDevDesc) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.on_exit(*dd);
        }

        unsafe extern "C" fn device_driver_new_frame_confirm<T: DeviceDriver>(
            dd: pDevDesc,
        ) -> Rboolean {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.new_frame_confirm(*dd).into()
        }

        unsafe extern "C" fn device_driver_holdflush<T: DeviceDriver>(
            dd: pDevDesc,
            level: c_int,
        ) -> c_int {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.holdflush(*dd, level as _)
        }

        unsafe extern "C" fn device_driver_locator<T: DeviceDriver>(
            x: *mut f64,
            y: *mut f64,
            dd: pDevDesc,
        ) -> Rboolean {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.locator(x, y, *dd).into()
        }

        unsafe extern "C" fn device_driver_eventHelper<T: DeviceDriver>(dd: pDevDesc, code: c_int) {
            let mut data = ((*dd).deviceSpecific as *mut T).read();
            data.eventHelper(*dd, code);
        }

        unsafe extern "C" fn device_driver_setPattern<T: DeviceDriver>(
            pattern: SEXP,
            dd: pDevDesc,
        ) -> SEXP {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.set_pattern(pattern, *dd)
        }

        unsafe extern "C" fn device_driver_releasePattern<T: DeviceDriver>(
            ref_: SEXP,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            data.release_pattern(ref_, *dd);
        }

        unsafe extern "C" fn device_driver_setClipPath<T: DeviceDriver>(
            path: SEXP,
            ref_: SEXP,
            dd: pDevDesc,
        ) -> SEXP {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            // TODO
            // data.setClipPath(path, ref_, *dd)
            R_NilValue
        }

        unsafe extern "C" fn device_driver_releaseClipPath<T: DeviceDriver>(
            ref_: SEXP,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            // TODO
            // data.releaseClipPath(ref_, *dd);
        }

        unsafe extern "C" fn device_driver_setMask<T: DeviceDriver>(
            path: SEXP,
            ref_: SEXP,
            dd: pDevDesc,
        ) -> SEXP {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            // TODO
            // data.setMask(path, ref_, *dd)
            R_NilValue
        }

        unsafe extern "C" fn device_driver_releaseMask<T: DeviceDriver>(ref_: SEXP, dd: pDevDesc) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            // TODO
            // data.releaseMask(ref_, *dd);
        }

        unsafe extern "C" fn device_driver_releaseGroup<T: DeviceDriver>(ref_: SEXP, dd: pDevDesc) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            // TODO
            // data.releaseMask(ref_, *dd);
        }

        unsafe extern "C" fn device_driver_capabilities<T: DeviceDriver>(cap: SEXP) -> SEXP {
            <T>::capabilities(cap)
        }

        unsafe extern "C" fn device_driver_glyph<T: DeviceDriver>(
            n: c_int,
            glyphs: *mut c_uint,
            x: *mut f64,
            y: *mut f64,
            font: SEXP,
            size: f64,
            colour: c_uint,
            rot: f64,
            dd: pDevDesc,
        ) {
            let data = ((*dd).deviceSpecific as *mut T).as_mut().unwrap();
            let glyphs = slice::from_raw_parts(glyphs, n as _);

            let x = slice::from_raw_parts(x, n as _);
            let y = slice::from_raw_parts(y, n as _);

            let fontfile = std::ffi::CStr::from_ptr(R_GE_glyphFontFile(font))
                .to_str()
                .unwrap_or_default();
            let index = R_GE_glyphFontIndex(font);
            let family = std::ffi::CStr::from_ptr(R_GE_glyphFontFamily(font))
                .to_str()
                .unwrap_or_default();
            let weight = R_GE_glyphFontWeight(font);
            let style = R_GE_glyphFontStyle(font);

            data.glyph(
                glyphs, x, y, fontfile, index, family, weight, style, rot, size, colour,
            );
        }

        //
        // ************* defining the wrapper functions ends here ****************
        //

        // `Box::new()` allocates memory on the heap and places `self` into it.
        // Then, an unsafe function `Box::into_raw()` converts it to a raw
        // pointer. By doing so, Rust won't drop the object so that it will
        // survive after after being passed to the R's side. Accordingly, it's
        // extendr's responsibility to drop it. This deallocation will be done
        // in the `close()` wrapper; the struct will be gotten back to the
        // Rust's side by `Box::from_raw()` so that Rust will drop it when
        // returning from the function.
        let deviceSpecific = Box::into_raw(Box::new(self)) as *mut std::os::raw::c_void;

        // When we go across the boundary of FFI, the general rule is that the
        // allocated memory needs to be deallocated by the same allocator; if we
        // allocate memory on Rust's side, it needs to be dropped on Rust's
        // side. If we allocate memory on R's side, it needs to be freed on R's
        // side. Here, `DevDesc` is the latter case.
        //
        // The problem is that, while `DevDesc` is supposed to be `free()`ed on
        // R's side when device is closed by `dev.off()` (more specifically, in
        // `GEdestroyDevDesc()`), there's no API that creates a `DevDesc`
        // instance; typically, it's created by `calloc()` and a manual cast to
        // `DevDesc*`. Please see [the example code on R Internals].
        //
        // Because of the absence of such an API, the only choice here is to use
        // `libc::calloc()` and treat it as `*DevDesc`, taking the risk of
        // uninitialized fields. This solves the problem if the same "libc" (or
        // C runtime) as R is used. In other words, there's still a risk of
        // allocator mismatch. We need to be careful to configure PATHs
        // correctly to make sure the same toolchain used for compiling R itself
        // is chosen when the program is compiled.
        //
        // [Example code on R Internals]:
        //     https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Device-structures
        let p_dev_desc = unsafe { libc::calloc(1, std::mem::size_of::<DevDesc>()) as *mut DevDesc };

        unsafe {
            (*p_dev_desc).left = device_descriptor.left;
            (*p_dev_desc).right = device_descriptor.right;
            (*p_dev_desc).bottom = device_descriptor.bottom;
            (*p_dev_desc).top = device_descriptor.top;

            // This should be the same as the size of the device
            (*p_dev_desc).clipLeft = device_descriptor.left;
            (*p_dev_desc).clipRight = device_descriptor.right;
            (*p_dev_desc).clipBottom = device_descriptor.bottom;
            (*p_dev_desc).clipTop = device_descriptor.top;

            // Not sure where these numbers came from, but it seems this is a
            // common practice, considering the postscript device and svglite
            // device do so.
            (*p_dev_desc).xCharOffset = 0.4900;
            (*p_dev_desc).yCharOffset = 0.3333;
            (*p_dev_desc).yLineBias = 0.2;

            (*p_dev_desc).ipr = device_descriptor.ipr;
            (*p_dev_desc).cra = device_descriptor.cra;

            // Gamma-related parameters are all ignored. R-internals indicates so:
            //
            // canChangeGamma – Rboolean: can the display gamma be adjusted? This is now
            // ignored, as gamma support has been removed.
            //
            // and actually it seems this parameter is never used.
            (*p_dev_desc).gamma = 1.0;

            (*p_dev_desc).canClip = Rboolean_TRUE;

            // As described above, gamma is not supported.
            (*p_dev_desc).canChangeGamma = Rboolean_FALSE;

            (*p_dev_desc).canHAdj = 2; // can do adjust of text continuously

            (*p_dev_desc).startps = device_descriptor.startps;
            (*p_dev_desc).startcol = device_descriptor.startcol;
            (*p_dev_desc).startfill = device_descriptor.startfill;
            (*p_dev_desc).startlty = device_descriptor.startlty;
            (*p_dev_desc).startfont = device_descriptor.startfont;

            (*p_dev_desc).startgamma = 1.0;

            // A raw pointer to the data specific to the device.
            (*p_dev_desc).deviceSpecific = deviceSpecific;

            (*p_dev_desc).displayListOn = Rboolean_FALSE; // TODO

            // These are currently not used, so just set FALSE.
            (*p_dev_desc).canGenMouseDown = Rboolean_FALSE;
            (*p_dev_desc).canGenMouseMove = Rboolean_FALSE;
            (*p_dev_desc).canGenMouseUp = Rboolean_FALSE;
            (*p_dev_desc).canGenKeybd = Rboolean_FALSE;
            (*p_dev_desc).canGenIdle = Rboolean_FALSE;

            // The header file says:
            //
            // This is set while getGraphicsEvent is actively looking for events.
            //
            // It seems no implementation sets this, so this is probably what is
            // modified on the engine's side.
            (*p_dev_desc).gettingEvent = Rboolean_FALSE;

            (*p_dev_desc).activate = Some(device_driver_activate::<T>);
            (*p_dev_desc).circle = Some(device_driver_circle::<T>);
            (*p_dev_desc).clip = Some(device_driver_clip::<T>);
            (*p_dev_desc).close = Some(device_driver_close::<T>);
            (*p_dev_desc).deactivate = Some(device_driver_deactivate::<T>);
            (*p_dev_desc).locator = Some(device_driver_locator::<T>); // TODO
            (*p_dev_desc).line = Some(device_driver_line::<T>);
            (*p_dev_desc).metricInfo = Some(device_driver_char_metric::<T>);
            (*p_dev_desc).mode = Some(device_driver_mode::<T>);
            (*p_dev_desc).newPage = Some(device_driver_new_page::<T>);
            (*p_dev_desc).polygon = Some(device_driver_polygon::<T>);
            (*p_dev_desc).polyline = Some(device_driver_polyline::<T>);
            (*p_dev_desc).rect = Some(device_driver_rect::<T>);
            (*p_dev_desc).path = Some(device_driver_path::<T>);
            (*p_dev_desc).raster = Some(device_driver_raster::<T>);
            (*p_dev_desc).cap = Some(device_driver_capture::<T>);
            (*p_dev_desc).size = Some(device_driver_size::<T>);
            (*p_dev_desc).strWidth = Some(device_driver_text_width::<T>);
            (*p_dev_desc).text = Some(device_driver_text::<T>);
            (*p_dev_desc).onExit = Some(device_driver_on_exit::<T>);

            // This is no longer used and exists only for backward-compatibility
            // of the structure.
            (*p_dev_desc).getEvent = None;

            (*p_dev_desc).newFrameConfirm = Some(device_driver_new_frame_confirm::<T>);

            // UTF-8 support
            (*p_dev_desc).hasTextUTF8 = Rboolean_TRUE;
            (*p_dev_desc).textUTF8 = Some(device_driver_text::<T>);
            (*p_dev_desc).strWidthUTF8 = Some(device_driver_text_width::<T>);
            (*p_dev_desc).wantSymbolUTF8 = Rboolean_TRUE;

            // R internals says:
            //
            //     Some devices can produce high-quality rotated text, but those based on
            //     bitmaps often cannot. Those which can should set useRotatedTextInContour
            //     to be true from graphics API version 4.
            //
            // It seems this is used only by plot3d, so FALSE should be appropriate in
            // most of the cases.
            (*p_dev_desc).useRotatedTextInContour = Rboolean_FALSE;

            (*p_dev_desc).eventEnv = R_EmptyEnv;
            (*p_dev_desc).eventHelper = Some(device_driver_eventHelper::<T>);

            (*p_dev_desc).holdflush = Some(device_driver_holdflush::<T>);

            // TODO: implement capability properly.
            (*p_dev_desc).haveTransparency = 2; // yes
            (*p_dev_desc).haveTransparentBg = 2; // fully

            (*p_dev_desc).haveRaster = 1;
            (*p_dev_desc).haveCapture = 1; // TODO
            (*p_dev_desc).haveLocator = 1; // TODO

            (*p_dev_desc).setPattern = Some(device_driver_setPattern::<T>);
            (*p_dev_desc).releasePattern = Some(device_driver_releasePattern::<T>);

            (*p_dev_desc).setClipPath = Some(device_driver_setClipPath::<T>);
            (*p_dev_desc).releaseClipPath = Some(device_driver_releaseClipPath::<T>);

            (*p_dev_desc).setMask = Some(device_driver_setMask::<T>);
            (*p_dev_desc).releaseMask = Some(device_driver_releaseMask::<T>);

            (*p_dev_desc).deviceVersion = R_GE_version as _;

            (*p_dev_desc).deviceClip = Rboolean_TRUE;

            (*p_dev_desc).defineGroup = None;
            (*p_dev_desc).useGroup = None;
            (*p_dev_desc).releaseGroup = Some(device_driver_releaseGroup::<T>);

            (*p_dev_desc).stroke = None;
            (*p_dev_desc).fill = None;
            (*p_dev_desc).fillStroke = None;

            (*p_dev_desc).capabilities = Some(device_driver_capabilities::<T>);

            (*p_dev_desc).glyph = Some(device_driver_glyph::<T>);
        } // unsafe ends here

        let device_name = CString::new(device_name).unwrap();

        let device = unsafe { GEcreateDevDesc(p_dev_desc) };

        unsafe {
            // NOTE: Some graphic device use `GEaddDevice2f()`, a version of
            // `GEaddDevice2()` with a filename, instead, but `GEaddDevice2()`
            // should be appropriate for general purposes.
            GEaddDevice2(device, device_name.as_ptr() as *mut i8);

            GEinitDisplayList(device);
        }

        Ok(())
    }
}
