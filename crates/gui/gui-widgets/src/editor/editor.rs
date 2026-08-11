use super::buffer::EditorBuffer;
use super::mode::{Mode, VimMode};
use crate::data::{
    Breakpoint, DocumentSymbol, GitLineState, GitStatus, LspDiagnostic, SyntaxToken, TokenKind,
};
use gpui::{
    App, AppContext, Context, ElementId, Entity, EntityId, FocusHandle, Focusable, HighlightStyle,
    Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    StyledText, Subscription, UnderlineStyle, Window, div, font, px, rgb,
};
use std::ops::Range;

const DEFAULT_FONT_FAMILY: &str = "Maple Mono";
const DEFAULT_FONT_SIZE: Pixels = px(14.);
const DEFAULT_LINE_HEIGHT_MULTIPLIER: f32 = 1.5;
const DEFAULT_LINE_NUMBER_DIGITS: u32 = 9;
const DEFAULT_BREAKPOINT_COLUMN_WIDTH: Pixels = px(20.);
const DEFAULT_GIT_MARKER_COLUMN_WIDTH: Pixels = px(4.);
const DEFAULT_STATUS_BAR_HEIGHT: Pixels = px(24.);
const CARET_WIDTH: Pixels = px(2.);
const BREAKPOINT_DOT_SIZE: Pixels = px(8.);
const GIT_MARKER_WIDTH: Pixels = px(2.);
const WAVE_THICKNESS: Pixels = px(1.5);
const LINE_NUMBER_PADDING_RIGHT: Pixels = px(8.);
const STATUS_BAR_PADDING_X: Pixels = px(8.);
const STATUS_BAR_TEXT_SIZE: Pixels = px(12.);
const STATUS_BAR_BORDER_WIDTH: Pixels = px(1.);
const TAB_TEXT: &str = "    ";
const RGB_LINE_NUMBER: u32 = 0x64748b;
const RGB_DIAGNOSTIC: u32 = 0xf87171;
const RGB_GIT_MODIFIED: u32 = 0xfbbf24;
const RGB_GIT_ADDED: u32 = 0x4ade80;
const RGB_GIT_DELETED: u32 = 0xf87171;
const RGB_BREAKPOINT: u32 = 0xf87171;
const RGB_STATUS_BAR_TEXT: u32 = 0x94a3b8;
const RGB_STATUS_BAR_BORDER: u32 = 0x334155;
const RGB_TOKEN_KEYWORD: u32 = 0x7dd3fc;
const RGB_TOKEN_IDENTIFIER: u32 = 0xe2e8f0;
const RGB_TOKEN_NUMBER: u32 = 0xfbbf24;
const RGB_TOKEN_STRING: u32 = 0x86efac;
const RGB_TOKEN_COMMENT: u32 = 0x94a3b8;
const RGB_TOKEN_OPERATOR: u32 = 0xc4b5fd;

#[derive(Clone, Debug)]
pub struct EditorStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub font_family: SharedString,
    pub font_size: Pixels,
    pub line_height_multiplier: f32,
    pub line_number_digits: u32,
    pub breakpoint_column_width: Pixels,
    pub git_marker_column_width: Pixels,
    pub status_bar_height: Pixels,
    pub background_color: Option<Hsla>,
    pub text_color: Hsla,
    pub caret_color: Hsla,
    pub line_number_color: Hsla,
    pub diagnostic_color: Hsla,
    pub git_modified_color: Hsla,
    pub git_added_color: Hsla,
    pub git_deleted_color: Hsla,
    pub breakpoint_color: Hsla,
    pub status_bar_text_color: Hsla,
    pub status_bar_border_color: Hsla,
}

