use crate::widgets::widgets_definition::WidgetId;

pub struct TreeItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    pub fn new(name: String, path: String, is_dir: bool) -> Self {
        Self { name, path, is_dir, children: Vec::new() }
    }

    pub fn add_child(&mut self, child: TreeItem) {
        self.children.push(child);
    }
}

pub struct FileTree {
    pub id: WidgetId,
    pub root: TreeItem,
}

impl FileTree {
    pub fn new(id: WidgetId, root: TreeItem) -> Self {
        Self { id, root }
    }
}
