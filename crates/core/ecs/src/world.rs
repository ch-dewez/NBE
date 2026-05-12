use crate::{archetype::{Archetype, ArchetypeCollumn, ArchetypeId, ComponentToArchetypeMap, SignatureToArchetypeMap}, component::{Component, ComponentTupple}, entity::{Entity, EntityId, EntityVersion}, system::{IntoSystem, System}};


pub struct World {
    archetypes: Vec<Archetype>, // index 0 will be the one with no component

    next_entity_id: EntityId,
    // important to re-use entity to avoid gaps in data
    // store full Entity to know what is the next version
    free_entity_id: Vec<Entity>,

    entity_to_archetype: Vec<Option<(ArchetypeId, EntityVersion)>>, // entity (index in the vec) -> &Archetype, row index
    component_to_archetype: ComponentToArchetypeMap,
    signature_to_archetype: SignatureToArchetypeMap,

    systems: Vec<Box<dyn System>>
}

impl World {
    pub fn new() -> Self {
        let mut world: World = World{
            archetypes: vec![],
            next_entity_id: 0,
            free_entity_id: Default::default(),
            entity_to_archetype: Default::default(),
            component_to_archetype: Default::default(),
            signature_to_archetype: Default::default(),
            systems: Default::default()
        };

        world.create_archetype(Archetype::new_blanck());

        world
    }

    fn create_archetype(&mut self, archetype:Archetype) -> ArchetypeId{
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

    pub(crate) fn get_all_archetypes(&mut self) -> &mut Vec<Archetype>{
        &mut self.archetypes
    }

    fn set_entity_to_archetype_map(&mut self, entity: Entity, archetype: ArchetypeId){
        if self.entity_to_archetype.len() <= entity.id{
            self.entity_to_archetype.resize(entity.id+1, None);
        }
        self.entity_to_archetype[entity.id] = Some((archetype, entity.id));
    }

    pub fn spawn_entity<T: ComponentTupple>(&mut self, components: T) -> (Entity, &mut Self) {
        let entity: Entity;
        if let Some(mut free_entity) = self.free_entity_id.pop(){
            free_entity.version += 1;
            entity = free_entity;
        }else {
            entity =  Entity { id: self.next_entity_id, version: 0 };
        }
    
        let signature = T::get_signature();
        let archetype_id: ArchetypeId; 
        if let  Some(id) = self.signature_to_archetype.get(&signature){
            archetype_id = *id;
        }else {
            archetype_id = self.create_archetype(T::create_archetype());
        }

        // SAFETY: components initialized just after
        unsafe {
            self.archetypes[archetype_id].add_entity(entity.id);
        }
        components.initialize_component(entity.id, &mut self.archetypes[archetype_id]);

        self.set_entity_to_archetype_map(entity, archetype_id);
        self.next_entity_id += 1;

        (entity, self)
    }

    // pub fn create_entity_with_components(&mut self) -> Entity {
    //     if let Some(mut entity) = self.free_entity_id.pop(){
    //         entity.version += 1;
    //         self.set_entity_to_archetype(entity.id, 0);
    //         return entity;
    //     }
    //
    //     let entity = Entity { id: self.next_entity_id, version: 0 };
    //     self.next_entity_id += 1;
    //     entity
    // }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<&mut World, &'static str> {
        // TODO: Add error handling here
        self.free_entity_id.push(entity);
        let archtype_id = self.entity_to_archetype[entity.id].take().ok_or("No entity in map")?;
        if archtype_id.1 == entity.version{
            self.archetypes[archtype_id.0].remove_entity(entity.id);
        }

        Ok(self)
    }

    pub fn add_component<T:ComponentTupple>(&mut self, entity: Entity, component:T) -> Result<&mut Self, &'static str>{
        let record = self.entity_to_archetype[entity.id].as_mut().ok_or("Entity not in entity map")?;
        let current_archetype_id = record.0;

        // get the new archetype
        let mut signature = self.archetypes[current_archetype_id].signature.clone();
        T::add_signature(&mut signature)?;

        // check if archetype already exist
        let new_archetype_id = match self.signature_to_archetype.get(&signature){
            Some(id) => *id,
            // if it doesn't, create it
            None => {
                self.create_archetype(Archetype::new_from_archetype_add::<T>(&self.archetypes[current_archetype_id])?)
            }
        };

