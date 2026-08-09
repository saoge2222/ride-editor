use crate::widgets::widgets_definition::WidgetId;

pub struct ListItem {
    pub label: String,
    pub value: String,
}

impl ListItem {
    pub fn new(label: String, value: String) -> Self {
        Self { label, value }
    }
}

pub struct List {
    pub id: WidgetId,
    pub items: Vec<ListItem>,
}

impl List {
    pub fn new(id: WidgetId) -> Self {
        Self { id, items: Vec::new() }
    }

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }
}
