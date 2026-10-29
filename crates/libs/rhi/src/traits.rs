use std::{any::Any, sync::Arc};

use anyhow::{Result, anyhow};
use arrayvec::ArrayVec;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct RHIWindowInfo<'a> {
    pub window: &'a dyn HasWindowHandle,
    pub display: &'a dyn HasDisplayHandle,
    pub window_res: (u32, u32),
}

pub trait RHIContext {
    fn create<'a>(windows_info: RHIWindowInfo<'a>) -> Result<Self>
    where
        Self: Sized;

    fn wait_idle(&self);
}

pub trait RHITypesTrait: Sized + 'static {
    type Texture: RHITexture;
    type Buffer: RHIBuffer;
    type FrameBuffer: RHIFrameBuffer;
    type CommandBuffer: RHICommandBuffer<Self>;
    type Pipeline: RHIPipeline;
    type RenderPass: RHIRenderPass;
    type Swapchain: RHISwapchain<Self>;
    type Semaphore: RHISemaphore;
    type Fence: RHIFence;
    type Queue: RHIQueue<Self>;
}

// TODO: I'm not sure it's a good idea to pass in Arc<Self> but for now, I intend context to be in Arc so currently it's fine
pub trait RHIContextCreate: RHITypesTrait {
    fn create_texture(self: Arc<Self>, info: RHITextureCreateInfo) -> Result<Self::Texture>;
    fn create_buffer(self: Arc<Self>, info: RHIBufferCreateInfo) -> Result<Self::Buffer>;
    fn create_framebuffer(
        self: Arc<Self>,
        info: RHIFrameBufferCreateInfo<Self>,
    ) -> Result<Self::FrameBuffer>;
    fn create_commandbuffers(
        self: Arc<Self>,
        info: RHICommandBufferCreateInfo,
    ) -> Result<Vec<Self::CommandBuffer>>;
    fn create_pipeline<'a>(
        self: Arc<Self>,
        info: RHIPipelineCreateInfo<'a, Self>,
    ) -> Result<Self::Pipeline>;
    fn create_renderpass<'a>(
        self: Arc<Self>,
        info: RHIRenderPassCreateInfo<'a, Self>,
    ) -> Result<Self::RenderPass>;
    fn create_swapchain(self: Arc<Self>, info: RHISwapchainCreateInfo) -> Result<Self::Swapchain>;
    fn create_semaphore(self: Arc<Self>, info: RHISemaphoreCreateInfo) -> Result<Self::Semaphore>;
    fn create_fence(self: Arc<Self>, info: RHIFenceCreateInfo) -> Result<Self::Fence>;
    fn create_queue(self: Arc<Self>, info: RHIQueueCreateInfo) -> Result<Self::Queue>;
}

// pub trait RHIContextCreateErased: RHIContext {
//     fn create_buffer_erased(
//         self: Arc<Self>,
//         info: RHIBufferCreateInfo,
//     ) -> Result<Box<dyn RHIBuffer>>;
//     fn create_texture_erased(
//         self: Arc<Self>,
//         info: RHITextureCreateInfo,
//     ) -> Result<Box<dyn RHITexture>>;
//     fn create_command_buffer_erased(
//         self: Arc<Self>,
//         info: RHICommandBufferCreateInfo,
//     ) -> Result<Box<dyn RHICommandBuffer>>;
//     fn create_pipeline_erased(
//         self: Arc<Self>,
//         info: RHIPipelineCreateInfo,
//     ) -> Result<Box<dyn RHIPipeline>>;
//     fn create_swapchain_erased(
//         self: Arc<Self>,
//         info: RHISwapchainCreateInfo,
//     ) -> Result<Box<dyn RHISwapchain>>;
// }
//
// macro_rules! impl_to_native {
//     ($fn_name:ident, $name:ident, $RHIType:ty, $SelfType:ty) => {
//         fn $fn_name($name: &$RHIType) -> &$SelfType {
//             ($name as &dyn Any)
//                 .downcast_ref::<$SelfType>()
//                 .expect(concat!(
//                     "Couldn't downcast ",
//                     stringify!($name),
//                     " to the current backend ",
//                     stringify!($name),
//                     " type"
//                 ))
//         }
//     };
// }
// pub trait RHIContextToNative: RHITypesTrait {
//     impl_to_native!(
//         to_native_command_buffer,
//         command_buffer,
//         dyn RHICommandBuffer,
//         Self::CommandBuffer
//     );
//
//     impl_to_native!(
//         to_native_semaphore,
//         semaphore,
//         dyn RHISemaphore,
//         Self::Semaphore
//     );
//
//     impl_to_native!(
//         to_native_pipeline,
//         pipeline,
//         dyn RHIPipeline,
//         Self::Pipeline
//     );
//
//     impl_to_native!(to_native_texture, texture, dyn RHITexture, Self::Texture);
//
//     impl_to_native!(to_native_buffer, buffer, dyn RHIBuffer, Self::Buffer);
//
//     impl_to_native!(
//         to_native_swapchain,
//         buffer,
//         dyn RHISwapchain,
//         Self::Swapchain
//     );
//
//     impl_to_native!(to_native_fence, fence, dyn RHIFence, Self::Fence);
// }

