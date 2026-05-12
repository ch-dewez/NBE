use crate::{
    archetype::{Archetype, ArchetypeRow, AccessComponentError},
    component::Component,
    system::SystemParam,
    world::World,
};
use std::marker::PhantomData;

pub struct Query<'a, T: QueryData, F: QueryFilter = ()> {
    pub archetypes: Vec<&'a mut Archetype>,
    _phantom_data: PhantomData<(T, F)>,
}

impl<'a, T: QueryData, F: QueryFilter> SystemParam for Query<'a, T, F> {
    type Item<'w> = Query<'w, T, F>;
    fn retrieve<'w>(world: &'w mut World) -> Self::Item<'w> {
        let mut archetypes: Vec<&mut Archetype> = world.get_all_archetypes().iter_mut().collect();

        T::filter(&mut archetypes);
        F::filter(&mut archetypes);

        Query {
            archetypes,
            _phantom_data: Default::default(),
        }
    }
}

impl<'a, T: QueryData, F: QueryFilter> IntoIterator for Query<'a, T, F> {
    type Item = T::Item<'a>;
    type IntoIter = QueryIter<'a, T, F>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            query: self,
            archetype_index: 0,
            row_index: 0,
        }
    }
}

pub struct QueryIter<'a, T: QueryData, F: QueryFilter> {
    query: Query<'a, T, F>,
    archetype_index: usize,
    row_index: ArchetypeRow,
}

impl<'a, T: QueryData, F: QueryFilter> Iterator for QueryIter<'a, T, F> {
    type Item = T::Item<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.query.archetypes.len() <= self.archetype_index {
            return None;
        }
        let mut row_count: usize;
        loop {
            row_count = self.query.archetypes[self.archetype_index].get_row_count();
            if self.row_index < row_count {
                break;
            }
            self.row_index = 0;
            self.archetype_index += 1;
            if self.query.archetypes.len() <= self.archetype_index {
                return None;
            }
        }

        //let archetype: &mut Archetype = self.query.archetypes[self.archetype_index];
        let archetype = unsafe {
            let archetype_ptr =
                (self.query.archetypes[self.archetype_index] as &mut Archetype) as *mut Archetype;
            &mut *archetype_ptr
        };

        let value = match T::retrieve(archetype, self.row_index) {
            Ok(result) => Some(result),
            Err(err) => {
                println!("Err {}", err);
                None
            },
        };

        self.row_index += 1;
        value
    }
}

pub trait QueryData {
    type Item<'a>;

    fn filter(archetypes: &mut Vec<&mut Archetype>);
    fn retrieve<'w>(
        archetype: &'w mut Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError>;
}
macro_rules! impl_query_tupple {
    ($( $params:ident ),*) => {
        #[allow(unused_parens)]
        //impl<'a, $($params:for<'b> QueryArgument<'b>),*> QueryData<'a> for ($($params),*)
        impl<$($params: QueryArgument),*> QueryData for ($($params),*)
        {
            type Item<'w> = ($($params::Item<'w>),*);

            fn filter(archetypes: &mut Vec<&mut Archetype>){
                $(
                    $params::filter(archetypes);
                )*
            }

        fn retrieve<'w>(archetype: &'w mut Archetype, row: ArchetypeRow) -> Result<Self::Item<'w>, AccessComponentError>{
                let archetype_ptr = archetype as * mut Archetype;

                // TODO: Maybe there's a way to remove this unsafe
                #[allow(clippy::needless_question_mark)]
                unsafe {
                    Ok((
                    $(
                        $params::retrieve(& mut *archetype_ptr, row)?
                        //$params::retrieve(archetype, row)?
                    ),*
                    ))
                }
            }
        }
    };
}
repeat_macro_with_argument_without_0!(impl_query_tupple, 16);

pub trait QueryFilter {
    fn filter(archetypes: &mut Vec<&mut Archetype>);
}
macro_rules! impl_query_tupple {
    ($( $params:ident ),*) => {
        #[allow(unused_parens)]
        #[allow(unused_variables)]
        impl<$($params:QueryFilterArgument),*> QueryFilter for ($($params),*)
        {
            fn filter(archetypes: &mut Vec<&mut Archetype>){
                $(
                    $params::filter(archetypes);
                )*
            }
        }
    };
}
repeat_macro_with_argument!(impl_query_tupple, 16);

pub trait QueryArgument {
    type Item<'w>;

    fn filter(archetypes: &mut Vec<&mut Archetype>);
    fn retrieve<'w>(
        archetype: &'w mut Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError>;
}
impl<T: Component> QueryArgument for &T {
    type Item<'w> = &'w T;

    fn filter(archetypes: &mut Vec<&mut Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }
    fn retrieve<'w>(
        archetype: &'w mut Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError> {
        //let archetype_ref = unsafe{};
        archetype.get_component_row::<T>(row)
    }
}
impl<T: Component> QueryArgument for &mut T {
    type Item<'w> = &'w mut T;

    fn filter(archetypes: &mut Vec<&mut Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }

    fn retrieve<'w>(
        archetype: &'w mut Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError> {
        //let archetype_ref = unsafe{};
        archetype.get_component_row_mut::<T>(row)
    }
}

pub trait QueryFilterArgument {
    fn filter(archetypes: &mut Vec<&mut Archetype>);
}
pub struct With<T: Component> {
    _phantom_data: PhantomData<T>,
}
impl<T: Component> QueryFilterArgument for With<T> {
    fn filter(archetypes: &mut Vec<&mut Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }
}
pub struct Without<T: Component> {
    _phantom_data: PhantomData<T>,
}
impl<T: Component> QueryFilterArgument for Without<T> {
    fn filter(archetypes: &mut Vec<&mut Archetype>) {
        archetypes.retain(|x| !x.signature.contains::<T>());
    }
}
