use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FillBrush {
    /// color
    Color(peniko::Color),
    /// index of the registered pattern
    PatternRef(u32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FillParams {
    pub brush: FillBrush,
    pub use_nonzero_rule: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StrokeParams {
    pub color: peniko::Color,
    pub stroke: kurbo::Stroke,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlyphParams<'a> {
    pub fontfile: &'a str,
    pub index: u32,
    pub family: &'a str,
    pub weight_raw: f32, // TODO: parley::FontWeight is not serializable
    pub style_raw: u32,  // TODO: parley::FontStyle is not serializable
    pub angle: f64,
    pub size: f32,
    pub color: peniko::Color,
}

impl<'a> GlyphParams<'a> {
    pub fn font(&self) -> std::io::Result<parley::Font> {
        let p = std::fs::canonicalize(self.fontfile).unwrap();
        let data = std::fs::read(p)?;
        Ok(parley::Font::new(data.into(), self.index))
    }

    pub fn weight(&self) -> parley::FontWeight {
        parley::FontWeight::new(self.weight_raw)
    }

    pub fn style(&self) -> parley::FontStyle {
        match self.style_raw {
            1 => parley::FontStyle::Normal,        // R_GE_text_style_normal
            2 => parley::FontStyle::Italic,        // R_GE_text_style_italic
            3 => parley::FontStyle::Oblique(None), // R_GE_text_style_oblique
            _ => parley::FontStyle::Normal,        // TODO: unreachable
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Request {
    ConnectionReady,
    NewWindow,
    RedrawWindow,
    CloseWindow,
    NewPage,

    SaveAsPng {
        filename: String,
    },

    PrepareForSaveAsTile {
        height: u32,
    },
    SaveAsTile {
        width: f64,
        height: f64,
        extend: peniko::Extend,
    },

    SetBaseColor {
        color: u32,
    },
    GetWindowSizes,
    DrawCircle {
        center: kurbo::Point,
        radius: f64,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawLine {
        p0: kurbo::Point,
        p1: kurbo::Point,
        stroke_params: StrokeParams,
    },
    DrawPolyline {
        path: kurbo::BezPath,
        stroke_params: StrokeParams,
    },
    DrawPolygon {
        path: kurbo::BezPath,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawRect {
        p0: kurbo::Point,
        p1: kurbo::Point,
        fill_params: Option<FillParams>,
        stroke_params: Option<StrokeParams>,
    },
    DrawText {
        pos: kurbo::Point,
        text: String,
        color: peniko::Color,
        size: f32,
        lineheight: f32,
        family: String,
        face: i32,
        angle: f32,
        hadj: f32,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Response {
    WindowSizes { width: u32, height: u32 },
    Connect { server_name: String },
    PatternRegistered { index: usize },
}

pub trait AppResponseRelay {
    fn respond(&self, response: Response);
}

impl AppResponseRelay for std::sync::mpsc::Sender<Response> {
    fn respond(&self, response: Response) {
        self.send(response).unwrap();
    }
}

impl AppResponseRelay for ipc_channel::ipc::IpcSender<Response> {
    fn respond(&self, response: Response) {
        self.send(response).unwrap();
    }
}
