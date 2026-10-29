use std::collections::HashSet;

use crate::{
    config::DEVICE_EXTENSIONS, context::VulkanAppData, queue_family::QueueFamilyIndices,
    swapchains::SwapchainSupport,
};
use anyhow::{Result, anyhow};
use logger::log::{info, warn};
use thiserror::Error;
use vulkanalia::prelude::v1_4::*;

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

// TODO: Implement a score based picking logic
pub fn pick_physical_device(instance: &Instance, data: &mut VulkanAppData) -> Result<()> {
    for physical_device in unsafe { instance.enumerate_physical_devices()? } {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!(
                "Skipping physical_device {}: {}",
                properties.device_name, error
            );
        } else {
            info!("Selecting physical device {}", properties.device_name);
            data.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

pub fn check_physical_device(
    instance: &Instance,
    data: &VulkanAppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    // let properties = instance.get_physical_device_properties(physical_device);
    // let features = instance.get_physical_device_features(physical_device);
    QueueFamilyIndices::get(instance, data, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support")));
    }
    Ok(())
}

fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device, None) }?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions."
        )))
    }
}
