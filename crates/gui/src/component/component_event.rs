#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

#[derive(Clone, Debug)]
pub enum KeyEvent {
    Pressed { key: String, modifiers: KeyModifiers },
    Released { key: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
    Moved { x: f32, y: f32 },
    Pressed { x: f32, y: f32, button: MouseButton },
    Released { x: f32, y: f32, button: MouseButton },
    Scrolled { delta: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct ResizeEvent {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub enum ComponentEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(ResizeEvent),
}
