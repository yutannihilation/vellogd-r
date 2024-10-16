use std::sync::{LazyLock, Mutex};

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
}
