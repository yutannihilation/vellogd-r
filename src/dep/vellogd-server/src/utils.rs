impl crate::StrokeParams {
    pub fn from_request(value: vellogd_protocol::StrokeParameters) -> Self {
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let join = match value.join {
            1 => vello::kurbo::Join::Round,
            2 => vello::kurbo::Join::Miter,
            3 => vello::kurbo::Join::Bevel,
            v => panic!("invalid join value: {v}"),
        };
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let cap = match value.cap {
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
        let dash_pattern: Vec<f64> = match value.linetype {
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

                    ptn.push(dash as f64 * value.width);
                    ptn.push(gap as f64 * value.width);
                }
                ptn
            }
        };
        Self {
            color: u32_to_color(value.color),
            stroke: vello::kurbo::Stroke {
                width: value.width,
                join,
                miter_limit: value.miter_limit,
                start_cap: cap,
                end_cap: cap,
                dash_pattern: dash_pattern.into(),
                dash_offset: 0.0,
            },
        }
    }
}

impl crate::FillParams {
    pub fn from_request(color: u32) -> Self {
        Self {
            color: u32_to_color(color),
        }
    }
}

pub(crate) fn u32_to_color(x: u32) -> vello::peniko::Color {
    let [r, g, b, a] = x.to_ne_bytes();
    vello::peniko::Color::rgba8(r, g, b, a)
}

// Note: BezPath allows more than lines, but R's graphics API currently uses lines only.
pub(crate) fn xy_to_path(x: Vec<f64>, y: Vec<f64>, close: bool) -> vello::kurbo::BezPath {
    let mut path = vello::kurbo::BezPath::new();

    let x_iter = x.into_iter();
    let y_iter = y.into_iter();
    let mut points = x_iter.zip(y_iter);
    if let Some(first) = points.next() {
        path.move_to(vello::kurbo::Point::new(first.0, first.1));
    } else {
        return path;
    }

    for (x, y) in points {
        path.line_to(vello::kurbo::Point::new(x, y));
    }

    if close {
        path.close_path();
    }

    path
}
