use crate::{
    component::{Component, ComponentId, get_component_id},
    component_storage::ComponentStorageErased,
    entity::EntityId,
};
use std::collections::HashMap;

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeSignature(pub Vec<ComponentId>); // need to be always sorted
impl ArchetypeSignature {
    pub fn add_sorted(&mut self, id: ComponentId) {
        for i in 0..self.0.len() {
            if self.0[i] >= id {
                self.0.insert(i, id);
                return;
            }
        }
        // no element bigger, so we push
        self.0.push(id);
    }

    pub fn remove(&mut self, id: ComponentId) -> Result<(), &'static str> {
        self.0.remove(
            self.0
                .iter()
                .position(|x| *x == id)
                .ok_or("Couldn't find id in signature")?,
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
        self.0.iter().position(|x| *x == id)
    }

    pub fn find_id(&self, id: ComponentId) -> Option<ArchetypeCollumn> {
        self.0.iter().position(|x| *x == id)
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

    pub(crate) edges: HashMap<ComponentId, ArchetypeEdge>,
}

impl Archetype {
    pub fn new_blanck() -> Self {
        Archetype {
            signature: Default::default(),
            components: Default::default(),
            next_row: 0,
            entity_to_row: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn new_from_archetype_add<T: Component>(
        archetype: &Archetype,
        archetype_id: ArchetypeId,
    ) -> Self {
        let mut signature = archetype.signature.clone();
        signature.add_sorted(get_component_id::<T>());

        let mut components: Vec<Box<dyn ComponentStorageErased>> = Default::default();

        for component_storage in &archetype.components {
            components.push(component_storage.new_vec_of_same_type());
        }
        let new_component_storage: Vec<T> = Vec::new();
        components.push(Box::new(new_component_storage));

        let mut edges: HashMap<ComponentId, ArchetypeEdge> = Default::default();
        edges.insert(get_component_id::<T>(), ArchetypeEdge::Remove(archetype_id));

        Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: vec![],

            edges,
        }
    }

    pub fn new_from_archetype_remove<T: Component>(
        archetype: &Archetype,
        archetype_id: ArchetypeId,
    ) -> Self {
        let mut signature = archetype.signature.clone();
        let _ = signature.remove(get_component_id::<T>());

        let mut components: Vec<Box<dyn ComponentStorageErased>> = Default::default();

        for (i, component_storage) in archetype.components.iter().enumerate() {
            if archetype.signature.0[i] == get_component_id::<T>() {
                continue;
            }
            components.push(component_storage.new_vec_of_same_type());
        }

        let mut edges: HashMap<ComponentId, ArchetypeEdge> = Default::default();
        edges.insert(get_component_id::<T>(), ArchetypeEdge::Add(archetype_id));

        Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: vec![],

            edges,
        }
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
        // move last row to the row, change entity record

        let max_row = self.next_row - 1;

        // special case, it's the last row
        if row == max_row {
            for component_storage in &mut self.components {
                component_storage.remove_last_row();
            }
        } else {
            for component_storage in &mut self.components {
                component_storage.copy_element_override(max_row, row);
                component_storage.remove_last_row();
            }
        }
        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor
    pub unsafe fn soft_remove(&mut self, row: ArchetypeRow) {
        // move last row to the row, change entity record
        let max_row = self.next_row - 1;

        // special case, it's the last row
        if row == max_row {
            for component_storage in &mut self.components {
                unsafe {
                    component_storage.soft_remove_last_row();
                }
            }
        } else {
            for component_storage in &mut self.components {
                component_storage.copy_element_override(max_row, row);
                unsafe {
                    component_storage.soft_remove_last_row();
                }
            }
        }
        // substract last row
        self.next_row -= 1;
    }

    /// doesn't call the destructor except for one collumn where it does call it
    pub unsafe fn soft_remove_except_one(
        &mut self,
        row: ArchetypeRow,
        except_col: ArchetypeCollumn,
    ) {
        // move last row to the row, change entity record
        let max_row = self.next_row - 1;

        // special case, it's the last row
        if row == max_row {
            for (i, component_storage) in self.components.iter_mut().enumerate() {
                if i == except_col {
                    component_storage.remove_last_row();
                    continue;
                }
                unsafe {
                    component_storage.soft_remove_last_row();
                }
            }
        } else {
            for (i, component_storage) in self.components.iter_mut().enumerate() {
                if i == except_col {
                    component_storage.remove_last_row();
                    continue;
                }
                component_storage.copy_element_override(max_row, row);
                unsafe {
                    component_storage.soft_remove_last_row();
                }
            }
        }
        // substract last row
        self.next_row -= 1;
    }

    pub fn get_component<T: Component>(&mut self, entity: EntityId) -> Result<&T, &'static str> {
        match self.get_row(entity) {
            Some(row) => self.get_component_row::<T>(row),
            None => Err("Entity Not Found"),
        }
    }
    pub fn get_component_row<T: Component>(
        &mut self,
        row: ArchetypeRow,
    ) -> Result<&T, &'static str> {
        let col = self
            .signature
            .find::<T>()
            .ok_or("Didn't find component in archetype")?;
        Ok(&(self.components[col]
            .as_any_ref()
            .downcast_ref::<Vec<T>>()
            .ok_or("Col not equal T, Should not happen")?)[row])
    }

    pub fn get_component_mut<T: Component>(
        &mut self,
        entity: EntityId,
    ) -> Result<&mut T, &'static str> {
        match self.get_row(entity) {
            Some(row) => self.get_component_row_mut::<T>(row),
            None => Err("Entity Not Found"),
        }
    }
    pub fn get_component_row_mut<T: Component>(
        &mut self,
        row: ArchetypeRow,
    ) -> Result<&mut T, &'static str> {
        let col = self
            .signature
            .find::<T>()
            .ok_or("Didn't find component in archetype")?;
        Ok(&mut (self.components[col]
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .ok_or("Col not equal T, Should not happen")?)[row])
    }

    pub fn set_component<T: Component>(&mut self, entity: EntityId, component: T) {
        //TODO: add error handling
        let col = self
            .signature
            .find::<T>()
            .expect("Didn't find component in archetype");
        let row = self.entity_to_row[entity].expect("didn't find entity");
        self.components[col]
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .expect("Col was not of type T, should not happen")[row] = component;
    }

    pub fn get_row_count(&self) -> ArchetypeRow {
        self.next_row
    }

    pub fn set_edge_add<T: Component>(&mut self, archetype_id: ArchetypeId) {
        match self.edges.get_mut(&get_component_id::<T>()) {
            Some(_edge) => {}
            None => {
                self.edges
                    .insert(get_component_id::<T>(), ArchetypeEdge::Add(archetype_id));
            }
        };
    }

    pub fn set_edge_remove<T: Component>(&mut self, archetype_id: ArchetypeId) {
        match self.edges.get_mut(&get_component_id::<T>()) {
            Some(_edge) => {}
            None => {
                self.edges
                    .insert(get_component_id::<T>(), ArchetypeEdge::Remove(archetype_id));
            }
        };
    }
}

pub enum ArchetypeEdge {
    Add(ArchetypeId),
    Remove(ArchetypeId),
}

pub type ComponentToArchetypeMap = HashMap<ComponentId, Vec<ArchetypeId>>;
pub type SignatureToArchetypeMap = HashMap<ArchetypeSignature, ArchetypeId>;
