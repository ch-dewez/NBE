use std::any::Any;

use ecs::world::World;

use crate::{plugin::{Plugin, PluginContext}};


pub struct Application<'w> {
    world: World<'w>,
    plugins: Vec<Box<dyn Plugin>>,
    
    stop: bool,
    //events: EventManager
}

impl<'w> Application<'w> {
    pub fn new() -> Self{
        Application { 
            world: World::new(),
            plugins: vec![],

            stop: false,
            //events: EventManager {}
        }
    }

    pub fn add_plugin<T: Plugin + Default>(&mut self) -> &mut Self 
    {
        let plugin = Box::new(T::default());
        {
            let context: PluginContext<'_, '_> = PluginContext{ application: self};
            plugin.init(context);
        }

        self.plugins.push(plugin);

        self
    }


    /// remove the plugin if it exists
    pub fn remove_plugin<T: Plugin>(&mut self) -> &mut Self{
        let index = self.plugins
                .iter()
                .position(|element | (element.as_ref() as &dyn Any).is::<T>());

        if let Some(idx) = index {
            self.plugins.remove(idx);
        }

        self
    }

    pub fn run(&mut self){
        while !self.stop  {
            self.world.step();
        }
    }
}

impl<'w> Default for Application<'w> {
    fn default() -> Self {
        Application::new()
    }
}
