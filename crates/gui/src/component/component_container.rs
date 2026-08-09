use crate::component::component_definition::{Component, ComponentId};
use crate::component::component_event::{ComponentEvent, EventResult, MouseEvent};
use crate::component::component_layout::{Child, Constraints, Flex, Rect, Size};
use crate::component::component_style::Style;
use crate::render::render_draw::DrawList;

pub struct Container {
    pub id: ComponentId,
    pub flex: Flex,
    pub style: Style,
    children: Vec<Child>,
    bounds: Rect,
    size: Size,
}

impl Container {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            flex: Flex::default(),
            style: Style::default(),
            children: Vec::new(),
            bounds: Rect::default(),
            size: Size::ZERO,
        }
    }

    pub fn add_child(&mut self, component: Box<dyn Component>) {
        self.children.push(Child::new(component));
    }

    pub fn children(&self) -> &[Child] {
        &self.children
    }
}

impl Component for Container {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        self.size = self.flex.layout_children(&mut self.children, constraints);
        self.size
    }

    fn arrange(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.flex
            .arrange_children(&mut self.children, bounds.inset(self.style.padding));
    }

    fn draw(&self, draw_list: &mut DrawList) {
        draw_list.rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            self.style.background.to_array(),
        );
        for child in &self.children {
            child.component.draw(draw_list);
        }
    }

    fn handle_event(&mut self, event: &ComponentEvent) -> EventResult {
        for child in self.children.iter_mut().rev() {
            if event_hits(child.bounds(), event) {
                if child.component.handle_event(event) == EventResult::HANDLED {
                    return EventResult::HANDLED;
                }
            }
        }
        EventResult::IGNORED
    }
}

fn event_hits(bounds: Rect, event: &ComponentEvent) -> bool {
    match event {
        ComponentEvent::Mouse(MouseEvent::Moved { x, y })
        | ComponentEvent::Mouse(MouseEvent::Pressed { x, y, .. })
        | ComponentEvent::Mouse(MouseEvent::Released { x, y, .. }) => {
            bounds.contains(*x, *y)
        }
        ComponentEvent::Mouse(MouseEvent::Scrolled { .. }) => false,
        ComponentEvent::Key(_) | ComponentEvent::Resize(_) => true,
    }
}
