use engine::{self, ApplicationCreationInfo, WindowCreationInfo};
use test_project::Game;

fn main() {
    let window_info: WindowCreationInfo = WindowCreationInfo{name:"test", width:1920, height:1080, mode:engine::WindowMode::Windowed};
    //let info: ApplicationCreationInfo = ApplicationCreationInfo{window_info};
    let info: ApplicationCreationInfo = ApplicationCreationInfo{};
    let mut application = engine::Application::new(info);

    let plugin: Box<Game> = Box::new(Game::default());
    application.add_plugin(plugin);

    application.run();


}