        // copy data to new archetype
        let new_row;
        // SAFETY: This will left uninitialized memory that will be filled with the memcpy just
        // after
        unsafe {
            new_row = self.archetypes[new_archetype_id].add_entity(entity.id);
        }
        let current_row = self.archetypes[current_archetype_id].get_row(entity.id).ok_or("Couldn't find entity in current entity -> row map ")?;

        for current_col in 0..self.archetypes[current_archetype_id].signature.0.len() {
            let component_id = self.archetypes[current_archetype_id].signature.0[current_col];
            let new_col = self.archetypes[new_archetype_id].signature.find_id(component_id).ok_or("Couldn't find col in signature")?;

            let size: usize = self.archetypes[current_archetype_id].components[current_col].get_size_of_element();
            let src: *const u8 = self.archetypes[current_archetype_id].components[current_col].get_pointer(current_row);
            let dst: *mut u8 = self.archetypes[new_archetype_id].components[new_col].get_pointer_mut(new_row);

            // SAFETY the dst is already allocated when we called add_entity
            unsafe {
                // pointer of u8 so size = count
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
        }

        T::initialize_component(component, entity.id, &mut self.archetypes[new_archetype_id]);
        
        // SAFE: we don't want to call the destructor because we moved the data
        unsafe {
            self.archetypes[current_archetype_id].soft_remove(current_row);
        }

        self.set_entity_to_archetype_map(entity, new_archetype_id);

        Ok(self)
    }

    pub fn remove_component<T:Component>(&mut self, entity: Entity) -> Result<&mut World, &'static str>{
        let record = self.entity_to_archetype[entity.id].as_mut().ok_or("Entity not in entity map")?;
        let current_archetype_id = record.0;

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
        // SAFETY: This will left uninitialized memory that will be filled with the memcpy just
        // after
        unsafe {
            new_row = self.archetypes[new_archetype_id].add_entity(entity.id);
        }
        let current_row = self.archetypes[current_archetype_id].get_row(entity.id).ok_or("Couldn't find entity in current entity -> row map ")?;

        let mut moved_cols: Vec<ArchetypeCollumn> = Vec::with_capacity(self.archetypes[new_row].signature.0.len());

        for new_col in 0..self.archetypes[new_archetype_id].signature.0.len() {
            let component_id = self.archetypes[new_archetype_id].signature.0[new_col];
            let current_col = self.archetypes[current_archetype_id].signature.find_id(component_id).ok_or("Couldn't find col in signature")?;
            moved_cols.push(current_col);

            let size: usize = self.archetypes[new_archetype_id].components[new_col].get_size_of_element();
            let src: *const u8 = self.archetypes[new_archetype_id].components[new_col].get_pointer(new_row);
            let dst: *mut u8 = self.archetypes[current_archetype_id].components[current_col].get_pointer_mut(current_row);

            // SAFETY the dst is already allocated when we called add_entity
            unsafe {
                // pointer of u8 so size = count
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
        }

        let max_current_col = self.archetypes[new_row].signature.0.len();

        let mut col_to_except: Vec<ArchetypeCollumn> = Vec::new();
        for col in 0..max_current_col{
            if !moved_cols.contains(&col){
                col_to_except.push(col);
            }
        }

        // SAFE: we don't want to call the destructor because we moved the data
        // But we want to call the destructor of the element that hasn't been copied, thus the
        // remove_except_one
        unsafe {
            self.archetypes[current_archetype_id].soft_remove_except(current_row, &col_to_except);
        }

        self.set_entity_to_archetype_map(entity, new_archetype_id);

        Ok(self)
    }

    pub fn add_system<T, S: System + 'static>(&mut self, system: impl IntoSystem<T, System = S> + 'static) -> &mut Self{
        self.systems.push(Box::new(system.into_system()));
        self
    }

    pub fn step(&mut self) {
        // SAFETY: System won't access other systems
        unsafe {
            for system in &mut *(&mut self.systems as *mut Vec<Box<dyn System>>){
                system.run(self);
            }
        }
    }
}

impl Default for World{
    fn default() -> Self {
        World::new()
    }
}
