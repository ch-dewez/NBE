use gl;
use glfw::{PWindow, WindowEvent};
use crate::{Plugin, scene::scene_manager::{self, SceneManager}, window::{Window, WindowCreationInfo}};

pub struct ApplicationCreationInfo {}

pub struct Application {
    scene_manager: SceneManager,
    plugins: Vec<Box<dyn Plugin>>
}

impl Application {
    pub fn new(info: ApplicationCreationInfo) -> Application{
        let plugins = Vec::default();
        let scene_manager = SceneManager::new();
        Application{scene_manager, plugins}
    }

    pub fn run(self: &mut Self) {
        // while self.window.update() {
        //    self.window.process_events(|window, event| {Application::handle_event(window, event)});
        //
        //     self.scene_manager.update();
        //
        //     unsafe {
        //         gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        //         gl::Clear(gl::COLOR_BUFFER_BIT);
        //     }
        // }
    }

    pub fn add_plugin(self: &mut Self, plugin:Box<dyn Plugin>){
        self.plugins.push(plugin);

    }

    //fn handle_event(_window: &mut PWindow, _event:WindowEvent) {
}

}
