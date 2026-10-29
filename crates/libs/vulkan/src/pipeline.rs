use std::sync::Arc;

use anyhow::Result;
use logger::log::info;
use rhi_lib::traits::{RHIPipeline, RHIPipelineCreateInfo};
use vulkanalia::{bytecode::Bytecode, prelude::v1_4::*};

use crate::{context::VulkanContext, shaders};

fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code(bytecode.code())
        .code_size(bytecode.code_size())
        .build();

    let shader_module = unsafe { device.create_shader_module(&info, None)? };

    Ok(shader_module)
}

pub struct VulkanPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,

    context: Arc<VulkanContext>,
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}

impl VulkanPipeline {
    pub fn new<'a>(
        context: Arc<VulkanContext>,
        info: RHIPipelineCreateInfo<'a, VulkanContext>,
    ) -> Result<Self> {
        info!("[Graphics] Creating Vulkan Pipeline");

        let device = &context.device;

        // TODO: Shader in info
        let shader_code = shaders::VulkanDefaultShaders::VulkanDefaultShader.bytes();
        let shader_module = create_shader_module(device, shader_code)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(shader_module)
            .name(b"main_vs\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(shader_module)
            .name(b"main_fs\0");

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let layout_info = vk::PipelineLayoutCreateInfo::builder();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        let stages = &[vert_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .dynamic_state(&dynamic_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(info.renderpass.renderpass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null()) // Optional.
            .base_pipeline_index(-1); // Optional.

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
                .0[0]
        };

        unsafe { device.destroy_shader_module(shader_module, None) };
        Ok(VulkanPipeline {
            pipeline,
            layout: pipeline_layout,
            context,
        })
    }
}

impl RHIPipeline for VulkanPipeline {}
