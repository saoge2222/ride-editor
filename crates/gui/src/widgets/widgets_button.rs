use crate::widgets::widgets_definition::WidgetId;

pub struct Button {
    pub id: WidgetId,
    pub label: String,
    pub width: f32,
    pub height: f32,
}

impl Button {
    pub fn new(id: WidgetId, label: String, width: f32, height: f32) -> Self {
        Self { id, label, width, height }
    }
}
