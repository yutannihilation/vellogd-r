use std::os::raw::c_uint;
use std::sync::atomic::Ordering;

use super::xy_to_path;
use super::WindowController;
use crate::add_tracing_point;
use crate::graphics::gc_to_fill_params;
use crate::graphics::gc_to_fill_params_with_flag;
use crate::graphics::gc_to_stroke_params;
use crate::graphics::DeviceDriver;
use crate::vello_device::xy_to_path_with_hole;
use vellogd_shared::ffi::*;
use vellogd_shared::protocol::GlyphParams;
use vellogd_shared::protocol::Request;
use vellogd_shared::protocol::Response;
use vellogd_shared::text_layouter::fontface_to_weight_and_style;
use vellogd_shared::text_layouter::TextLayouter;
use vellogd_shared::text_layouter::TextMetric;
use vellogd_shared::winit_app::convert_to_image;
use vellogd_shared::winit_app::FillPattern;
use vellogd_shared::winit_app::VELLO_APP_PROXY;

pub struct VelloGraphicsDevice {
    filename: String,
    layout: parley::Layout<peniko::Brush>,
}

impl VelloGraphicsDevice {
    pub(crate) fn new(filename: &str, width: f64, height: f64) -> savvy::Result<Self> {
        VELLO_APP_PROXY.set_size(width as u32, height as u32);
        Ok(Self {
            filename: filename.into(),
            layout: parley::Layout::new(),
        })
    }
}

impl WindowController for VelloGraphicsDevice {
    fn send_event(&self, event: Request) -> savvy::Result<()> {
        VELLO_APP_PROXY
            .tx
            .send_event(event)
            .map_err(|e| format!("Failed to send event {e:?}").into())
    }

    fn recv_response(&self) -> savvy::Result<Response> {
        let receiver = VELLO_APP_PROXY.rx.lock().map_err(|e| e.to_string())?;
        receiver.recv().map_err(|e| e.to_string().into())
    }
}

impl TextLayouter for VelloGraphicsDevice {
    fn layout_mut(&mut self) -> &mut parley::Layout<peniko::Brush> {
        &mut self.layout
    }

    fn layout_ref(&self) -> &parley::Layout<peniko::Brush> {
        &self.layout
    }
}

