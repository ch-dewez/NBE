use std::{collections::HashMap, default, sync::Arc};

use anyhow::{Result, anyhow};
use dag::{DAG, DagNodeId};
use rhi_lib::traits::{
    RHIAttachment, RHICommandBeginRenderPass, RHICommandBuffer, RHICommandBufferCreateInfo,
    RHICommandBufferLevel, RHICommands, RHIContextAllTraits, RHIRenderPassCreateInfo,
    RHITypesTrait,
};
use thiserror::Error;

use crate::stages::{RenderStage, RenderStageCreateInfo, RenderStageUseContext};

pub type RenderStageHandle = DagNodeId;
pub type RenderRessourceHandle = usize;

/// a render graph
/// N: the number of frame in flights
pub struct Rendergraph<C: RHIContextAllTraits, const N: usize> {
    stages: DAG<RenderStage<C, N>>,
    ressources: HashMap<RenderRessourceHandle, RHIAttachment<C, N>>,

    output_attachment: Option<RenderStageHandle>,
    current_image_index: usize,
    next_ressource_id: usize,

    context: Arc<C>,
}

pub struct AddRenderStageInfo<'a, C: RHITypesTrait> {
    pub renderpass_info: RHIRenderPassCreateInfo<'a, C>,
}

#[derive(Debug, Error)]
#[error("Couldn't find renderstage with handle")]
pub struct RenderStageNotFoundError;

impl<C: RHIContextAllTraits, const N: usize> Rendergraph<C, N> {
    pub fn new(context: Arc<C>) -> Self {
        Self {
            stages: Default::default(),
            ressources: Default::default(),

            current_image_index: 0,
            output_attachment: None,
            next_ressource_id: 0,

            context,
        }
    }

    pub fn set_image_index(&mut self, index: usize) {
        self.current_image_index = index;
    }

    /// add a stage that doesn't have any inputs and create an output color
    pub fn add_stage<'a>(&mut self, info: AddRenderStageInfo<'a, C>) -> Result<RenderStageHandle> {
        // TODO: use thiserror crate, and use other output than default
        let output_handle = self.output_attachment.ok_or(anyhow!("No output set"))?;
        let output = self
            .get_ressource(output_handle)
            .ok_or(anyhow!("Output doesn't exist"))?;

        let info = RenderStageCreateInfo {
            renderpass_info: info.renderpass_info,
            output_handle,
            output_attachment: output,
        };
        let stage = RenderStage::new(Arc::clone(&self.context), info)?;

        let handle = self.stages.add_node_from_inputs(vec![], stage)?;

        Ok(handle)
    }

    pub fn get_stage(
        &mut self,
        stage: RenderStageHandle,
    ) -> Result<&RenderStage<C, N>, RenderStageNotFoundError> {
        self.stages
            .get_node(stage)
            .ok_or(RenderStageNotFoundError)
            .map(|node| &node.value)
    }

    // pub fn add_compositor_stage();

    pub fn set_output_attachment(&mut self, attachment: RHIAttachment<C, N>) {
        self.output_attachment = Some(self.add_ressource(attachment));
    }

    pub fn add_ressource(&mut self, attachment: RHIAttachment<C, N>) -> RenderRessourceHandle {
        let id = self.next_ressource_id;
        self.ressources.insert(id, attachment);
        self.next_ressource_id += 1;
        id
    }

    pub fn get_ressource(&self, attachment: RenderRessourceHandle) -> Option<&RHIAttachment<C, N>> {
        self.ressources.get(&attachment)
    }

    pub fn get_ressource_mut(
        &mut self,
        attachment: RenderRessourceHandle,
    ) -> Option<&mut RHIAttachment<C, N>> {
        self.ressources.get_mut(&attachment)
    }

    pub fn begin_stage_record(
        &'_ mut self,
        stage: RenderStageHandle,
        clear_values: [f32; 4],
    ) -> Option<RenderStageUseContext<'_, C>> {
        let stage = &mut self.stages.get_node_mut(stage)?.value;

        let command_buffer = &mut stage.command_buffers[self.current_image_index];
        command_buffer.reset();
        command_buffer.begin();
        let begin_info = RHICommandBeginRenderPass {
            renderpass: &stage.renderpass,
            framebuffer: &stage.frame_buffers[self.current_image_index],
            render_area: stage.output_size,
            clear_values,
        };
        command_buffer.push_command(RHICommands::SetViewport(stage.output_size));
        command_buffer.push_command(RHICommands::SetScissor(stage.output_size));
        command_buffer.push_command(RHICommands::BeginRenderPass(begin_info));

        Some(RenderStageUseContext {
            command_buffer,
            //TODO: inptus
            inputs: vec![],
        })
    }

    pub fn end_stage_record(
        &mut self,
        stage: RenderStageHandle,
    ) -> Result<(), RenderStageNotFoundError> {
        let stage = &mut self
            .stages
            .get_node_mut(stage)
            .ok_or(RenderStageNotFoundError)?
            .value;

        let command_buffer = &mut stage.command_buffers[self.current_image_index];
        command_buffer.push_command(RHICommands::EndRenderPass);
        command_buffer.end();

        Ok(())
    }

    pub fn submit(
        &mut self,
        wait_semaphores: &[&C::Semaphore],
        signal_semaphores: &[&C::Semaphore],
        fence: Option<&C::Fence>,
    ) {
        self.stages
            .get_all_nodes_mut()
            .values_mut()
            .for_each(|stage| {
                stage.value.command_buffers[self.current_image_index].submit(
                    wait_semaphores,
                    signal_semaphores,
                    fence,
                )
            })
    }
}
