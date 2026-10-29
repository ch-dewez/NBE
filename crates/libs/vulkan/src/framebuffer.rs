use std::sync::Arc;

use crate::context::{VulkanAppData, VulkanContext};
use anyhow::Result;
use logger::log::info;
use rhi_lib::traits::{RHIFrameBuffer, RHIFrameBufferCreateInfo};
use vulkanalia::prelude::v1_4::*;

pub struct VulkanFrameBuffer {
    pub frame_buffer: vk::Framebuffer,
}

impl VulkanFrameBuffer {
    pub fn new(
        context: Arc<VulkanContext>,
        info: RHIFrameBufferCreateInfo<VulkanContext>,
    ) -> Result<Self> {
        info!("[Graphics] Creating Vulkan Frame Buffers");
        let device = &context.device;

        let attachments = &[info.texture.get_view()];
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(info.renderpass.renderpass)
            .attachments(attachments)
            .width(info.size.0)
            .height(info.size.1)
            .layers(1);

        let frame_buffer = unsafe { device.create_framebuffer(&create_info, None) }?;

        Ok(VulkanFrameBuffer { frame_buffer })
    }
}

impl RHIFrameBuffer for VulkanFrameBuffer {}
