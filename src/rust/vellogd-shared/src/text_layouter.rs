use std::sync::{LazyLock, Mutex};

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
        // TODO
        // family: String,
        // face: i32,
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
        layout_builder.push_default(parley::GenericFamily::SansSerif); // TODO: specify family

        // TODO: use build_into() to reuse a Layout?
        let layout = self.layout_mut();
        layout_builder.build_into(layout, text);

        // It seems this is mandatory, otherwise no text is drawn. Why?
        layout.break_all_lines(None);

        layout.align(None, parley::Alignment::Start);
    }

    fn get_text_width<T: AsRef<str>>(&mut self, text: T, gc: R_GE_gcontext) -> f64 {
        // TODO
        // let family = unsafe {
        //     CStr::from_ptr(gc.fontfamily.as_ptr())
        //         .to_str()
        //         .unwrap_or("Arial")
        // }
        // .to_string();
        let size = gc.cex * gc.ps;
        self.build_layout(text, size as _, gc.lineheight as _);
        self.layout_ref().width() as _
    }

    fn get_char_metric(&mut self, c: char, gc: R_GE_gcontext) -> TextMetric {
        // TODO
        // let _family = unsafe {
        //     CStr::from_ptr(gc.fontfamily.as_ptr())
        //         .to_str()
        //         .unwrap_or("Arial")
        // }
        // .to_string();
        let size = gc.cex * gc.ps;
        self.build_layout(c.to_string(), size as _, gc.lineheight as _);
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
