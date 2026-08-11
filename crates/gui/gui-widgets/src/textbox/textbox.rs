use gpui::{
    App, AppContext, Context, ElementId, Entity, EntityId, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, div, px,
};

const DEFAULT_BORDER_WIDTH: Pixels = px(1.);
const DEFAULT_TEXT_SIZE: Pixels = px(14.);
const DEFAULT_TITLE_SIZE: Pixels = px(12.);
const DEFAULT_LINE_COUNT: usize = 1;
const LINE_HEIGHT_MULTIPLIER: f32 = 1.5;
const TEXT_PADDING_X: Pixels = px(8.);
const CARET_WIDTH: Pixels = px(2.);
const TITLE_INSET_X: Pixels = px(12.);
const TITLE_INSET_Y: Pixels = px(8.);

#[derive(Clone, Debug)]
pub struct TextBoxTitle {
    pub text: SharedString,
    pub font_family: Option<SharedString>,
    pub text_size: Pixels,
    pub color: Hsla,
}

impl TextBoxTitle {
    pub fn new(text: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        Self {
            text: text.into(),
            font_family: None,
            text_size: DEFAULT_TITLE_SIZE,
            color: color.into(),
        }
    }

    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn text_size(mut self, size: Pixels) -> Self {
        self.text_size = size;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Placeholder {
    pub text: SharedString,
    pub icon_path: Option<SharedString>,
    pub icon_width: Pixels,
    pub icon_height: Pixels,
}

impl Placeholder {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            icon_path: None,
            icon_width: px(14.),
            icon_height: px(14.),
        }
    }

    pub fn icon(mut self, path: impl Into<SharedString>, width: Pixels, height: Pixels) -> Self {
        self.icon_path = Some(path.into());
        self.icon_width = width;
        self.icon_height = height;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TextboxStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub fill_color: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: Pixels,
    pub title_top: Option<TextBoxTitle>,
    pub title_bottom: Option<TextBoxTitle>,
    pub title_left: Option<TextBoxTitle>,
    pub title_right: Option<TextBoxTitle>,
    pub font_family: Option<SharedString>,
    pub text_size: Pixels,
    pub text_color: Hsla,
    pub caret_color: Hsla,
    pub placeholder_color: Hsla,
    pub line_count: usize,
    pub placeholder: Option<Placeholder>,
}

impl TextboxStyles {
    pub fn new(
        width: Pixels,
        height: Pixels,
        text_color: impl Into<Hsla>,
        caret_color: impl Into<Hsla>,
        placeholder_color: impl Into<Hsla>,
    ) -> Self {
        Self {
            width,
            height,
            fill_color: None,
            border_color: None,
            border_width: DEFAULT_BORDER_WIDTH,
            title_top: None,
            title_bottom: None,
            title_left: None,
            title_right: None,
            font_family: None,
            text_size: DEFAULT_TEXT_SIZE,
            text_color: text_color.into(),
            caret_color: caret_color.into(),
            placeholder_color: placeholder_color.into(),
            line_count: DEFAULT_LINE_COUNT,
            placeholder: None,
        }
    }

    pub fn fill_color(mut self, color: impl Into<Hsla>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn border_width(mut self, width: Pixels) -> Self {
        self.border_width = width;
        self
    }

    pub fn title_top(mut self, title: TextBoxTitle) -> Self {
        self.title_top = Some(title);
        self
    }

    pub fn title_bottom(mut self, title: TextBoxTitle) -> Self {
        self.title_bottom = Some(title);
        self
    }

    pub fn title_left(mut self, title: TextBoxTitle) -> Self {
        self.title_left = Some(title);
        self
    }

    pub fn title_right(mut self, title: TextBoxTitle) -> Self {
        self.title_right = Some(title);
        self
    }

    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn text_size(mut self, size: Pixels) -> Self {
        self.text_size = size;
        self
    }

    pub fn line_count(mut self, count: usize) -> Self {
        self.line_count = count;
        self
    }

    pub fn placeholder(mut self, placeholder: Placeholder) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
}

pub struct TextboxConfig {
    id: SharedString,
    styles: TextboxStyles,
    on_text_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_hover: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
    on_has_text: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
}

impl TextboxConfig {
    pub fn new(id: impl Into<SharedString>, styles: TextboxStyles) -> Self {
        Self {
            id: id.into(),
            styles,
            on_text_changed: None,
            on_hover: None,
            on_has_text: None,
        }
    }

    pub fn on_text_changed(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_text_changed = Some(Box::new(handler));
        self
    }

