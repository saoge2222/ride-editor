use crate::button::{Button, ButtonConfig, ButtonStyles};
use crate::data::{GitFileState, GitStatus};
use gpui::{
    App, AppContext, ClickEvent, Context, ElementId, Entity, EntityId, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement, Styled,
    WeakEntity, Window, div, px,
};
use std::collections::{HashMap, HashSet};

const DEFAULT_INDENT_PER_LEVEL: Pixels = px(16.);
const DEFAULT_ITEM_HEIGHT: Pixels = px(24.);
const DEFAULT_ITEM_TEXT_SIZE: Pixels = px(14.);
const DEFAULT_BORDER_WIDTH: Pixels = px(1.);
const ARROW_COLLAPSED: &str = "▸";
const ARROW_EXPANDED: &str = "▾";
const GUIDE_WIDTH: Pixels = px(1.);
const ARROW_WIDTH: Pixels = px(16.);
const ARROW_TEXT_SIZE: Pixels = px(12.);

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: SharedString,
    pub text: SharedString,
    pub icon_path: Option<SharedString>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon_path: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TreeStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub indent_per_level: Pixels,
    pub show_indent_guides: bool,
    pub background_color: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub item_height: Pixels,
    pub item_background_color: Option<Hsla>,
    pub item_border_color: Option<Hsla>,
    pub item_text_size: Pixels,
    pub item_text_color: Option<Hsla>,
    pub guide_color: Hsla,
    pub arrow_color: Hsla,
}

impl TreeStyles {
    pub fn new(width: Pixels, height: Pixels, guide_color: Hsla, arrow_color: Hsla) -> Self {
        Self {
            width,
            height,
            indent_per_level: DEFAULT_INDENT_PER_LEVEL,
            show_indent_guides: false,
            background_color: None,
            border_color: None,
            item_height: DEFAULT_ITEM_HEIGHT,
            item_background_color: None,
            item_border_color: None,
            item_text_size: DEFAULT_ITEM_TEXT_SIZE,
            item_text_color: None,
            guide_color,
            arrow_color,
        }
    }

    pub fn indent_per_level(mut self, indent: Pixels) -> Self {
        self.indent_per_level = indent;
        self
    }

    pub fn show_indent_guides(mut self, show: bool) -> Self {
        self.show_indent_guides = show;
        self
    }

    pub fn background_color(mut self, color: impl Into<Hsla>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn item_height(mut self, height: Pixels) -> Self {
        self.item_height = height;
        self
    }

    pub fn item_background_color(mut self, color: impl Into<Hsla>) -> Self {
        self.item_background_color = Some(color.into());
        self
    }

    pub fn item_border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.item_border_color = Some(color.into());
        self
    }

    pub fn item_text_size(mut self, size: Pixels) -> Self {
        self.item_text_size = size;
        self
    }

    pub fn item_text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.item_text_color = Some(color.into());
        self
    }
}

