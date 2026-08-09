use crate::component::component_event::{ComponentEvent, EventResult};
use crate::component::component_layout::{Constraints, Rect, Size};
use crate::render::render_draw::DrawList;

pub type ComponentId = u64;

pub trait Component {
    fn id(&self) -> ComponentId;
    fn bounds(&self) -> Rect;
    fn layout(&mut self, constraints: Constraints) -> Size;
    fn arrange(&mut self, bounds: Rect);
    fn draw(&self, draw_list: &mut DrawList);
    fn handle_event(&mut self, event: &ComponentEvent) -> EventResult;
}
