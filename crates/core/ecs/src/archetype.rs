use thiserror::Error;

use crate::{
    component::{Component, ComponentId, ComponentTupple, get_component_id},
    component_storage::ComponentStorageErased,
    entity::{Entity}
};
use std::{cell::{self, Ref, RefCell, RefMut}, collections::HashMap};

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeSignature(pub Vec<ComponentId>); // need to be always sorted

#[derive(Error, Debug)]
#[error("The Component is already present in the signature/archetype/entity.")]
pub struct AddSignatureError;

#[derive(Error, Debug)]
#[error("Try to remove a component which is not present in the signature/archetype/entity.")]
pub struct RemoveSignatureError;

impl ArchetypeSignature {
    pub fn add_sorted<T: Component>(&mut self) -> Result<usize, AddSignatureError> {
        self.add_sorted_id(get_component_id::<T>())
    }

    pub fn add_sorted_id(&mut self, id: ComponentId) -> Result<usize, AddSignatureError> {
        let pos = match self.0.binary_search(&id) {
            Ok(_pos) => Err(AddSignatureError),
            // if doesn't exist
            Err(pos) => Ok(pos),
        }?;
        self.0.insert(pos, id);
        Ok(pos)
    }

    pub fn remove<T: Component>(&mut self) -> Result<(), RemoveSignatureError> {
        self.remove_id(get_component_id::<T>())
    }
    
    pub fn remove_id(&mut self, id: ComponentId) -> Result<(), RemoveSignatureError> {
        self.0.remove(
            self.0
                .iter()
                .position(|x| *x == id)
                .ok_or(RemoveSignatureError)?,
        );
        Ok(())
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.0.contains(&get_component_id::<T>())
    }

    pub fn contains_id(&self, id: ComponentId) -> bool {
        self.0.contains(&id)
    }

    pub fn find<T: Component>(&self) -> Option<ArchetypeCollumn> {
        let id: ComponentId = get_component_id::<T>();
        self.find_id(id)
    }

    pub fn find_id(&self, id: ComponentId) -> Option<ArchetypeCollumn> {
        self.0.binary_search(&id).ok()
    }
}

pub type ArchetypeRow = usize;
pub type ArchetypeCollumn = usize;
pub type ArchetypeId = usize;

pub struct Archetype {
    pub(crate) signature: ArchetypeSignature, // the index of the type id represent the collumn
    pub(crate) components: Vec<Box<RefCell<dyn ComponentStorageErased>>>, // each collumns = a component

    pub(crate) next_row: ArchetypeRow, // each row = entity
    pub(crate) entity_to_row: HashMap<Entity, ArchetypeRow>,
    pub(crate) row_to_entity: Vec<Entity>,
}

#[derive(Error, Debug)]
pub enum AccessComponentError {
    #[error("Trying to get a component from an entity that is not present in the archetype")]
    EntityNotFound,
    #[error("Trying to get a component that is not present in the archetype/entity")]
    ComponentNotInArchetype,
    #[error("Should not happen, meaning that the signature don't represent the data")]
    SignatureTypeUnsynced,
    #[error("Cannot borrow immutably the data, it is already borrowed")]
    BorrowError(#[from] cell::BorrowError),
    #[error("Cannot borrow the data mutably, it is already borrowed")]
    BorrowMutError(#[from] cell::BorrowMutError),
} 

impl Archetype {
    pub fn new_blanck() -> Self {
        Archetype {
            signature: Default::default(),
            components: Default::default(),
            next_row: 0,
            entity_to_row: Default::default(),
            row_to_entity: Default::default(),
        }
    }

    pub fn new_from_archetype_add<T: ComponentTupple>(
        archetype: &Archetype,
    ) -> Result<Self, AddSignatureError> {
        let mut components: Vec<Box<RefCell<dyn ComponentStorageErased>>> = Default::default();

        for component_storage in &archetype.components {
            let borrowed = component_storage.borrow();
            components.push(borrowed.new_vec_of_same_type());
        }

        let mut signature = archetype.signature.clone();
        T::add_component_storage(&mut signature, &mut components)?;

        Ok(Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: Default::default(),
            row_to_entity: Default::default()
        })
    }

    pub fn new_from_archetype_remove<T: ComponentTupple>(
        archetype: &Archetype,
    ) -> Result<Self, RemoveSignatureError> {
        let mut signature = archetype.signature.clone();
        T::remove_signature(&mut signature)?;

        let mut components: Vec<Box<RefCell<dyn ComponentStorageErased>>> = Default::default();

        for (i, component_storage) in archetype.components.iter().enumerate() {
            if T::component_id_match_any_component(archetype.signature.0[i]) {
                continue;
            }
            let borrowed = component_storage.borrow();
            components.push(borrowed.new_vec_of_same_type());
        }

        Ok(Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: Default::default(),
            row_to_entity: Default::default()
        })
    }

    /// The row are unintialized
    /// you NEED to initialize the components just after this.
    /// if the signature is [], there's no need because there's no component
    pub fn add_entity<T: ComponentTupple>(&mut self, entity: Entity, component_tupple: T) -> ArchetypeRow {
        let row = self.next_row;
        self.entity_to_row
            .insert(entity, row);

        self.row_to_entity.push(entity);

        self.next_row += 1;

        let _ = component_tupple.initialize_component(entity, self);

        row
    }

    /// update entity_to_row, row_to_entity and next_row but does not increase the size of the
    /// component vectors
    /// self.components[0][row] will panic.
    /// After calling this function, the caller needs to manually increase the size
    pub unsafe fn add_entity_no_push(&mut self, entity: Entity) -> ArchetypeRow {
        let row = self.next_row;
        self.entity_to_row
            .insert(entity, row);

        self.row_to_entity.push(entity);

        self.next_row += 1;

        row
    }


