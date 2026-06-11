use ecs::{event::EventReader, ressource::{Res, ResMut, Ressource}, system::SystemParam};
use engine::plugin::Plugin;
use glow::{Context, HasContext};
use window::{glfw::{self, WindowEvent}, window::{GLFWRes, GLFWWindowRes, WindowPlugin}};

use crate::graphics_ressources::GraphicsRessourceManager;

pub struct OpenGlContext (pub Context);
impl Ressource for OpenGlContext {}

#[derive(Default)]
pub struct OpenGlPlugin {

}

fn resize_viewport(mut events: EventReader<window::window::EcsWindowEvent>, gl: Res<OpenGlContext>){
    for event in events.iter(){
        if let WindowEvent::FramebufferSize(x, y) = event.0 {
            println!("Resize : {}, {}", x, y);
            unsafe { gl.0.viewport(0, 0, x, y) };
        }
    }
}

impl Plugin for OpenGlPlugin {
    fn init<'a, 'b>(&self, context:engine::plugin::PluginContext<'a, 'b>) {
        let mut glfw = ResMut::<GLFWRes>::retrieve_no_local(&context.application.world).expect("GLFW window not loaded, cannot load glow");

        glfw.0.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.0.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        drop(glfw);

        WindowPlugin::create_window(&mut context.application.world);

        let mut window = ResMut::<GLFWWindowRes>::retrieve_no_local(&context.application.world).expect("GLFW window not loaded, cannot load glow");

        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.0.get_proc_address(s).map_or(std::ptr::null(), |p| p as *const _))
        };

        let size = window.0.get_framebuffer_size();
        unsafe {
            gl.viewport(0, 0, size.0, size.1);
        }

        drop(window);

        context.application.world
            .add_ressource(OpenGlContext ( gl ));
        context.application.world
            .add_ressource(GraphicsRessourceManager::default());
        context.application.world
            .add_system(resize_viewport);
    }

    fn uninit<'a, 'b>(&self, _context:engine::plugin::PluginContext<'a, 'b>) {
        
    }
}


