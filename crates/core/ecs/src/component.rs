use std::any::TypeId;

pub trait Component: 'static + Clone {}
pub type ComponentId = TypeId;

pub fn get_component_id<T: Component>() -> ComponentId {
    TypeId::of::<T>()
}
