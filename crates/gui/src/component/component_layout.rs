use crate::component::component_definition::Component;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    pub fn inset(&self, padding: [f32; 4]) -> Rect {
        Rect {
            x: self.x + padding[0],
            y: self.y + padding[1],
            width: (self.width - padding[0] - padding[2]).max(0.0),
            height: (self.height - padding[1] - padding[3]).max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const ZERO: Size = Size::new(0.0, 0.0);
}

#[derive(Clone, Copy, Debug)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub const fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub const fn loose(max_width: f32, max_height: f32) -> Self {
        Self {
            min_width: 0.0,
            max_width,
            min_height: 0.0,
            max_height,
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Constraints::loose(f32::INFINITY, f32::INFINITY)
    }
}

pub trait Layout {
    fn layout(&mut self, constraints: Constraints) -> Size;
    fn arrange(&mut self, bounds: Rect);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

pub struct Child {
    pub component: Box<dyn Component>,
    size: Size,
    bounds: Rect,
}

impl Child {
    pub fn new(component: Box<dyn Component>) -> Self {
        Self {
            component,
            size: Size::ZERO,
            bounds: Rect::default(),
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Flex {
    pub axis: Axis,
    pub main_alignment: Alignment,
    pub cross_alignment: Alignment,
    pub gap: f32,
    pub padding: [f32; 4],
}

impl Default for Flex {
    fn default() -> Self {
        Self {
            axis: Axis::Vertical,
            main_alignment: Alignment::Start,
            cross_alignment: Alignment::Start,
            gap: 0.0,
            padding: [0.0; 4],
        }
    }
}

impl Flex {
    pub fn layout_children(
        &self,
        children: &mut [Child],
        constraints: Constraints,
    ) -> Size {
        let inner = Constraints::loose(
            constraints.max_width - self.padding[0] - self.padding[2],
            constraints.max_height - self.padding[1] - self.padding[3],
        );
        let mut main_total: f32 = 0.0;
        let mut cross_max: f32 = 0.0;
        for child in children.iter_mut() {
            let size = child.component.layout(inner);
            child.size = size;
            match self.axis {
                Axis::Horizontal => {
                    main_total += size.width;
                    cross_max = cross_max.max(size.height);
                }
                Axis::Vertical => {
                    main_total += size.height;
                    cross_max = cross_max.max(size.width);
                }
            }
        }
        let main_total = main_total + self.gap * (children.len().saturating_sub(1) as f32);
        let (width, height) = match self.axis {
            Axis::Horizontal => (main_total, cross_max),
            Axis::Vertical => (cross_max, main_total),
        };
        constraints.constrain(Size::new(
            width + self.padding[0] + self.padding[2],
            height + self.padding[1] + self.padding[3],
        ))
    }

    pub fn arrange_children(&self, children: &mut [Child], bounds: Rect) {
        let inner = bounds.inset(self.padding);
        let mut content_main = 0.0;
        for child in children.iter() {
            match self.axis {
                Axis::Horizontal => content_main += child.size.width,
                Axis::Vertical => content_main += child.size.height,
            }
        }
        content_main += self.gap * (children.len().saturating_sub(1) as f32);
        let free_main = match self.axis {
            Axis::Horizontal => inner.width - content_main,
            Axis::Vertical => inner.height - content_main,
        };
        let mut cursor = match self.main_alignment {
            Alignment::Start => inner.x,
            Alignment::Center => inner.x + free_main / 2.0,
            Alignment::End => inner.x + free_main,
        };
        for child in children.iter_mut() {
            let size = child.size;
            let (pos_x, pos_y) = match self.axis {
                Axis::Horizontal => {
                    let y = align_position(self.cross_alignment, inner.y, inner.height, size.height);
                    (cursor, y)
                }
                Axis::Vertical => {
                    let x = align_position(self.cross_alignment, inner.x, inner.width, size.width);
                    (x, cursor)
                }
            };
            let rect = Rect::new(pos_x, pos_y, size.width, size.height);
            child.bounds = rect;
            child.component.arrange(rect);
            match self.axis {
                Axis::Horizontal => cursor += size.width + self.gap,
                Axis::Vertical => cursor += size.height + self.gap,
            }
        }
    }
}

fn align_position(alignment: Alignment, start: f32, span: f32, size: f32) -> f32 {
    match alignment {
        Alignment::Start => start,
        Alignment::Center => start + (span - size) / 2.0,
        Alignment::End => start + span - size,
    }
}
