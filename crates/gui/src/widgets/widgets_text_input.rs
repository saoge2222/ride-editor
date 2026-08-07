use crate::widgets::widgets_definition::WidgetId;

pub struct TextInput {
    pub id: WidgetId,
    pub text: String,
    pub placeholder: String,
    pub cursor: u32,
}

impl TextInput {
    pub fn new(id: WidgetId, placeholder: String) -> Self {
        Self {
            id,
            text: String::new(),
            placeholder,
            cursor: 0,
        }
    }
}
