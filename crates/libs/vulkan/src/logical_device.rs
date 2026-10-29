use std::collections::HashSet;

use anyhow::Result;
use logger::log::info;
use vulkanalia::prelude::v1_4::*;

use crate::{
    config::{DEVICE_EXTENSIONS, PORTABILITY_MACOS_VERSION},
    context::VulkanAppData,
    queue_family::QueueFamilyIndices,
};

pub fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    data: &mut VulkanAppData,
) -> Result<Device> {
    info!("Creating logical device");
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    // layers create an validation error
    // let layers = if VALIDATION_ENABLED {
    //     vec![VALIDATION_LAYER.as_ptr()]
    // } else {
    //     vec![]
    // };

    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    let features = vk::PhysicalDeviceFeatures::builder();
    let mut vulkan_12_features =
        vk::PhysicalDeviceVulkan12Features::builder().vulkan_memory_model(true);

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        // .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features)
        .push_next(&mut vulkan_12_features);

    let device = unsafe { instance.create_device(data.physical_device, &info, None)? };

    data.graphics_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
    data.present_queue = unsafe { device.get_device_queue(indices.present, 0) };

    Ok(device)
}
