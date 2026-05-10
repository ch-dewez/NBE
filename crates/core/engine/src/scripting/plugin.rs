use crate::Application;


struct PluginInitInfo {
    application: &Application
}

pub trait Plugin{
    fn init(self: &Self, info: &PluginInitInfo) {}
    fn scene_start(self: &Self) {}
}
