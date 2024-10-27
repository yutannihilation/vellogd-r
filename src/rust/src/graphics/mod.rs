// Copied from https://github.com/extendr/extendr/blob/master/extendr-api/src/graphics/

mod device_descriptor;
mod device_driver;

pub use device_descriptor::DeviceDescriptor;

pub use device_driver::DeviceDriver;
use vellogd_shared::{
    ffi::{R_GE_gcontext, R_NilValue, INTEGER},
    protocol::{FillBrush, FillParams, StrokeParams},
};

pub fn gc_to_fill_params(gc: R_GE_gcontext) -> Option<FillParams> {
    gc_to_fill_params_with_flag(gc, true)
}

pub fn gc_to_fill_params_with_flag(
    gc: R_GE_gcontext,
    use_nonzero_rule: bool,
) -> Option<FillParams> {
    if gc.fill == 0 {
        return None;
    }

    let brush = if unsafe { gc.patternFill != R_NilValue } {
        let index = unsafe { *INTEGER(gc.patternFill) };
        FillBrush::PatternRef(index as u32)
    } else {
        let [r, g, b, a] = gc.fill.to_ne_bytes();
        let color = peniko::Color::rgba8(r, g, b, a);
        FillBrush::Color(color)
    };

    Some(FillParams {
        brush,
        use_nonzero_rule,
    })
}

pub fn gc_to_stroke_params(gc: R_GE_gcontext) -> Option<StrokeParams> {
    if gc.col == 0 || gc.lty == -1 {
        return None;
    }

    let [r, g, b, a] = gc.col.to_ne_bytes();
    let color = peniko::Color::rgba8(r, g, b, a);

    let width = gc.lwd;

    // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
    let join = match gc.ljoin {
        1 => kurbo::Join::Round,
        2 => kurbo::Join::Miter,
        3 => kurbo::Join::Bevel,
        v => panic!("invalid join value: {v}"),
    };
    // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
    let cap = match gc.lend {
        1 => kurbo::Cap::Round,
        2 => kurbo::Cap::Butt,
        3 => kurbo::Cap::Square,
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

    Some(StrokeParams {
        color,
        stroke: kurbo::Stroke {
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
