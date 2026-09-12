use std::{any::TypeId, cell::RefCell};

use macro_utils::repeat_macro_with_argument;

use crate::{
    archetype::{
        AccessComponentError, AddSignatureError, Archetype, ArchetypeSignature,
        RemoveSignatureError,
    },
    component_storage::ComponentStorageErased,
    entity::Entity,
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

    fn initialize_component(
        self,
        entity: Entity,
        archetype: &mut Archetype,
    ) -> Result<(), AccessComponentError>;
    fn create_archetype() -> Archetype;

    /// add empty component storage to the container
    fn add_component_storage(
        signature: &mut ArchetypeSignature,
        container: &mut Vec<Box<RefCell<dyn ComponentStorageErased>>>,
    ) -> Result<(), AddSignatureError>;

    fn component_id_match_any_component(id: ComponentId) -> bool;
}

impl<A: Component> ComponentTupple for A {
    fn get_signature() -> ArchetypeSignature {
        #[allow(unused_mut)]
        let mut signature: ArchetypeSignature = Default::default();
        let _ = signature.add_sorted::<A>();
        signature
    }

    #[allow(unused_variables)]
    fn add_signature(signature: &mut ArchetypeSignature) -> Result<Vec<usize>, AddSignatureError> {
        Ok(vec![signature.add_sorted::<A>()?])
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    fn remove_signature(signature: &mut ArchetypeSignature) -> Result<(), RemoveSignatureError> {
        let _ = signature.remove::<A>()?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn initialize_component(
        self,
        entity: Entity,
        archetype: &mut Archetype,
    ) -> Result<(), AccessComponentError> {
        archetype.set_component::<A>(entity, self)?;

        Ok(())
    }

    fn create_archetype() -> Archetype {
        let signature = Self::get_signature();
        let mut components: Vec<Box<RefCell<dyn ComponentStorageErased>>> =
            vec![Box::new(RefCell::new(Vec::<A>::new()))];

        components.sort_by(|a, b| {
            a.borrow()
                .get_component_id()
                .cmp(&b.borrow().get_component_id())
        });

        Archetype {
            signature,
            components,

            next_row: 0,
            entity_to_row: Default::default(),
            row_to_entity: Default::default(),
        }
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    fn add_component_storage(
        signature: &mut ArchetypeSignature,
        container: &mut Vec<Box<RefCell<dyn ComponentStorageErased>>>,
    ) -> Result<(), AddSignatureError> {
        let index = signature.add_sorted::<A>()?;
        let new_component_storage: Vec<A> = Vec::new();
        container.insert(index, Box::new(RefCell::new(new_component_storage)));

        Ok(())
    }

    #[allow(unused_variables)]
    fn component_id_match_any_component(id: ComponentId) -> bool {
        if get_component_id::<A>() == id {
            return true;
        }
        false
    }
}

macro_rules! impl_component_tupple_for_component_tupple_tupple {
    ($( $params:ident ),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($params:ComponentTupple,)*> ComponentTupple for ($($params,)*)
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
                // the mut is necessary
                #[allow(unused_mut)]
                let mut result: Vec<usize> = Vec::new();
                $(
                    result.extend($params::add_signature(signature)?);
                )*
                Ok(result)
            }

            #[allow(unused_mut)]
            #[allow(unused_variables)]
            fn remove_signature(signature: &mut ArchetypeSignature) -> Result<(), RemoveSignatureError> {
                $(
                    $params::remove_signature(signature)?;
                )*
                Ok(())
            }

            #[allow(unused_variables)]
            #[allow(unconditional_recursion)] // the recursion can go to the next other `ComponentTupple` implementation that does not recurse
            fn initialize_component(self, entity: Entity, archetype: &mut Archetype)-> Result<(), AccessComponentError>
            {
                let ($($params),*) = self;
                $(
                    $params.initialize_component(entity, archetype)?;
                )*

                Ok(())
            }

            fn create_archetype()-> Archetype{
                let mut signature = ArchetypeSignature::default();
                let mut components: Vec<Box<RefCell<dyn ComponentStorageErased>>> = Vec::new();
                let _ = Self::add_component_storage(&mut signature, &mut components).unwrap();

                // no need to sort, already sorted in add_component_storage
                //components.sort_by(|a, b| a.borrow().get_component_id().cmp(&b.borrow().get_component_id()));

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
            fn add_component_storage(signature: &mut ArchetypeSignature, container: &mut Vec<Box<RefCell<dyn ComponentStorageErased>>>) -> Result<(), AddSignatureError>{
                $(
                    $params::add_component_storage(signature, container)?;
                )*

                Ok(())
            }

            #[allow(unused_variables)]
            fn component_id_match_any_component(id: ComponentId) -> bool{
                $(
                    if $params::component_id_match_any_component(id) {
                        return true;
                    }
                )*
                false
            }
        }
    };
}

repeat_macro_with_argument!(impl_component_tupple_for_component_tupple_tupple, 32);
