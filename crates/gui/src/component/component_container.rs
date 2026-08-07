use crate::component::component_definition::{Component, ComponentId};
use crate::component::component_event::ComponentEvent;
use crate::component::component_layout::{Layout, Rect};

pub struct Container {
    pub id: ComponentId,
    pub bounds: Rect,
    pub children: Vec<Box<dyn Component>>,
}

impl Container {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            bounds: Rect::default(),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }
}

impl Component for Container {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn handle_event(&mut self, event: &ComponentEvent) {
        for child in &mut self.children {
            child.handle_event(event);
        }
    }
}

impl Layout for Container {
    fn arrange(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
}
