use std::cell::RefCell;

use crate::{system::{IntoSystem, System}, world::World};

pub struct SystemManager<'w>{
    cache_invalidated: bool,
    systems: Vec<Box<RefCell<dyn System + 'w>>>
}

impl<'w> SystemManager<'w>{
    pub fn new() -> Self{
        SystemManager { cache_invalidated: false, systems: Default::default() }

    }

    pub fn invalidate_cache(&mut self){
        self.cache_invalidated = true;
    }


    pub fn add_system<T, S: System + 'w>(&mut self, system: impl IntoSystem<T, System = S>) {
        self.systems.push(Box::new(RefCell::new(system.into_system())));
    }

    

    pub fn step(&self, world: &World) {
        if self.cache_invalidated{
            for system in &self.systems{
                system.borrow_mut().cache(world);
            }
        }
        for system in &self.systems{

            system.borrow_mut().run(world);
        }

    }
}
