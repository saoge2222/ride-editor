#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub const fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct Style {
    pub background: Color,
    pub foreground: Color,
    pub padding: [f32; 4],
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Color::default(),
            foreground: Color::new(1.0, 1.0, 1.0, 1.0),
            padding: [0.0; 4],
        }
    }
}
