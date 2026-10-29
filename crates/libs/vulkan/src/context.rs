use std::collections::HashSet;
use std::ffi::{CStr, c_void};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use logger::log::{debug, error, info, trace, warn};
use raw_window_handle::HasWindowHandle;
use rhi_lib::traits::{
    RHIBufferCreateInfo, RHICommandBufferCreateInfo, RHIContext, RHIContextAllTraits,
    RHIContextCreate, RHIPipelineCreateInfo, RHIRenderPassCreateInfo, RHISwapchainCreateInfo,
    RHITextureCreateInfo, RHITypesTrait, RHIWindowInfo,
};
use vulkanalia::loader::{LIBRARY, LibloadingLoader};
use vulkanalia::prelude::v1_4::*;
use vulkanalia::vk::{ExtDebugUtilsExtensionInstanceCommands, KhrSwapchainExtensionDeviceCommands};
use vulkanalia::window as vk_window;

use crate::buffer::VulkanBuffer;
use crate::commands::{VulkanCommandBuffer, create_command_pool};
use crate::config::{PORTABILITY_MACOS_VERSION, VALIDATION_ENABLED, VALIDATION_LAYER};
use crate::framebuffer::VulkanFrameBuffer;
use crate::logical_device::create_logical_device;
use crate::physical_device::pick_physical_device;
use crate::pipeline::VulkanPipeline;
use crate::queue_family::VulkanQueue;
use crate::renderpass::VulkanRenderPass;
use crate::swapchains::VulkanSwapchain;
use crate::sync_objects::{VulkanFence, VulkanSemaphore};
use crate::texture::VulkanTexture;

unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("[Vulkan Validation Layer] [{:?}] {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("[Vulkan Validation Layer] [{:?}] {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("[Vulkan Validation Layer] [{:?}] {}", type_, message);
    } else {
        trace!("[Vulkan Validation Layer] [{:?}] {}", type_, message);
    }

    vk::FALSE
}

#[derive(Debug)]
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub data: VulkanAppData,
    pub device: Device,
}

#[derive(Debug, Default)]
pub struct VulkanAppData {
    pub messenger: vk::DebugUtilsMessengerEXT,

    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,

    pub command_pool: vk::CommandPool,
}

impl VulkanContext {
    fn create_instance(
        window: &dyn HasWindowHandle,
        entry: &Entry,
        data: &mut VulkanAppData,
    ) -> Result<Instance> {
        info!("[Graphics] Creating Vulkan Instance");
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        if VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            info!("Enabling extensions for macOs portability");
            extensions.push(
                vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                    .name
                    .as_ptr(),
            );
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let available_layers = unsafe { entry.enumerate_instance_layer_properties()? }
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();

        if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
            return Err(anyhow!("Validation layer requested but not supported."));
        }

        let layers = if VALIDATION_ENABLED {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            Vec::new()
        };

        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"NBE Vulkan\0")
            .api_version(vk::make_version(1, 4, 0));

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers)
            .flags(flags);

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(debug_callback));

        if VALIDATION_ENABLED {
            info.push_next(&mut debug_info);
        }

        let instance = unsafe { entry.create_instance(&info, None)? };

        if VALIDATION_ENABLED {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .user_callback(Some(debug_callback));

            data.messenger =
                unsafe { instance.create_debug_utils_messenger_ext(&debug_info, None) }?;
        }

        Ok(instance)
    }

    fn destroy(&mut self) {
        unsafe {
            self.device.device_wait_idle();

            self.device
                .destroy_command_pool(self.data.command_pool, None);

            self.device.destroy_device(None);

            if VALIDATION_ENABLED {
                self.instance
                    .destroy_debug_utils_messenger_ext(self.data.messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl RHIContext for VulkanContext {
    fn create<'a>(windows_info: RHIWindowInfo<'a>) -> Result<Self> {
        info!("[Graphics] Creating VULKAN context");
        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)
                .map_err(|err| anyhow!("Couldn't create vulkan loader: {}", err))?;
            let entry = Entry::new(loader).map_err(|err| anyhow!("{}", err))?;
            let mut data = VulkanAppData::default();
            let instance = VulkanContext::create_instance(windows_info.window, &entry, &mut data)?;

            data.surface =
                vk_window::create_surface(&instance, windows_info.display, windows_info.window)?;

            pick_physical_device(&instance, &mut data)?;

            let device = create_logical_device(&entry, &instance, &mut data)?;

            create_command_pool(&instance, &device, &mut data)?;

            info!("[Graphics] VULKAN context created");
            Ok(Self {
                entry,
                instance,
                data,
                device,
            })
        }
    }

    fn wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle();
        }
    }
}

impl RHIContextCreate for VulkanContext {
    fn create_buffer(self: Arc<Self>, info: RHIBufferCreateInfo) -> Result<Self::Buffer> {
        unimplemented!()
    }

    fn create_texture(self: Arc<Self>, info: RHITextureCreateInfo) -> Result<Self::Texture> {
        unimplemented!()
    }

    fn create_commandbuffers(
        self: Arc<Self>,
        info: RHICommandBufferCreateInfo,
    ) -> Result<Vec<Self::CommandBuffer>> {
        VulkanCommandBuffer::new(self, info)
    }

    fn create_pipeline<'a>(
        self: Arc<Self>,
        info: RHIPipelineCreateInfo<'a, Self>,
    ) -> Result<Self::Pipeline> {
        VulkanPipeline::new(self, info)
    }

    fn create_swapchain(self: Arc<Self>, info: RHISwapchainCreateInfo) -> Result<Self::Swapchain> {
        // TODO: use the info better
        VulkanSwapchain::new(info.window_info.window_res, self)
    }

    fn create_renderpass<'a>(
        self: Arc<Self>,
        info: RHIRenderPassCreateInfo<'a, Self>,
    ) -> Result<Self::RenderPass> {
        VulkanRenderPass::new(self, info)
    }

    fn create_framebuffer(
        self: Arc<Self>,
        info: rhi_lib::traits::RHIFrameBufferCreateInfo<Self>,
    ) -> Result<Self::FrameBuffer> {
        VulkanFrameBuffer::new(self, info)
    }

    fn create_semaphore(
        self: Arc<Self>,
        info: rhi_lib::traits::RHISemaphoreCreateInfo,
    ) -> Result<Self::Semaphore> {
        VulkanSemaphore::new(self, info)
    }

    fn create_fence(
        self: Arc<Self>,
        info: rhi_lib::traits::RHIFenceCreateInfo,
    ) -> Result<Self::Fence> {
        VulkanFence::new(self, info)
    }

    fn create_queue(
        self: Arc<Self>,
        info: rhi_lib::traits::RHIQueueCreateInfo,
    ) -> Result<Self::Queue> {
        todo!()
    }
}

impl RHITypesTrait for VulkanContext {
    type Texture = VulkanTexture;
    type Buffer = VulkanBuffer;
    type CommandBuffer = VulkanCommandBuffer;
    type Pipeline = VulkanPipeline;
    type RenderPass = VulkanRenderPass;
    type Swapchain = VulkanSwapchain;
    type Semaphore = VulkanSemaphore;
    type Fence = VulkanFence;
    type Queue = VulkanQueue;
    type FrameBuffer = VulkanFrameBuffer;
}

impl RHIContextAllTraits for VulkanContext {}
