use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::device::Device;
use vulkano::format::ClearValue;
use vulkano::instance::Instance;
use vulkano::swapchain::{self, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use super::vulkano_base_frame::FrameContext;
use super::vulkano_base_instance::InstanceContext;
use super::vulkano_base_render_pass::RenderPassContext;
use super::vulkano_base_surface::{pick_swapchain_format, SurfaceContext};
use super::vulkano_base_window::WindowContext;

const BACKGROUND_COLOR: [f32; 4] = [0.08, 0.08, 0.10, 1.0];
const MIN_EXTENT: u32 = 1;

pub type CommandBufferBuilder = AutoCommandBufferBuilder<
    PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>,
    StandardCommandBufferAllocator,
>;

pub struct RenderLoop {
    event_loop: EventLoop<()>,
    pub instance_context: InstanceContext,
    pub window: Arc<Window>,
    surface_context: SurfaceContext,
    render_pass_context: RenderPassContext,
    frame_context: FrameContext,
    swapchain_needs_recreate: bool,
}

impl RenderLoop {
    pub fn new(window_context: WindowContext) -> Result<Self, Box<dyn std::error::Error>> {
        let WindowContext {
            event_loop,
            instance_context,
            window,
            surface,
        } = window_context;

        let extent = window_size(&window);
        let format = pick_swapchain_format(&instance_context.device, &surface)?;
        let render_pass_context =
            RenderPassContext::new(instance_context.device.clone(), format)?;
        let surface_context = SurfaceContext::new(
            instance_context.device.clone(),
            surface.clone(),
            extent,
            render_pass_context.render_pass.clone(),
        )?;
        let frame_context = FrameContext::new(instance_context.device.clone());

        Ok(Self {
            event_loop,
            instance_context,
            window,
            surface_context,
            render_pass_context,
            frame_context,
            swapchain_needs_recreate: false,
        })
    }

    pub fn device(&self) -> Arc<Device> {
        self.instance_context.device.clone()
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.instance_context.instance.clone()
    }

    pub fn queue_family_index(&self) -> u32 {
        self.instance_context.queue_family_index()
    }

    pub fn render_pass(&self) -> Arc<vulkano::render_pass::RenderPass> {
        self.render_pass_context.render_pass.clone()
    }

    pub fn memory_allocator(
        &self,
    ) -> Arc<vulkano::memory::allocator::StandardMemoryAllocator> {
        self.frame_context.memory_allocator.clone()
    }

    pub fn run<F>(mut self, mut frame_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(&mut CommandBufferBuilder, [u32; 2], u32) + 'static,
    {
        let queue = self.instance_context.queue.clone();
        let queue_family_index = queue.queue_family_index();

        self.event_loop.run(move |event, _event_loop_window_target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.swapchain_needs_recreate = true;
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    if self.swapchain_needs_recreate {
                        let extent = window_size(&self.window);
                        match self.surface_context.recreate(
                            self.render_pass_context.render_pass.clone(),
                            extent,
                        ) {
                            Ok(()) => self.swapchain_needs_recreate = false,
                            Err(_) => return,
                        }
                    }
                    match render_frame(
                        &mut self.surface_context,
                        &self.frame_context,
                        &queue,
                        queue_family_index,
                        &mut frame_fn,
                    ) {
                        Ok(true) => self.swapchain_needs_recreate = true,
                        Ok(false) => {}
                        Err(_) => {}
                    }
                    self.window.request_redraw();
                }
                _ => {}
            }
        })
    }
}

fn render_frame<F>(
    surface_context: &mut SurfaceContext,
    frame_context: &FrameContext,
    queue: &Arc<vulkano::device::Queue>,
    queue_family_index: u32,
    frame_fn: &mut F,
) -> Result<bool, Box<dyn std::error::Error>>
where
    F: FnMut(&mut CommandBufferBuilder, [u32; 2], u32) + 'static,
{
    let (image_index, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(surface_context.swapchain.clone(), None) {
            Ok(result) => result,
            Err(Validated::Error(VulkanError::OutOfDate)) => return Ok(true),
            Err(error) => return Err(error.into()),
        };

    let extent = surface_context.extent;
    let mut builder = AutoCommandBufferBuilder::primary(
        &frame_context.command_allocator,
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

    frame_fn(&mut builder, extent, image_index);

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

fn window_size(window: &Window) -> [u32; 2] {
    let size = window.inner_size();
    [
        size.width.max(MIN_EXTENT),
        size.height.max(MIN_EXTENT),
    ]
}
