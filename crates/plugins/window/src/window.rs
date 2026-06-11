use ecs::{command::Command, event::{Event, EventWriter}, ressource::{ResMut, Ressource}, system::SystemParam, world::World};
use engine::{app_command::StopAppCommand, plugin::Plugin};
use glfw::{Context, Glfw, GlfwReceiver, PWindow};

use crate::input::{InputManager, reset_mouse_delta, update_input_manager};

pub struct GLFWRes (pub Glfw);
impl Ressource for GLFWRes {}

pub struct GLFWWindowRes (pub PWindow);
impl Ressource for GLFWWindowRes {}
pub struct GLFWEventRes (pub GlfwReceiver<(f64, glfw::WindowEvent)>);

impl Ressource for GLFWEventRes  {}

#[derive(Default)]
pub struct WindowPlugin {

}

pub struct EcsWindowEvent (pub glfw::WindowEvent);
impl Event for EcsWindowEvent{}

fn window_update(mut glfw: ResMut<GLFWRes> , window: ResMut<GLFWWindowRes>, events: ResMut<GLFWEventRes>, mut event_writer: EventWriter<EcsWindowEvent>, mut command: ResMut<Command>){
    if window.0.should_close() {
        command.0.push(Box::new(StopAppCommand{}));
        return;
    }
    glfw.0.poll_events();

    for (_, event) in glfw::flush_messages(&events.0) {
        event_writer.write(EcsWindowEvent(event));
    }
}

impl Plugin for WindowPlugin {
    fn init<'a, 'b>(&self, context:engine::plugin::PluginContext<'a, 'b>) {
        let glfw = glfw::init(glfw::fail_on_errors).unwrap();


        context.application.world
            .add_ressource(GLFWRes ( glfw ));
    }

    fn uninit<'a, 'b>(&self, _context:engine::plugin::PluginContext<'a, 'b>) {
        
    }
}

impl WindowPlugin {
    pub fn create_window(world: &mut World){
        let mut glfw = ResMut::<GLFWRes>::retrieve_no_local(world).expect("Couldn't retrieve glfw");

        let (mut window, events) = glfw.0.create_window(1000, 800, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();

        window.set_framebuffer_size_polling(true);

        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);

        drop(glfw);

        world
            .add_ressource(GLFWWindowRes( window ))
            .add_ressource(GLFWEventRes ( events ))
            .add_ressource(InputManager::default())
            .add_event::<EcsWindowEvent>()
            .add_system(window_update)
            .add_system(reset_mouse_delta)
            .add_system(update_input_manager);
    }
}