impl EditorStyles {
    pub fn new(
        width: Pixels,
        height: Pixels,
        text_color: impl Into<Hsla>,
        caret_color: impl Into<Hsla>,
    ) -> Self {
        Self {
            width,
            height,
            font_family: DEFAULT_FONT_FAMILY.into(),
            font_size: DEFAULT_FONT_SIZE,
            line_height_multiplier: DEFAULT_LINE_HEIGHT_MULTIPLIER,
            line_number_digits: DEFAULT_LINE_NUMBER_DIGITS,
            breakpoint_column_width: DEFAULT_BREAKPOINT_COLUMN_WIDTH,
            git_marker_column_width: DEFAULT_GIT_MARKER_COLUMN_WIDTH,
            status_bar_height: DEFAULT_STATUS_BAR_HEIGHT,
            background_color: None,
            text_color: text_color.into(),
            caret_color: caret_color.into(),
            line_number_color: rgb(RGB_LINE_NUMBER).into(),
            diagnostic_color: rgb(RGB_DIAGNOSTIC).into(),
            git_modified_color: rgb(RGB_GIT_MODIFIED).into(),
            git_added_color: rgb(RGB_GIT_ADDED).into(),
            git_deleted_color: rgb(RGB_GIT_DELETED).into(),
            breakpoint_color: rgb(RGB_BREAKPOINT).into(),
            status_bar_text_color: rgb(RGB_STATUS_BAR_TEXT).into(),
            status_bar_border_color: rgb(RGB_STATUS_BAR_BORDER).into(),
        }
    }

    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = family.into();
        self
    }

    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    pub fn line_height_multiplier(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier;
        self
    }

    pub fn line_number_digits(mut self, digits: u32) -> Self {
        self.line_number_digits = digits;
        self
    }

    pub fn breakpoint_column_width(mut self, width: Pixels) -> Self {
        self.breakpoint_column_width = width;
        self
    }

    pub fn git_marker_column_width(mut self, width: Pixels) -> Self {
        self.git_marker_column_width = width;
        self
    }

    pub fn status_bar_height(mut self, height: Pixels) -> Self {
        self.status_bar_height = height;
        self
    }

    pub fn background_color(mut self, color: impl Into<Hsla>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn line_number_color(mut self, color: impl Into<Hsla>) -> Self {
        self.line_number_color = color.into();
        self
    }

    pub fn diagnostic_color(mut self, color: impl Into<Hsla>) -> Self {
        self.diagnostic_color = color.into();
        self
    }

    pub fn git_modified_color(mut self, color: impl Into<Hsla>) -> Self {
        self.git_modified_color = color.into();
        self
    }

    pub fn git_added_color(mut self, color: impl Into<Hsla>) -> Self {
        self.git_added_color = color.into();
        self
    }

    pub fn git_deleted_color(mut self, color: impl Into<Hsla>) -> Self {
        self.git_deleted_color = color.into();
        self
    }

    pub fn breakpoint_color(mut self, color: impl Into<Hsla>) -> Self {
        self.breakpoint_color = color.into();
        self
    }

    pub fn status_bar_text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.status_bar_text_color = color.into();
        self
    }

    pub fn status_bar_border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.status_bar_border_color = color.into();
        self
    }
}

pub struct EditorConfig {
    id: SharedString,
    styles: EditorStyles,
    initial_text: String,
    on_write_file: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_text_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_mode_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_toggle_breakpoint: Option<Box<dyn Fn(&str, u32, bool, &mut Window, &mut App) + 'static>>,
}

