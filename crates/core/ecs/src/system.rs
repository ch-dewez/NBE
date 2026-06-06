// use crate::world::World;
//
// pub trait System {
//     fn run(&self, world: &mut World);
// }
// macro_rules! impl_into_system {
//     ($( $params:ident ),*) => {
//         #[allow(unused_variables)]
//         impl<$($params:for<'a>SystemParam<'a>),*> System for fn ($($params),*) -> ()
//         {
//             fn run(&self, world: & mut World)
//             {
//                 self($($params::retrieve(world)),*);
//             }
//         }
//     }
// }
// //repeat_macro_with_argument!(impl_into_system, 32);
// repeat_macro_with_argument_without_0!(impl_into_system, 32);
//
// pub trait SystemParam<'a>{
//     fn retrieve(world: &'a mut World) -> Self;
// }
//


use std::{collections::{HashMap, HashSet}, sync::Arc};

use crate::{archetype::ArchetypeId, ressource::RessourceId, system_local::LocalStorage , world::World};


pub type StoredSystem = Box<dyn System>;

pub trait System{
    fn init(&mut self, world: &World);

    fn run(&mut self, world: &World);
    fn cache(&mut self, world: &World);
    fn clear_cache(&mut self);

    fn get_dependencies(&self) -> Option<Arc<[SystemDependency]>>;
    fn calculate_dependencies(&mut self, world: &World);
    fn calculate_dependencies_from_cache(&mut self, world: &World);
    fn clear_dependencies(&mut self);
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum SystemDependency {
    Command,
    Archetype(ArchetypeId),
    Ressource(RessourceId)
}

pub trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

macro_rules! impl_into_system {
    ($($params:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
        impl<F: FnMut($($params),*), $($params: SystemParam + 'static),*> IntoSystem<($($params),*)> for F 
        where
            for<'a, 'b, 'c> &'a mut F:
            FnMut($($params),*) +
            FnMut($(<$params as SystemParam>::Item<'b, 'c>),*)
        {
            type System = FunctionSystem<($($params),*), Self>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    cache: None,
                    dependencies: Default::default(),

                    local: HashMap::new()

                }
            }
        }
    };
}


macro_rules! impl_system  {
    ($($params:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
    impl<F: FnMut($($params),*), $($params: SystemParam + 'static),*> System for FunctionSystem<($($params),*), F> 
        where
            for<'a, 'b, 'c> &'a mut F:
                FnMut($($params),*) +
                FnMut($(<$params as SystemParam>::Item<'b, 'c>),*),
        {
            fn init(&mut self, world: &World){
                $(
                    $params::init(world, &mut self.local);
                )*
            }

            fn run(&mut self, world: &World) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($params),*> (
                    mut f: impl FnMut($($params),*),
                    $($params: $params),*
                ) {
                    f($($params),*)
                }

                if let Some(cache) = &self.cache.as_ref(){
                    let ($($params),*) = cache;
                    $(
                        let $params = $params::from_cache($params, world, &self.local);
                        if $params.is_none(){
                            return;
                        }
                        let $params = $params.unwrap();
                    )*
                    call_inner(&mut self.f, $($params),*);
                }else {
                    $(
                        let $params = $params::retrieve(world, &self.local);
                        if $params.is_none(){
                            return;
                        }
                        let $params = $params.unwrap();
                    )*
                    call_inner(&mut self.f, $($params),*);
                }
            }

            fn cache(&mut self, world: &World)
            {
                $(
                    let $params = $params::cache(world, &self.local);
                    if $params.is_none(){
                        self.cache = None;
                        return;
                    }
                    let $params = $params.unwrap();
                )*
                self.cache = Some(($($params),*));
            }

            fn clear_cache(&mut self){
                self.cache = None;
            }


            fn calculate_dependencies(&mut self, world: &World){
                let total_deps = std::collections::HashSet::new();

                $(
                    let $params = $params::get_dependencies(world, &self.local);
                )*

                $(
                    let total_deps:HashSet<SystemDependency> = total_deps.union(&$params).copied().collect();
                )*
                let deps_vec: Vec<SystemDependency> = total_deps.into_iter().collect();
                self.dependencies = Some(Arc::from(deps_vec));
            }

            fn calculate_dependencies_from_cache(&mut self, world: &World){
                let total_deps = std::collections::HashSet::new();
                if self.cache.is_none(){
                    self.cache(world);
                }

                let ($($params),*) = self.cache.as_ref().unwrap();

                $(
                    let $params = $params::get_dependencies_from_cache(world, $params, &self.local);
                )*

                $(
                    let total_deps:HashSet<SystemDependency> = total_deps.union(&$params).copied().collect();
                )*
                let deps_vec: Vec<SystemDependency> = total_deps.into_iter().collect();
                self.dependencies = Some(Arc::from(deps_vec));
            }

            fn get_dependencies(&self) -> Option<Arc<[SystemDependency]>>{
                self.dependencies.clone()
            }

            fn clear_dependencies(&mut self){
                self.dependencies = None;
            }
        }
    };
}

repeat_macro_with_argument!(impl_system, 32);
repeat_macro_with_argument!(impl_into_system, 32);

pub trait SystemParamTupple {
    type Item<'w, 'l>;
    type Cache;
}

macro_rules! impl_system_param_tupple {
    ($($params:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
        impl<$($params: SystemParam),*> SystemParamTupple for ($($params),*){
            type Item<'w, 'l> = ($($params::Item<'w, 'l>),*);
            type Cache = ($($params::Cache),*);
        }
    };
}
repeat_macro_with_argument!(impl_system_param_tupple, 32);


pub struct FunctionSystem<Input: SystemParamTupple, F> {
    f: F,
    cache: Option<Input::Cache>,
    dependencies: Option<Arc<[SystemDependency]>>,

    local: LocalStorage
}
unsafe impl<Input: SystemParamTupple, F> Send for FunctionSystem<Input, F>{}
unsafe impl<Input: SystemParamTupple, F> Sync for FunctionSystem<Input, F>{}

pub trait SystemParam{
    type Item<'w, 'l>;
    type Cache;

    fn init(world:&World, local: &mut LocalStorage);

    fn retrieve<'w, 'l>(world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>;

    fn from_cache<'w, 'l>(cache: &Self::Cache, world: &'w World, local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>>;
    fn cache(world: &World, local: &LocalStorage) -> Option<Self::Cache>;

    fn get_dependencies(world: &World, local: &LocalStorage) -> HashSet<SystemDependency>;
    fn get_dependencies_from_cache(world: &World, cache: &Self::Cache, local: &LocalStorage) -> HashSet<SystemDependency>;
}

