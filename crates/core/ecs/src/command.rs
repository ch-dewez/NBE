use std::{any::{Any, TypeId}, marker::PhantomData};

use crate::{component::ComponentTupple, entity::Entity, ressource::Ressource, world::World};

pub enum CommandHandler{
    World,
    External
}
pub trait CommandTrait: Any + 'static {
    // if the struct has generics, we can't use the struct type id, so in that case, we use the
    // type id of another struct by overwriting this function. See AddEntityComponent and
    // AddEntityComponentId
    fn get_id(&self) -> CommandId {TypeId::of::<Self>()}

    fn get_handler(&self) -> CommandHandler;
    fn handle_from_world(&mut self, _world:&mut World){}
    fn handle_from_external(&mut self, _handler: &mut dyn CommandHandlerTrait){}
}

pub trait CommandHandlerTrait: Any { }

#[derive(Default)]
pub struct Command (pub Vec<Box<dyn CommandTrait>>);
impl Ressource for Command { }

pub type CommandId = TypeId;

// =======================================
// World commands
// ===================================

pub(crate) struct AddEntityCommandId {}
pub struct AddEntityCommand<T: ComponentTupple> {
    components: Option<T>,

    callback: Option<Box<dyn FnOnce(Entity)>>
}
impl<T: ComponentTupple + 'static> CommandTrait for AddEntityCommand<T>{
    fn get_id(&self) -> CommandId {
        TypeId::of::<AddEntityCommandId>()
    }

    fn get_handler(&self) -> CommandHandler {
        CommandHandler::World
    }

    fn handle_from_world(&mut self, world:&mut World) {
        let entity = if let Some(components) = self.components.take() {
            world.spawn_entity(components).0
        }else {
            world.spawn_entity(()).0
        };

        if let Some(callback) = self.callback.take(){
            callback(entity);
        }
    }
}
impl<T: ComponentTupple> AddEntityCommand<T>{
    /// a new entity with components and a callback, you can use a () for components
    pub fn new(components: T, callback: impl FnOnce(Entity) + 'static) -> Self{
        Self{
            components: Some(components),
            callback: Some(Box::new(callback)) 
        }
    }

    /// a new entity, but no callbacks, you can use a () for components;
    pub fn new_no_callbacks(components: T) -> Self{
        Self{
            components: Some(components),
            callback: None
        }
    }
}

pub struct RemoveEntityCommand{
    entity: Entity
}
impl CommandTrait for RemoveEntityCommand {
    fn get_handler(&self) -> CommandHandler {
        CommandHandler::World
    }

    fn handle_from_world(&mut self, world:&mut World) {
        let _ = world.remove_entity(self.entity);
    }
}
impl RemoveEntityCommand {
    pub fn new(entity: Entity) -> Self{
        Self{
            entity
        }
    }
}

struct AddComponentsCommandId {}
pub struct AddComponentsCommand<T: ComponentTupple> {
    entity: Entity,
    components: Option<T>
}
impl<T: ComponentTupple + 'static> CommandTrait for AddComponentsCommand<T>{
    fn get_id(&self) -> CommandId {
        TypeId::of::<AddComponentsCommandId>()
    }

    fn get_handler(&self) -> CommandHandler {
        CommandHandler::World
    }

    fn handle_from_world(&mut self, world:&mut World) {
        let _ = world.add_component(self.entity, self.components.take().expect("Add Component Command with no commponent given"));
    }
}
impl<T: ComponentTupple> AddComponentsCommand<T>{
    /// add component to the entity
    pub fn new(entity: Entity, components: T) -> Self{
        Self{
            entity,
            components: Some(components),
        }
    }
}

struct RemoveComponentCommandId {}
pub struct RemoveComponentCommand<T: ComponentTupple> {
    entity: Entity,
    phantom_data: PhantomData<T>
}
impl<T: ComponentTupple + 'static> CommandTrait for RemoveComponentCommand<T>{
    fn get_id(&self) -> CommandId {
        TypeId::of::<RemoveComponentCommandId>()
    }

    fn get_handler(&self) -> CommandHandler {
        CommandHandler::World
    }

    fn handle_from_world(&mut self, world:&mut World) {
        let _ = world.remove_component::<T>(self.entity);
    }
}
impl<T: ComponentTupple> RemoveComponentCommand<T>{
    /// add component to the entity
    pub fn new(entity: Entity) -> Self{
        Self{
            entity,
            phantom_data: Default::default(),
        }
    }
}
