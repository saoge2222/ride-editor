use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventResult {
    handled: bool,
}

impl EventResult {
    pub const HANDLED: EventResult = EventResult { handled: true };
    pub const IGNORED: EventResult = EventResult { handled: false };

    pub fn is_handled(&self) -> bool {
        self.handled
    }
}

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

pub struct EventTranslator {
    last_x: f32,
    last_y: f32,
}

impl EventTranslator {
    pub const fn new() -> Self {
        Self {
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    pub fn translate(&mut self, event: &WindowEvent) -> Option<ComponentEvent> {
        match event {
            WindowEvent::Resized(size) => Some(ComponentEvent::Resize(ResizeEvent {
                width: size.width,
                height: size.height,
            })),
            WindowEvent::CursorMoved { position, .. } => {
                self.last_x = position.x as f32;
                self.last_y = position.y as f32;
                Some(ComponentEvent::Mouse(MouseEvent::Moved {
                    x: self.last_x,
                    y: self.last_y,
                }))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = translate_button(*button)?;
                let event = match state {
                    ElementState::Pressed => MouseEvent::Pressed {
                        x: self.last_x,
                        y: self.last_y,
                        button,
                    },
                    ElementState::Released => MouseEvent::Released {
                        x: self.last_x,
                        y: self.last_y,
                        button,
                    },
                };
                Some(ComponentEvent::Mouse(event))
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                Some(ComponentEvent::Mouse(MouseEvent::Scrolled { delta }))
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key = event.text.as_ref().map(|text| text.to_string()).unwrap_or_default();
                let event = match event.state {
                    ElementState::Pressed => KeyEvent::Pressed {
                        key,
                        modifiers: KeyModifiers {
                            shift: false,
                            control: false,
                            alt: false,
                        },
                    },
                    ElementState::Released => KeyEvent::Released { key },
                };
                Some(ComponentEvent::Key(event))
            }
            _ => None,
        }
    }
}

impl Default for EventTranslator {
    fn default() -> Self {
        Self::new()
    }
}

fn translate_button(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left => Some(MouseButton::Left),
        WinitMouseButton::Middle => Some(MouseButton::Middle),
        WinitMouseButton::Right => Some(MouseButton::Right),
        _ => None,
    }
}
