use std::marker::PhantomData;
use std::sync::Arc;

use arrayvec::ArrayVec;
use ecs::ressource::{Res, ResMut, Ressource};
use ecs::system::{SystemExecutionTime, SystemParam};
use engine::plugin::Plugin;
use logger::log::info;
use rendergraph_lib::rendergraph::{AddRenderStageInfo, RenderStageHandle};
use rendergraph_plugin::RenderGraphRes;
use rhi_lib::traits::{
    ColorOutputFormat, RHIAttachment, RHIAttachmentType, RHICommandBuffer, RHICommandDraw,
    RHICommands, RHIContextAllTraits, RHIFence, RHIFenceCreateInfo, RHIPipelineCreateInfo,
    RHIRenderPassCreateInfo, RHISemaphoreCreateInfo, RHISwapchain, RHISwapchainCreateInfo,
    RHITypesTrait, RHIWindowInfo,
};
use rhi_plugin::ressource::RHIContextRes;
use window::window::GLFWWindowRes;
// use vulkan::vulkanalia::prelude::v1_4::*;
// use vulkan::vulkanalia::vk::Fence;
// use vulkan::{context::VulkanContext, vulkanalia::vk::KhrSwapchainExtensionDeviceCommands};

fn render<C: RHIContextAllTraits, const N: usize>(
    mut simple_renderer_context: ResMut<SimpleRendererContext<C, N>>,
    mut rendergraph_res: ResMut<RenderGraphRes<C, N>>,
) {
    info!("Begin render frame");
    let image_index = simple_renderer_context
        .swapchain
        .acquire_next_image(
            Some(&simple_renderer_context.image_available_semaphore[simple_renderer_context.frame]),
            None,
        )
        .expect("Couldn't acquire next_image") as usize;

    rendergraph_res.set_image_index(image_index);

    // If the acquired image is in flight, wait for it
    if let Some(fence_id) = simple_renderer_context.images_in_flight[image_index] {
        simple_renderer_context.in_flight_fences[fence_id].wait()
    }

    simple_renderer_context.images_in_flight[image_index] = Some(simple_renderer_context.frame);

    let mut renderstage_context = rendergraph_res
        .begin_stage_record(simple_renderer_context.renderstage, [0.0, 0.0, 0.0, 1.0])
        .expect("Couldn't get renderstage");

    let command_buffer = &mut renderstage_context.command_buffer;
    command_buffer.push_command(RHICommands::BindPipeline(&simple_renderer_context.pipeline));
    command_buffer.push_command(RHICommands::Draw(RHICommandDraw {
        vertex_count: 3,
        instance_count: 1,
        first_vertex: 0,
        first_instance: 0,
    }));

    rendergraph_res
        .end_stage_record(simple_renderer_context.renderstage)
        .expect("Couldn't end stage record");

    let submit_fence = &simple_renderer_context.in_flight_fences[simple_renderer_context.frame];
    submit_fence.reset();
    rendergraph_res.submit(
        &[&simple_renderer_context.image_available_semaphore[simple_renderer_context.frame]],
        &[&simple_renderer_context.render_finished_semaphore[simple_renderer_context.frame]],
        Some(submit_fence),
    );

    simple_renderer_context.swapchain.present(
        image_index as u32,
        &[&simple_renderer_context.render_finished_semaphore[simple_renderer_context.frame]],
        &[],
    );

    simple_renderer_context.frame = (simple_renderer_context.frame + 1)
        % simple_renderer_context.render_finished_semaphore.len();
}

pub struct SimpleRendererContext<C: RHIContextAllTraits, const N: usize = 2> {
    // TODO: move swapchain elsewhere
    swapchain: C::Swapchain,

    renderstage: RenderStageHandle,

    image_available_semaphore: ArrayVec<C::Semaphore, N>,
    render_finished_semaphore: ArrayVec<C::Semaphore, N>,

    in_flight_fences: ArrayVec<C::Fence, N>,
    images_in_flight: ArrayVec<Option<usize>, N>,

    frame: usize,

    pipeline: C::Pipeline,

