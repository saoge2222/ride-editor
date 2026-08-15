use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::caret::CaretShape;
use gui_widgets::textbox::{Placeholder, TextBoxTitle, Textbox, TextboxConfig, TextboxStyles};
use std::borrow::Cow;

struct FsAssetSource;

impl AssetSource for FsAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(_) => Ok(None),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let _ = path;
        Ok(vec![])
    }
}

struct RootView {
    title_textbox: Entity<Textbox>,
    multi_textbox: Entity<Textbox>,
    status: SharedString,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_changed = cx.weak_entity();
        let weak_changed_notes = weak_changed.clone();
        let weak_has_text = cx.weak_entity();

        let title_styles = TextboxStyles::new(px(340.), px(56.), rgb(0xe2e8f0), rgb(0x60a5fa), rgb(0x64748b))
            .fill_color(rgb(0x0f172a))
            .border_color(rgb(0x334155))
            .title_top(
                TextBoxTitle::new("Name", rgb(0x94a3b8))
                    .font_family("Maple Mono")
                    .text_size(px(12.)),
            )
            .title_bottom(TextBoxTitle::new("Hint", rgb(0x64748b)).text_size(px(12.)))
            .title_left(TextBoxTitle::new("L", rgb(0x64748b)).text_size(px(12.)))
            .title_right(TextBoxTitle::new("R", rgb(0x64748b)).text_size(px(12.)))
            .placeholder(
                Placeholder::new("type here")
                    .icon(
                        format!("{}/examples/assets/icon.svg", env!("CARGO_MANIFEST_DIR")),
                        px(14.),
                        px(14.),
                    ),
            );

        let title_textbox = Textbox::new(
            TextboxConfig::new("title-box", title_styles)
                .on_text_changed(move |id, text, _window, cx| {
                    println!("text_changed: {id} text={text:?}");
                    let _ = weak_changed.update(cx, |root, cx| {
                        root.status = format!("{id}: {text}").into();
                        cx.notify();
                    });
                })
                .on_has_text(move |id, has_text, _window, cx| {
                    println!("has_text: {id} {has_text}");
                    let _ = weak_has_text.update(cx, |root, cx| {
                        root.status = format!("{id} has_text={has_text}").into();
                        cx.notify();
                    });
                }),
            cx,
        );

        let multi_styles = TextboxStyles::new(px(340.), px(96.), rgb(0xe2e8f0), rgb(0x60a5fa), rgb(0x64748b))
            .fill_color(rgb(0x0f172a))
            .border_color(rgb(0x334155))
            .title_top(TextBoxTitle::new("Notes", rgb(0x94a3b8)).text_size(px(12.)))
            .line_count(3)
            .placeholder(Placeholder::new("multi-line notes"))
            .caret_style(CaretShape::Block)
            .caret_blink_ms(800);

        let multi_textbox = Textbox::new(
            TextboxConfig::new("notes-box", multi_styles)
                .on_text_changed(move |id, text, _window, cx| {
                    println!("text_changed: {id} text={text:?}");
                    let _ = weak_changed_notes.update(cx, |root, cx| {
                        root.status = format!("{id}: {text}").into();
                        cx.notify();
                    });
                }),
            cx,
        );
        multi_textbox.update(cx, |textbox, cx| {
            textbox.set_text(
                "line 01\nline 02\nline 03\nline 04\nline 05\nline 06\nline 07\nline 08\nline 09",
                cx,
            );
        });

        RootView {
            title_textbox,
            multi_textbox,
            status: "idle".into(),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(relative(1.0))
            .h(relative(1.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(0x111827))
            .child(self.title_textbox.clone())
            .child(div().mt(px(24.)).child(self.multi_textbox.clone()))
            .child(
                div()
                    .mt(px(24.))
                    .text_color(rgb(0x94a3b8))
                    .text_size(px(14.))
                    .child(self.status.clone()),
            )
    }
}

fn main() {
    Application::new()
        .with_assets(FsAssetSource)
        .run(|cx: &mut App| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(800.), px(600.)), cx)),
                ..Default::default()
            };
            let _ = cx.open_window(options, |_window, cx| cx.new(RootView::new));
            cx.activate(true);
        });
}
