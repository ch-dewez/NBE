use std::{any::{Any, TypeId}, cell::{Ref, RefCell, RefMut, UnsafeCell}, collections::HashMap, ops::Deref};
use thiserror::Error;

use crate::{archetype::{AddSignatureError, Archetype, ArchetypeId, ComponentToArchetypeMap, RemoveSignatureError, SignatureToArchetypeMap}, component::{Component, ComponentTupple}, entity::{Entity, EntityId, EntityVersion}, event::{Event, EventRes, SingleEventRes}, ressource::{ResMut, Ressource, get_ressource_id}, system::{IntoSystem, System}, system_manager::SystemManager};


pub struct World<'w> {
    pub(crate) archetypes: Vec<Archetype>, // index 0 will be the one with no component

    next_entity_id: EntityId,
    // important to re-use entity to avoid gaps in data
    // store full Entity to know what is the next version
    free_entity_id: Vec<Entity>,

    entity_to_archetype: Vec<Option<(ArchetypeId, EntityVersion)>>, // entity (index in the vec) -> &Archetype
    component_to_archetype: ComponentToArchetypeMap,
    signature_to_archetype: SignatureToArchetypeMap,

    ressources: HashMap<TypeId, Box<RefCell<dyn Ressource>>>,

    system_manager: UnsafeCell<SystemManager<'w>>
}

pub trait FromWorld {
    fn from_world(world: &World) -> Self;
}

unsafe impl<'w> Sync for World<'w> {}

#[derive(Error, Debug)]
pub enum GetArchetypeFromEntityError{
    #[error("Trying to get an entity id that is not present")]
    EntityIdNotFound,
    #[error("Trying to get an entity that doesn't have the right version")]
    EntityVersionMismatch 
}