    context: Arc<C>,
}
impl<C: RHIContextAllTraits, const N: usize> Ressource for SimpleRendererContext<C, N> {}

impl<C: RHIContextAllTraits, const N: usize> Drop for SimpleRendererContext<C, N> {
    fn drop(&mut self) {
        self.context.wait_idle();
    }
}

pub struct SimpleRendererPlugin<C: RHITypesTrait, const N: usize>(PhantomData<C>);

impl<C: RHITypesTrait, const N: usize> Default for SimpleRendererPlugin<C, N> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C: RHIContextAllTraits, const N: usize> Plugin for SimpleRendererPlugin<C, N> {
    fn init<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {
        let rhi_context = Res::<RHIContextRes<C>>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get Graphics backend context");

        // TODO: Move window elsewhere
        let window = Res::<GLFWWindowRes>::retrieve_no_local(&context.application.world)
            .expect("Couldn't get glfw window handle");
        let window_res = window.0.get_size();
        let window_info = RHIWindowInfo {
            window: &window.0,
            display: &window.0,
            window_res: (window_res.0 as u32, window_res.1 as u32),
        };

        let swapchain_info = RHISwapchainCreateInfo { window_info };
        let mut swapchain = Arc::clone(&rhi_context)
            .create_swapchain(swapchain_info)
            .expect("Couldn't create swapchain");

        let semaphore_info = RHISemaphoreCreateInfo {};
        let semaphore_1 = (0..N)
            .into_iter()
            .map(|_| {
                Arc::clone(&rhi_context)
                    .create_semaphore(semaphore_info.clone())
                    .expect("Couldn't create semaphore")
            })
            .collect();
        let semaphore_2 = (0..N)
            .into_iter()
            .map(|_| {
                Arc::clone(&rhi_context)
                    .create_semaphore(semaphore_info.clone())
                    .expect("Couldn't create semaphore")
            })
            .collect();

        let fence_info = RHIFenceCreateInfo { signaled: true };
        let in_flight_fences = (0..N)
            .into_iter()
            .map(|_| {
                Arc::clone(&rhi_context)
                    .create_fence(fence_info.clone())
                    .expect("Couldn't create Fence")
            })
            .collect();
        let images_in_flight = (0..N).into_iter().map(|_| None).collect();

        let mut rendergraph =
            ResMut::<RenderGraphRes<C, N>>::retrieve_no_local(&context.application.world)
                .expect("Couldn't get rendergraph");

        let output_attachement: RHIAttachment<C, N> = RHIAttachment {
            attachment_type: RHIAttachmentType::Color,
            texture: swapchain.take_textures().unwrap().into_iter().collect(),
        };
        rendergraph.set_output_attachment(output_attachement);

        let renderpass_info = RHIRenderPassCreateInfo {
            color_output_format: ColorOutputFormat::SameAsSwapchain(&swapchain),
        };
        let renderstage_info = AddRenderStageInfo { renderpass_info };
        let renderstage_handle = rendergraph
            .add_stage(renderstage_info)
            .expect("Couldn't create renderstage");

        let renderstage = rendergraph.get_stage(renderstage_handle).unwrap();

        let pipeline_info = RHIPipelineCreateInfo {
            renderpass: &renderstage.renderpass,
        };
        let pipeline = Arc::clone(&rhi_context)
            .create_pipeline(pipeline_info)
            .expect("Couldn't create pipeline");

        let simple_renderer_context = SimpleRendererContext::<C, N> {
            swapchain,

            renderstage: renderstage_handle,

            pipeline,

            image_available_semaphore: semaphore_1,
            render_finished_semaphore: semaphore_2,

            in_flight_fences,
            images_in_flight,

            frame: 0,

            context: Arc::clone(&rhi_context),
        };

        drop(window);
        drop(rendergraph);
        drop(rhi_context);

        context
            .application
            .world
            .add_ressource(simple_renderer_context)
            .add_system(render::<C, N>, SystemExecutionTime::Rendering);
    }

    fn uninit<'a, 'b>(&self, context: engine::plugin::PluginContext<'a, 'b>) {}
}
