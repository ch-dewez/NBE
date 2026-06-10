use crate::{
    archetype::{AccessComponentError, Archetype, ArchetypeId, ArchetypeRow}, component::Component, entity::Entity, system::{SystemDependency, SystemParam}, system_local::LocalStorage, world::World
};
use std::{cell::{Ref, RefMut}, collections::HashSet, marker::PhantomData};

pub struct Query<'a, T: QueryData, F: QueryFilter = ()> {
    pub archetypes: Vec<&'a Archetype>,
    _phantom_data: PhantomData<(T, F)>,
}

impl<'a, T: QueryData, F: QueryFilter> Query<'a, T, F> {
    pub fn count(&self) -> usize{
        self
            .archetypes
            .iter()
            .map(|archetype| archetype.get_row_count())
            .sum()
    }
}

pub struct QueryCache{
    pub archetypes: Vec<ArchetypeId>
}

#[allow(dead_code)]
enum QueryDependency {
    Archetypes,
    Command, // no yet implemented
}

impl<'a, T: QueryData, F: QueryFilter> SystemParam for Query<'a, T, F> {
    type Item<'w, 'l> = Query<'w, T, F>;
    type Cache = QueryCache;

    fn init(_world:&World, _local: &mut LocalStorage) {}

    fn retrieve<'w, 'l>(world: &'w World, _local:&'l LocalStorage) -> Option<Self::Item<'w, 'l>> {
        let mut archetypes: Vec<&Archetype> = world.get_all_archetypes().iter().collect();

        T::filter(&mut archetypes);
        F::filter(&mut archetypes);

        if archetypes.is_empty(){
            return None;
        }

        Some(Query {
            archetypes,
            _phantom_data: Default::default(),
        })
    }

    fn cache(world: &World, _local: &LocalStorage) -> Option<Self::Cache> {
        let mut archetypes : Vec<ArchetypeId> = (0..world.archetypes.len()).collect();

        T::filter_id(&mut archetypes, world);
        F::filter_id(&mut archetypes, world);

        Some(QueryCache{
            archetypes
        })
    }

    fn from_cache<'w, 'l>(cache: &Self::Cache, world: &'w World, _local: &'l LocalStorage) -> Option<Self::Item<'w, 'l>> {
        let archetypes: Vec<&Archetype> = world.get_all_archetypes().iter().collect();
        Some(Query {
            archetypes: cache.archetypes.iter().map(|id| archetypes[*id]).collect(),
            _phantom_data: Default::default(),
        })
    }


    fn get_dependencies(world: &World, local: &LocalStorage) -> HashSet<SystemDependency> {
        let cache = Self::cache(world, local);
        if cache.is_none(){
            return HashSet::new();
        }
        let cache = cache.unwrap();
        Self::get_dependencies_from_cache(world, &cache, local)
    }

    fn get_dependencies_from_cache(_world: &World, cache: &Self::Cache, _local: &LocalStorage) -> HashSet<SystemDependency>{
        cache
            .archetypes
            .iter()
            .map(|id| SystemDependency::Archetype(*id))
            .collect()

    }
}

impl<'a, T: QueryData, F: QueryFilter> Clone for Query<'a, T, F> {
    fn clone(&self) -> Self {
        Self { archetypes: self.archetypes.clone(), _phantom_data: Default::default() }
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
        let archetype = self.query.archetypes[self.archetype_index];

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

    fn filter(archetypes: &mut Vec<&Archetype>);
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World);
    
    #[allow(private_interfaces)]
    fn get_dependency() -> QueryDependency;

    fn retrieve<'w>(
        archetype: &'w Archetype,
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

