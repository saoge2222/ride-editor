use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::button::{Button, ButtonConfig, ButtonStyles};
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
    text_button: Entity<Button>,
    icon_button: Entity<Button>,
    status: SharedString,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_click = cx.weak_entity();
        let weak_hover = weak_click.clone();

        let text_style = ButtonStyles::new(px(140.), px(40.))
            .border_color(rgb(0x3b82f6))
            .fill_color(rgb(0x1e293b))
            .text("Click Me")
            .text_color(rgb(0xf1f5f9))
            .font_family("Maple Mono")
            .text_size(px(16.));

        let text_button = Button::new(
            ButtonConfig::new("text-button", text_style)
                .on_click(move |id, _window, cx| {
                    println!("clicked: {id}");
                    let _ = weak_click.update(cx, |root, cx| {
                        root.status = format!("{id} clicked").into();
                        cx.notify();
                    });
                })
                .on_hover(move |id, hovered, _window, cx| {
                    println!("hovered: {id} {hovered}");
                    let _ = weak_hover.update(cx, |root, cx| {
                        root.status = format!("{id} hovered={hovered}").into();
                        cx.notify();
                    });
                }),
            cx,
        );

        let icon_path = format!("{}/examples/assets/icon.svg", env!("CARGO_MANIFEST_DIR"));
        let icon_style = ButtonStyles::new(px(40.), px(40.))
            .border_color(rgb(0x10b981))
            .fill_color(rgb(0x0f172a))
            .text_color(rgb(0x10b981))
            .icon(icon_path, px(22.), px(22.));

        let icon_button = Button::new(ButtonConfig::new("icon-button", icon_style), cx);

        RootView {
            text_button,
            icon_button,
            status: "idle".into(),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status.clone();
        div()
            .w(relative(1.0))
            .h(relative(1.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(0x111827))
            .child(self.text_button.clone())
            .child(div().mt(px(16.)).child(self.icon_button.clone()))
            .child(
                div()
                    .mt(px(16.))
                    .text_color(rgb(0x94a3b8))
                    .text_size(px(14.))
                    .child(status),
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
