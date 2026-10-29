use engine::application::Application;
use gl_renderer::renderer::GlRenderer;
use logger::logger::LoggerBuilder;
use opengl::context::OpenGlPlugin;
use rendergraph_plugin::RenderGraphPlugin;
use rhi_plugin::plugin::RHIPlugin;
use simple_renderer::SimpleRendererPlugin;
use vulkan::context::VulkanContext;
use window::window::WindowPlugin;

use crate::{minimal_plugin::MinimalPlugin, test_vulkan::TestVulkanPlugin};

mod assets {
    include!(concat!(env!("OUT_DIR"), "/generated_assets.rs"));
}

mod camera_movement;
mod minimal_plugin;
mod movement;
mod test_vulkan;

fn main() {
    let _ = LoggerBuilder::build_default();

    Application::new()
        .add_plugin::<WindowPlugin>()
        .add_plugin::<RHIPlugin<VulkanContext>>()
        // .add_plugin::<OpenGlPlugin>()
        // .add_plugin::<GlRenderer>()
        // .add_plugin::<MinimalPlugin>()
        .add_plugin::<RenderGraphPlugin<VulkanContext, 4>>()
        .add_plugin::<SimpleRendererPlugin<VulkanContext, 4>>()
        .add_plugin::<TestVulkanPlugin>()
        .run();
}
