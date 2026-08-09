use std::sync::Arc;

use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

const WINDOW_TITLE: &str = "ride-editor";
const WINDOW_WIDTH: f64 = 1280.0;
const WINDOW_HEIGHT: f64 = 800.0;
const MIN_EXTENT: u32 = 1;

pub fn create_window(event_loop: &ActiveEventLoop) -> Result<Arc<Window>, OsError> {
    let attributes = Window::default_attributes()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let window = event_loop.create_window(attributes)?;
    Ok(Arc::new(window))
}

pub fn window_size(window: &Window) -> [u32; 2] {
    let size = window.inner_size();
    [
        size.width.max(MIN_EXTENT),
        size.height.max(MIN_EXTENT),
    ]
}
