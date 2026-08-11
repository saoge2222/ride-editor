use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::button::{Button, ButtonConfig, ButtonStyles};
use gui_widgets::data::{GitFileState, GitStatus};
use gui_widgets::list::{List, ListConfig, ListItemConfig, ListStyles};
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
    list: Entity<List>,
    refresh_button: Entity<Button>,
    status: SharedString,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_click = cx.weak_entity();
        let weak_hover = weak_click.clone();
        let weak_refresh = cx.weak_entity();

        let list_styles = ListStyles::new(px(360.), px(240.))
            .border_color(rgb(0x334155))
            .background_color(rgb(0x0f172a))
            .item_height(px(36.))
            .item_border_color(rgb(0x1e293b))
            .item_text_color(rgb(0xe2e8f0));

        let items = vec![
            ListItemConfig::new("src/main.rs", "M src/main.rs"),
            ListItemConfig::new("src/parser.rs", "?? src/parser.rs"),
            ListItemConfig::new("src/lexer.rs", "D src/lexer.rs"),
        ];

        let list = List::new(
            ListConfig::new("git-list", list_styles, items)
                .on_item_click(move |_list_id, item_id, _window, cx| {
                    println!("item clicked: {item_id}");
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

        let refresh_button = Button::new(
            ButtonConfig::new(
                "refresh",
                ButtonStyles::new(px(360.), px(36.))
                    .border_color(rgb(0x3b82f6))
                    .fill_color(rgb(0x1e293b))
                    .text("Refresh Git Status")
                    .text_color(rgb(0xf1f5f9))
                    .text_size(px(14.)),
            )
            .on_click(move |_id, _window, cx| {
                let statuses = vec![
                    GitFileState {
                        path: "src/main.rs".into(),
                        status: GitStatus::Unmodified,
                    },
                    GitFileState {
                        path: "src/parser.rs".into(),
                        status: GitStatus::Added,
                    },
                    GitFileState {
                        path: "src/lexer.rs".into(),
                        status: GitStatus::Deleted,
                    },
                    GitFileState {
                        path: "docs/guide.md".into(),
                        status: GitStatus::Modified,
                    },
                ];
                let _ = weak_refresh.update(cx, |root, cx| {
                    root.list.update(cx, |list, cx| {
                        list.set_git_status(statuses, cx);
                    });
                    root.status = "git status refreshed".into();
                    cx.notify();
                });
            }),
            cx,
        );

        RootView {
            list,
            refresh_button,
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
            .child(self.list.clone())
            .child(div().mt(px(16.)).child(self.refresh_button.clone()))
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
