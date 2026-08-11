use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::button::{Button, ButtonConfig, ButtonStyles};
use gui_widgets::tree::{Tree, TreeConfig, TreeNode, TreeStyles};
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

fn sample_root() -> TreeNode {
    TreeNode::new("src", "src")
        .child(TreeNode::new("gui-widgets", "gui-widgets").child(
            TreeNode::new("button", "button")
                .child(TreeNode::new("button-rs", "button.rs"))
                .child(TreeNode::new("button-mod", "mod.rs")),
        ))
        .child(TreeNode::new("gui-workbench", "gui-workbench"))
    .child(TreeNode::new("cargo-toml", "Cargo.toml"))
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_click = cx.weak_entity();
        let weak_hover = weak_click.clone();
        let weak_toggle = weak_click.clone();
        let weak_guides = cx.weak_entity();

        let tree_styles =
            TreeStyles::new(px(360.), px(300.), rgb(0x334155).into(), rgb(0x94a3b8).into())
            .background_color(rgb(0x0f172a))
            .border_color(rgb(0x334155))
            .item_height(px(30.))
            .item_text_color(rgb(0xe2e8f0))
            .show_indent_guides(false);

        let tree = Tree::new(
            TreeConfig::new("file-tree", tree_styles, sample_root())
                .on_item_click(move |_tree_id, node_id, _window, cx| {
                    println!("node clicked: {node_id}");
                    let _ = weak_click.update(cx, |root, cx| {
                        root.status = format!("clicked: {node_id}").into();
                        cx.notify();
                    });
                })
                .on_item_hover(move |_tree_id, node_id, hovered, _window, cx| {
                    let _ = weak_hover.update(cx, |root, cx| {
                        root.status = format!("hovered: {node_id} {hovered}").into();
                        cx.notify();
                    });
                })
                .on_toggle(move |_tree_id, node_id, expanded, _window, cx| {
                    println!("toggled: {node_id} expanded={expanded}");
                    let _ = weak_toggle.update(cx, |root, cx| {
                        root.status = format!("toggled: {node_id} {expanded}").into();
                        cx.notify();
                    });
                }),
            cx,
        );

        let guides_button = Button::new(
            ButtonConfig::new(
                "guides",
                ButtonStyles::new(px(360.), px(36.))
                    .border_color(rgb(0x3b82f6))
                    .fill_color(rgb(0x1e293b))
                    .text("Toggle Indent Guides")
                    .text_color(rgb(0xf1f5f9))
                    .text_size(px(14.)),
            )
            .on_click(move |_id, _window, cx| {
                let _ = weak_guides.update(cx, |root, cx| {
                    let show = root.guides_visible;
                    root.guides_visible = !show;
                    root.tree.update(cx, |tree, cx| {
                        let styles =
                            TreeStyles::new(px(360.), px(300.), rgb(0x334155).into(), rgb(0x94a3b8).into())
                            .background_color(rgb(0x0f172a))
                            .border_color(rgb(0x334155))
                            .item_height(px(30.))
                            .item_text_color(rgb(0xe2e8f0))
                            .show_indent_guides(!show);
                        tree.set_styles(styles, cx);
                    });
                    root.status = format!("indent guides: {}", !show).into();
                    cx.notify();
                });
            }),
            cx,
        );

        RootView {
            tree,
            guides_button,
            guides_visible: false,
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
            .child(self.tree.clone())
            .child(div().mt(px(16.)).child(self.guides_button.clone()))
            .child(
                div()
                    .mt(px(16.))
                    .text_color(rgb(0x94a3b8))
                    .text_size(px(14.))
                    .child(self.status.clone()),
            )
    }
}

struct RootView {
    tree: Entity<Tree>,
    guides_button: Entity<Button>,
    guides_visible: bool,
    status: SharedString,
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
