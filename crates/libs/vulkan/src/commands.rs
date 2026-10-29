use std::sync::Arc;

use anyhow::Result;
use logger::log::info;
use vulkanalia::{prelude::v1_4::*, vk::CommandBufferResetFlags};

pub(crate) fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut VulkanAppData,
) -> Result<()> {
    info!("[Graphics] Creating Vulkan Command Pool");

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) // Optional.
        .queue_family_index(indices.graphics);

    data.command_pool = unsafe { device.create_command_pool(&info, None)? };

    Ok(())
}

// Things I was recoding in the command buffer
//
// let render_area = vk::Rect2D::builder()
//     .offset(vk::Offset2D::default())
//     .extent(data.swapchain_extent);
//
// let color_clear_value = vk::ClearValue {
//     color: vk::ClearColorValue {
//         float32: [0.0, 0.0, 0.0, 1.0],
//     },
// };
//
// let clear_values = &[color_clear_value];
// let info = vk::RenderPassBeginInfo::builder()
//     .render_pass(data.render_pass)
//     .framebuffer(data.framebuffers[i])
//     .render_area(render_area)
//     .clear_values(clear_values);
//
// unsafe {
//     device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);
// }
//
// unsafe {
//     device.cmd_bind_pipeline(
//         *command_buffer,
//         vk::PipelineBindPoint::GRAPHICS,
//         data.pipeline,
//     );
//      device.cmd_draw(*command_buffer, 3, 1, 0, 0);
//
//     device.cmd_end_render_pass(*command_buffer);
//
//     device.end_command_buffer(*command_buffer)?;
// }

use rhi_lib::traits::{
    RHICommandBuffer, RHICommandBufferCreateInfo, RHICommandBufferLevel, RHICommands,
};

use crate::{
    context::{VulkanAppData, VulkanContext},
    queue_family::QueueFamilyIndices,
    sync_objects::{VulkanFence, VulkanSemaphore},
};

pub struct VulkanCommandBuffer {
    pub command_buffer: vk::CommandBuffer,
    context: Arc<VulkanContext>,
}

impl VulkanCommandBuffer {
    pub fn new(context: Arc<VulkanContext>, info: RHICommandBufferCreateInfo) -> Result<Vec<Self>> {
        info!("[Graphics] Creating Vulkan Command Buffer");

        let device = &context.device;
        let data = &context.data;

        let buffer_level = match info.level {
            RHICommandBufferLevel::Primary => vk::CommandBufferLevel::PRIMARY,
            RHICommandBufferLevel::Secondary => vk::CommandBufferLevel::SECONDARY,
        };

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pool)
            .level(buffer_level)
            .command_buffer_count(info.count);

        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info)? }
            .into_iter()
            .map(|el| VulkanCommandBuffer {
                command_buffer: el,
                context: Arc::clone(&context),
            })
            .collect::<Vec<_>>();

        Ok(command_buffer)
    }
}

impl RHICommandBuffer<VulkanContext> for VulkanCommandBuffer {
    fn begin(&self) {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
            .inheritance_info(&inheritance); // Optional.

        unsafe {
            self.context
                .device
                .begin_command_buffer(self.command_buffer, &info)
                .expect("Couldn't bein command buffer");
        }
    }

    fn end(&self) {
        unsafe {
            self.context.device.end_command_buffer(self.command_buffer);
        }
    }

    fn reset(&mut self) {
        unsafe {
            self.context
                .device
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::default());
        }
    }

    fn push_command(&mut self, command: RHICommands<'_, VulkanContext>) {
        let device = &self.context.device;
        unsafe {
            match command {
                RHICommands::BeginRenderPass(info) => {
                    let render_area = vk::Rect2D::builder()
                        .offset(vk::Offset2D::default())
                        .extent(vk::Extent2D {
                            width: info.render_area.0,
                            height: info.render_area.1,
                        });
                    let color_clear_value = vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: info.clear_values,
                        },
                    };
                    let info = vk::RenderPassBeginInfo::builder()
                        .render_pass(info.renderpass.renderpass)
                        .framebuffer(info.framebuffer.frame_buffer)
                        .render_area(render_area)
                        .clear_values(std::slice::from_ref(&color_clear_value));
                    device.cmd_begin_render_pass(
                        self.command_buffer,
                        &info,
                        vk::SubpassContents::INLINE,
                    );
                }
                RHICommands::EndRenderPass => {
                    device.cmd_end_render_pass(self.command_buffer);
                }
                RHICommands::BindPipeline(pipeline) => {
                    device.cmd_bind_pipeline(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.pipeline,
                    );
                }
                RHICommands::Draw(draw_info) => {
                    device.cmd_draw(
                        self.command_buffer,
                        draw_info.vertex_count,
                        draw_info.instance_count,
                        draw_info.first_vertex,
                        draw_info.first_instance,
                    );
                }
                RHICommands::SetViewport(size) => {
                    let viewport = vk::Viewport::builder()
                        .width(size.0 as f32)
                        .height(size.1 as f32)
                        .max_depth(1.0)
                        .build();
                    device.cmd_set_viewport(self.command_buffer, 0, &[viewport]);
                }
                RHICommands::SetScissor(size) => {
                    let scissor = vk::Rect2D::builder()
                        .offset(vk::Offset2D { x: 0, y: 0 })
                        .extent(vk::Extent2D {
                            width: size.0,
                            height: size.1,
                        });
                    device.cmd_set_scissor(self.command_buffer, 0, &[scissor]);
                }
            }
        }
    }

    fn submit(
        &mut self,
        wait_semaphores: &[&VulkanSemaphore],
        signal_semahpores: &[&VulkanSemaphore],
        fence: Option<&VulkanFence>,
    ) {
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|semaphore| semaphore.0)
            .collect::<Vec<_>>();
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_buffer];
        let signal_semaphores = signal_semahpores.iter().map(|el| el.0).collect::<Vec<_>>();
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context
                .device
                .queue_submit(
                    self.context.data.graphics_queue,
                    &[submit_info],
                    fence.map_or(vk::Fence::null(), |fence| fence.0),
                )
                .expect("Couldn't submit to the queue");
        }
    }
}
