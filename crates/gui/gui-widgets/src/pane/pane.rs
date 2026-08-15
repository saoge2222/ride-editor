use crate::button::{Button, ButtonConfig, ButtonStyles};
use gpui::{
    App, AppContext, Context, Div, ElementId, Entity, EntityId, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Point, Render, SharedString, Styled, WeakEntity, Window,
    div, px,
};

const DEFAULT_COLUMNS: usize = 1;
const DEFAULT_ROWS: usize = 1;
const DEFAULT_ITEM_WIDTH: Pixels = px(120.);
const DEFAULT_ITEM_HEIGHT: Pixels = px(32.);
const DEFAULT_ITEM_TEXT_SIZE: Pixels = px(14.);
const DEFAULT_BORDER_WIDTH: Pixels = px(1.);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneAlign {
    Start,
    Center,
    End,
    SpaceBetween,
}

impl PaneAlign {
    fn apply(self, row: Div) -> Div {
        match self {
            PaneAlign::Start => row.justify_start(),
            PaneAlign::Center => row.justify_center(),
            PaneAlign::End => row.justify_end(),
            PaneAlign::SpaceBetween => row.justify_between(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaneStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub background_color: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub columns: usize,
    pub rows: usize,
    pub align: PaneAlign,
    pub item_width: Pixels,
    pub item_height: Pixels,
    pub item_border_color: Option<Hsla>,
    pub item_background_color: Option<Hsla>,
    pub item_text_size: Pixels,
    pub item_text_color: Option<Hsla>,
}

impl PaneStyles {
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self {
            width,
            height,
            background_color: None,
            border_color: None,
            columns: DEFAULT_COLUMNS,
            rows: DEFAULT_ROWS,
            align: PaneAlign::Start,
            item_width: DEFAULT_ITEM_WIDTH,
            item_height: DEFAULT_ITEM_HEIGHT,
            item_border_color: None,
            item_background_color: None,
            item_text_size: DEFAULT_ITEM_TEXT_SIZE,
            item_text_color: None,
        }
    }

    pub fn background_color(mut self, color: impl Into<Hsla>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn align(mut self, align: PaneAlign) -> Self {
        self.align = align;
        self
    }

    pub fn item_width(mut self, width: Pixels) -> Self {
        self.item_width = width;
        self
    }

    pub fn item_height(mut self, height: Pixels) -> Self {
        self.item_height = height;
        self
    }

    pub fn item_border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.item_border_color = Some(color.into());
        self
    }

    pub fn item_background_color(mut self, color: impl Into<Hsla>) -> Self {
        self.item_background_color = Some(color.into());
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

#[derive(Clone, Debug)]
pub struct PaneItemConfig {
    pub id: SharedString,
    pub text: SharedString,
    pub row: usize,
    pub column: usize,
    pub offset: Point<Pixels>,
}

impl PaneItemConfig {
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            row: 0,
            column: 0,
            offset: Point::new(px(0.), px(0.)),
        }
    }

    pub fn row_col(mut self, row: usize, column: usize) -> Self {
        self.row = row;
        self.column = column;
        self
    }

    pub fn offset(mut self, x: Pixels, y: Pixels) -> Self {
        self.offset = Point::new(x, y);
        self
    }
}

pub struct PaneConfig {
    id: SharedString,
    styles: PaneStyles,
    items: Vec<PaneItemConfig>,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl PaneConfig {
    pub fn new(
        id: impl Into<SharedString>,
        styles: PaneStyles,
        items: Vec<PaneItemConfig>,
    ) -> Self {
        Self {
            id: id.into(),
            styles,
            items,
            on_item_click: None,
            on_item_hover: None,
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
}

pub struct Pane {
    id: SharedString,
    styles: PaneStyles,
    pending: Vec<PaneItemConfig>,
    items: Vec<Entity<Button>>,
    meta: Vec<(usize, usize, Point<Pixels>)>,
    visible: bool,
    entity_id: EntityId,
    weak_self: WeakEntity<Pane>,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl Pane {
    pub fn new(config: PaneConfig, cx: &mut App) -> Entity<Pane> {
        let PaneConfig {
            id,
            styles,
            items,
            on_item_click,
            on_item_hover,
        } = config;
        cx.new(|cx| {
            let mut pane = Pane {
                id,
                styles,
                pending: Vec::new(),
                items: Vec::new(),
                meta: Vec::new(),
                visible: true,
                entity_id: cx.entity_id(),
                weak_self: cx.weak_entity(),
                on_item_click,
                on_item_hover,
            };
            pane.set_items(items, cx);
            pane
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut App) {
        self.visible = visible;
        cx.notify(self.entity_id);
    }

    pub fn set_items(&mut self, items: Vec<PaneItemConfig>, cx: &mut App) {
        self.pending = items;
        let styles = self.styles.clone();
        let weak_click = self.weak_self.clone();
        let weak_hover = weak_click.clone();
        let button_width = if styles.border_color.is_some() {
            Pixels::from(f32::from(styles.item_width) - 2. * f32::from(DEFAULT_BORDER_WIDTH))
        } else {
            styles.item_width
        };
        let mut buttons = Vec::with_capacity(self.pending.len());
        let mut meta = Vec::with_capacity(self.pending.len());
        for item in &self.pending {
            let weak_click = weak_click.clone();
            let weak_hover = weak_hover.clone();
            let item_id = item.id.clone();
            let item_text = item.text.clone();
            let click_id = item_id.clone();
            let hover_id = item_id.clone();
            let mut button_styles = ButtonStyles::new(button_width, styles.item_height)
                .text(item_text)
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
            buttons.push(Button::new(
                ButtonConfig::new(item_id, button_styles)
                    .on_click(move |_button_id, window, cx| {
                        let _ = weak_click.update(cx, |pane, cx| {
                            if let Some(handler) = &pane.on_item_click {
                                handler(&pane.id, &click_id, window, cx);
                            }
                        });
                    })
                    .on_hover(move |_button_id, hovered, window, cx| {
                        let _ = weak_hover.update(cx, |pane, cx| {
                            if let Some(handler) = &pane.on_item_hover {
                                handler(&pane.id, &hover_id, hovered, window, cx);
                            }
                        });
                    }),
                cx,
            ));
            meta.push((item.row, item.column, item.offset));
        }
        self.items = buttons;
        self.meta = meta;
        cx.notify(self.entity_id);
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().id(ElementId::Name(format!("{}-pane-hidden", self.id).into()));
        }

        let mut element = div()
            .id(ElementId::Name(format!("{}-pane", self.id).into()))
            .w(self.styles.width)
            .h(self.styles.height)
            .flex()
            .flex_col()
            .overflow_hidden();
        if let Some(color) = self.styles.background_color {
            element = element.bg(color);
        }
        if let Some(color) = self.styles.border_color {
            element = element.border_color(color).border(DEFAULT_BORDER_WIDTH);
        }

        let rows = self.styles.rows;
        let columns = self.styles.columns;
        let row_height = px(f32::from(self.styles.height) / rows as f32);
        for row in 0..rows {
            let mut row_element = self.styles.align.apply(div().flex().h(row_height).items_center());
            for index in 0..self.items.len() {
                let (item_row, item_column, offset) = self.meta[index];
                if item_row == row && item_column < columns {
                    let button = self.items[index].clone();
                    row_element = row_element.child(
                        div()
                            .relative()
                            .ml(offset.x)
                            .mt(offset.y)
                            .child(button),
                    );
                }
            }
            element = element.child(row_element);
        }
        element
    }
}
