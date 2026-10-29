use std::sync::Arc;

use anyhow::Result;
use logger::log::info;
use rhi_lib::traits::RHISwapchain;
use vulkanalia::{
    prelude::v1_4::*,
    vk::{KhrSurfaceExtensionInstanceCommands, KhrSwapchainExtensionDeviceCommands},
};

use crate::{
    context::{VulkanAppData, VulkanContext},
    queue_family::QueueFamilyIndices,
    sync_objects::{VulkanFence, VulkanSemaphore},
    texture::VulkanTexture,
};

pub struct VulkanSwapchain {
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_textures: Option<Vec<VulkanTexture>>,

    pub context: Arc<VulkanContext>,
}

impl RHISwapchain<VulkanContext> for VulkanSwapchain {
    fn acquire_next_image(
        &self,
        semaphore: Option<&VulkanSemaphore>,
        fence: Option<&VulkanFence>,
    ) -> Result<u32> {
        Ok(unsafe {
            self.context.device.acquire_next_image_khr(
                self.swapchain,
                u64::MAX,
                semaphore.map_or(vk::Semaphore::null(), |semaphore| semaphore.0),
                fence.map_or(vk::Fence::null(), |fence| fence.0),
            )
        }?
        .0)
    }

    fn present(&self, image_index: u32, semaphores: &[&VulkanSemaphore], fences: &[&VulkanFence]) {
        // TODO: Fence?
        let swapchains = &[self.swapchain];
        let image_indices = &[image_index];
        let semaphores = semaphores
            .iter()
            .map(|semaphore| semaphore.0)
            .collect::<Vec<_>>();
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            self.context
                .device
                .queue_present_khr(self.context.data.present_queue, &present_info)
                .expect("Couldn't present to the window");
        }
    }

    fn take_textures(&mut self) -> Option<Vec<VulkanTexture>> {
        self.swapchain_textures.take()
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            drop(std::mem::take(&mut self.swapchain_textures));
            self.context
                .device
                .destroy_swapchain_khr(self.swapchain, None);
        }
    }
}

impl VulkanSwapchain {
    // TODO: use more info
    pub fn new(window_res: (u32, u32), context: Arc<VulkanContext>) -> Result<Self> {
        let device = &context.device;

        let instance = &context.instance;
        let data = &context.data;
        info!("[Graphics] Creating VULKAN Swapchain");

        let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
        let support = SwapchainSupport::get(instance, data, data.physical_device)?;

        let surface_format = get_swapchain_surface_format(&support.formats);
        let present_mode = get_swapchain_present_mode(&support.present_modes);
        let extent = get_swapchain_extent(window_res, support.capabilities);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if indices.graphics != indices.present {
            queue_family_indices.push(indices.graphics);
            queue_family_indices.push(indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(data.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe { device.create_swapchain_khr(&info, None)? };
        let images = unsafe { device.get_swapchain_images_khr(swapchain)? };

        let views = images
            .iter()
            .map(|i| {
                let components = vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY);

                let subresource_range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);

                let info = vk::ImageViewCreateInfo::builder()
                    .image(*i)
                    .view_type(vk::ImageViewType::_2D)
                    .format(surface_format.format)
                    .components(components)
                    .subresource_range(subresource_range);

                unsafe { device.create_image_view(&info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let textures = images
            .iter()
            .zip(views.iter())
            .map(|(i, v)| {
                VulkanTexture::new(
                    Arc::clone(&context),
                    *i,
                    *v,
                    surface_format.format,
                    (extent.width, extent.height),
                )
            })
            .collect::<Vec<_>>();

        // Ok((surface_format.format, extent, swapchain, textures))
        Ok(Self {
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain,
            swapchain_textures: Some(textures),

            context,
        })
    }
}

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn get(
        instance: &Instance,
        data: &VulkanAppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        unsafe {
            Ok(Self {
                capabilities: instance
                    .get_physical_device_surface_capabilities_khr(physical_device, data.surface)?,
                formats: instance
                    .get_physical_device_surface_formats_khr(physical_device, data.surface)?,
                present_modes: instance
                    .get_physical_device_surface_present_modes_khr(physical_device, data.surface)?,
            })
        }
    }
}

// TODO: Add config
pub fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

// TODO: Add config
pub fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        // FIFO is guarenteed to be present
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn get_swapchain_extent(
    window_res: (u32, u32),
    capabilties: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilties.current_extent.width != u32::MAX {
        capabilties.current_extent
    } else {
        vk::Extent2D::builder()
            .width(window_res.0.clamp(
                capabilties.min_image_extent.width,
                capabilties.max_image_extent.width,
            ))
            .height(window_res.1.clamp(
                capabilties.min_image_extent.height,
                capabilties.max_image_extent.height,
            ))
            .build()
    }
}
