use anyhow::{Result, anyhow};
use rhi_lib::traits::RHIQueue;
use vulkanalia::{prelude::v1_4::*, vk::KhrSurfaceExtensionInstanceCommands};

use crate::{
    commands::VulkanCommandBuffer,
    context::{VulkanAppData, VulkanContext},
    physical_device::SuitabilityError,
};

pub struct VulkanQueue {}
impl RHIQueue<VulkanContext> for VulkanQueue {
    fn submit_commands(&self, command_buffer: VulkanCommandBuffer) {}
}

pub(crate) struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub fn get(
        instance: &Instance,
        data: &VulkanAppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        unsafe {
            let properties = instance.get_physical_device_queue_family_properties(physical_device);

            let graphics = properties
                .iter()
                .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|i| i as u32);

            let mut present = None;

            for (index, properties) in properties.iter().enumerate() {
                if instance.get_physical_device_surface_support_khr(
                    physical_device,
                    index as u32,
                    data.surface,
                )? {
                    present = Some(index as u32);
                    break;
                }
            }

            if let Some(graphics) = graphics
                && let Some(present) = present
            {
                Ok(Self { graphics, present })
            } else {
                Err(anyhow!(SuitabilityError(
                    "Missing required queue families."
                )))
            }
        }
    }
}
