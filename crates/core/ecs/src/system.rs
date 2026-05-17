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


use crate::{world::World};


pub type StoredSystem = Box<dyn System>;

pub trait System{
    fn run(&mut self, world: &World);
    fn cache(&mut self, world: &World);
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
                    cache: None
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
    cache: Option<Input::Cache>
}


pub trait SystemParam{
    type Item<'w>;
    type Cache;
    fn retrieve<'w>(world: &'w World) -> Self::Item<'w>;
    fn from_cache<'w>(cache: &Self::Cache, world: &'w World) -> Self::Item<'w>;
    fn cache(world: &World) -> Self::Cache;
}

