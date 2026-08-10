use gpui::{
    App, AppContext, ClickEvent, Context, ElementId, Entity, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px, svg,
};

const DEFAULT_TEXT_SIZE: Pixels = px(14.);
const DEFAULT_ICON_SIZE: Pixels = px(16.);
const DEFAULT_BORDER_WIDTH: Pixels = px(1.);

#[derive(Clone, Debug)]
pub struct ButtonStyles {
    pub width: Pixels,
    pub height: Pixels,
    pub border_color: Option<Hsla>,
    pub fill_color: Option<Hsla>,
    pub text: Option<SharedString>,
    pub text_color: Option<Hsla>,
    pub font_family: Option<SharedString>,
    pub text_size: Pixels,
    pub icon_path: Option<SharedString>,
    pub icon_width: Pixels,
    pub icon_height: Pixels,
}

impl ButtonStyles {
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self {
            width,
            height,
            border_color: None,
            fill_color: None,
            text: None,
            text_color: None,
            font_family: None,
            text_size: DEFAULT_TEXT_SIZE,
            icon_path: None,
            icon_width: DEFAULT_ICON_SIZE,
            icon_height: DEFAULT_ICON_SIZE,
        }
    }

    pub fn border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn fill_color(mut self, color: impl Into<Hsla>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
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

    pub fn icon(mut self, path: impl Into<SharedString>, width: Pixels, height: Pixels) -> Self {
        self.icon_path = Some(path.into());
        self.icon_width = width;
        self.icon_height = height;
        self
    }
}

pub struct ButtonConfig {
    id: SharedString,
    styles: ButtonStyles,
    on_click: Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_hover: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
}

impl ButtonConfig {
    pub fn new(id: impl Into<SharedString>, styles: ButtonStyles) -> Self {
        Self {
            id: id.into(),
            styles,
            on_click: None,
            on_hover: None,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_hover(mut self, handler: impl Fn(&str, bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Box::new(handler));
        self
    }
}

pub struct Button {
    id: SharedString,
    styles: ButtonStyles,
    hovered: bool,
    clicked: bool,
    on_click: Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_hover: Option<Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(config: ButtonConfig, cx: &mut App) -> Entity<Button> {
        cx.new(|_| Button {
            id: config.id,
            styles: config.styles,
            hovered: false,
            clicked: false,
            on_click: config.on_click,
            on_hover: config.on_hover,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_clicked(&self) -> bool {
        self.clicked
    }

    pub fn set_styles(&mut self, styles: ButtonStyles) {
        self.styles = styles;
    }
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = &self.styles;

        let mut element = div()
            .id(ElementId::Name(self.id.clone()))
            .w(s.width)
            .h(s.height)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer();

        if let Some(color) = s.border_color {
            element = element.border_color(color).border(DEFAULT_BORDER_WIDTH);
        }
        if let Some(color) = s.fill_color {
            element = element.bg(color);
        }

        if let Some(text) = s.text.clone() {
            let mut label = div().child(text).text_size(s.text_size);
            if let Some(color) = s.text_color {
                label = label.text_color(color);
            }
            if let Some(family) = s.font_family.clone() {
                label = label.font_family(family);
            }
            element = element.child(label);
        }

        if let Some(path) = s.icon_path.clone() {
            let mut icon = svg().path(path).w(s.icon_width).h(s.icon_height);
            if let Some(color) = s.text_color {
                icon = icon.text_color(color);
            }
            element = element.child(icon);
        }

        element
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                this.clicked = true;
                if let Some(handler) = &this.on_click {
                    handler(&this.id, window, &mut **cx);
                }
                cx.notify();
            }))
            .on_hover(cx.listener(|this, hovered: &bool, window, cx| {
                this.hovered = *hovered;
                if let Some(handler) = &this.on_hover {
                    handler(&this.id, *hovered, window, &mut **cx);
                }
                cx.notify();
            }))
    }
}
