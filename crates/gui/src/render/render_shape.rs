use super::render_vertex::Vertex2D;

pub const CIRCLE_SEGMENT_COUNT: u32 = 32;
pub const TWO_PI: f32 = std::f32::consts::TAU;

pub fn rect_mesh(x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> Vec<Vertex2D> {
    let x0 = x;
    let y0 = y;
    let x1 = x + width;
    let y1 = y + height;
    vec![
        vertex(x0, y0, color),
        vertex(x1, y0, color),
        vertex(x0, y1, color),
        vertex(x0, y1, color),
        vertex(x1, y0, color),
        vertex(x1, y1, color),
    ]
}

pub fn line_mesh(start: [f32; 2], end: [f32; 2], thickness: f32, color: [f32; 4]) -> Vec<Vertex2D> {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = dx.hypot(dy);
    let nx = -dy / length;
    let ny = dx / length;
    let half = thickness / 2.0;
    let ox = nx * half;
    let oy = ny * half;
    let p0 = [start[0] + ox, start[1] + oy];
    let p1 = [start[0] - ox, start[1] - oy];
    let p2 = [end[0] + ox, end[1] + oy];
    let p3 = [end[0] - ox, end[1] - oy];
    vec![
        vertex(p0[0], p0[1], color),
        vertex(p1[0], p1[1], color),
        vertex(p2[0], p2[1], color),
        vertex(p2[0], p2[1], color),
        vertex(p1[0], p1[1], color),
        vertex(p3[0], p3[1], color),
    ]
}

pub fn circle_mesh(cx: f32, cy: f32, radius: f32, color: [f32; 4]) -> Vec<Vertex2D> {
    let mut vertices = Vec::new();
    for index in 0..CIRCLE_SEGMENT_COUNT {
        let angle0 = (index as f32 / CIRCLE_SEGMENT_COUNT as f32) * TWO_PI;
        let angle1 = ((index + 1) as f32 / CIRCLE_SEGMENT_COUNT as f32) * TWO_PI;
        vertices.push(vertex(cx, cy, color));
        vertices.push(vertex(cx + radius * angle0.cos(), cy + radius * angle0.sin(), color));
        vertices.push(vertex(cx + radius * angle1.cos(), cy + radius * angle1.sin(), color));
    }
    vertices
}

fn vertex(x: f32, y: f32, color: [f32; 4]) -> Vertex2D {
    Vertex2D {
        position: [x, y],
        color,
    }
}
