mod graphics;
use std::ffi::CStr;
use std::sync::LazyLock;

use graphics::ClippingStrategy;
use graphics::DevDesc;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use graphics::R_GE_gcontext;
use graphics::R_NilValue;
use savvy::savvy;
use tonic::transport::Channel;
use vellogd_protocol::graphics_device_client::GraphicsDeviceClient;
use vellogd_protocol::DrawCircleRequest;
use vellogd_protocol::DrawLineRequest;
use vellogd_protocol::DrawPolygonRequest;
use vellogd_protocol::DrawPolylineRequest;
use vellogd_protocol::DrawRectRequest;
use vellogd_protocol::DrawTextRequest;
use vellogd_protocol::Empty;
use vellogd_protocol::GetTextMetricRequest;
use vellogd_protocol::StrokeParameters;

#[cfg(debug_assertions)]
mod debug_device;

pub struct VelloGraphicsDevice {
    filename: String,
    client: Option<GraphicsDeviceClient<Channel>>,
}

impl VelloGraphicsDevice {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.into(),
            client: None,
        }
    }

    // TODO: if the connection is lost, how to detect it and reconnect?
    pub fn client(&mut self) -> &mut GraphicsDeviceClient<Channel> {
        if self.client.is_none() {
            self.client = Some(
                RUNTIME
                    .block_on(async { GraphicsDeviceClient::connect("http://[::1]:50051").await })
                    .unwrap(),
            )
        }
        self.client.as_mut().unwrap()
    }
}

fn gc_to_stroke_params(gc: R_GE_gcontext, optional: bool) -> Option<StrokeParameters> {
    let stroke_color = unsafe { std::mem::transmute::<i32, u32>(gc.col) };
    if optional && stroke_color == 0 {
        return None;
    }
    Some(StrokeParameters {
        color: stroke_color,
        width: gc.lwd,
        linetype: gc.lty,
        join: gc.ljoin as _,
        miter_limit: gc.lmitre,
        cap: gc.lend as _,
    })
}

fn gc_to_fill_color(gc: R_GE_gcontext, optional: bool) -> Option<u32> {
    let fill_color = unsafe { std::mem::transmute::<i32, u32>(gc.fill) };
    if optional && fill_color == 0 {
        None
    } else {
        Some(fill_color)
    }
}

static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

impl DeviceDriver for VelloGraphicsDevice {
    const USE_RASTER: bool = true;

    const USE_CAPTURE: bool = true;

    const USE_LOCATOR: bool = true;

    const USE_PLOT_HISTORY: bool = false;

    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::DeviceAndEngine;

    const ACCEPT_UTF8_TEXT: bool = true;

    fn activate(&mut self, _: DevDesc) {}

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        let fill_color = unsafe { std::mem::transmute::<i32, u32>(gc.fill) };
        let fill_color = if fill_color != 0 {
            Some(fill_color)
        } else {
            None
        };

        let request = tonic::Request::new(DrawCircleRequest {
            cx: center.0,
            cy: center.1,
            radius: r,
            fill_color,
            stroke_params: gc_to_stroke_params(gc, true),
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_circle(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw circle: {e:?}"),
        }
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {}

    fn close(&mut self, _: DevDesc) {
        let client = self.client();
        let res = RUNTIME.block_on(async { client.close_window(Empty {}).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to close the device: {e:?}"),
        }
    }

    fn deactivate(&mut self, _: DevDesc) {}

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let request = tonic::Request::new(DrawLineRequest {
            x0: from.0,
            y0: from.1,
            x1: from.0,
            y1: from.1,
            stroke_params: gc_to_stroke_params(gc, false),
        });
        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_line(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw line: {e:?}"),
        }
    }

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> graphics::TextMetric {
        let family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let request = tonic::Request::new(GetTextMetricRequest {
            text: c.to_string(),
            size: (gc.cex * gc.ps) as _,
            lineheight: gc.lineheight as _,
            face: gc.fontface as _,
            family,
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.get_text_metric(request).await });
        match res {
            Ok(res) => {
                let metric = res.into_inner();
                graphics::TextMetric {
                    ascent: metric.ascent,
                    descent: metric.descent,
                    width: metric.width,
                }
            }
            Err(e) => {
                savvy::r_eprintln!("failed to draw text: {e:?}");
                graphics::TextMetric {
                    ascent: 0.0,
                    descent: 0.0,
                    width: 0.0,
                }
            }
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {
        let client = self.client();
        let res = RUNTIME.block_on(async { client.new_page(Empty {}).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to request new page: {e:?}"),
        }
    }

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        let request = tonic::Request::new(DrawPolygonRequest {
            x: x.to_vec(), // TODO: avoid copy?
            y: y.to_vec(), // TODO: avoid copy?
            fill_color: gc_to_fill_color(gc, true),
            stroke_params: gc_to_stroke_params(gc, true),
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_polygon(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw polygon: {e:?}"),
        }
    }

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {
        let request = tonic::Request::new(DrawPolylineRequest {
            x: x.to_vec(), // TODO: avoid copy?
            y: y.to_vec(), // TODO: avoid copy?
            stroke_params: gc_to_stroke_params(gc, false),
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_polyline(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw polyline: {e:?}"),
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let request = tonic::Request::new(DrawRectRequest {
            x0: from.0,
            y0: from.1,
            x1: to.0,
            y1: to.1,
            fill_color: gc_to_fill_color(gc, true),
            stroke_params: gc_to_stroke_params(gc, true),
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_rect(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw rect: {e:?}"),
        }
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
        let family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let request = tonic::Request::new(GetTextMetricRequest {
            text: text.to_string(),
            size: (gc.cex * gc.ps) as _,
            lineheight: gc.lineheight as _,
            face: gc.fontface as _,
            family,
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.get_text_width(request).await });
        match res {
            Ok(res) => res.into_inner().width,
            Err(e) => {
                savvy::r_eprintln!("failed to draw text: {e:?}");
                0.0
            }
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
        let color = unsafe { std::mem::transmute::<i32, u32>(gc.col) };
        let family = unsafe {
            CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let request = tonic::Request::new(DrawTextRequest {
            x: pos.0,
            y: pos.1,
            text: text.to_string(),
            color,
            size: (gc.cex * gc.ps) as _,
            lineheight: gc.lineheight as _,
            face: gc.fontface as _,
            family,
            angle: angle as _,
            hadj: hadj as _,
        });

        let client = self.client();
        let res = RUNTIME.block_on(async { client.draw_text(request).await });
        match res {
            Ok(_) => {}
            Err(e) => savvy::r_eprintln!("failed to draw text: {e:?}"),
        }
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
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDevice::new(filename);

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd");

    Ok(())
}

#[savvy]
fn debugdg() -> savvy::Result<()> {
    let device_driver = debug_device::DebugDevice {};

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(480.0, 480.0);

    device_driver.create_device::<debug_device::DebugDevice>(device_descriptor, "debug");

    Ok(())
}
