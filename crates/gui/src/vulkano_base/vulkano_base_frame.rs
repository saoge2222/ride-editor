use std::sync::Arc;

use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::device::Device;
use vulkano::memory::allocator::StandardMemoryAllocator;

pub struct FrameContext {
    pub command_allocator: Arc<StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
}

impl FrameContext {
    pub fn new(device: Arc<Device>) -> Self {
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device));
        Self {
            command_allocator,
            memory_allocator,
        }
    }
}
