// The code related to vello is based on
//
// - the example code on linbender/vello (examples/simple/main.rs).
// - the example code on linbender/parley (examples/vello_editor/src/main.rs).

use crate::{build_layout, utils, FillParams, StrokeParams, UserEvent};

use parley::FontContext;
use utils::u32_to_color;
use winit::event_loop::EventLoopProxy;

use tonic::{Request, Response, Status};

use vellogd_protocol::graphics_device_server::GraphicsDevice;
use vellogd_protocol::*;

pub struct VelloGraphicsDevice {
    event_loop_proxy: EventLoopProxy<UserEvent>,
    // TODO: how to share this with VelloApp?
    font_ctx: tokio::sync::Mutex<FontContext>,
}

impl VelloGraphicsDevice {
    pub fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Self {
        Self {
            event_loop_proxy,
            font_ctx: tokio::sync::Mutex::new(FontContext::new()),
        }
    }
}

#[tonic::async_trait]
impl GraphicsDevice for VelloGraphicsDevice {
    async fn close_window(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        self.event_loop_proxy
            .send_event(UserEvent::CloseWindow)
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn new_page(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        self.event_loop_proxy
            .send_event(UserEvent::NewPage)
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn draw_circle(
        &self,
        request: Request<DrawCircleRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawCircleRequest {
            cx,
            cy,
            radius,
            fill_color,
            stroke_params,
        } = request.into_inner();

        let fill_params = fill_color.map(FillParams::from_request);
        let stroke_params = stroke_params.map(StrokeParams::from_request);

        self.event_loop_proxy
            .send_event(UserEvent::DrawCircle {
                center: vello::kurbo::Point::new(cx, cy),
                radius,
                fill_params,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;
        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_line(
        &self,
        request: Request<DrawLineRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawLineRequest {
            x0,
            y0,
            x1,
            y1,
            stroke_params,
        } = request.into_inner();

        let stroke_params = stroke_params.ok_or_else(|| {
            Status::new(
                tonic::Code::InvalidArgument,
                "stroke_params must be specified",
            )
        })?;

        let stroke_params = StrokeParams::from_request(stroke_params);

        self.event_loop_proxy
            .send_event(UserEvent::DrawLine {
                p0: vello::kurbo::Point::new(x0, y0),
                p1: vello::kurbo::Point::new(x1, y1),
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_polyline(
        &self,
        request: Request<DrawPolylineRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawPolylineRequest {
            x,
            y,
            stroke_params,
        } = request.into_inner();

        let stroke_params = stroke_params.ok_or_else(|| {
            Status::new(
                tonic::Code::InvalidArgument,
                "stroke_params must be specified",
            )
        })?;

        let path = utils::xy_to_path(x, y, false);

        let stroke_params = StrokeParams::from_request(stroke_params);

        self.event_loop_proxy
            .send_event(UserEvent::DrawPolyline {
                path,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_polygon(
        &self,
        request: Request<DrawPolygonRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawPolygonRequest {
            x,
            y,
            fill_color,
            stroke_params,
        } = request.into_inner();

        let fill_params = fill_color.map(FillParams::from_request);
        let stroke_params = stroke_params.map(StrokeParams::from_request);
        let path = utils::xy_to_path(x, y, true);

        self.event_loop_proxy
            .send_event(UserEvent::DrawPolygon {
                path,
                fill_params,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_rect(
        &self,
        request: Request<DrawRectRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawRectRequest {
            x0,
            y0,
            x1,
            y1,
            fill_color,
            stroke_params,
        } = request.into_inner();

        let fill_params = fill_color.map(FillParams::from_request);
        let stroke_params = stroke_params.map(StrokeParams::from_request);

        self.event_loop_proxy
            .send_event(UserEvent::DrawRect {
                p0: vello::kurbo::Point::new(x0, y0),
                p1: vello::kurbo::Point::new(x1, y1),
                fill_params,
                stroke_params,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn draw_text(
        &self,
        request: Request<DrawTextRequest>,
    ) -> Result<Response<Empty>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let DrawTextRequest {
            x,
            y,
            text,
            color,
            size,
            lineheight,
            face,
            family,
            angle,
            hadj,
        } = request.into_inner();

        self.event_loop_proxy
            .send_event(UserEvent::DrawText {
                pos: vello::kurbo::Point::new(x, y),
                text,
                color: u32_to_color(color),
                size,
                lineheight,
                family,
                angle: angle.to_radians(),
                hadj,
            })
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let reply = Empty {};
        Ok(Response::new(reply))
    }

    async fn get_text_width(
        &self,
        request: Request<GetTextMetricRequest>,
    ) -> Result<Response<GetTextWidthResponse>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let GetTextMetricRequest {
            text,
            size,
            lineheight,
            face,
            family,
        } = request.into_inner();

        let mut font_ctx = self.font_ctx.lock().await;
        let layout = build_layout(&mut font_ctx, text, size, lineheight);

        let reply = GetTextWidthResponse {
            width: layout.width() as _,
        };
        Ok(Response::new(reply))
    }

    async fn get_text_metric(
        &self,
        request: Request<GetTextMetricRequest>,
    ) -> Result<Response<GetTextMetricResponse>, Status> {
        log::debug!("Recieved request: {:?}", request);

        let GetTextMetricRequest {
            text,
            size,
            lineheight,
            face,
            family,
        } = request.into_inner();

        let mut font_ctx = self.font_ctx.lock().await;
        let layout = build_layout(&mut font_ctx, text, size, lineheight);

        let reply = match layout.lines().next() {
            Some(line) => {
                let metrics = line.metrics();
                GetTextMetricResponse {
                    ascent: metrics.ascent as _,
                    descent: metrics.descent as _,
                    width: layout.width() as _, // TOOD: should this be run.metrics().width of the first char?
                }
            }
            None => {
                log::warn!("Failed to get line metrics");
                GetTextMetricResponse {
                    ascent: 0.0,
                    descent: 0.0,
                    width: 0.0,
                }
            }
        };
        Ok(Response::new(reply))
    }
}
