use std::sync::Arc;

use rhi_lib::traits::RHITexture;
use vulkanalia::prelude::v1_4::*;

use crate::context::VulkanContext;

pub struct VulkanTexture {
    image: vk::Image,
    view: vk::ImageView,
    format: vk::Format,

    size: (u32, u32),

    context: Arc<VulkanContext>,
}

impl Drop for VulkanTexture {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_image_view(self.view, None);
        }
    }
}

impl Clone for VulkanTexture {
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context),
            view: self.view,
            image: self.image,
            format: self.format,
            size: self.size,
        }
    }
}

impl VulkanTexture {
    pub fn new(
        context: Arc<VulkanContext>,
        image: vk::Image,
        view: vk::ImageView,
        format: vk::Format,
        size: (u32, u32),
    ) -> Self {
        VulkanTexture {
            context,
            image,
            view,
            format,
            size,
        }
    }

    pub fn get_image(&self) -> vk::Image {
        self.image
    }

    pub fn get_view(&self) -> vk::ImageView {
        self.view
    }

    pub fn get_format(&self) -> vk::Format {
        self.format
    }
}

impl RHITexture for VulkanTexture {
    fn get_size(&self) -> (u32, u32) {
        self.size
    }
}
