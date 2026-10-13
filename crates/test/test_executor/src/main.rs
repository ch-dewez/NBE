use engine::application::Application;
use gl_renderer::renderer::GlRenderer;
use logger::logger::LoggerBuilder;
use opengl::context::OpenGlPlugin;
use test_project::plugin::GamePlugin;
use window::window::WindowPlugin;

fn main() {
    let _ = LoggerBuilder::build_default();

    Application::new()
        .add_plugin::<WindowPlugin>()
        .add_plugin::<OpenGlPlugin>()
        .add_plugin::<GlRenderer>()
        .add_plugin::<GamePlugin>()
        .run();
}
