use std::any::Any;
use crate::application::Application;


pub trait Plugin: Any {
    fn init<'a, 'b>(&self, context:PluginContext<'a, 'b>);
    fn uninit<'a, 'b>(&self, context:PluginContext<'a, 'b>);
}

// TODO: Maybe I should not put the whole app in it, but that's the easiest way to do it
pub struct PluginContext<'a, 'b> {
    pub application: &'a mut Application<'b>,
}
