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
// //repeat_macro_with_argument!(impl_into_system, 16);
// repeat_macro_with_argument_without_0!(impl_into_system, 16);
//
// pub trait SystemParam<'a>{
//     fn retrieve(world: &'a mut World) -> Self;
// }
//


use std::marker::PhantomData;

use crate::{world::World};



pub type StoredSystem = Box<dyn System>;

pub trait System {
    fn run(&mut self, world: &World);
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
                    marker: Default::default()
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
                FnMut($(<$params as SystemParam>::Item<'b>),*)
        {

            fn run(&mut self, world: &World) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($params),*> (
                    mut f: impl FnMut($($params),*),
                    $($params: $params),*
                ) {
                    f($($params),*)
                }
                $(
                    let $params = $params::retrieve(world);
                )*

                call_inner(&mut self.f, $($params),*);
            }
        }
    };
}

repeat_macro_with_argument!(impl_system, 16);
repeat_macro_with_argument!(impl_into_system, 16);

pub struct FunctionSystem<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>
}


pub trait SystemParam{
    type Item<'w>;
    fn retrieve<'w>(world: &'w World) -> Self::Item<'w>;
}

