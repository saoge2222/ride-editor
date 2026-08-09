use std::sync::Arc;

use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::ImageLayout;
use vulkano::render_pass::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, RenderPass,
    RenderPassCreateInfo, SubpassDescription,
};

const COLOR_ATTACHMENT_INDEX: u32 = 0;

pub struct RenderPassContext {
    pub render_pass: Arc<RenderPass>,
}

impl RenderPassContext {
    pub fn new(device: Arc<Device>, format: Format) -> Result<Self, Box<dyn std::error::Error>> {
        let attachment = AttachmentDescription {
            format,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        };
        let render_pass = RenderPass::new(
            device,
            RenderPassCreateInfo {
                attachments: vec![attachment],
                subpasses: vec![SubpassDescription {
                    color_attachments: vec![Some(AttachmentReference {
                        attachment: COLOR_ATTACHMENT_INDEX,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    })],
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        Ok(Self { render_pass })
    }
}
