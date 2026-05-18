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


use std::collections::HashSet;

use crate::{archetype::ArchetypeId, world::World};


pub type StoredSystem = Box<dyn System>;

pub trait System{
    fn run(&mut self, world: &World);
    fn cache(&mut self, world: &World);
    fn clear_cache(&mut self);

    fn get_dependencies(&self) -> &HashSet<SystemDependency>;
    fn calculate_dependencies(&mut self, world: &World);
    fn calculate_dependencies_from_cache(&mut self, world: &World);
    fn clear_dependencies(&mut self);
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum SystemDependency {
    Command,
    Archetype(ArchetypeId)
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
            for<'a, 'b> &'a mut F:
            FnMut($($params),*) +
            FnMut($(<$params as SystemParam>::Item<'b>),*)
        {
            type System = FunctionSystem<($($params),*), Self>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    cache: None,
                    dependencies: Default::default()
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
            for<'a, 'b> &'a mut F:
                FnMut($($params),*) +
                FnMut($(<$params as SystemParam>::Item<'b>),*),
        {

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
                        let $params = $params::from_cache($params, world);
                    )*
                    call_inner(&mut self.f, $($params),*);
                }else {
                    $(
                        let $params = $params::retrieve(world);
                    )*
                    call_inner(&mut self.f, $($params),*);
                }
            }

            fn cache(&mut self, world: &World)
            {
                self.cache = Some(
                    ($($params::cache(world)),*)
                );
            }

            fn clear_cache(&mut self){
                self.cache = None;
            }


            fn calculate_dependencies(&mut self, world: &World){
                let total_deps = std::collections::HashSet::new();

                $(
                    let $params = $params::get_dependencies(world);
                )*

                $(
                    let total_deps:HashSet<SystemDependency> = total_deps.union(&$params).copied().collect();
                )*
                self.dependencies = total_deps;
            }

            fn calculate_dependencies_from_cache(&mut self, world: &World){
                let total_deps = std::collections::HashSet::new();
                if self.cache.is_none(){
                    self.cache(world);
                }

                let ($($params),*) = self.cache.as_ref().unwrap();

                $(
                    let $params = $params::get_dependencies_from_cache(world, $params);
                )*

                $(
                    let total_deps:HashSet<SystemDependency> = total_deps.union(&$params).copied().collect();
                )*
                self.dependencies = total_deps;
            }

            fn get_dependencies(&self) -> &HashSet<SystemDependency>{
                &self.dependencies
            }

            fn clear_dependencies(&mut self){
                self.dependencies = Default::default();
            }
        }
    };
}

repeat_macro_with_argument!(impl_system, 32);
repeat_macro_with_argument!(impl_into_system, 32);

pub trait SystemParamTupple {
    type Item<'w>;
    type Cache;
}

macro_rules! impl_system_param_tupple {
    ($($params:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused)]
        impl<$($params: SystemParam),*> SystemParamTupple for ($($params),*){
            type Item<'w> = ($($params::Item<'w>),*);
            type Cache = ($($params::Cache),*);
        }
    };
}
repeat_macro_with_argument!(impl_system_param_tupple, 32);


pub struct FunctionSystem<Input: SystemParamTupple, F> {
    f: F,
    cache: Option<Input::Cache>,
    dependencies: HashSet<SystemDependency>
}


pub trait SystemParam{
    type Item<'w>;
    type Cache;
    fn retrieve<'w>(world: &'w World) -> Self::Item<'w>;

    fn from_cache<'w>(cache: &Self::Cache, world: &'w World) -> Self::Item<'w>;
    fn cache(world: &World) -> Self::Cache;

    fn get_dependencies(world: &World) -> HashSet<SystemDependency>;
    fn get_dependencies_from_cache(world: &World, cache: &Self::Cache) -> HashSet<SystemDependency>;
}

