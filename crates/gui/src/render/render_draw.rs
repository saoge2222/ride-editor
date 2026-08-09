use super::render_shape;
use super::render_vertex::Vertex2D;

pub struct SolidRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

pub struct SolidLine {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub thickness: f32,
    pub color: [f32; 4],
}

pub struct SolidCircle {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
}

pub enum DrawCommand {
    Rect(SolidRect),
    Line(SolidLine),
    Circle(SolidCircle),
}

pub struct DrawList {
    commands: Vec<DrawCommand>,
}

impl DrawList {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.commands
            .push(DrawCommand::Rect(SolidRect { x, y, width, height, color }));
    }

    pub fn line(&mut self, start: [f32; 2], end: [f32; 2], thickness: f32, color: [f32; 4]) {
        self.commands
            .push(DrawCommand::Line(SolidLine { start, end, thickness, color }));
    }

    pub fn circle(&mut self, center: [f32; 2], radius: f32, color: [f32; 4]) {
        self.commands
            .push(DrawCommand::Circle(SolidCircle { center, radius, color }));
    }

    pub fn build_mesh(&self) -> Vec<Vertex2D> {
        let mut vertices = Vec::new();
        for command in &self.commands {
            match command {
                DrawCommand::Rect(rect) => vertices.extend(render_shape::rect_mesh(
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    rect.color,
                )),
                DrawCommand::Line(line) => vertices.extend(render_shape::line_mesh(
                    line.start,
                    line.end,
                    line.thickness,
                    line.color,
                )),
                DrawCommand::Circle(circle) => vertices.extend(render_shape::circle_mesh(
                    circle.center[0],
                    circle.center[1],
                    circle.radius,
                    circle.color,
                )),
            }
        }
        vertices
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}