impl DeviceDriver for VelloGraphicsDevice {
    fn activate(&mut self, _: DevDesc) {
        add_tracing_point!();

        match self.request_new_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to activate: {e}"),
        }
    }

    fn close(&mut self, _: DevDesc) {
        add_tracing_point!();

        match self.request_close_window() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to close window: {e}"),
        }
    }

    // TODO
    // fn deactivate(&mut self, _: DevDesc) {}

    // GraphicsDevice.h says:
    //
    //     device_Mode is called whenever the graphics engine
    //     starts drawing (mode=1) or stops drawing (mode=0)
    fn mode(&mut self, mode: i32, _: DevDesc) {
        VELLO_APP_PROXY
            .stop_rendering
            .store(mode == 1, Ordering::Relaxed);
    }

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        VELLO_APP_PROXY.set_base_color(gc.fill);
        match self.request_new_page() {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("Failed to create a new page: {e}"),
        }
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {
        let window_width = VELLO_APP_PROXY.width.load(Ordering::Relaxed) as f64;
        let window_height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;

        if from.0 <= 0.0 && from.1 <= 0.0 && to.0 >= window_width && to.1 >= window_height {
            VELLO_APP_PROXY.scene.pop_clip();
        } else {
            VELLO_APP_PROXY.scene.push_clip(from.into(), to.into());
        }
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = gc_to_fill_params(gc);
        let stroke_params = gc_to_stroke_params(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_circle(center.into(), r, fill_params, stroke_params);
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        if let Some(stroke_params) = gc_to_stroke_params(gc) {
            VELLO_APP_PROXY
                .scene
                .draw_line(from.into(), to.into(), stroke_params);
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = gc_to_fill_params(gc);
        let stroke_params = gc_to_stroke_params(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_polygon(xy_to_path(x, y, true), fill_params, stroke_params);
        }
    }

    fn path(
        &mut self,
        x: &[f64],
        y: &[f64],
        nper: &[i32],
        winding: bool,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();

        let fill_params = gc_to_fill_params_with_flag(gc, winding);
        let stroke_params = gc_to_stroke_params(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY.scene.draw_polygon(
                xy_to_path_with_hole(x, y, nper),
                fill_params,
                stroke_params,
            );
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let stroke_params = gc_to_stroke_params(gc);
        if let Some(stroke_params) = stroke_params {
            VELLO_APP_PROXY
                .scene
                .draw_polyline(xy_to_path(x, y, false), stroke_params);
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        add_tracing_point!();

        let fill_params = gc_to_fill_params(gc);
        let stroke_params = gc_to_stroke_params(gc);
        if fill_params.is_some() || stroke_params.is_some() {
            VELLO_APP_PROXY
                .scene
                .draw_rect(from.into(), to.into(), fill_params, stroke_params);
        }
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
        add_tracing_point!();

        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = peniko::Color::rgba8(r, g, b, a);
        let family = unsafe {
            std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = (gc.cex * gc.ps) as f32;
        let lineheight = gc.lineheight as f32;
        let (weight, style) = fontface_to_weight_and_style(gc.fontface);
        self.build_layout(text, &family, weight, style, size, lineheight);

        let layout_width = self.layout.width() as f64;
        let window_height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;

        for line in self.layout.lines() {
            let line_metrics = line.metrics();
            let transform = vello::kurbo::Affine::translate((
                -(layout_width * hadj),
                (line_metrics.baseline - line_metrics.line_height) as f64, // TODO: is this correct?
            ))
            .then_rotate(-angle.to_radians())
            .then_translate((pos.0, window_height - pos.1).into()); // Y-axis is flipped

            for item in line.items() {
                // ignore inline box
                let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                VELLO_APP_PROXY
                    .scene
                    .draw_glyph(glyph_run, color, transform);
            }
        }
    }

    fn glyph(
        &mut self,
        glyph_ids: &[u32],
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

        let [r, g, b, a] = colour.to_ne_bytes();
        let color = peniko::Color::rgba8(r, g, b, a);
        let glyph_params = GlyphParams {
            fontfile,
            index: index as u32,
            family,
            weight_raw: weight as f32,
            style_raw: style as u32,
            angle: angle.to_radians(),
            size: size as f32,
            color,
        };

        VELLO_APP_PROXY
            .scene
            .draw_glyph_raw(glyph_ids, x, y, glyph_params);
    }

    fn raster(
        &mut self,
        raster: &[u8],
        pixels: (u32, u32),
        pos: (f64, f64), // bottom left corner
        size: (f64, f64),
        angle: f64,
        _interpolate: bool, // TODO
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        add_tracing_point!();

        let alpha = gc.col.to_ne_bytes()[3];

        let scale = (size.0 / pixels.0 as f64, size.1 / pixels.1 as f64);

        // when the pixel is small enough, it's not a problem, but if it's
        // large, this needs a tweak.
        let with_extended_edge = scale.0 > 1.0 || scale.1 > 1.0;

        #[cfg(debug_assertions)]
        {
            savvy::r_eprintln!("with_extended_edge : {with_extended_edge}");
        }

        let image = convert_to_image(
            raster,
            pixels.0 as usize,
            pixels.1 as usize,
            peniko::Extend::Pad,
            alpha,
            with_extended_edge,
        );

        let window_height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;
        let pos = (pos.0, window_height - (pos.1 + size.1)); // change to top-left corner

        VELLO_APP_PROXY
            .scene
            .draw_raster(&image, scale, pos.into(), angle, with_extended_edge);
    }

    // TODO
    // fn capture(&mut self, _: DevDesc) -> savvy::ffi::SEXP {
    //     unsafe { R_NilValue }
    // }

    fn size(&mut self, width: &mut f64, height: &mut f64, _: DevDesc) {
        add_tracing_point!();

        *width = VELLO_APP_PROXY.width.load(Ordering::Relaxed) as f64;
        *height = VELLO_APP_PROXY.height.load(Ordering::Relaxed) as f64;
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        add_tracing_point!();

        self.get_char_metric(c, gc)
    }

    fn text_width(&mut self, text: &str, gc: R_GE_gcontext, _: DevDesc) -> f64 {
        add_tracing_point!();

        self.get_text_width(text, gc)
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
                let extend = match R_GE_linearGradientExtend(pattern) {
                    1 => peniko::Extend::Pad,     // R_GE_patternExtendPad
                    2 => peniko::Extend::Repeat,  // R_GE_patternExtendRepeat
                    3 => peniko::Extend::Reflect, // R_GE_patternExtendReflect
                    _ => peniko::Extend::Pad, // TODO: what should I do when R_GE_patternExtendNone?
                };

                let num_stops = R_GE_linearGradientNumStops(pattern);

                let color_stops_iter = (0..num_stops).map(|i| {
                    let offset = R_GE_linearGradientStop(pattern, i) as f32;
                    let [r, g, b, a] = R_GE_linearGradientColour(pattern, i).to_ne_bytes();
                    let color = peniko::Color::rgba8(r, g, b, a);
                    peniko::ColorStop { offset, color }
                });
                let color_stops = peniko::ColorStops::from_iter(color_stops_iter);

                let mut gradient =
                    peniko::Gradient::new_linear((x1, y1), (x2, y2)).with_extend(extend);
                // Note: with_stops doesn't accept &[ColorStop] or ColorStops. Why?
                gradient.stops = color_stops;

                let index = VELLO_APP_PROXY
                    .scene
                    .register_pattern(FillPattern::Gradient(gradient));
                Rf_ScalarInteger(index as i32)
            },
            2 => unsafe {
                let cx1 = R_GE_radialGradientCX1(pattern);
                let cy1 = R_GE_radialGradientCY1(pattern);
                let r1 = R_GE_radialGradientR1(pattern) as f32;
                let cx2 = R_GE_radialGradientCX2(pattern);
                let cy2 = R_GE_radialGradientCY2(pattern);
                let r2 = R_GE_radialGradientR2(pattern) as f32;
                let extend = match R_GE_radialGradientExtend(pattern) {
                    1 => peniko::Extend::Pad,     // R_GE_patternExtendPad
                    2 => peniko::Extend::Repeat,  // R_GE_patternExtendRepeat
                    3 => peniko::Extend::Reflect, // R_GE_patternExtendReflect
                    _ => peniko::Extend::Pad, // TODO: what should I do when R_GE_patternExtendNone?
                };

                let num_stops = R_GE_radialGradientNumStops(pattern);

                let color_stops_iter = (0..num_stops).map(|i| {
                    let offset = R_GE_radialGradientStop(pattern, i) as f32;
                    let [r, g, b, a] = R_GE_radialGradientColour(pattern, i).to_ne_bytes();
                    let color = peniko::Color::rgba8(r, g, b, a);
                    peniko::ColorStop { offset, color }
                });
                let color_stops = peniko::ColorStops::from_iter(color_stops_iter);
                let mut gradient =
                    peniko::Gradient::new_two_point_radial((cx1, cy1), r1, (cx2, cy2), r2)
                        .with_extend(extend);
                // Note: with_stops doesn't accept &[ColorStop] or ColorStops. Why?
                gradient.stops = color_stops;

                let index = VELLO_APP_PROXY
                    .scene
                    .register_pattern(FillPattern::Gradient(gradient));
                Rf_ScalarInteger(index as i32)
            },
            3 => unsafe {
                let x = R_GE_tilingPatternX(pattern);
                let y = R_GE_tilingPatternY(pattern);
                let width = R_GE_tilingPatternWidth(pattern);
                let height = R_GE_tilingPatternHeight(pattern);
                let extend = match R_GE_tilingPatternExtend(pattern) {
                    1 => peniko::Extend::Pad,     // R_GE_patternExtendPad
                    2 => peniko::Extend::Repeat,  // R_GE_patternExtendRepeat
                    3 => peniko::Extend::Reflect, // R_GE_patternExtendReflect
                    _ => peniko::Extend::Pad, // TODO: what should I do when R_GE_patternExtendNone?
                };

                // Do not reflect the tile drawing to screen
                //
                // TODO: this is not perfect because mode() API call will mess
                // this flag. So, probably another flag or using accumulated
                // value instead of a bool is needed.
                VELLO_APP_PROXY
                    .stop_rendering
                    .store(true, Ordering::Relaxed);

                // the pattern tile is drawn on the screen of original sizes,
                // but it needs to be clipped at the specified area.
                let mut y_transform = VELLO_APP_PROXY.y_transform.lock().unwrap();
                let orig_y_transform = *y_transform;
                // TODO: to match the actual pixels and logical sizes, this needs to be scaled.
                *y_transform = y_transform.then_translate((-x, -y).into());
                // release lock, otherwise it deadlocks...
                drop(y_transform);

                // Use a new scene to preserve the current scene
                let tmp_scene = vello::Scene::new();
                let orig_scene = VELLO_APP_PROXY.scene.replace_edited_scene(tmp_scene);

                // Run drawing function
                let fun = R_GE_tilingPatternFunction(pattern);
                let call = Rf_protect(Rf_lang1(fun));
                Rf_eval(call, R_GlobalEnv);
                Rf_unprotect(1);

                self.request_register_tile(width, height, extend).unwrap();
                let index = VELLO_APP_PROXY.rx.lock().unwrap().recv().unwrap();

                // restore
                let _ = VELLO_APP_PROXY.scene.replace_edited_scene(orig_scene);
                *VELLO_APP_PROXY.y_transform.lock().unwrap() = orig_y_transform;
                VELLO_APP_PROXY
                    .stop_rendering
                    .store(false, Ordering::Relaxed);

                if let Response::PatternRegistered { index } = index {
                    Rf_ScalarInteger(index as i32)
                } else {
                    R_NilValue
                }
            },
            _ => unsafe { R_NilValue },
        }
    }

    fn release_pattern(&mut self, _ref: SEXP, _: DevDesc) {
        // TODO: ref_ contains the index of the pattern, so it can be released.
        VELLO_APP_PROXY.scene.release_pattern();
    }

    // TODO
    // fn on_exit(&mut self, _: DevDesc) {}

    // TODO
    // fn new_frame_confirm(&mut self, _: DevDesc) -> bool {
    //     true
    // }

    // TODO
    // fn holdflush(&mut self, _: DevDesc, level: i32) -> i32 {
    //     0
    // }

    // TODO
    // fn locator(&mut self, x: *mut f64, y: *mut f64, _: DevDesc) -> bool {
    //     true
    // }

    // TODO
    // fn eventHelper(&mut self, _: DevDesc, code: i32) {}
}
