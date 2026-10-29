use std::sync::Arc;

use anyhow::Result;
use arrayvec::ArrayVec;
use rhi_lib::traits::{
    RHIAttachment, RHICommandBufferCreateInfo, RHICommandBufferLevel, RHIContextAllTraits,
    RHIFrameBufferCreateInfo, RHIRenderPassCreateInfo, RHITexture, RHITypesTrait,
};

use crate::rendergraph::RenderRessourceHandle;

pub struct RenderStage<C: RHITypesTrait, const N: usize> {
    pub renderpass: C::RenderPass,
    pub frame_buffers: ArrayVec<C::FrameBuffer, N>,
    pub inputs: Vec<RenderRessourceHandle>,
    pub outputs: Vec<RenderRessourceHandle>,

    pub output_size: (u32, u32),

    pub command_buffers: ArrayVec<C::CommandBuffer, N>,
}

pub struct RenderStageCreateInfo<'a, C: RHITypesTrait, const N: usize> {
    pub renderpass_info: RHIRenderPassCreateInfo<'a, C>,
    // TODO: inputs
    // TODO: multiple attachments for a frame buffer
    pub output_handle: RenderRessourceHandle,
    pub output_attachment: &'a RHIAttachment<C, N>,
}

impl<C: RHIContextAllTraits, const N: usize> RenderStage<C, N> {
    pub fn new<'a>(context: Arc<C>, info: RenderStageCreateInfo<'a, C, N>) -> Result<Self> {
        let renderpass = Arc::clone(&context).create_renderpass(info.renderpass_info)?;

        let size = info.output_attachment.texture[0].get_size();

        let frame_buffers = info
            .output_attachment
            .texture
            .iter()
            .map(|texture| {
                let frame_buffer_info = RHIFrameBufferCreateInfo {
                    renderpass: &renderpass,
                    texture,
                    size,
                };
                Arc::clone(&context).create_framebuffer(frame_buffer_info)
            })
            .collect::<Result<ArrayVec<_, N>, _>>()?;

        let command_buffer_info = RHICommandBufferCreateInfo {
            count: frame_buffers.len() as u32,
            level: RHICommandBufferLevel::Primary,
        };
        // NOTE: Wanted to use try_from_note but it's in nightly, rn it does an allocation for nothing
        let command_buffers: ArrayVec<C::CommandBuffer, N> = Arc::clone(&context)
            .create_commandbuffers(command_buffer_info)?
            .into_iter()
            .collect();

        Ok(Self {
            renderpass,
            frame_buffers,
            inputs: Vec::new(),
            outputs: vec![info.output_handle],
            output_size: size,
            command_buffers,
        })
    }
}

pub struct RenderStageRequestInfo {}

pub struct RenderStageUseContext<'a, C: RHITypesTrait> {
    pub command_buffer: &'a mut C::CommandBuffer,
    pub inputs: Vec<&'a mut C::Texture>,
}