impl EditorConfig {
    pub fn new(
        id: impl Into<SharedString>,
        styles: EditorStyles,
        initial_text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            styles,
            initial_text: initial_text.into(),
            on_write_file: None,
            on_text_changed: None,
            on_mode_changed: None,
            on_toggle_breakpoint: None,
        }
    }

    pub fn on_write_file(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_write_file = Some(Box::new(handler));
        self
    }

    pub fn on_text_changed(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_text_changed = Some(Box::new(handler));
        self
    }

    pub fn on_mode_changed(
        mut self,
        handler: impl Fn(&str, &str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_mode_changed = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_breakpoint(
        mut self,
        handler: impl Fn(&str, u32, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_breakpoint = Some(Box::new(handler));
        self
    }
}

pub struct Editor {
    id: SharedString,
    styles: EditorStyles,
    buffer: EditorBuffer,
    mode: VimMode,
    line_height: Pixels,
    line_number_width: Pixels,
    scroll: ScrollHandle,
    diagnostics: Vec<LspDiagnostic>,
    tokens: Vec<SyntaxToken>,
    git_lines: Vec<GitLineState>,
    breakpoints: Vec<Breakpoint>,
    document_symbols: Vec<DocumentSymbol>,
    focused: bool,
    entity_id: EntityId,
    focus_handle: FocusHandle,
    focus_events_registered: bool,
    focus_subscriptions: Vec<Subscription>,
    on_write_file: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_text_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_mode_changed: Option<Box<dyn Fn(&str, &str, &mut Window, &mut App) + 'static>>,
    on_toggle_breakpoint: Option<Box<dyn Fn(&str, u32, bool, &mut Window, &mut App) + 'static>>,
}

impl Editor {
    pub fn new(config: EditorConfig, cx: &mut App) -> Entity<Editor> {
        let EditorConfig {
            id,
            styles,
            initial_text,
            on_write_file,
            on_text_changed,
            on_mode_changed,
            on_toggle_breakpoint,
        } = config;
        cx.new(|cx| {
            let font_id = cx.text_system().resolve_font(&font(styles.font_family.clone()));
            let char_width = cx
                .text_system()
                .ch_width(font_id, styles.font_size)
                .expect("Editor::new requires the font family to be registered first");
            let line_number_width =
                px(f32::from(char_width) * styles.line_number_digits as f32);
            let line_height = px(f32::from(styles.font_size) * styles.line_height_multiplier);
            Editor {
                id,
                styles,
                buffer: EditorBuffer::new(initial_text),
                mode: VimMode::new(),
                line_height,
                line_number_width,
                scroll: ScrollHandle::new(),
                diagnostics: Vec::new(),
                tokens: Vec::new(),
                git_lines: Vec::new(),
                breakpoints: Vec::new(),
                document_symbols: Vec::new(),
                focused: false,
                entity_id: cx.entity_id(),
                focus_handle: cx.focus_handle(),
                focus_events_registered: false,
                focus_subscriptions: Vec::new(),
                on_write_file,
                on_text_changed,
                on_mode_changed,
                on_toggle_breakpoint,
            }
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode.mode() {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        }
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<LspDiagnostic>, cx: &mut App) {
        self.diagnostics = diagnostics;
        cx.notify(self.entity_id);
    }

    pub fn set_tokens(&mut self, tokens: Vec<SyntaxToken>, cx: &mut App) {
        self.tokens = tokens;
        cx.notify(self.entity_id);
    }

    pub fn set_git_status(&mut self, git_lines: Vec<GitLineState>, cx: &mut App) {
        self.git_lines = git_lines;
        cx.notify(self.entity_id);
    }

    pub fn set_breakpoints(&mut self, breakpoints: Vec<Breakpoint>, cx: &mut App) {
        self.breakpoints = breakpoints;
        cx.notify(self.entity_id);
    }

    pub fn set_document_symbols(&mut self, symbols: Vec<DocumentSymbol>, cx: &mut App) {
        self.document_symbols = symbols;
        cx.notify(self.entity_id);
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(handler) = &self.on_write_file {
            let id = self.id.clone();
            let text = self.buffer.text().to_string();
            handler(&id, &text, window, cx);
        }
    }

    fn handle_editor_key(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.mode.handle_key(key) {
            if let Some(handler) = &self.on_mode_changed {
                let id = self.id.clone();
                let mode_name = self.mode_name().to_string();
                handler(&id, &mode_name, window, cx);
            }
            return;
        }
        match self.mode.mode() {
            Mode::Insert => match key {
                "backspace" => self.buffer.backspace_at_cursor(),
                "enter" => self.buffer.newline_at_cursor(),
                "left" => self.buffer.move_cursor_left(),
                "right" => self.buffer.move_cursor_right(),
                "up" => self.buffer.move_cursor_up(),
                "down" => self.buffer.move_cursor_down(),
                "home" => self.buffer.move_cursor_to_line_start(),
                "end" => self.buffer.move_cursor_to_line_end(),
                "delete" => self.buffer.delete_at_cursor(),
                "tab" => self.buffer.insert_at_cursor(TAB_TEXT),
                _ => {
                    if let Some(ch) = key_char {
                        self.buffer.insert_at_cursor(ch);
                    }
                }
            },
            Mode::Normal => match key {
                "h" => self.buffer.move_cursor_left(),
                "j" => self.buffer.move_cursor_down(),
                "k" => self.buffer.move_cursor_up(),
                "l" => self.buffer.move_cursor_right(),
                "0" => self.buffer.move_cursor_to_line_start(),
                "$" => self.buffer.move_cursor_to_line_end(),
                _ => {}
            },
        }
        if let Some(handler) = &self.on_text_changed {
            let id = self.id.clone();
            let text = self.buffer.text().to_string();
            handler(&id, &text, window, cx);
        }
    }

    fn toggle_breakpoint(&mut self, line: u32, window: &mut Window, cx: &mut App) {
        let added = if let Some(index) = self.breakpoints.iter().position(|b| b.line == line) {
            self.breakpoints.remove(index);
            false
        } else {
            self.breakpoints.push(Breakpoint { line });
            true
        };
        if let Some(handler) = &self.on_toggle_breakpoint {
            handler(&self.id, line, added, window, cx);
        }
        cx.notify(self.entity_id);
    }

    fn scroll_to_cursor(&self, cx: &mut App) {
        self.scroll.scroll_to_item(self.buffer.cursor_line() as usize);
        cx.notify(self.entity_id);
    }

    fn current_symbol(&self) -> Option<String> {
        let line = self.buffer.cursor_line();
        self.document_symbols
            .iter()
            .find(|s| line >= s.range.start.line && line <= s.range.end.line)
            .map(|s| s.name.to_string())
    }

    fn line_highlights(&self, line: usize) -> Vec<(Range<usize>, HighlightStyle)> {
        let line_text = self.buffer.line_text(line);
        let line_start = self.buffer.line_start_index(line);
        let line_end = line_start + line_text.len();
        let mut highlights = Vec::new();
        for token in &self.tokens {
            let start = token.start as usize;
            let end = token.end as usize;
            if start >= line_start && end <= line_end && end > start {
                highlights.push((
                    start - line_start..end - line_start,
                    HighlightStyle {
                        color: Some(token_color(token.kind)),
                        ..Default::default()
                    },
                ));
            }
        }
        for diagnostic in &self.diagnostics {
            let range = &diagnostic.range;
            if range.start.line as usize == line || range.end.line as usize == line {
                let start = if range.start.line as usize == line {
                    self.buffer.column_to_char_index(line, range.start.character)
                } else {
                    line_start
                };
                let end = if range.end.line as usize == line {
                    self.buffer.column_to_char_index(line, range.end.character)
                } else {
                    line_end
                };
                if end > start {
                    highlights.push((
                        start - line_start..end - line_start,
                        HighlightStyle {
                            color: Some(self.styles.diagnostic_color),
                            underline: Some(UnderlineStyle {
                                thickness: WAVE_THICKNESS,
                                color: Some(self.styles.diagnostic_color),
                                wavy: true,
                            }),
                            ..Default::default()
                        },
                    ));
                }
            }
        }
        highlights
    }

    fn render_line(&self, line: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let line_height = self.line_height;
        let is_cursor_line = line == self.buffer.cursor_line() as usize;
        let has_breakpoint = self.breakpoints.iter().any(|b| b.line == line as u32);

        let mut breakpoint_cell = div()
            .id(ElementId::Name(format!("{}-bp-{}", self.id, line).into()))
            .w(self.styles.breakpoint_column_width)
            .h(line_height)
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                    this.toggle_breakpoint(line as u32, window, cx);
                }),
            );
        if has_breakpoint {
            breakpoint_cell = breakpoint_cell
                .child(div().w(BREAKPOINT_DOT_SIZE).h(BREAKPOINT_DOT_SIZE).bg(self.styles.breakpoint_color));
        }

        let git_status = self
            .git_lines
            .iter()
            .find(|g| g.line == line as u32)
            .map(|g| g.status);
        let mut git_cell = div()
            .w(self.styles.git_marker_column_width)
            .h(line_height)
            .flex()
            .items_center()
            .justify_center();
        if let Some(status) = git_status {
            if let Some(color) = git_status_color(status, &self.styles) {
                git_cell = git_cell
                    .child(div().w(GIT_MARKER_WIDTH).h(line_height).bg(color));
            }
        }

        let number_cell = div()
            .w(self.line_number_width)
            .h(line_height)
            .flex()
            .items_center()
            .justify_end()
            .pr(LINE_NUMBER_PADDING_RIGHT)
            .text_color(self.styles.line_number_color)
            .child(format!("{}", line + 1));

        let line_text = self.buffer.line_text(line);
        let highlights = self.line_highlights(line);
        let text_cell = if is_cursor_line && self.focused {
            let cursor_column = self.buffer.cursor_column();
            let before_idx = self.buffer.column_to_char_index(line, cursor_column)
                - self.buffer.line_start_index(line);
            let (before_highlights, after_highlights) = split_highlights(&highlights, before_idx);
            let before: SharedString = line_text[..before_idx].to_string().into();
            let after: SharedString = line_text[before_idx..].to_string().into();
            let mut row = div().flex().items_center();
            if !before.is_empty() {
                row = row.child(StyledText::new(before).with_highlights(before_highlights));
            }
            row = row.child(div().w(CARET_WIDTH).h(line_height).bg(self.styles.caret_color));
            if !after.is_empty() {
                row = row.child(StyledText::new(after).with_highlights(after_highlights));
            }
            row
        } else {
            let line_text: SharedString = line_text.to_string().into();
            div().child(StyledText::new(line_text).with_highlights(highlights))
        };

        div()
            .id(ElementId::Integer(line as u64))
            .flex()
            .h(line_height)
            .child(breakpoint_cell)
            .child(git_cell)
            .child(number_cell)
            .child(text_cell)
    }

    fn render_status_bar(&self) -> impl IntoElement {
        let status_text = format!(
            "{} | {}",
            self.mode_name(),
            self.current_symbol().unwrap_or_else(|| "--".to_string())
        );
        div()
            .h(self.styles.status_bar_height)
            .flex()
            .flex_col()
            .child(div().w(self.styles.width).h(STATUS_BAR_BORDER_WIDTH).bg(self.styles.status_bar_border_color))
            .child(
                div()
                    .h(px(f32::from(self.styles.status_bar_height) - f32::from(STATUS_BAR_BORDER_WIDTH)))
                    .flex()
                    .items_center()
                    .px(STATUS_BAR_PADDING_X)
                    .text_size(STATUS_BAR_TEXT_SIZE)
                    .text_color(self.styles.status_bar_text_color)
                    .child(status_text),
            )
    }
}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_events_registered {
            self.focus_events_registered = true;
            let handle = self.focus_handle.clone();
            let weak_in = cx.weak_entity();
            let weak_out = cx.weak_entity();
            let subscription_in = window.on_focus_in(&handle, cx, move |_window, cx| {
                let _ = weak_in.update(cx, |editor, cx| {
                    editor.focused = true;
                    cx.notify();
                });
            });
            let subscription_out = window.on_focus_out(&handle, cx, move |_event, _window, cx| {
                let _ = weak_out.update(cx, |editor, cx| {
                    editor.focused = false;
                    cx.notify();
                });
            });
            self.focus_subscriptions.push(subscription_in);
            self.focus_subscriptions.push(subscription_out);
        }

        let line_count = self.buffer.line_count();
        let mut scroll_area = div()
            .id(ElementId::Name(format!("{}-scroll", self.id).into()))
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .font_family(self.styles.font_family.clone())
            .text_size(self.styles.font_size)
            .text_color(self.styles.text_color)
            .cursor_text()
            .focusable()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, _| {
                    window.focus(&this.focus_handle);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                if key_char.is_some()
                    && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                {
                    return;
                }
                let before_line = this.buffer.cursor_line();
                this.handle_editor_key(key, key_char, window, cx);
                if this.buffer.cursor_line() != before_line {
                    this.scroll_to_cursor(cx);
                }
                cx.notify();
            }));
        for line in 0..line_count {
            scroll_area = scroll_area.child(self.render_line(line, cx));
        }

        let mut root = div()
            .w(self.styles.width)
            .h(self.styles.height)
            .flex()
            .flex_col();
        if let Some(color) = self.styles.background_color {
            root = root.bg(color);
        }
        root = root.child(scroll_area);
        root = root.child(self.render_status_bar());
        root
    }
}