#[derive(Error, Debug)]
pub enum AddComponentError {
    #[error("The entity was not found in the world")]
    GetEntity(#[from] GetArchetypeFromEntityError),
    #[error("Trying to add component to an entity that already has it.")]
    AlreadyExistingComponent(#[from] AddSignatureError),
    #[error("Should not happen, the archetype does not have the entity in its map, the map or the world is unsynced")]
    ArchetypeEntityNotFound,
    #[error("Should not happen, the archetype does not have the component in its map, the map or the world is unsynced")]
    ComponentCollumnNotFound,
}

#[derive(Error, Debug)]
pub enum RemoveComponentError {
    #[error("The entity was not found in the world")]
    GetEntity(#[from] GetArchetypeFromEntityError),
    #[error("Trying to remove component to an entity that doesn't have it.")]
    NotExistingComponent(#[from] RemoveSignatureError),
    #[error("Should not happen, the archetype does not have the entity in its map, the map or the world is unsynced")]
    ArchetypeEntityNotFound,
    #[error("Should not happen, the archetype does not have the component in its map, the map or the world is unsynced")]
    ComponentCollumnNotFound,
}


pub type RemoveEntityError = GetArchetypeFromEntityError;

impl<'w> World<'w> {
    pub fn new() -> Self {
        let mut world: World = World{
            archetypes: vec![],
            next_entity_id: 0,
            free_entity_id: Default::default(),
            entity_to_archetype: Default::default(),
            component_to_archetype: Default::default(),
            signature_to_archetype: Default::default(),

            ressources: Default::default(),

            system_manager: UnsafeCell::new(SystemManager::new())
        };

        world.create_archetype(Archetype::new_blanck());

        world
    }

    fn create_archetype(&mut self, archetype:Archetype) -> ArchetypeId{
        self.system_manager.get_mut().invalidate_cache();

        let index: ArchetypeId = self.archetypes.len();

        for component_id in &archetype.signature.0 {
            match self.component_to_archetype.get_mut(component_id){
                Some(archetype_array) => {archetype_array.push(index);} 
                None => {self.component_to_archetype.insert(*component_id, vec![index]);}
            };
        }

        self.signature_to_archetype.insert(archetype.signature.clone(), index);

        self.archetypes.push(archetype);

        index
    }

    pub(crate) fn get_all_archetypes(&self) -> &Vec<Archetype>{
        &self.archetypes
    }

    fn set_entity_to_archetype_map(&mut self, entity: Entity, archetype: ArchetypeId){
        if self.entity_to_archetype.len() <= entity.id{
            self.entity_to_archetype.resize(entity.id+1, None);
        }
        self.entity_to_archetype[entity.id] = Some((archetype, entity.version));
    }

    fn get_archetype_from_entity(&self, entity: Entity) -> Result<ArchetypeId, GetArchetypeFromEntityError>{
        if self.entity_to_archetype.len() <= entity.id{
            return Err(GetArchetypeFromEntityError::EntityIdNotFound);
        }
        let value = self.entity_to_archetype[entity.id]
            .ok_or(GetArchetypeFromEntityError::EntityIdNotFound)?;
        if value.1 != entity.version{
            return Err(GetArchetypeFromEntityError::EntityVersionMismatch); 
        }

        Ok(value.0)
    }

    pub fn spawn_entity<T: ComponentTupple>(&mut self, components: T) -> (Entity, &mut Self) {
        let entity: Entity;
        if let Some(mut free_entity) = self.free_entity_id.pop(){
            free_entity.version += 1;
            entity = free_entity;
        }else {
            entity =  Entity { id: self.next_entity_id, version: 0 };
            self.next_entity_id += 1;
        }
    
        let signature = T::get_signature();
        let archetype_id: ArchetypeId; 
        if let  Some(id) = self.signature_to_archetype.get(&signature){
            archetype_id = *id;
        }else {
            archetype_id = self.create_archetype(T::create_archetype());
        }

        self.archetypes[archetype_id].add_entity(entity, components);

        self.set_entity_to_archetype_map(entity, archetype_id);

        (entity, self)
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<&mut Self, RemoveEntityError> {
        if self.entity_to_archetype.len() <= entity.id{
            return Err(RemoveEntityError::EntityIdNotFound);
        }
        let value = self.entity_to_archetype[entity.id]
            .take()
            .ok_or(RemoveEntityError::EntityIdNotFound)?;
        if value.1 != entity.version{
            return Err(RemoveEntityError::EntityVersionMismatch); 
        }

        let archtype_id = value.0;
        self.archetypes[archtype_id].remove_entity(entity);

        self.free_entity_id.push(entity);

        Ok(self)
    }

    /// add the ressource, if it was already present, it overrides it
    pub fn add_ressource<T: Ressource>(&mut self, ressource:T){
        self.ressources.insert(get_ressource_id::<T>(), Box::new(RefCell::new(ressource)));
    }

    /// remove the ressource, if it was not there, it does nothing
    pub fn remove_ressource<T: Ressource>(&mut self){
        self.ressources.remove(&get_ressource_id::<T>());
    }

    pub fn get_ressource<T: Ressource>(&'_ self) -> Option<Ref<'_, T>> {
        let trait_ref = self.ressources.get(&get_ressource_id::<T>()).and_then(|el| el.try_borrow().ok());
        trait_ref.map(|cell_ref|{
            Ref::map(cell_ref, |reference| {
                (reference as &dyn Any).downcast_ref::<T>().unwrap()
            })
        })
    }

    pub fn get_ressource_mut<'a, T: Ressource>(&'a self) -> Option<RefMut<'a, T>> {
        let trait_ref = self.ressources.get(&get_ressource_id::<T>()).and_then(|el| el.try_borrow_mut().ok());
        trait_ref.map(|cell_ref|{
            RefMut::map(cell_ref, |reference| {
                (reference as &mut dyn Any).downcast_mut::<T>().unwrap()
            })
        })
    }

    pub fn add_component<T:ComponentTupple>(&mut self, entity: Entity, component:T) -> Result<&mut Self, AddComponentError>{
        let current_archetype_id = self.get_archetype_from_entity(entity)?;

        // get the new archetype
        let mut signature = self.archetypes[current_archetype_id].signature.clone();
        T::add_signature(&mut signature)?;

        // check if archetype already exist
        let new_archetype_id = match self.signature_to_archetype.get(&signature){ Some(id) => *id,
            // if it doesn't, create it
            None => {
                self.create_archetype(Archetype::new_from_archetype_add::<T>(&self.archetypes[current_archetype_id])?)
            }
        };

        // copy data to new archetype
        let new_row;
        // SAFETY: The component storage won't be filled with a new row, but this is fine
        // because copy_element_from_another_storage take this case into account
        unsafe {
            new_row = self.archetypes[new_archetype_id].add_entity_no_push(entity);
            debug_assert!(new_row == self.archetypes[new_archetype_id].next_row - 1, "new entity row is not the last row, in add component");
        }
        let current_row = self.archetypes[current_archetype_id].get_row(entity).ok_or(AddComponentError::ArchetypeEntityNotFound)?;

        for current_col_id in 0..self.archetypes[current_archetype_id].signature.0.len() {
            let mut current_col = self.archetypes[current_archetype_id].components[current_col_id].borrow_mut();

            let component_id = self.archetypes[current_archetype_id].signature.0[current_col_id];
            let new_col_id = self.archetypes[new_archetype_id].signature.find_id(component_id).ok_or(AddComponentError::ComponentCollumnNotFound)?;
            let mut new_col = self.archetypes[new_archetype_id].components[new_col_id].borrow_mut();

            new_col
                .copy_element_from_another_storage(
                    current_row, 
                    new_row,

                    current_col.deref()
                );

            // SAFETY: We don't want to call the destructor because we copied the data
            unsafe {
                current_col
                    .soft_swap_remove_erased(current_row);
            }
        }

        T::initialize_component(component, entity, &mut self.archetypes[new_archetype_id]).ok().ok_or(AddComponentError::ArchetypeEntityNotFound)?;

        // SAFETY: We don't want to update the comopnent as it has already been done in the loop
        // above
        unsafe {
            self.archetypes[current_archetype_id]
                .remove_entity_no_storage_update(entity);
        }
        
        self.set_entity_to_archetype_map(entity, new_archetype_id);

        Ok(self)
    }

    pub fn remove_component<T:Component>(&'_ mut self, entity: Entity) -> Result<&'_ mut Self, RemoveComponentError>{
        let current_archetype_id = self.get_archetype_from_entity(entity)?;
        // get the new archetype

        // get the new archetype
        let mut signature = self.archetypes[current_archetype_id].signature.clone();
        T::remove_signature(&mut signature)?;

        // check if archetype already exist
        let new_archetype_id = match self.signature_to_archetype.get(&signature){
            Some(id) => *id,
            // if it doesn't, create it
            None => {
                self.create_archetype(Archetype::new_from_archetype_remove::<T>(&self.archetypes[current_archetype_id])?)
            }
        };

        // copy data to new archetype

        let new_row;
        // SAFETY: The component storage won't be filled with a new row, but this is fine
        // because copy_element_from_another_storage take this case into account
        unsafe {
            new_row = self.archetypes[new_archetype_id].add_entity_no_push(entity);
        }
        let current_row = self.archetypes[current_archetype_id].get_row(entity).ok_or(RemoveComponentError::ArchetypeEntityNotFound)?;

        for current_col_id in 0..self.archetypes[current_archetype_id].signature.0.len() {
            let mut current_col = self.archetypes[current_archetype_id].components[current_col_id].borrow_mut();

            let component_id = self.archetypes[current_archetype_id].signature.0[current_col_id];
            let new_col_id = self.archetypes[new_archetype_id].signature.find_id(component_id);

            // a component that nees to be removed
            if new_col_id.is_none(){
                current_col
                    .swap_remove_erased(current_row);
                continue;
            }
            let new_col_id = new_col_id.unwrap();

            let mut new_col = self.archetypes[new_archetype_id].components[new_col_id].borrow_mut();

            new_col
                .copy_element_from_another_storage(
                    current_row, 
                    new_row,

                    current_col.deref()
                );

            // SAFETY: We don't want to call the destructor because we copied the data
            unsafe {
                current_col
                    .soft_swap_remove_erased(current_row);
            }
        }

        // SAFETY: We don't want to update the comopnent as it has already been done in the loop
        // above
        unsafe {
            self.archetypes[current_archetype_id].remove_entity_no_storage_update(entity);
        }

        self.set_entity_to_archetype_map(entity, new_archetype_id);

        Ok(self)
    }

    pub fn add_system<T, S: System + 'w + Send + Sync>(&mut self, system: impl IntoSystem<T, System = S>) -> &mut Self{
        // Safe, system manager doesn't access it self from world
        unsafe {
            (*self.system_manager.get()).add_system(system, self);
        }
        self
    }

    pub fn add_single_event<T: Event>(&mut self) -> &mut Self{
        self.add_ressource(SingleEventRes::<T>::new());
        self.add_system(|mut res: ResMut<SingleEventRes<T>>| {
            res.swap();
            res.clear_this_frame();
        });
        self
    }

    pub fn add_event<T: Event>(&mut self)-> &mut Self{
        self.add_ressource(EventRes::<T>::new());
        self.add_system(|mut res: ResMut<EventRes<T>>|{
            res.swap();
            res.clear_current();
        });
        self
    }

    pub fn step(&mut self){
        // Safe, system manager doesn't access it self
        unsafe {
            (*self.system_manager.get()).step(self);
        }
    }
}

impl<'w> Default for World<'w>{
    fn default() -> Self {
        World::new()
    }
}
