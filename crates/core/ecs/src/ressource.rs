use std::{any::{Any, TypeId}, cell::{Ref, RefMut}, collections::HashSet};

use crate::{system::{SystemDependency, SystemParam}, world::World};


pub trait Ressource: Any + 'static {}
pub type RessourceId = TypeId;

pub fn get_ressource_id<T: Ressource>() -> RessourceId {
    TypeId::of::<T>()
}

#[allow(dead_code)]
pub struct Res<'a, T: Ressource>(Ref<'a, T>);
impl<'a, T: Ressource> std::ops::Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
pub struct ResMut<'a, T: Ressource>(RefMut<'a, T>);
impl<'a, T: Ressource> std::ops::Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T: Ressource> std::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Ressource> SystemParam for Res<'a, T> {
    type Item<'w> = Res<'w, T>;
    type Cache = ();

    fn retrieve<'w>(world: &'w World) -> Option<Self::Item<'w>> {
        world.get_ressource::<T>().map(|reference| Res(reference))
    }

    fn cache(_world: &World) -> Option<Self::Cache> {
        Some(())
    }

    fn from_cache<'w>(_cache: &Self::Cache, world: &'w World) -> Option<Self::Item<'w>> {
        Self::retrieve(world)
    }


    fn get_dependencies(_world: &World) -> HashSet<SystemDependency> {
        let mut result = HashSet::with_capacity(1);
        result.insert(SystemDependency::Ressource(get_ressource_id::<T>()));
        result
    }

    fn get_dependencies_from_cache(world: &World, _cache: &Self::Cache) -> HashSet<SystemDependency>{
        Self::get_dependencies(world)

    }
}

impl<'a, T: Ressource> SystemParam for ResMut<'a, T> {
    type Item<'w> = ResMut<'w, T>;
    type Cache = ();

    fn retrieve<'w>(world: &'w World) -> Option<Self::Item<'w>> {
        world.get_ressource_mut::<T>().map(|reference: RefMut<'w, T>| ResMut::<'w, T>(reference))
    }

    fn cache(_world: &World) -> Option<Self::Cache> {
        Some(())
    }

    fn from_cache<'w>(_cache: &Self::Cache, world: &'w World) -> Option<Self::Item<'w>> {
        Self::retrieve(world)
    }


    fn get_dependencies(_world: &World) -> HashSet<SystemDependency> {
        let mut result = HashSet::with_capacity(1);
        result.insert(SystemDependency::Ressource(get_ressource_id::<T>()));
        result
    }

    fn get_dependencies_from_cache(world: &World, _cache: &Self::Cache) -> HashSet<SystemDependency>{
        Self::get_dependencies(world)
    }
}
