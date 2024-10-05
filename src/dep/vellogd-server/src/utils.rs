impl crate::StrokeParams {
    pub fn from_request(value: crate::StrokeParameters) -> Self {
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
            2 => vello::kurbo::Cap::Round,
            3 => vello::kurbo::Cap::Round,
            v => panic!("invalid cap value: {v}"),
        };

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L413C1-L419C50
        //
        // TODO: I need to figure out the conversion logic. What is this `& 15`...?
        //
        // https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/modules/X11/devX11.c#L1224
        // https://github.com/r-lib/ragg/blob/6e8bfd1264dfaa36aa6f92592e13a1169986e7b9/src/AggDevice.h#L195C8-L205
        let dash_pattern = match value.linetype {
            -1 => Default::default(), // TODO
            0 => Default::default(),
            49 => vello::kurbo::Dashes::from_const([1.0, 1.0, 1.0, 1.0]), // LTY_DOTTED	1 + (3<<4)
            68 => vello::kurbo::Dashes::from_const([1.0, 1.0, 1.0, 1.0]), // LTY_DASHED	4 + (4<<4)
            _ => Default::default(),
        };
        Self {
            color: u32_to_color(value.color),
            stroke: vello::kurbo::Stroke {
                width: value.width,
                join,
                miter_limit: value.miter_limit,
                start_cap: cap,
                end_cap: cap,
                dash_pattern,
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
