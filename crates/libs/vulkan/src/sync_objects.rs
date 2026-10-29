use std::sync::Arc;

use anyhow::Result;
use logger::log::info;
use rhi_lib::traits::{RHIFence, RHIFenceCreateInfo, RHISemaphore, RHISemaphoreCreateInfo};
use vulkanalia::prelude::v1_4::*;

use crate::context::{VulkanAppData, VulkanContext};

pub struct VulkanSemaphore(pub vk::Semaphore, Arc<VulkanContext>);
impl RHISemaphore for VulkanSemaphore {}

impl Clone for VulkanSemaphore {
    fn clone(&self) -> Self {
        Self(self.0, Arc::clone(&self.1))
    }
}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.1.device.destroy_semaphore(self.0, None);
        }
    }
}

impl VulkanSemaphore {
    pub fn new(context: Arc<VulkanContext>, _info: RHISemaphoreCreateInfo) -> Result<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let semaphore = unsafe { context.device.create_semaphore(&semaphore_info, None)? };

        Ok(VulkanSemaphore(semaphore, context))
    }
}

#[derive(Debug, Clone)]
pub struct VulkanFence(pub vk::Fence, pub Arc<VulkanContext>);
impl RHIFence for VulkanFence {
    fn wait(&self) {
        unsafe {
            self.1.device.wait_for_fences(&[self.0], true, u64::MAX);
        }
    }

    fn reset(&self) {
        unsafe {
            self.1.device.reset_fences(&[self.0]);
        }
    }
}

impl VulkanFence {
    pub fn new(context: Arc<VulkanContext>, info: RHIFenceCreateInfo) -> Result<Self> {
        let mut flags = vk::FenceCreateFlags::empty();
        if info.signaled {
            flags = flags.intersection(vk::FenceCreateFlags::SIGNALED);
        }
        let fence_info = vk::FenceCreateInfo::builder().flags(flags);
        let fence = unsafe { context.device.create_fence(&fence_info, None)? };

        Ok(VulkanFence(fence, context))
    }
}