pub struct TreeConfig {
    id: SharedString,
    styles: TreeStyles,
    root: TreeNode,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
    on_toggle: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl TreeConfig {
    pub fn new(
        id: impl Into<SharedString>,
        styles: TreeStyles,
        root: TreeNode,
    ) -> Self {
        Self {
            id: id.into(),
            styles,
            root,
            on_item_click: None,
            on_item_hover: None,
            on_toggle: None,
        }
    }

    pub fn on_item_click(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_item_click = Some(Box::new(handler));
        self
    }

    pub fn on_item_hover(
        mut self,
        handler: impl Fn(&str, &str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_item_hover = Some(Box::new(handler));
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&str, &str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

pub struct Tree {
    id: SharedString,
    styles: TreeStyles,
    root: TreeNode,
    collapsed: HashSet<SharedString>,
    buttons: HashMap<SharedString, Entity<Button>>,
    entity_id: EntityId,
    weak_self: WeakEntity<Tree>,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
    on_toggle: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl Tree {
    pub fn new(config: TreeConfig, cx: &mut App) -> Entity<Tree> {
        let TreeConfig {
            id,
            styles,
            root,
            on_item_click,
            on_item_hover,
            on_toggle,
        } = config;
        cx.new(|cx| {
            let mut tree = Tree {
                id,
                styles,
                root: TreeNode::new("", ""),
                collapsed: HashSet::new(),
                buttons: HashMap::new(),
                entity_id: cx.entity_id(),
                weak_self: cx.weak_entity(),
                on_item_click,
                on_item_hover,
                on_toggle,
            };
            tree.set_root(root, cx);
            tree
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_styles(&mut self, styles: TreeStyles, cx: &mut App) {
        self.styles = styles;
        cx.notify(self.entity_id);
    }

    pub fn set_root(&mut self, root: TreeNode, cx: &mut App) {
        self.root = root;
        self.buttons.clear();
        let styles = self.styles.clone();
        let weak_click = self.weak_self.clone();
        let weak_hover = weak_click.clone();
        let nodes = collect_node_refs(&self.root);
        for node in nodes {
            let weak_click = weak_click.clone();
            let weak_hover = weak_hover.clone();
            let mut button_styles = ButtonStyles::new(px(0.), styles.item_height)
                .auto_width()
                .text(node.text.clone())
                .text_size(styles.item_text_size);
            if let Some(color) = styles.item_border_color {
                button_styles = button_styles.border_color(color);
            }
            if let Some(color) = styles.item_background_color {
                button_styles = button_styles.fill_color(color);
            }
            if let Some(color) = styles.item_text_color {
                button_styles = button_styles.text_color(color);
            }
            if let Some(path) = &node.icon_path {
                button_styles = button_styles.icon(path.clone(), px(16.), px(16.));
            }
            let click_id = node.id.clone();
            let hover_id = node.id.clone();
            let button = Button::new(
                ButtonConfig::new(node.id.clone(), button_styles)
                    .on_click(move |_button_id, window, cx| {
                        let _ = weak_click.update(cx, |tree, cx| {
                            if let Some(handler) = &tree.on_item_click {
                                handler(&tree.id, &click_id, window, cx);
                            }
                        });
                    })
                    .on_hover(move |_button_id, hovered, window, cx| {
                        let _ = weak_hover.update(cx, |tree, cx| {
                            if let Some(handler) = &tree.on_item_hover {
                                handler(&tree.id, &hover_id, hovered, window, cx);
                            }
                        });
                    }),
                cx,
            );
            self.buttons.insert(node.id.clone(), button);
        }
        cx.notify(self.entity_id);
    }

    pub fn toggle_node(&mut self, id: &str, window: &mut Window, cx: &mut App) {
        let expanded = if self.collapsed.contains(id) {
            self.collapsed.remove(id);
            true
        } else {
            self.collapsed.insert(id.to_string().into());
            false
        };
        if let Some(handler) = &self.on_toggle {
            handler(&self.id, id, expanded, window, cx);
        }
        cx.notify(self.entity_id);
    }

    pub fn is_collapsed(&self, id: &str) -> bool {
        self.collapsed.contains(id)
    }

    pub fn set_git_status(&mut self, statuses: Vec<GitFileState>, cx: &mut App) {
        update_git_status(&mut self.root, &statuses);
        self.set_root(self.root.clone(), cx);
    }

    fn visible_rows(&self) -> Vec<VisibleRow> {
        let mut rows = Vec::new();
        collect_visible(&self.root, 0, &self.collapsed, &mut rows);
        rows
    }
}

struct VisibleRow {
    depth: u32,
    node_id: SharedString,
    has_children: bool,
}

fn status_text(status: &GitFileState) -> String {
    let prefix = match status.status {
        GitStatus::Unmodified => " ",
        GitStatus::Modified => "M",
        GitStatus::Added => "A",
        GitStatus::Deleted => "D",
        GitStatus::Renamed => "R",
        GitStatus::Untracked => "??",
    };
    format!("{prefix} {}", status.path)
}

fn collect_visible(node: &TreeNode, depth: u32, collapsed: &HashSet<SharedString>, rows: &mut Vec<VisibleRow>) {
    rows.push(VisibleRow {
        depth,
        node_id: node.id.clone(),
        has_children: !node.children.is_empty(),
    });
    if collapsed.contains(&node.id) {
        return;
    }
    for child in &node.children {
        collect_visible(child, depth + 1, collapsed, rows);
    }
}

fn collect_node_refs(node: &TreeNode) -> Vec<&TreeNode> {
    let mut nodes = vec![node];
    for child in &node.children {
        nodes.extend(collect_node_refs(child));
    }
    nodes
}

fn update_git_status(node: &mut TreeNode, statuses: &[GitFileState]) {
    for status in statuses {
        if node.id == status.path {
            node.text = status_text(status).into();
        }
    }
    for child in &mut node.children {
        update_git_status(child, statuses);
    }
}

impl Render for Tree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut element = div()
            .w(self.styles.width)
            .h(self.styles.height)
            .flex()
            .flex_col()
            .overflow_y_hidden();

        if let Some(color) = self.styles.background_color {
            element = element.bg(color);
        }
        if let Some(color) = self.styles.border_color {
            element = element.border_color(color).border(DEFAULT_BORDER_WIDTH);
        }

        let rows = self.visible_rows();
        for row in rows {
            let row_element = self.render_row(&row, cx);
            element = element.child(row_element);
        }

        element
    }
}

impl Tree {
    fn render_row(&self, row: &VisibleRow, cx: &mut Context<Self>) -> impl IntoElement {
        let indent = self.styles.indent_per_level;
        let mut indents = Vec::new();
        for _ in 0..row.depth {
            let mut cell = div().w(indent).h(self.styles.item_height).flex().justify_end();
            if self.styles.show_indent_guides {
                cell = cell.child(
                    div()
                        .w(GUIDE_WIDTH)
                        .h(self.styles.item_height)
                        .bg(self.styles.guide_color),
                );
            }
            indents.push(cell);
        }

        let arrow_id = format!("{}-{}-arrow", self.id, row.node_id);
        let mut arrow = div()
            .id(ElementId::Name(arrow_id.into()))
            .w(ARROW_WIDTH)
            .h(self.styles.item_height)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_size(ARROW_TEXT_SIZE)
            .text_color(self.styles.arrow_color);
        if row.has_children {
            let node_id = row.node_id.clone();
            let collapsed = self.collapsed.contains(&row.node_id);
            let glyph = if collapsed { ARROW_COLLAPSED } else { ARROW_EXPANDED };
            arrow = arrow
                .child(glyph)
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.toggle_node(&node_id, window, cx);
                }));
        }

        let button = self.buttons.get(&row.node_id).cloned();

        let mut row_element = div().flex().h(self.styles.item_height);
        for cell in indents {
            row_element = row_element.child(cell);
        }
        row_element = row_element.child(arrow);
        if let Some(button) = button {
            row_element = row_element.child(button);
        }
        row_element
    }
}
