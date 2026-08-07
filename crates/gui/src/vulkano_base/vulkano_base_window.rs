use std::sync::Arc;

use vulkano::swapchain::Surface;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use super::vulkano_base_instance::InstanceContext;

const WINDOW_TITLE: &str = "ride-editor";
const WINDOW_WIDTH: f64 = 1280.0;
const WINDOW_HEIGHT: f64 = 800.0;

pub struct WindowContext {
    pub event_loop: EventLoop<()>,
    pub instance_context: InstanceContext,
    pub window: Arc<Window>,
    pub surface: Arc<Surface>,
}

impl WindowContext {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        let instance_context = InstanceContext::new(&event_loop)?;
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(WINDOW_TITLE)
                .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .build(&event_loop)?,
        );
        let surface = Surface::from_window(instance_context.instance.clone(), window.clone())?;

        Ok(Self {
            event_loop,
            instance_context,
            window,
            surface,
        })
    }
}
