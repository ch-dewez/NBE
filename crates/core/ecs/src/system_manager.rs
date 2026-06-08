use std::sync::Arc;

//use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{system::{IntoSystem, System, SystemDependency}, world::World};

type SystemIndependence<'w> = Vec<Vec<(Arc<[SystemDependency]>, usize)>>;
pub struct SystemManager<'w>{
    cache_invalidated: bool,
    //system_independence_cache: SystemIndependence<'w>,
    systems: Vec<Box<dyn System + 'w + Send + Sync>>,
}

impl<'w> SystemManager<'w>{
    pub fn new() -> Self{
        SystemManager { 
            cache_invalidated: true, // starts at true so that it is initialized first frame
            //system_independence_cache: Default::default(),
            systems: Default::default()
        }

    }

    pub fn invalidate_cache(&mut self){
        self.cache_invalidated = true;
    }


    pub fn add_system<T, S: System + 'w + Send + Sync>(&mut self, system: impl IntoSystem<T, System = S>, world: &World) {
        let mut system = Box::new(system.into_system());
        system.init(world);
        self.systems.push(system);
    }

    #[allow(dead_code)]
    fn update_system_independence(system_independence: &mut SystemIndependence, new_dependencies: Arc<[SystemDependency]>, new_system: usize){
        let mut overlapping_groups: Vec<usize> = Vec::new();
        let mut found_match: bool = false;

        // Find first overlaping group
        for (index, system_independence_group) in system_independence.iter().enumerate(){
            'system: for system in system_independence_group {
                for dependency in system.0.iter(){
                    if new_dependencies.contains(dependency) {
                        overlapping_groups.push(index);
                        found_match = true;
                        break 'system;
                    }
                }
            }
        }

        // if there's none -> add it as a new gropu
        if !found_match {
            system_independence.push(vec![(new_dependencies, new_system)]);
            return;
        }

        // if there's overlapping, we need to merge all the groups
        while overlapping_groups.len() >= 2 {
            let group_index = overlapping_groups.pop().unwrap();
            println!("1. len {}", system_independence.len());
            let group = system_independence.swap_remove(group_index);
            println!("2. len {}", system_independence.len());
            system_independence[overlapping_groups[0]].extend(group);
            println!("3. len {}", system_independence.len());
        }

        // add the system to the group
        system_independence[overlapping_groups[0]].push((new_dependencies, new_system));
    }

    fn cache(&mut self, world: &World){
        for system in &mut self.systems{
            system.cache(world);
            system.calculate_dependencies_from_cache(world);
        }

        // let mut system_independence: SystemIndependence<'w> = vec![];
        //
        // for (index, system) in self.systems.iter().enumerate(){
        //     let dependencies = system.get_dependencies().expect("Fail to get system dependencies even though we calculated them. Should not be possible.");
        //     Self::update_system_independence(&mut system_independence, dependencies, index);
        // }
        //
        // self.system_independence_cache = system_independence;
    }

    pub fn step(&mut self, world: &World) {
        if self.cache_invalidated{
                self.cache(world);
        }
         // self
         //    .system_independence_cache
         //    .par_iter()
         //    .iter()
         //    .for_each(|group|{
         //        for (_, system_idx) in group {
         //            unsafe {
         //                let self_ref = &mut *(self as *const Self as *mut Self);
         //
         //                self_ref.systems[*system_idx].run(world);
         //            }
         //        }
         //    });
        self
            .systems
            .iter_mut()
            .for_each(|system| {
                system.run(world);
            });
    }
}

unsafe impl<'w> Sync for SystemManager<'w> {}