    pub fn on_hover(
        mut self,
        handler: impl Fn(&str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover = Some(Box::new(handler));
        self
    }

    pub fn on_has_text(
        mut self,
        handler: impl Fn(&str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_has_text = Some(Box::new(handler));
        self
    }
}

pub struct Textbox {
    id: SharedString,
    styles: TextboxStyles,
    text: String,
    cursor_column: u32,
    hovered: bool,
    focused: bool,
    prev_has_text: bool,
    entity_id: EntityId,
    focus_handle: FocusHandle,
    focus_events_registered: bool,
    focus_subscriptions: Vec<Subscription>,
    on_text_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_hover: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
    on_has_text: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
}

impl Textbox {
    pub fn new(config: TextboxConfig, cx: &mut App) -> Entity<Textbox> {
        cx.new(|cx| Textbox {
            id: config.id,
            styles: config.styles,
            text: String::new(),
            cursor_column: 0,
            hovered: false,
            focused: false,
            prev_has_text: false,
            entity_id: cx.entity_id(),
            focus_handle: cx.focus_handle(),
            focus_events_registered: false,
            focus_subscriptions: Vec::new(),
            on_text_changed: config.on_text_changed,
            on_hover: config.on_hover,
            on_has_text: config.on_has_text,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut App) {
        self.text = text.into();
        self.cursor_column = self.column_width(&self.text);
        cx.notify(self.entity_id);
    }

    pub fn set_styles(&mut self, styles: TextboxStyles) {
        self.styles = styles;
    }

    fn column_width(&self, text: &str) -> u32 {
        text.chars()
            .map(|c| crate::editor::buffer::char_width_kind(c as u32) as u32)
            .sum()
    }

    fn emit_text_events(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(handler) = &self.on_text_changed {
            let id = self.id.clone();
            let text = self.text.clone();
            handler(&id, &text, window, cx);
        }
        let has_text = self.has_text();
        if has_text != self.prev_has_text {
            self.prev_has_text = has_text;
            if let Some(handler) = &self.on_has_text {
                let id = self.id.clone();
                handler(&id, has_text, window, cx);
            }
        }
    }

    fn char_at_cursor(&self) -> Option<char> {
        let index = self.char_index_at_cursor();
        self.text[index..].chars().next()
    }

    fn char_before_cursor(&self) -> Option<char> {
        if self.cursor_column == 0 {
            return None;
        }
        let mut column = 0;
        let mut last = None;
        for c in self.text.chars() {
            if column >= self.cursor_column {
                break;
            }
            last = Some(c);
            column += crate::editor::buffer::char_width_kind(c as u32) as u32;
        }
        last
    }

    fn char_index_at_cursor(&self) -> usize {
        let mut column = 0;
        for (i, c) in self.text.char_indices() {
            if column >= self.cursor_column {
                return i;
            }
            column += crate::editor::buffer::char_width_kind(c as u32) as u32;
        }
        self.text.len()
    }

    fn handle_key(&mut self, key: &str, key_char: Option<&str>, window: &mut Window, cx: &mut App) {
        match key {
            "backspace" => {
                if let Some(c) = self.char_before_cursor() {
                    let index = self.char_index_at_cursor() - c.len_utf8();
                    self.text.remove(index);
                    self.cursor_column -=
                        crate::editor::buffer::char_width_kind(c as u32) as u32;
                    self.emit_text_events(window, cx);
                }
            }
            "left" => {
                if let Some(c) = self.char_before_cursor() {
                    self.cursor_column -=
                        crate::editor::buffer::char_width_kind(c as u32) as u32;
                }
            }
            "right" => {
                if let Some(c) = self.char_at_cursor() {
                    self.cursor_column +=
                        crate::editor::buffer::char_width_kind(c as u32) as u32;
                }
            }
            "home" => self.cursor_column = 0,
            "end" => self.cursor_column = self.column_width(&self.text),
            _ => {
                if let Some(ch) = key_char {
                    let index = self.char_index_at_cursor();
                    self.text.insert_str(index, ch);
                    self.cursor_column += crate::editor::buffer::char_width_kind(
                        ch.chars().next().map(|c| c as u32).unwrap_or(0),
                    ) as u32;
                    self.emit_text_events(window, cx);
                }
            }
        }
    }
}

impl Focusable for Textbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Textbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_events_registered {
            self.focus_events_registered = true;
            let handle = self.focus_handle.clone();
            let weak_in = cx.weak_entity();
            let weak_out = cx.weak_entity();
            let subscription_in = window.on_focus_in(&handle, cx, move |_window, cx| {
                let _ = weak_in.update(cx, |textbox, cx| {
                    textbox.focused = true;
                    cx.notify();
                });
            });
            let subscription_out = window.on_focus_out(&handle, cx, move |_event, _window, cx| {
                let _ = weak_out.update(cx, |textbox, cx| {
                    textbox.focused = false;
                    cx.notify();
                });
            });
            self.focus_subscriptions.push(subscription_in);
            self.focus_subscriptions.push(subscription_out);
        }

        let styles = self.styles.clone();
        let content_id = format!("{}-content", self.id);
        let mut content = div()
            .absolute()
            .left(px(0.))
            .right(px(0.))
            .top(px(0.))
            .bottom(px(0.))
            .flex()
            .items_center()
            .px(TEXT_PADDING_X)
            .cursor_text()
            .id(ElementId::Name(content_id.into()))
            .focusable()
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _: &gpui::MouseDownEvent, window, _| {
                window.focus(&this.focus_handle);
            }))
            .on_hover(cx.listener(|this, hovered: &bool, window, cx| {
                this.hovered = *hovered;
                if let Some(handler) = &this.on_hover {
                    handler(&this.id, *hovered, window, cx);
                }
                cx.notify();
            }))
            .on_key_down(cx.listener(
                |this, event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let key_char = event.keystroke.key_char.as_deref();
                    if key_char.is_some()
                        && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                    {
                        return;
                    }
                    this.handle_key(key, key_char, window, cx);
                    cx.notify();
                },
            ));

