use crate::graphics::R_GE_gcontext;

#[derive(Debug, Clone)]
pub struct FillParams {
    pub color: vello::peniko::Color,
}

#[derive(Debug, Clone)]
pub struct StrokeParams {
    pub color: vello::peniko::Color,
    pub stroke: vello::kurbo::Stroke,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    NewWindow,
    RedrawWindow,
    CloseWindow,
    NewPage,
    GetWindowSizes,
    DrawCircle {
        center: vello::kurbo::Point,
        radius: f64,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawLine {
        p0: vello::kurbo::Point,
        p1: vello::kurbo::Point,
        stroke_params: StrokeParams,
    },
    DrawPolyline {
        path: vello::kurbo::BezPath,
        stroke_params: StrokeParams,
    },
    DrawPolygon {
        path: vello::kurbo::BezPath,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawRect {
        p0: vello::kurbo::Point,
        p1: vello::kurbo::Point,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawText {
        pos: vello::kurbo::Point,
        text: String,
        color: vello::peniko::Color,
        size: f32,
        lineheight: f32,
        // TODO
        // face
        family: String,
        angle: f32,
        hadj: f32,
    },
}

#[derive(Debug, Clone)]
pub enum UserResponse {
    WindowSizes { width: u32, height: u32 },
}

impl StrokeParams {
    pub fn from_gc(gc: R_GE_gcontext) -> Option<Self> {
        if gc.col == 0 || gc.lty == -1 {
            return None;
        }

        let [r, g, b, a] = gc.col.to_ne_bytes();
        let color = vello::peniko::Color::rgba8(r, g, b, a);

        let width = gc.lwd;

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let join = match gc.ljoin {
            1 => vello::kurbo::Join::Round,
            2 => vello::kurbo::Join::Miter,
            3 => vello::kurbo::Join::Bevel,
            v => panic!("invalid join value: {v}"),
        };
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let cap = match gc.lend {
            1 => vello::kurbo::Cap::Round,
            2 => vello::kurbo::Cap::Butt,
            3 => vello::kurbo::Cap::Square,
            v => panic!("invalid cap value: {v}"),
        };

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L413C1-L419C50
        //
        // Based on these implementations
        //
        // https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/modules/X11/devX11.c#L1224
        // https://github.com/r-lib/ragg/blob/6e8bfd1264dfaa36aa6f92592e13a1169986e7b9/src/AggDevice.h#L195C8-L205
        let dash_pattern: Vec<f64> = match gc.lty {
            -1 => vec![], // LTY_BLANK;
            0 => vec![],  // LTY_SOLID;
            lty => {
                let ptn_bytes = lty.to_ne_bytes();
                let mut ptn = Vec::new();
                for b in ptn_bytes {
                    let dash = b & 0b00001111;
                    let gap = (b & 0b11110000) >> 4;

                    if dash == 0 {
                        break;
                    }

                    ptn.push(dash as f64 * width);
                    ptn.push(gap as f64 * width);
                }
                ptn
            }
        };

        Some(Self {
            color,
            stroke: vello::kurbo::Stroke {
                width,
                join,
                miter_limit: gc.lmitre,
                start_cap: cap,
                end_cap: cap,
                dash_pattern: dash_pattern.into(),
                dash_offset: 0.0,
            },
        })
    }
}

impl FillParams {
    pub fn from_gc(gc: R_GE_gcontext) -> Option<Self> {
        if gc.fill == 0 {
            return None;
        }
        let [r, g, b, a] = gc.fill.to_ne_bytes();
        let color = vello::peniko::Color::rgba8(r, g, b, a);
        Some(Self { color })
    }
}
