use std::sync::Arc;

use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::image::view::ImageView;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{
    ColorSpace, CompositeAlpha, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo,
};

const MIN_IMAGE_COUNT: u32 = 2;
const DEFAULT_LAYERS: u32 = 1;

pub fn pick_swapchain_format(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
) -> Result<Format, Box<dyn std::error::Error>> {
    let formats = device
        .physical_device()
        .surface_formats(surface, SurfaceInfo::default())?;
    Ok(pick_format(&formats).0)
}

pub struct SurfaceContext {
    pub swapchain: Arc<Swapchain>,
    pub image_views: Vec<Arc<ImageView>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub extent: [u32; 2],
    pub format: Format,
}

impl SurfaceContext {
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        extent: [u32; 2],
        render_pass: Arc<RenderPass>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let physical_device = device.physical_device();
        let surface_info = SurfaceInfo::default();
        let capabilities = physical_device.surface_capabilities(&surface, surface_info.clone())?;
        let formats = physical_device.surface_formats(&surface, surface_info.clone())?;
        let present_modes: Vec<PresentMode> =
            physical_device.surface_present_modes(&surface, surface_info)?.collect();

        let (format, _) = pick_format(&formats);
        let present_mode = pick_present_mode(&present_modes);
        let composite_alpha = capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .unwrap_or(CompositeAlpha::Opaque);

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: MIN_IMAGE_COUNT,
                image_format: format,
                image_extent: extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                pre_transform: capabilities.current_transform,
                composite_alpha,
                present_mode,
                ..Default::default()
            },
        )?;

        let image_views = images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).map_err(Into::into))
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

        let framebuffers = image_views
            .iter()
            .map(|view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view.clone()],
                        extent,
                        layers: DEFAULT_LAYERS,
                        ..Default::default()
                    },
                )
                .map_err(Into::into)
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

        Ok(Self {
            swapchain,
            image_views,
            framebuffers,
            extent,
            format,
        })
    }

    pub fn recreate(
        &mut self,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (swapchain, images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_format: self.format,
            image_extent: extent,
            ..self.swapchain.create_info()
        })?;

        let image_views = images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).map_err(Into::into))
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

        let framebuffers = image_views
            .iter()
            .map(|view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view.clone()],
                        extent,
                        layers: DEFAULT_LAYERS,
                        ..Default::default()
                    },
                )
                .map_err(Into::into)
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

        self.swapchain = swapchain;
        self.image_views = image_views;
        self.framebuffers = framebuffers;
        self.extent = extent;
        Ok(())
    }
}

fn pick_format(formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
    formats
        .first()
        .copied()
        .unwrap_or((Format::B8G8R8A8_UNORM, ColorSpace::SrgbNonLinear))
}

fn pick_present_mode(present_modes: &[PresentMode]) -> PresentMode {
    for preferred in [PresentMode::Mailbox, PresentMode::Immediate, PresentMode::Fifo] {
        if present_modes.contains(&preferred) {
            return preferred;
        }
    }
    PresentMode::Fifo
}
