use std::sync::Arc;

use winit::window::Window;

const EMPTY_TEXT: &str = "";

pub struct ClipboardContext {
    window: Arc<Window>,
}

impl ClipboardContext {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window }
    }

    pub fn get_text(&self) -> Option<String> {
        let _ = &self.window;
        Some(EMPTY_TEXT.to_owned())
    }

    pub fn set_text(&self, text: &str) {
        let _ = &self.window;
        let _ = text;
    }
}
