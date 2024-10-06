mod graphics;
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
use vellogd_protocol::Empty;
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

        let stroke_color = unsafe { std::mem::transmute::<i32, u32>(gc.col) };
        let stroke_params = if stroke_color != 0 {
            Some(StrokeParameters {
                color: stroke_color,
                width: gc.lwd,
                linetype: 1,
                join: 1,
                miter_limit: 1.0,
                cap: 1,
            })
        } else {
            None
        };

        let request = tonic::Request::new(DrawCircleRequest {
            cx: center.0,
            cy: center.1,
            radius: r,
            fill_color,
            stroke_params,
        });

        let client = self.client();
        let res = RUNTIME
            .block_on(async { client.draw_circle(request).await })
            .unwrap();
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

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {}

    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> graphics::TextMetric {
        graphics::TextMetric {
            ascent: 0.0,
            descent: 0.0,
            width: 0.0,
        }
    }

    fn mode(&mut self, mode: i32, _: DevDesc) {}

    fn new_page(&mut self, gc: R_GE_gcontext, _: DevDesc) {}

    fn polygon(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {}

    fn polyline(&mut self, x: &[f64], y: &[f64], gc: R_GE_gcontext, _: DevDesc) {}

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {}

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
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDevice::new(filename);

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new().device_size(0.0, width as _, 0.0, height as _);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd");

    Ok(())
}
