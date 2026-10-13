use engine::application::Application;
use gl_renderer::renderer::GlRenderer;
use logger::logger::LoggerBuilder;
use opengl::context::OpenGlPlugin;
use window::window::WindowPlugin;

use crate::minimal_plugin::MinimalPlugin;

mod assets {
    include!(concat!(env!("OUT_DIR"), "/generated_assets.rs"));
}

mod camera_movement;
mod minimal_plugin;
mod movement;

fn main() {
    let _ = LoggerBuilder::build_default();

    Application::new()
        .add_plugin::<WindowPlugin>()
        .add_plugin::<OpenGlPlugin>()
        .add_plugin::<GlRenderer>()
        .add_plugin::<MinimalPlugin>()
        .run();
}
