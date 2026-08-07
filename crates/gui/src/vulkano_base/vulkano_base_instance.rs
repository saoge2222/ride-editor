use std::sync::Arc;

use raw_window_handle::HasRawDisplayHandle;
use vulkano::VulkanLibrary;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::Surface;
use vulkano::Version;

const APPLICATION_NAME: &str = "ride-editor";
const ENGINE_NAME: &str = "ride-gui";
const APPLICATION_VERSION: Version = Version::V1_0;
const ENGINE_VERSION: Version = Version::V1_0;
const QUEUE_PRIORITY: f32 = 1.0;

pub struct InstanceContext {
    pub instance: Arc<Instance>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl InstanceContext {
    pub fn new(
        display_handle: &impl HasRawDisplayHandle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let library = VulkanLibrary::new()?;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                application_name: Some(APPLICATION_NAME.to_owned()),
                application_version: APPLICATION_VERSION,
                engine_name: Some(ENGINE_NAME.to_owned()),
                engine_version: ENGINE_VERSION,
                enabled_extensions: Surface::required_extensions(display_handle),
                ..Default::default()
            },
        )?;

        let physical_device = instance
            .enumerate_physical_devices()?
            .into_iter()
            .find(|device| Self::supports_graphics(device))
            .ok_or("no physical device with graphics queue support")?;

        let queue_family_index = Self::graphics_queue_family(&physical_device)?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    queues: vec![QUEUE_PRIORITY],
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().ok_or("no graphics queue created")?;

        Ok(Self {
            instance,
            physical_device,
            device,
            queue,
        })
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue.queue_family_index()
    }

    fn supports_graphics(device: &PhysicalDevice) -> bool {
        device
            .queue_family_properties()
            .iter()
            .any(|family| family.queue_flags.intersects(QueueFlags::GRAPHICS))
    }

    fn graphics_queue_family(device: &PhysicalDevice) -> Result<u32, Box<dyn std::error::Error>> {
        device
            .queue_family_properties()
            .iter()
            .position(|family| family.queue_flags.intersects(QueueFlags::GRAPHICS))
            .map(|index| index as u32)
            .ok_or_else(|| "no graphics queue family".into())
    }
}
