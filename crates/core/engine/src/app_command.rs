use std::{any::{Any, TypeId}, marker::PhantomData};

use ecs::command::{CommandHandler, CommandHandlerTrait, CommandId, CommandTrait};

use crate::plugin::Plugin;

#[derive(Default)]
pub(crate) struct AppCommandHandler {
    pub stop: bool,
    pub plugins_to_add: Vec<Box<dyn Plugin>>,
    pub plugins_to_remove: Vec<TypeId>,
}

impl CommandHandlerTrait for AppCommandHandler {}

// =============
// App commands
// =========

#[derive(Default)]
pub struct StopAppCommand {}
impl CommandTrait for StopAppCommand{
    fn get_handler(&self) -> CommandHandler {
        CommandHandler::External
    }

    fn handle_from_external(&mut self, handler: &mut dyn CommandHandlerTrait){
        let handler = (handler as &mut dyn Any).downcast_mut::<AppCommandHandler>().unwrap();

        handler.stop = true;
    }
}
impl StopAppCommand{
    /// add component to the entity
    pub fn new() -> Self{
        Self{}
    }
}


struct AddPluginCommandId {}
#[derive(Default)]
pub struct AddPluginCommand<T: Plugin> {
    phantom_data: PhantomData<T>
}
impl<T: Plugin + Default> CommandTrait for AddPluginCommand<T>{
    fn get_id(&self) -> CommandId {
        TypeId::of::<AddPluginCommandId>()
    }

    fn get_handler(&self) -> CommandHandler {
        CommandHandler::External
    }

    fn handle_from_external(&mut self, handler: &mut dyn CommandHandlerTrait){
        let handler = (handler as &mut dyn Any).downcast_mut::<AppCommandHandler>().unwrap();

        let plugin = Box::new(T::default());

        handler.plugins_to_add.push(plugin);
    }
}
impl<T: Plugin> AddPluginCommand<T>{
    /// add component to the entity
    pub fn new() -> Self{
        Self{
            phantom_data: Default::default(),
        }
    }
}

struct RemovePluginCommandId {}
#[derive(Default)]
pub struct RemovePluginCommand<T: Plugin> {
    phantom_data: PhantomData<T>
}
impl<T: Plugin + Default> CommandTrait for RemovePluginCommand<T>{
    fn get_id(&self) -> CommandId {
        TypeId::of::<RemovePluginCommandId>()
    }

    fn get_handler(&self) -> CommandHandler {
        CommandHandler::External
    }

    fn handle_from_external(&mut self, handler: &mut dyn CommandHandlerTrait){
        let handler = (handler as &mut dyn Any).downcast_mut::<AppCommandHandler>().unwrap();

        handler.plugins_to_remove.push(TypeId::of::<T>());
    }
}
impl<T: Plugin> RemovePluginCommand<T>{
    /// add component to the entity
    pub fn new() -> Self{
        Self{
            phantom_data: Default::default(),
        }
    }
}
