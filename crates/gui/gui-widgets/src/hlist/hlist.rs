use crate::button::{Button, ButtonConfig, ButtonStyles};
use crate::data::{GitFileState, GitStatus, LspDiagnostic};
use gpui::{
    App, AppContext, Context, ElementId, Entity, EntityId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    WeakEntity, Window, div, px,
};

const DEFAULT_ITEM_WIDTH: Pixels = px(120.);
const DEFAULT_ITEM_HEIGHT: Pixels = px(32.);
const DEFAULT_ITEM_TEXT_SIZE: Pixels = px(14.);
const DEFAULT_BORDER_WIDTH: Pixels = px(1.);

#[derive(Clone, Debug)]
pub struct HListStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub background_color: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub item_width: Pixels,
    pub item_height: Pixels,
    pub item_border_color: Option<Hsla>,
    pub item_background_color: Option<Hsla>,
    pub item_text_size: Pixels,
    pub item_text_color: Option<Hsla>,
}

impl HListStyles {
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self {
            width,
            height,
            background_color: None,
            border_color: None,
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
pub struct HListItemConfig {
    pub id: SharedString,
    pub text: SharedString,
    pub icon_path: Option<SharedString>,
}

impl HListItemConfig {
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon_path: None,
        }
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon_path = Some(path.into());
        self
    }
}

pub struct HListConfig {
    id: SharedString,
    styles: HListStyles,
    items: Vec<HListItemConfig>,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl HListConfig {
    pub fn new(
        id: impl Into<SharedString>,
        styles: HListStyles,
        items: Vec<HListItemConfig>,
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

pub struct HList {
    id: SharedString,
    styles: HListStyles,
    pending: Vec<HListItemConfig>,
    items: Vec<Entity<Button>>,
    scroll: ScrollHandle,
    entity_id: EntityId,
    weak_self: WeakEntity<HList>,
    on_item_click: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_item_hover: Option<Box<dyn Fn(&str, &str, bool, &mut Window, &mut App) + 'static>>,
}

impl HList {
    pub fn new(config: HListConfig, cx: &mut App) -> Entity<HList> {
        let HListConfig {
            id,
            styles,
            items,
            on_item_click,
            on_item_hover,
        } = config;
        cx.new(|cx| {
            let mut list = HList {
                id,
                styles,
                pending: Vec::new(),
                items: Vec::new(),
                scroll: ScrollHandle::new(),
                entity_id: cx.entity_id(),
                weak_self: cx.weak_entity(),
                on_item_click,
                on_item_hover,
            };
            list.set_items(items, cx);
            list
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_items(&mut self, items: Vec<HListItemConfig>, cx: &mut App) {
        self.pending = items;
        let styles = self.styles.clone();
        let weak_click = self.weak_self.clone();
        let weak_hover = weak_click.clone();
        let button_width = if styles.border_color.is_some() {
            Pixels::from(f32::from(styles.item_width) - 2. * f32::from(DEFAULT_BORDER_WIDTH))
        } else {
            styles.item_width
        };
        self.items = self
            .pending
            .iter()
            .map(|item| {
                let weak_click = weak_click.clone();
                let weak_hover = weak_hover.clone();
                let item_id = item.id.clone();
                let item_text = item.text.clone();
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
                if let Some(path) = &item.icon_path {
                    button_styles = button_styles.icon(path.clone(), px(16.), px(16.));
                }
                let click_item = item_id.clone();
                let hover_item = item_id.clone();
                Button::new(
                    ButtonConfig::new(item_id, button_styles)
                        .on_click(move |_button_id, window, cx| {
                            let _ = weak_click.update(cx, |list, cx| {
                                if let Some(handler) = &list.on_item_click {
                                    handler(&list.id, &click_item, window, cx);
                                }
                            });
                        })
                        .on_hover(move |_button_id, hovered, window, cx| {
                            let _ = weak_hover.update(cx, |list, cx| {
                                if let Some(handler) = &list.on_item_hover {
                                    handler(&list.id, &hover_item, hovered, window, cx);
                                }
                            });
                        }),
                    cx,
                )
            })
            .collect();
        cx.notify(self.entity_id);
    }

    pub fn set_git_status(&mut self, statuses: Vec<GitFileState>, cx: &mut App) {
        let items = statuses
            .into_iter()
            .map(|status| {
                let prefix = match status.status {
                    GitStatus::Unmodified => " ",
                    GitStatus::Modified => "M",
                    GitStatus::Added => "A",
                    GitStatus::Deleted => "D",
                    GitStatus::Renamed => "R",
                    GitStatus::Untracked => "??",
                };
                HListItemConfig::new(status.path.clone(), format!("{prefix} {}", status.path))
            })
            .collect();
        self.set_items(items, cx);
    }

    pub fn set_lsp_result(&mut self, diagnostics: Vec<LspDiagnostic>, cx: &mut App) {
        let items = diagnostics
            .into_iter()
            .map(|diagnostic| {
                let severity = match diagnostic.severity {
                    crate::data::DiagnosticSeverity::Error => "error",
                    crate::data::DiagnosticSeverity::Warning => "warning",
                    crate::data::DiagnosticSeverity::Information => "info",
                    crate::data::DiagnosticSeverity::Hint => "hint",
                };
                HListItemConfig::new(
                    diagnostic.message.clone(),
                    format!(
                        "{}:{} {}: {}",
                        diagnostic.range.start.line + 1,
                        diagnostic.range.start.character + 1,
                        severity,
                        diagnostic.message
                    ),
                )
            })
            .collect();
        self.set_items(items, cx);
    }
}

impl Render for HList {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut element = div()
            .id(ElementId::Name(format!("{}-scroll", self.id).into()))
            .w(self.styles.width)
            .h(self.styles.height)
            .flex()
            .flex_row()
            .overflow_x_scroll()
            .track_scroll(&self.scroll);

        if let Some(color) = self.styles.background_color {
            element = element.bg(color);
        }
        if let Some(color) = self.styles.border_color {
            element = element.border_color(color).border(DEFAULT_BORDER_WIDTH);
        }

        for button in &self.items {
            element = element.child(button.clone());
        }

        element
    }
}
