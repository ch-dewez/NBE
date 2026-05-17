use std::any::{Any, TypeId};
use crate::{archetype::ArchetypeRow, component::{Component, ComponentId, get_component_id}};

pub trait ComponentStorageErased: Any {
    fn as_any_ref(&self) -> & dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_component_id(&self) -> ComponentId;

    fn swap_remove_erased(&mut self, index: usize);
    /// doesn't call the destructor
    unsafe fn soft_swap_remove_erased(&mut self, index: usize);

    fn get_size_of_element(&self) -> usize;
    fn get_pointer(&self, index: usize) -> *const u8;
    fn get_pointer_mut(&mut self, index: usize) -> *mut u8;

    fn new_vec_of_same_type(&self) -> Box<dyn ComponentStorageErased>;

    fn copy_element_from_another_storage(&mut self, source_row: ArchetypeRow, dst_row: ArchetypeRow, other: &dyn ComponentStorageErased);
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

    /// memcpy to dst_row,
    /// if dst_row == len -> first reserve and set_len, then memcpy
    fn copy_element_from_another_storage(&mut self, source_row: ArchetypeRow, dst_row: ArchetypeRow, other: & dyn ComponentStorageErased){
        let size: usize = self.get_size_of_element();
        let src: *const u8 = other.get_pointer(source_row);
        let dst;

        if dst_row < self.len(){
            dst = self.get_pointer_mut(dst_row);
        }else if dst_row == self.len(){
            if self.capacity() < self.len() + 1 {
                self.reserve(1);
            }
            // SAFETY: We have the capacity and the element will be initiliazed in the
            // following copy_non_overlaping
            unsafe {
                self.set_len(self.len() + 1);
            }
            dst = self.get_pointer_mut(dst_row);
        }else {
            panic!("Trying to copy data to this component storage but dst row is out of bounds (and not next row)");
        }

        debug_assert_eq!(other.type_id(), TypeId::of::<Vec<T>>());

        // SAFETY: dst is allocated, the size has been calculated, we asserted that the type are
        // the same 
        unsafe {
            // pointer of u8 so size = count
            std::ptr::copy_nonoverlapping(src, dst, size);
        }
    }
}

