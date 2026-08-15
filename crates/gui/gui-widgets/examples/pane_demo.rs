use gpui::{
    AnyElement, App, Application, AppContext, AssetSource, Context, Entity, IntoElement,
    ParentElement, Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div,
    px, relative, rgb, size,
};
use gui_widgets::button::{Button, ButtonConfig, ButtonStyles};
use gui_widgets::pane::{Pane, PaneAlign, PaneConfig, PaneItemConfig, PaneStyles};
use gui_widgets::textbox::{TextBoxTitle, Textbox, TextboxConfig, TextboxStyles};
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
    pane: Entity<Pane>,
    title: Entity<Textbox>,
    hide_pane: Entity<Button>,
    show_pane: Entity<Button>,
    title_visible: bool,
    pane_visible: bool,
    status: SharedString,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_click = cx.weak_entity();

        let pane = Pane::new(
            PaneConfig::new(
                "main-pane",
                PaneStyles::new(px(360.), px(220.))
                    .background_color(rgb(0x1e293b))
                    .border_color(rgb(0x334155))
                    .columns(2)
                    .rows(2)
                    .align(PaneAlign::Center)
                    .item_width(px(120.))
                    .item_height(px(32.))
                    .item_border_color(rgb(0x475569))
                    .item_background_color(rgb(0x0f172a))
                    .item_text_color(rgb(0xf1f5f9)),
                vec![
                    PaneItemConfig::new("item-a", "A").row_col(0, 0),
                    PaneItemConfig::new("item-b", "B").row_col(0, 1),
                    PaneItemConfig::new("item-c", "C").row_col(1, 0),
                    PaneItemConfig::new("hide-title", "Hide Title").row_col(1, 1),
                ],
            )
            .on_item_click(move |_pane_id, item_id, _window, cx| {
                let _ = weak_click.update(cx, |root, cx| {
                    if item_id == "hide-title" {
                        root.title_visible = !root.title_visible;
                        root.status = format!("title visible={}", root.title_visible).into();
                    } else {
                        root.status = format!("clicked {item_id}").into();
                    }
                    cx.notify();
                });
            }),
            cx,
        );

        let title = Textbox::new(
            TextboxConfig::new(
                "demo-title",
                TextboxStyles::new(px(300.), px(56.), rgb(0xe2e8f0), rgb(0x60a5fa), rgb(0x64748b))
                    .fill_color(rgb(0x0f172a))
                    .border_color(rgb(0x334155))
                    .title_top(TextBoxTitle::new("Title", rgb(0x94a3b8)).text_size(px(12.))),
            ),
            cx,
        );

        let weak_hide = cx.weak_entity();
        let hide_pane = Button::new(
            ButtonConfig::new(
                "hide-pane",
                ButtonStyles::new(px(120.), px(32.))
                    .border_color(rgb(0x3b82f6))
                    .fill_color(rgb(0x1e293b))
                    .text("Hide Pane")
                    .text_color(rgb(0xf1f5f9))
                    .text_size(px(14.)),
            )
            .on_click(move |_id, _window, cx| {
                let _ = weak_hide.update(cx, |root, cx| {
                    root.pane.update(cx, |pane, cx| pane.set_visible(false, cx));
                    root.pane_visible = false;
                    cx.notify();
                });
            }),
            cx,
        );

        let weak_show = cx.weak_entity();
        let show_pane = Button::new(
            ButtonConfig::new(
                "show-pane",
                ButtonStyles::new(px(120.), px(32.))
                    .border_color(rgb(0x3b82f6))
                    .fill_color(rgb(0x1e293b))
                    .text("Show Pane")
                    .text_color(rgb(0xf1f5f9))
                    .text_size(px(14.)),
            )
            .on_click(move |_id, _window, cx| {
                let _ = weak_show.update(cx, |root, cx| {
                    root.pane.update(cx, |pane, cx| pane.set_visible(true, cx));
                    root.pane_visible = true;
                    cx.notify();
                });
            }),
            cx,
        );

        RootView {
            pane,
            title,
            hide_pane,
            show_pane,
            title_visible: true,
            pane_visible: true,
            status: "idle".into(),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let title_element: AnyElement = if self.title_visible {
            self.title.clone().into_any_element()
        } else {
            div().into_any_element()
        };
        div()
            .w(relative(1.0))
            .h(relative(1.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(0x111827))
            .child(title_element)
            .child(div().mt(px(16.)).child(self.pane.clone()))
            .child(
                div()
                    .mt(px(16.))
                    .flex()
                    .child(self.hide_pane.clone())
                    .child(div().w(px(8.)))
                    .child(self.show_pane.clone()),
            )
            .child(
                div()
                    .mt(px(16.))
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
