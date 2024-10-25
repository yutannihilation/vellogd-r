use std::{
    io,
    sync::{LazyLock, Mutex},
};

use crate::ffi::R_GE_gcontext;

pub struct TextMetric {
    pub ascent: f64,
    pub descent: f64,
    pub width: f64,
}

static FONT_CTX: LazyLock<Mutex<parley::FontContext>> =
    LazyLock::new(|| Mutex::new(parley::FontContext::new()));

pub trait TextLayouter {
    fn layout_mut(&mut self) -> &mut parley::Layout<peniko::Brush>;
    fn layout_ref(&self) -> &parley::Layout<peniko::Brush>;

    fn build_layout(
        &mut self,
        text: impl AsRef<str>,
        family: impl AsRef<str>,
        weight: parley::FontWeight,
        style: parley::FontStyle,
        size: f32,
        lineheight: f32,
    ) {
        let text = text.as_ref();
        let mut font_ctx = FONT_CTX.lock().unwrap();
        // Note: parley is probably a little bit overkill, but it seems
        // this is the only interface.
        let mut layout_ctx: parley::LayoutContext<peniko::Brush> = parley::LayoutContext::new();
        let mut layout_builder = layout_ctx.ranged_builder(&mut font_ctx, text, 1.0);
        // TODO: should scale be configurable?
        layout_builder.push_default(parley::StyleProperty::FontSize(size));
        layout_builder.push_default(parley::StyleProperty::LineHeight(lineheight));

        let family = parley::FontFamily::parse(family.as_ref())
            .unwrap_or(parley::GenericFamily::SansSerif.into());
        layout_builder.push_default(family);

        layout_builder.push_default(parley::StyleProperty::FontWeight(weight));
        layout_builder.push_default(parley::StyleProperty::FontStyle(style));

        // TODO: use build_into() to reuse a Layout?
        let layout = self.layout_mut();
        layout_builder.build_into(layout, text);

        // It seems this is mandatory, otherwise no text is drawn. Why?
        layout.break_all_lines(None);

        layout.align(None, parley::Alignment::Start);
    }

    fn get_text_width<T: AsRef<str>>(&mut self, text: T, gc: R_GE_gcontext) -> f64 {
        let family = unsafe {
            std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = gc.cex * gc.ps;
        let (weight, style) = fontface_to_weight_and_style(gc.fontface);
        self.build_layout(text, &family, weight, style, size as _, gc.lineheight as _);
        self.layout_ref().width() as _
    }

    fn get_char_metric(&mut self, c: char, gc: R_GE_gcontext) -> TextMetric {
        let family = unsafe {
            std::ffi::CStr::from_ptr(gc.fontfamily.as_ptr())
                .to_str()
                .unwrap_or("Arial")
        }
        .to_string();
        let size = gc.cex * gc.ps;
        let text = c.to_string();
        let (weight, style) = fontface_to_weight_and_style(gc.fontface);
        self.build_layout(text, &family, weight, style, size as _, gc.lineheight as _);
        let layout_ref = self.layout_ref();
        let line = layout_ref.lines().next();
        match line {
            Some(line) => {
                let metrics = line.metrics();
                TextMetric {
                    ascent: metrics.ascent as _,
                    descent: metrics.descent as _,
                    width: layout_ref.width() as _, // TOOD: should this be run.metrics().width of the first char?
                }
            }
            None => TextMetric {
                ascent: 0.0,
                descent: 0.0,
                width: 0.0,
            },
        }
    }
}

pub fn fontface_to_weight_and_style(fontface: i32) -> (parley::FontWeight, parley::FontStyle) {
    match fontface {
        1 => (parley::FontWeight::NORMAL, parley::FontStyle::Normal), // Plain
        2 => (parley::FontWeight::BOLD, parley::FontStyle::Normal),   // Bold
        3 => (parley::FontWeight::NORMAL, parley::FontStyle::Italic), // Italic
        4 => (parley::FontWeight::BOLD, parley::FontStyle::Italic),   // BoldItalic
        _ => (parley::FontWeight::NORMAL, parley::FontStyle::Normal), // Symbolic or unknown
    }
}