            fn filter(archetypes: &mut Vec<&Archetype>){
                $(
                    $params::filter(archetypes);
                )*
            }
            fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World){
                $(
                    $params::filter_id(archetypes, world);
                )*
            }

            fn retrieve<'w>(archetype: &'w Archetype, row: ArchetypeRow) -> Result<Self::Item<'w>, AccessComponentError>{
                #[allow(clippy::needless_question_mark)]
                Ok((
                $(
                    $params::retrieve(archetype, row)?
                    //$params::retrieve(archetype, row)?
                    ),*
                ))
            }

            #[allow(private_interfaces)]
            fn get_dependency() -> QueryDependency{
                QueryDependency::Archetypes
            }
        }
    };
}
repeat_macro_with_argument_without_0!(impl_query_tupple, 32);

pub trait QueryFilter {
    fn filter(archetypes: &mut Vec<&Archetype>);
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World);

    #[allow(private_interfaces)]
    fn get_dependency() -> QueryDependency;
}
macro_rules! impl_query_tupple {
    ($( $params:ident ),*) => {
        #[allow(unused_parens)]
        #[allow(unused_variables)]
        impl<$($params:QueryFilterArgument),*> QueryFilter for ($($params),*)
        {
            fn filter(archetypes: &mut Vec<&Archetype>){
                $(
                    $params::filter(archetypes);
                )*
            }

            fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World){
                $(
                    $params::filter_id(archetypes, world);
                )*
            }

            #[allow(private_interfaces)]
            fn get_dependency() -> QueryDependency{
                QueryDependency::Archetypes
            }
        }
    };
}
repeat_macro_with_argument!(impl_query_tupple, 32);


pub trait QueryArgument {
    type Item<'w>;

    fn filter(archetypes: &mut Vec<& Archetype>);
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World);
    fn retrieve<'w>(
        archetype: &'w Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError>;
}
impl<T: Component> QueryArgument for &T {
    type Item<'w> = Ref<'w, T>;

    fn filter(archetypes: &mut Vec<& Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World) {
        archetypes.retain(|id| world.archetypes[*id].signature.contains::<T>());
    }
    fn retrieve<'w>(
        archetype: &'w Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError> {
        archetype.get_component_row::<T>(row)
    }
}
impl<T: Component> QueryArgument for &mut T {
    type Item<'w> = RefMut<'w, T>;

    fn filter(archetypes: &mut Vec<& Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World) {
        archetypes.retain(|id| world.archetypes[*id].signature.contains::<T>());
    }

    fn retrieve<'w>(
        archetype: &'w Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError> {
        archetype.get_component_row_mut::<T>(row)
    }
}

pub struct EntityArgument;
impl QueryArgument for EntityArgument{
    type Item<'w> = Entity;
    fn filter(_archetypes: &mut Vec<&Archetype>) {}
    fn filter_id(_archetypes: &mut Vec<ArchetypeId>, _world: &World) {}

    fn retrieve<'w>(
        archetype: &'w Archetype,
        row: ArchetypeRow,
    ) -> Result<Self::Item<'w>, AccessComponentError> {
        Ok(archetype.get_entity(row).expect("Row out of bounds in system iteration"))
    }
}

pub trait QueryFilterArgument {
    fn filter(archetypes: &mut Vec<&Archetype>);
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World);
}
pub struct With<T: Component> {
    _phantom_data: PhantomData<T>,
}
impl<T: Component> QueryFilterArgument for With<T> {
    fn filter(archetypes: &mut Vec<&Archetype>) {
        archetypes.retain(|x| x.signature.contains::<T>());
    }
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World) {
        archetypes.retain(|id| world.archetypes[*id].signature.contains::<T>());
    }
}
pub struct Without<T: Component> {
    _phantom_data: PhantomData<T>,
}
impl<T: Component> QueryFilterArgument for Without<T> {
    fn filter(archetypes: &mut Vec<&Archetype>) {
        archetypes.retain(|x| !x.signature.contains::<T>());
    }
    fn filter_id(archetypes: &mut Vec<ArchetypeId>, world: &World) {
        archetypes.retain(|id| world.archetypes[*id].signature.contains::<T>());
    }
}
