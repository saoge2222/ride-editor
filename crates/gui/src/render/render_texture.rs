use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::sampler::{Sampler, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageTiling, ImageType, ImageUsage};
use vulkano::memory::allocator::{
    AllocationCreateInfo, StandardMemoryAllocator, MemoryTypeFilter,
};
use vulkano::sync::{now, GpuFuture};

const MIP_LEVELS: u32 = 1;
const ARRAY_LAYERS: u32 = 1;
const IMAGE_DEPTH: u32 = 1;
const RGBA_BYTES_PER_PIXEL: usize = 4;

pub struct SamplerContext {
    pub sampler: Arc<Sampler>,
}

impl SamplerContext {
    pub fn new(device: Arc<Device>) -> Result<Self, Box<dyn std::error::Error>> {
        let sampler = Sampler::new(device, SamplerCreateInfo::default())?;
        Ok(Self { sampler })
    }
}

pub struct Texture {
    pub image: Arc<Image>,
    pub image_view: Arc<ImageView>,
}

impl Texture {
    pub fn from_rgba(
        device: Arc<Device>,
        queue: Arc<Queue>,
        allocator: Arc<StandardMemoryAllocator>,
        command_allocator: &StandardCommandBufferAllocator,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        assert_eq!(
            rgba.len(),
            (width * height) as usize * RGBA_BYTES_PER_PIXEL,
            "rgba data length must match width * height * 4"
        );

        let extent = [width, height, IMAGE_DEPTH];
        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent,
                mip_levels: MIP_LEVELS,
                array_layers: ARRAY_LAYERS,
                tiling: ImageTiling::Optimal,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )?;

        let staging = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            rgba.iter().copied(),
        )?;

        let queue_family_index = queue.queue_family_index();
        let mut builder = AutoCommandBufferBuilder::primary(
            command_allocator,
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )?;
        builder.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging,
            image.clone(),
        ))?;
        let command_buffer = builder.build()?;

        now(device)
            .then_execute(queue, command_buffer)?
            .then_signal_fence_and_flush()?
            .wait(None)?;

        let image_view = ImageView::new_default(image.clone())?;

        Ok(Self { image, image_view })
    }
}
