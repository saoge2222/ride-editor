use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::hlist::{HList, HListConfig, HListItemConfig, HListStyles};
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

fn sample_tabs() -> Vec<HListItemConfig> {
    vec![
        HListItemConfig::new("main.rs", "main.rs"),
        HListItemConfig::new("parser.rs", "parser.rs"),
        HListItemConfig::new("lexer.rs", "lexer.rs"),
        HListItemConfig::new("ast.rs", "ast.rs"),
        HListItemConfig::new("codegen.rs", "codegen.rs"),
        HListItemConfig::new("optimizer.rs", "optimizer.rs"),
        HListItemConfig::new("lsp.rs", "lsp.rs"),
        HListItemConfig::new("prelude.rs", "prelude.rs"),
    ]
}

struct RootView {
    tabs: Entity<HList>,
    status: SharedString,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_click = cx.weak_entity();
        let weak_hover = weak_click.clone();

        let styles = HListStyles::new(px(640.), px(40.))
            .background_color(rgb(0x0f172a))
            .border_color(rgb(0x334155))
            .item_width(px(96.))
            .item_height(px(40.))
            .item_border_color(rgb(0x1e293b))
            .item_background_color(rgb(0x1e293b))
            .item_text_color(rgb(0xe2e8f0));

        let tabs = HList::new(
            HListConfig::new("tabs", styles, sample_tabs())
                .on_item_click(move |_list_id, item_id, _window, cx| {
                    println!("tab clicked: {item_id}");
                    let _ = weak_click.update(cx, |root, cx| {
                        root.status = format!("clicked: {item_id}").into();
                        cx.notify();
                    });
                })
                .on_item_hover(move |_list_id, item_id, hovered, _window, cx| {
                    let _ = weak_hover.update(cx, |root, cx| {
                        root.status = format!("hovered: {item_id} {hovered}").into();
                        cx.notify();
                    });
                }),
            cx,
        );

        RootView {
            tabs,
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
            .child(self.tabs.clone())
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