// impl<T: RHITypesTrait> RHIContextToNative for T {}
// impl<T: RHIContextCreate + RHIContext> RHIContextCreateErased for T {
//     fn create_buffer_erased(
//         self: Arc<Self>,
//         info: RHIBufferCreateInfo,
//     ) -> Result<Box<dyn RHIBuffer>> {
//         Ok(Box::new(self.create_buffer(info)?))
//     }
//     fn create_texture_erased(
//         self: Arc<Self>,
//         info: RHITextureCreateInfo,
//     ) -> Result<Box<dyn RHITexture>> {
//         Ok(Box::new(self.create_texture(info)?))
//     }
//     fn create_command_buffer_erased(
//         self: Arc<Self>,
//         info: RHICommandBufferCreateInfo,
//     ) -> Result<Box<dyn RHICommandBuffer>> {
//         Ok(Box::new(self.create_command_buffer(info)?))
//     }
//     fn create_pipeline_erased(
//         self: Arc<Self>,
//         info: RHIPipelineCreateInfo,
//     ) -> Result<Box<dyn RHIPipeline>> {
//         Ok(Box::new(self.create_pipeline(info)?))
//     }
//     fn create_swapchain_erased(
//         self: Arc<Self>,
//         info: RHISwapchainCreateInfo,
//     ) -> Result<Box<dyn RHISwapchain>> {
//         Ok(Box::new(self.create_swapchain(info)?))
//     }
// }

// pub trait RHIContextAllTraits: RHIContext + RHITypesTrait + RHIContextCreate + RHIContextCreateErased+ RHIContextToNative + 'static {}
pub trait RHIContextAllTraits: RHIContext + RHITypesTrait + RHIContextCreate + 'static {}

pub enum RHIAttachmentType {
    Color,
    Depth,
}

/// An attachment for render stages
/// multiple texture for multiple frame in flights
pub struct RHIAttachment<C: RHITypesTrait, const N: usize> {
    pub attachment_type: RHIAttachmentType,
    pub texture: ArrayVec<C::Texture, N>,
}

pub trait RHIBuilder {
    type Item;
    fn build(self) -> Result<Self::Item>;
}

pub enum RHITextureFormat {}

pub struct RHITextureCreateInfo {}
pub trait RHITexture {
    fn get_size(&self) -> (u32, u32);
}

pub struct RHIBufferCreateInfo {}
pub trait RHIBuffer {}

pub struct RHIFrameBufferCreateInfo<'a, C: RHITypesTrait> {
    pub renderpass: &'a C::RenderPass,
    pub texture: &'a C::Texture,
    pub size: (u32, u32),
}
pub trait RHIFrameBuffer {}

pub struct RHIPipelineCreateInfo<'a, C: RHITypesTrait> {
    pub renderpass: &'a C::RenderPass,
}
pub trait RHIPipeline {}

pub enum ColorOutputFormat<'a, C: RHITypesTrait> {
    TextureFormat(RHITextureFormat),
    SameAsSwapchain(&'a C::Swapchain),
}
pub struct RHIRenderPassCreateInfo<'a, C: RHITypesTrait> {
    pub color_output_format: ColorOutputFormat<'a, C>,
}
pub trait RHIRenderPass {}

pub struct RHISwapchainCreateInfo<'a> {
    pub window_info: RHIWindowInfo<'a>,
}
pub trait RHISwapchain<C: RHITypesTrait> {
    fn acquire_next_image(
        &self,
        semaphore: Option<&C::Semaphore>,
        fence: Option<&C::Fence>,
    ) -> Result<u32>;
    fn present(&self, image_index: u32, semaphores: &[&C::Semaphore], fences: &[&C::Fence]);
    fn take_textures(&mut self) -> Option<Vec<C::Texture>>;
}

pub struct RHIQueueCreateInfo {}
pub trait RHIQueue<C: RHITypesTrait> {
    fn submit_commands(&self, command_buffer: C::CommandBuffer);
}

pub struct RHICommandDraw {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}
pub struct RHICommandBeginRenderPass<'a, C: RHITypesTrait> {
    pub renderpass: &'a C::RenderPass,
    pub framebuffer: &'a C::FrameBuffer,
    pub render_area: (u32, u32),
    pub clear_values: [f32; 4],
}
pub enum RHICommands<'a, C: RHITypesTrait> {
    BeginRenderPass(RHICommandBeginRenderPass<'a, C>),
    EndRenderPass,
    BindPipeline(&'a C::Pipeline),
    Draw(RHICommandDraw),

    SetViewport((u32, u32)),
    SetScissor((u32, u32)),
}

pub enum RHICommandBufferLevel {
    Primary,
    Secondary,
}
pub struct RHICommandBufferCreateInfo {
    pub count: u32,
    pub level: RHICommandBufferLevel,
}
pub trait RHICommandBuffer<C: RHITypesTrait> {
    fn begin(&self);
    fn end(&self);
    fn push_command<'a>(&'a mut self, command: RHICommands<'a, C>);
    fn reset(&mut self);
    fn submit(
        &mut self,
        wait_semaphores: &[&C::Semaphore],
        signal_semaphores: &[&C::Semaphore],
        fences: Option<&C::Fence>,
    );
}
#[derive(Debug, Clone)]
pub struct RHISemaphoreCreateInfo {}
pub trait RHISemaphore {}
#[derive(Debug, Clone)]
pub struct RHIFenceCreateInfo {
    pub signaled: bool,
}
pub trait RHIFence {
    fn wait(&self);
    fn reset(&self);
}
