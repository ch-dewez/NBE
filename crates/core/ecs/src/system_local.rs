use std::{any::{Any, TypeId}, cell::{RefCell, RefMut}, collections::{HashMap, HashSet}};

use crate::{system::{SystemDependency, SystemParam}, world::{FromWorld, World}};

pub type LocalStorage = HashMap<SystemLocalId, Box<RefCell<dyn SystemLocal>>>;

pub trait SystemLocal: Any +'static {}
pub(crate) type SystemLocalId = TypeId;

pub fn get_local_id<T: SystemLocal>() -> SystemLocalId {
    TypeId::of::<T>()
}

pub struct Local<'a, T: SystemLocal> (RefMut<'a, T>);

impl<'a, T: SystemLocal> std::ops::Deref for Local<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: SystemLocal> std::ops::DerefMut for Local<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T:SystemLocal + FromWorld> SystemParam for Local<'a, T>{
    type Item<'world, 'local> = Local<'local, T>;
    type Cache = ();

    fn init(world:&World, local: &mut LocalStorage){
        local.insert(get_local_id::<T>(), Box::new(RefCell::new(T::from_world(world))));
    }

    fn retrieve<'w, 'l>(_world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>> 
    {

        let trait_ref = local.get(&get_local_id::<T>()).and_then(|el| el.try_borrow_mut().ok());
        trait_ref.map(|cell_ref|{
            Local(
                RefMut::map(cell_ref, |reference| {
                    (reference as &mut dyn Any).downcast_mut::<T>().unwrap()
                })
            )
        })
    }

    fn cache(_world: &World, _local: &LocalStorage) -> Option<Self::Cache> {
        Some(())
    }

    fn from_cache<'w, 'l>(_cache: &Self::Cache, world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>
    {
        Self::retrieve(world, local)
    }


    fn get_dependencies(_world: &World, _local: &LocalStorage) -> HashSet<SystemDependency> {
        HashSet::new()
    }

    fn get_dependencies_from_cache(world: &World, _cache: &Self::Cache, local: &LocalStorage) -> HashSet<SystemDependency>{
        Self::get_dependencies(world, local)

    }
}
