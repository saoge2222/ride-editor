use std::sync::Arc;

use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::device::Device;
use vulkano::memory::allocator::{StandardMemoryAllocator, GenericMemoryAllocatorCreateInfo};

pub struct FrameContext {
    pub command_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
}

impl FrameContext {
    pub fn new(device: Arc<Device>) -> Self {
        let command_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );
        let memory_allocator = Arc::new(StandardMemoryAllocator::new(
            device,
            GenericMemoryAllocatorCreateInfo::default(),
        ));
        Self {
            command_allocator,
            memory_allocator,
        }
    }
}
