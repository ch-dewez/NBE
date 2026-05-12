use thiserror::Error;

use crate::{
    component::{Component, ComponentId, ComponentTupple, get_component_id},
    component_storage::ComponentStorageErased,
    entity::EntityId,
};
use std::collections::HashMap;

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
    pub(crate) components: Vec<Box<dyn ComponentStorageErased>>, // each collumns = a component

    pub(crate) next_row: ArchetypeRow, // each row = entity
    pub(crate) entity_to_row: Vec<Option<ArchetypeRow>>,
}

#[derive(Error, Debug)]
pub enum AccessComponentError {
    #[error("Trying to get a component from an entity that is not present in the archetype")]
    EntityNotFound,
    #[error("Trying to get a component that is not present in the archetype/entity")]
    ComponentNotInArchetype,
    #[error("Should not happen, meaning that the signature don't represent the data")]
    SignatureTypeUnsynced 
} 

impl Archetype {
    pub fn new_blanck() -> Self {
        Archetype {
            signature: Default::default(),
            components: Default::default(),
            next_row: 0,
            entity_to_row: Default::default(),
        }
    }

    pub fn new_from_archetype_add<T: ComponentTupple>(
        archetype: &Archetype,
    ) -> Result<Self, AddSignatureError> {
        let mut components: Vec<Box<dyn ComponentStorageErased>> = Default::default();

        for component_storage in &archetype.components {
            components.push(component_storage.new_vec_of_same_type());
        }

        let mut signature = archetype.signature.clone();
        T::add_component_storage(&mut signature, &mut components)?;

        Ok(Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: vec![],
        })
    }

    pub fn new_from_archetype_remove<T: ComponentTupple>(
        archetype: &Archetype,
    ) -> Result<Self, RemoveSignatureError> {
        let mut signature = archetype.signature.clone();
        T::remove_signature(&mut signature)?;

        let mut components: Vec<Box<dyn ComponentStorageErased>> = Default::default();

        for (i, component_storage) in archetype.components.iter().enumerate() {
            if T::component_id_match_any_component(archetype.signature.0[i]) {
                continue;
            }
            components.push(component_storage.new_vec_of_same_type());
        }

        Ok(Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: vec![],
        })
    }

    /// The row are unintialized
    /// you NEED to initialize the components just after this.
    /// if the signature is [], there's no need because there's no component
    pub unsafe fn add_entity(&mut self, entity: EntityId) -> ArchetypeRow {
        if self.entity_to_row.len() <= entity {
            self.entity_to_row.resize(entity + 1, None);
        }

        let row = self.next_row;
        self.entity_to_row[entity] = Some(row);
        self.next_row += 1;

        // SAFETY: intented uninitialized memory, it needs to be initialized after, left to the
        // caller: push raw bytes
        unsafe {
            self.push_uninitialized_row();
        }

        row
    }

    unsafe fn push_uninitialized_row(&mut self) {
        for component_storage in &mut self.components {
            // SAFETY: intented uninitialized memory: push raw bytes
            unsafe {
                component_storage.push_uninitialized();
            }
        }
    }

    pub fn get_row(&self, entity: EntityId) -> Option<ArchetypeRow> {
        self.entity_to_row[entity]
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        if let Some(row) = self.get_row(entity) {
            self.remove_row(row);
        }
    }

    pub fn remove_row(&mut self, row: ArchetypeRow) {
        // special case, it's the last row
        for component_storage in &mut self.components {
            component_storage.swap_remove_erased(row);
        }
        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor
    pub unsafe fn soft_remove(&mut self, row: ArchetypeRow) {
        // special case, it's the last row
        for component_storage in &mut self.components {
            unsafe {
                component_storage.soft_swap_remove_erased(row);
            }
        }
        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor except for one collumn where it does call it
    pub unsafe fn soft_remove_except(
        &mut self,
        row: ArchetypeRow,
        except_col: &[ArchetypeCollumn],
    ) {
        for (i, component_storage) in self.components.iter_mut().enumerate() {
            if except_col.contains(&i) {
                component_storage.swap_remove_erased(row);
                continue;
            }
            unsafe {
                component_storage.soft_swap_remove_erased(row);
            }
        }
        // substract last row
        self.next_row -= 1;
    }

    pub fn get_component<T: Component>(&mut self, entity: EntityId) -> Result<&T, AccessComponentError> {
        match self.get_row(entity) {
            Some(row) => self.get_component_row::<T>(row),
            None => Err(AccessComponentError::EntityNotFound),
        }
    }
    pub fn get_component_row<T: Component>(
        &mut self,
        row: ArchetypeRow,
    ) -> Result<&T, AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::EntityNotFound)?;
        Ok(&(self.components[col]
            .as_any_ref()
            .downcast_ref::<Vec<T>>()
            .ok_or(AccessComponentError::SignatureTypeUnsynced)?)[row])
    }

    pub fn get_component_mut<T: Component>(
        &mut self,
        entity: EntityId,
    ) -> Result<&mut T, AccessComponentError> {
        match self.get_row(entity) {
            Some(row) => self.get_component_row_mut::<T>(row),
            None => Err(AccessComponentError::EntityNotFound),
        }
    }
    pub fn get_component_row_mut<T: Component>(
        &mut self,
        row: ArchetypeRow,
    ) -> Result<&mut T, AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::ComponentNotInArchetype)?;
        Ok(&mut (self.components[col]
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .ok_or(AccessComponentError::SignatureTypeUnsynced)?)[row])
    }

    pub fn set_component<T: Component>(&mut self, entity: EntityId, component: T) -> Result<(), AccessComponentError> {
        let col = self
            .signature
            .find::<T>()
            .ok_or(AccessComponentError::ComponentNotInArchetype)?;
        let row = self.entity_to_row[entity].ok_or(AccessComponentError::EntityNotFound)?;
        self.components[col]
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .ok_or(AccessComponentError::SignatureTypeUnsynced)?[row] = component;
        Ok(())
    }

    pub fn get_row_count(&self) -> ArchetypeRow {
        self.next_row
    }
}

pub type ComponentToArchetypeMap = HashMap<ComponentId, Vec<ArchetypeId>>;
pub type SignatureToArchetypeMap = HashMap<ArchetypeSignature, ArchetypeId>;