fn git_status_color(status: GitStatus, styles: &EditorStyles) -> Option<Hsla> {
    match status {
        GitStatus::Modified => Some(styles.git_modified_color),
        GitStatus::Added => Some(styles.git_added_color),
        GitStatus::Deleted => Some(styles.git_deleted_color),
        _ => None,
    }
}

fn token_color(kind: TokenKind) -> Hsla {
    match kind {
        TokenKind::Keyword => rgb(RGB_TOKEN_KEYWORD).into(),
        TokenKind::Identifier => rgb(RGB_TOKEN_IDENTIFIER).into(),
        TokenKind::Number => rgb(RGB_TOKEN_NUMBER).into(),
        TokenKind::String => rgb(RGB_TOKEN_STRING).into(),
        TokenKind::Comment => rgb(RGB_TOKEN_COMMENT).into(),
        TokenKind::Operator => rgb(RGB_TOKEN_OPERATOR).into(),
    }
}

fn split_highlights(
    highlights: &[(Range<usize>, HighlightStyle)],
    split: usize,
) -> (Vec<(Range<usize>, HighlightStyle)>, Vec<(Range<usize>, HighlightStyle)>) {
    let mut before = Vec::new();
    let mut after = Vec::new();
    for (range, style) in highlights {
        if range.end <= split {
            before.push((range.clone(), style.clone()));
        } else if range.start >= split {
            after.push((range.start - split..range.end - split, style.clone()));
        } else {
            before.push((range.start..split, style.clone()));
            after.push((0..range.end - split, style.clone()));
        }
    }
    (before, after)
}
