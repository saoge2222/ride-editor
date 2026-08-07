use crate::component::component_event::ComponentEvent;
use crate::component::component_layout::Rect;

pub type ComponentId = u64;

pub trait Component {
    fn id(&self) -> ComponentId;
    fn bounds(&self) -> Rect;
    fn handle_event(&mut self, event: &ComponentEvent);
}
