use std::any::Any;
use crate::component::{Component, ComponentId, get_component_id};

pub trait ComponentStorageErased: Any {
    fn as_any_ref(&self) -> & dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_component_id(&self) -> ComponentId;

    fn swap_remove_erased(&mut self, index: usize);
    /// doesn't call the destructor
    unsafe fn soft_swap_remove_erased(&mut self, index: usize);

    /// push raw bytes 
    /// This will leave uninitialized/corrupt memory
    /// use case: have an index, just to be initialized just after
    unsafe fn push_uninitialized(&mut self);

    fn get_size_of_element(&self) -> usize;
    fn get_pointer(&self, index: usize) -> *const u8;
    fn get_pointer_mut(&mut self, index: usize) -> *mut u8;

    fn new_vec_of_same_type(&self) -> Box<dyn ComponentStorageErased>;
}

impl<T: Component> ComponentStorageErased for Vec<T>{
    fn as_any_ref(&self) -> & dyn Any{
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any{
        self
    }

    fn get_component_id(&self) -> ComponentId {
        get_component_id::<T>()
    }

    fn swap_remove_erased(&mut self, index: usize){
        self.swap_remove(index);
    }

    unsafe fn soft_swap_remove_erased(&mut self, index: usize){
        let last_index = self.len()-1;
        self.swap(index, last_index);
        unsafe{
            self.set_len(self.len()-1);
        }
    }

    unsafe fn push_uninitialized(&mut self){
        let raw_bytes = vec![0;self.get_size_of_element()];
        let element: T = unsafe {
            std::ptr::read_unaligned(raw_bytes.as_ptr() as *const T)
        };
        
        self.push(element);
    }

    fn get_size_of_element(&self) -> usize {
        size_of::<T>()
    }

    fn get_pointer(&self, index: usize) -> *const u8 {
        (&raw const self[index]) as *const u8
    }
    fn get_pointer_mut(&mut self, index: usize) -> *mut u8 {
        (&raw mut self[index]) as *mut u8
    }

    fn new_vec_of_same_type(&self) -> Box<dyn ComponentStorageErased>{
        let vec_of_type: Vec<T> = Vec::new();
        Box::new(vec_of_type)
    }
}