        if let Some(color) = styles.border_color {
            content = content.border_color(color).border(styles.border_width);
        }
        if let Some(color) = styles.fill_color {
            content = content.bg(color);
        }

        if self.text.is_empty() {
            if let Some(placeholder) = &styles.placeholder {
                let mut placeholder_div = div()
                    .flex()
                    .items_center()
                    .text_size(styles.text_size)
                    .text_color(styles.placeholder_color)
                    .child(placeholder.text.clone());
                if let Some(path) = &placeholder.icon_path {
                    placeholder_div = placeholder_div.child(
                        gpui::svg()
                            .path(path.clone())
                            .w(placeholder.icon_width)
                            .h(placeholder.icon_height)
                            .text_color(styles.placeholder_color),
                    );
                }
                content = content.child(placeholder_div);
            }
        } else {
            let before = self.char_index_at_cursor();
            let after = before + self.char_at_cursor().map(|c| c.len_utf8()).unwrap_or(0);
            let before_text: SharedString = self.text[..before].to_string().into();
            let after_text: SharedString = self.text[after..].to_string().into();
            let caret_height = px(f32::from(styles.text_size) * LINE_HEIGHT_MULTIPLIER);
            let mut text_row = div().flex().items_center();
            if !before_text.is_empty() {
                let mut before_div = div()
                    .child(before_text.clone())
                    .text_size(styles.text_size)
                    .text_color(styles.text_color);
                if let Some(family) = &styles.font_family {
                    before_div = before_div.font_family(family.clone());
                }
                text_row = text_row.child(before_div);
            }
            if self.focused {
                text_row = text_row.child(
                    div().w(CARET_WIDTH).h(caret_height).bg(styles.caret_color),
                );
            }
            if !after_text.is_empty() {
                let mut after_div = div()
                    .child(after_text.clone())
                    .text_size(styles.text_size)
                    .text_color(styles.text_color);
                if let Some(family) = &styles.font_family {
                    after_div = after_div.font_family(family.clone());
                }
                text_row = text_row.child(after_div);
            }
            content = content.child(text_row);
        }

        let mut root = div().relative().w(styles.width).h(styles.height);
        root = root.child(content);
        let titles = [
            ("top", &styles.title_top),
            ("bottom", &styles.title_bottom),
            ("left", &styles.title_left),
            ("right", &styles.title_right),
        ];
        for (position, title) in titles {
            if let Some(title) = title {
                let mut title_div = div()
                    .absolute()
                    .text_size(title.text_size)
                    .text_color(title.color)
                    .child(title.text.clone());
                if let Some(family) = &title.font_family {
                    title_div = title_div.font_family(family.clone());
                }
                if let Some(fill) = styles.fill_color {
                    title_div = title_div.bg(fill);
                }
                match position {
                    "top" => {
                        title_div = title_div
                            .left(TITLE_INSET_X)
                            .top(px(-f32::from(title.text_size) / 2.));
                    }
                    "bottom" => {
                        title_div = title_div
                            .left(TITLE_INSET_X)
                            .bottom(px(-f32::from(title.text_size) / 2.));
                    }
                    "left" => {
                        title_div = title_div
                            .top(TITLE_INSET_Y)
                            .left(px(-f32::from(title.text_size) / 2.));
                    }
                    _ => {
                        title_div = title_div
                            .top(TITLE_INSET_Y)
                            .right(px(-f32::from(title.text_size) / 2.));
                    }
                }
                root = root.child(title_div);
            }
        }
        root
    }
}
