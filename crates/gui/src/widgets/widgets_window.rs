use crate::widgets::widgets_definition::WidgetId;

pub struct Window {
    pub id: WidgetId,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(id: WidgetId, title: String, width: u32, height: u32) -> Self {
        Self { id, title, width, height }
    }
}
