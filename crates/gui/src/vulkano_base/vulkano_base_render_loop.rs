use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::device::{Device, Queue};
use vulkano::format::ClearValue;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::{self, Surface, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{Validated, VulkanError};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use super::vulkano_base_frame::FrameContext;
use super::vulkano_base_instance::InstanceContext;
use super::vulkano_base_render_pass::RenderPassContext;
use super::vulkano_base_surface::{pick_swapchain_format, SurfaceContext};
use super::vulkano_base_window::{create_window, window_size};

const BACKGROUND_COLOR: [f32; 4] = [0.08, 0.08, 0.10, 1.0];

pub type CommandBufferBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

pub struct FrameResources<'a> {
    pub builder: &'a mut CommandBufferBuilder,
    pub extent: [u32; 2],
    pub image_index: u32,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_allocator: Arc<StandardCommandBufferAllocator>,
    pub queue_family_index: u32,
}

pub struct RenderLoop {
    event_loop: EventLoop<()>,
}

impl RenderLoop {
    pub fn new() -> Result<Self, winit::error::EventLoopError> {
        Ok(Self {
            event_loop: EventLoop::new()?,
        })
    }

    pub fn run<F>(self, frame_fn: F) -> Result<(), winit::error::EventLoopError>
    where
        F: FnMut(&mut FrameResources) + 'static,
    {
        let mut app = RenderApp {
            frame_fn,
            instance_context: None,
            window: None,
            surface_context: None,
            render_pass_context: None,
            frame_context: None,
            swapchain_needs_recreate: false,
        };
        self.event_loop.run_app(&mut app)
    }
}

struct RenderApp<F> {
    frame_fn: F,
    instance_context: Option<InstanceContext>,
    window: Option<Arc<Window>>,
    surface_context: Option<SurfaceContext>,
    render_pass_context: Option<RenderPassContext>,
    frame_context: Option<FrameContext>,
    swapchain_needs_recreate: bool,
}

impl<F> ApplicationHandler for RenderApp<F>
where
    F: FnMut(&mut FrameResources) + 'static,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.instance_context.is_some() {
            return;
        }

        let init_result = self.initialize(event_loop);
        if let Err(error) = init_result {
            eprintln!("ride-gui initialization failed: {error:?}");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let owned_window_id = self.window.as_ref().map(|window| window.id());
        if owned_window_id != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.swapchain_needs_recreate = true;
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl<F> RenderApp<F>
where
    F: FnMut(&mut FrameResources) + 'static,
{
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let instance_context = InstanceContext::new(event_loop)?;
        let window = create_window(event_loop)?;
        let surface = Surface::from_window(instance_context.instance.clone(), window.clone())?;
        let extent = window_size(&window);
        let format = pick_swapchain_format(&instance_context.device, &surface)?;
        let render_pass_context =
            RenderPassContext::new(instance_context.device.clone(), format)?;
        let surface_context = SurfaceContext::new(
            instance_context.device.clone(),
            surface,
            extent,
            render_pass_context.render_pass.clone(),
        )?;
        let frame_context = FrameContext::new(instance_context.device.clone());

        self.instance_context = Some(instance_context);
        self.window = Some(window);
        self.surface_context = Some(surface_context);
        self.render_pass_context = Some(render_pass_context);
        self.frame_context = Some(frame_context);
        Ok(())
    }

    fn render_frame(&mut self) {
        let surface_context = match &mut self.surface_context {
            Some(context) => context,
            None => return,
        };
        let frame_context = match &self.frame_context {
            Some(context) => context,
            None => return,
        };
        let instance_context = match &self.instance_context {
            Some(context) => context,
            None => return,
        };
        let render_pass_context = match &self.render_pass_context {
            Some(context) => context,
            None => return,
        };

        if self.swapchain_needs_recreate {
            let extent = window_size(self.window.as_ref().unwrap());
            match surface_context.recreate(
                render_pass_context.render_pass.clone(),
                extent,
            ) {
                Ok(()) => self.swapchain_needs_recreate = false,
                Err(_) => return,
            }
        }

        let queue = instance_context.queue.clone();
        let queue_family_index = instance_context.queue_family_index();
        let device = instance_context.device.clone();
        let render_pass = render_pass_context.render_pass.clone();
        let memory_allocator = frame_context.memory_allocator.clone();

        let suboptimal = match render_frame_inner(
            surface_context,
            &frame_context.command_allocator,
            &queue,
            queue_family_index,
            &render_pass,
            device,
            memory_allocator,
            &mut self.frame_fn,
        ) {
            Ok(suboptimal) => suboptimal,
            Err(_) => return,
        };

        if suboptimal {
            self.swapchain_needs_recreate = true;
        }
    }
}

fn render_frame_inner<F>(
    surface_context: &mut SurfaceContext,
    command_allocator: &Arc<StandardCommandBufferAllocator>,
    queue: &Arc<Queue>,
    queue_family_index: u32,
    render_pass: &Arc<RenderPass>,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    frame_fn: &mut F,
) -> Result<bool, Box<dyn std::error::Error>>
where
    F: FnMut(&mut FrameResources) + 'static,
{
    let (image_index, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(surface_context.swapchain.clone(), None) {
            Ok(result) => result,
            Err(Validated::Error(VulkanError::OutOfDate)) => return Ok(true),
            Err(error) => return Err(error.into()),
        };

    let extent = surface_context.extent;
    let mut builder = AutoCommandBufferBuilder::primary(
        command_allocator.clone(),
        queue_family_index,
        CommandBufferUsage::OneTimeSubmit,
    )?;

    builder.begin_render_pass(
        RenderPassBeginInfo {
            clear_values: vec![Some(ClearValue::Float(BACKGROUND_COLOR))],
            ..RenderPassBeginInfo::framebuffer(
                surface_context.framebuffers[image_index as usize].clone(),
            )
        },
        SubpassBeginInfo {
            contents: SubpassContents::Inline,
            ..Default::default()
        },
    )?;

    let mut frame_resources = FrameResources {
        builder: &mut builder,
        extent,
        image_index,
        device,
        queue: queue.clone(),
        render_pass: render_pass.clone(),
        memory_allocator,
        command_allocator: command_allocator.clone(),
        queue_family_index,
    };
    frame_fn(&mut frame_resources);

    builder.end_render_pass(SubpassEndInfo::default())?;
    let command_buffer = builder.build()?;

    acquire_future
        .then_execute(queue.clone(), command_buffer)?
        .then_swapchain_present(
            queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(
                surface_context.swapchain.clone(),
                image_index,
            ),
        )
        .then_signal_fence_and_flush()?
        .wait(None)?;

    Ok(suboptimal)
}
