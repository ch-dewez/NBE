use std::any::TypeId;

use crate::{
    archetype::{AccessComponentError, AddSignatureError, Archetype, ArchetypeSignature, RemoveSignatureError},
    component_storage::ComponentStorageErased,
    entity::EntityId,
};

pub trait Component: 'static {}
pub type ComponentId = TypeId;

pub fn get_component_id<T: Component>() -> ComponentId {
    TypeId::of::<T>()
}


pub trait ComponentTupple {
    fn get_signature() -> ArchetypeSignature;
    fn add_signature(signature: &mut ArchetypeSignature) -> Result<Vec<usize>, AddSignatureError>;
    /// remove the comp id in the signature
    fn remove_signature(signature: &mut ArchetypeSignature) -> Result<(), RemoveSignatureError>;

    fn initialize_component(self, entity: EntityId, archetype: &mut Archetype) -> Result<(), AccessComponentError>;
    fn create_archetype() -> Archetype;

    /// add empty component storage to the container
    fn add_component_storage(signature: &mut ArchetypeSignature, container: &mut Vec<Box<dyn ComponentStorageErased>>)->Result<(), AddSignatureError>;

    fn component_id_match_any_component(id: ComponentId) -> bool;
}

macro_rules! impl_component_tupple {
    ($( $params:ident ),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($params:Component),*> ComponentTupple for ($($params),*)
        {
            fn get_signature() -> ArchetypeSignature{
                #[allow(unused_mut)]
                let mut signature: ArchetypeSignature = Default::default();
                // Safety: won't fail because there are no duplicates
                Self::add_signature(&mut signature).unwrap();
                signature
            }

            #[allow(unused_variables)]
            fn add_signature(signature: &mut ArchetypeSignature) -> Result<Vec<usize>, AddSignatureError>{
                Ok(vec![
                    $(
                        signature.add_sorted_id(get_component_id::<$params>())?
                    ),*
                ])
            }

            #[allow(unused_mut)]
            #[allow(unused_variables)]
            fn remove_signature(signature: &mut ArchetypeSignature) -> Result<(), RemoveSignatureError> {
                $(
                    let _ = signature.remove::<$params>()?;
                )*
                Ok(())
            }

            #[allow(unused_variables)]
            fn initialize_component(self, entity: EntityId, archetype: &mut Archetype)-> Result<(), AccessComponentError>
            {
                let ($($params),*) = self;
                $(
                    archetype.set_component::<$params>(entity, $params)?;
                )*

                Ok(())
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
                    row_to_entity: Default::default()
                }
            }

            #[allow(unused_mut)]
            #[allow(unused_variables)]
            fn add_component_storage(signature: &mut ArchetypeSignature, container: &mut Vec<Box<dyn ComponentStorageErased>>) -> Result<(), AddSignatureError>{
                let mut index: usize;
                $(
                    index = signature.add_sorted::<$params>()?;
                    let new_component_storage: Vec<$params> = Vec::new();
                    container.insert(index, Box::new(new_component_storage));
                )*

                Ok(())
            }

            #[allow(unused_variables)]
            fn component_id_match_any_component(id: ComponentId) -> bool{
                $(
                    if get_component_id::<$params>() == id{
                        return true;
                    }
                )*
                false
            }
        }
    };
}

repeat_macro_with_argument!(impl_component_tupple, 16);