    pub fn get_row(&self, entity: Entity) -> Option<ArchetypeRow> {
        self.entity_to_row.get(&entity).cloned()
    }

    pub fn get_entity(&self, row: ArchetypeRow) -> Option<Entity> {
        self.row_to_entity.get(row).cloned()
    }

    fn removed_entity_map_update_from_row(&mut self, row: usize){
        let last_entity = self.row_to_entity[self.next_row - 1];
        self.entity_to_row.insert(last_entity, row);

        let new_entity = self.row_to_entity[row];
        self.entity_to_row.remove(&new_entity);

        self.row_to_entity[row] = last_entity;
        let _ = self.row_to_entity.pop();
    }

    /// remove the entity from the maps, if the entity does not exist, it does nothing
    fn removed_entity_map_update_from_entity(&mut self, entity: Entity){
        let current_row = self.entity_to_row.get(&entity);
        if current_row.is_none(){
            return;
        }
        let current_row = *current_row.unwrap();

        let last_entity = self.row_to_entity[self.next_row - 1];

        self.entity_to_row.insert(last_entity, current_row);
        self.entity_to_row.remove(&entity);

        self.row_to_entity[current_row] = last_entity;
        let _ = self.row_to_entity.pop();
    }

    pub unsafe fn remove_entity_no_storage_update(&mut self, entity: Entity){
        self.removed_entity_map_update_from_entity(entity);
        self.next_row -= 1;
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(row) = self.get_row(entity) {
            self.remove_row(row);
        }
    }

    pub fn remove_row(&mut self, row: ArchetypeRow) {
        for component_storage in &mut self.components {
            component_storage.borrow_mut().swap_remove_erased(row);
        }

        self.removed_entity_map_update_from_row(row);

        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor
    pub unsafe fn soft_remove(&mut self, row: ArchetypeRow) {
        for component_storage in &mut self.components {
            unsafe {
                component_storage.borrow_mut().soft_swap_remove_erased(row);
            }
        }

        self.removed_entity_map_update_from_row(row);

        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor except for excepted collumns where it does call it
    pub unsafe fn soft_remove_except(
        &mut self,
        row: ArchetypeRow,
        except_col: &[ArchetypeCollumn],
    ) {
        for (i, component_storage) in self.components.iter_mut().enumerate() {
            if except_col.contains(&i) {
                component_storage.borrow_mut().swap_remove_erased(row);
                continue;
            }
            unsafe {
                component_storage.borrow_mut().soft_swap_remove_erased(row);
            }
        }

        self.removed_entity_map_update_from_row(row);

        // substract last row
        self.next_row -= 1;
    }

    pub fn get_component<T: Component>(&'_ self, entity: Entity) -> Result<Ref<'_, T>, AccessComponentError>{
        match self.get_row(entity) {
            Some(row) => self.get_component_row::<T>(row),
            None => Err(AccessComponentError::EntityNotFound),
        }
    }
    pub fn get_component_row<T: Component>(
        &'_ self,
        row: ArchetypeRow,
    ) -> Result<Ref<'_, T>, AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::EntityNotFound)?;
        let component_storage_ref_dyn = self.components[col].try_borrow()?;
        let component_ref: Ref<'_, T> = Ref::filter_map(
            component_storage_ref_dyn,
            |e | e
                .as_any_ref()
                .downcast_ref::<Vec<T>>()
                .map(|e| &e[row])
        ).map_err(|_e| AccessComponentError::SignatureTypeUnsynced)?;
        Ok(component_ref)
    }

    pub fn get_component_mut<T: Component>(
        &'_ self,
        entity: Entity,
    ) -> Result<RefMut<'_, T>, AccessComponentError> {
        match self.get_row(entity) {
            Some(row) => self.get_component_row_mut::<T>(row),
            None => Err(AccessComponentError::EntityNotFound),
        }
    }
    pub fn get_component_row_mut<T: Component>(
        &'_ self,
        row: ArchetypeRow,
    ) -> Result<RefMut<'_, T>, AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::ComponentNotInArchetype)?;
        let component_storage_ref_dyn = self.components[col].try_borrow_mut()?;
        let component_ref = RefMut::filter_map(
            component_storage_ref_dyn,
            |e | e
                .as_any_mut()
                .downcast_mut::<Vec<T>>()
                .map(|e| &mut e[row])
        ).map_err(|_e| AccessComponentError::SignatureTypeUnsynced)?;
        Ok(component_ref)

        // Ok(&mut (self.components[col]
        //     .as_any_mut()
        //     .downcast_mut::<Vec<T>>()
        //     .ok_or(AccessComponentError::SignatureTypeUnsynced)?)[row])
    }

    /// set the component
    /// if the row == len, it pushes
    pub fn set_component<T: Component>(&self, entity: Entity, component: T) -> Result<(), AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::ComponentNotInArchetype)?;
        let row = self.entity_to_row.get(&entity).cloned().ok_or(AccessComponentError::EntityNotFound)?;
        let mut borrowed = self.components[col].try_borrow_mut()?;
        let component_storage = borrowed
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .ok_or(AccessComponentError::SignatureTypeUnsynced)?;
        if component_storage.len() == row {
            component_storage.push(component);
        } else {
            component_storage[row] = component;
        }
        Ok(())
    }

    pub fn get_row_count(&self) -> ArchetypeRow {
        self.next_row
    }
}

pub type ComponentToArchetypeMap = HashMap<ComponentId, Vec<ArchetypeId>>;
pub type SignatureToArchetypeMap = HashMap<ArchetypeSignature, ArchetypeId>;
