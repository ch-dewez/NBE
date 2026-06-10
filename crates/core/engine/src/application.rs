use std::any::{TypeId};

use ecs::{world::World};

use crate::{app_command::AppCommandHandler, plugin::{Plugin, PluginContext}};


pub struct Application<'w> {
    pub world: World<'w>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl<'w> Application<'w> {
    pub fn new() -> Self{
        Application { 
            world: World::new(),
            plugins: vec![],
        }
    }

    pub fn add_plugin<T: Plugin + Default>(&mut self) -> &mut Self 
    {
        let mut plugin = Box::new(T::default());

        self.init_plugin(plugin.as_mut());

        self.plugins.push(plugin);

        self
    }

    fn init_plugin(&mut self, plugin: &mut dyn Plugin){
        let context: PluginContext<'_, '_> = PluginContext{ application: self};
        plugin.init(context);
    }


    /// remove the plugin if it exists
    pub fn remove_plugin<T: Plugin>(&mut self) -> &mut Self{
        self.remove_plugin_type_id(TypeId::of::<T>());

        self
    }

    pub fn remove_plugin_type_id(&mut self, id: TypeId){
        let index = self.plugins
                .iter()
                .position(|element | element.as_ref().type_id() == id);

        if let Some(idx) = index {
            self.plugins.remove(idx);
        }
    }

    pub fn run(&mut self){
        loop {
            self.world.step();
            // temporarily replace self.plugins so that there are no lifetimes, then we will
            // place it back
            let mut command_handler = AppCommandHandler::default();
            self.world.handle_command(&mut command_handler);

            if command_handler.stop {
                break;
            }

            for plugin in command_handler.plugins_to_remove {
                self.remove_plugin_type_id(plugin);
            }
            for mut plugin in command_handler.plugins_to_add {
                self.init_plugin(plugin.as_mut());
                self.plugins.push(plugin);
            }
        }
    }
}

impl<'w> Default for Application<'w> {
    fn default() -> Self {
        Application::new()
    }
}
