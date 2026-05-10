use crate::{
    archetype::{Archetype, ArchetypeSignature},
    component::{Component, get_component_id},
    component_storage::ComponentStorageErased,
    entity::EntityId,
};

//fn somethign (somethi: Archetype, entity: EntityId);
pub trait SpawnableTupple {
    fn get_signature() -> ArchetypeSignature;
    fn initialize_component(&self, entity: EntityId, archetype: &mut Archetype);
    fn create_archetype() -> Archetype;
}

macro_rules! impl_component_tupple {
    ($( $params:ident ),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($params:Component),*> SpawnableTupple for ($($params),*)
        {
            fn get_signature() -> ArchetypeSignature{
                #[allow(unused_mut)]
                let mut signature: ArchetypeSignature = Default::default();
                $(
                    signature.add_sorted(get_component_id::<$params>());
                )*
                signature
            }

            #[allow(unused_variables)]
            fn initialize_component(&self, entity: EntityId, archetype: &mut Archetype){
                let ($($params),*) = self;
                $(
                    archetype.set_component::<$params>(entity, $params.clone());
                )*
            }

            fn create_archetype()-> Archetype{
                let signature = Self::get_signature();
                let mut components: Vec<Box<dyn ComponentStorageErased>> = vec![
                    $( Box::new(Vec::<$params>::new()) ),*
                ];

                components.sort_by(|a, b| a.get_component_id().cmp(&b.get_component_id()));

                Archetype {
                    signature,
                    components,

                    next_row: 0,
                    entity_to_row: Default::default(),

                    edges: Default::default()
                }

            }
        }
    };
}

repeat_macro_with_argument!(impl_component_tupple, 16);
