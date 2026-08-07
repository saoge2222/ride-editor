use crate::widgets::widgets_definition::WidgetId;

pub struct EditorBuffer {
    pub id: WidgetId,
    pub path: String,
    pub text: String,
    pub cursor_line: u32,
    pub cursor_column: u32,
}

impl EditorBuffer {
    pub fn new(id: WidgetId, path: String, text: String) -> Self {
        Self {
            id,
            path,
            text,
            cursor_line: 0,
            cursor_column: 0,
        }
    }
}
