use ecs::{component::Component, world::World, query::Query};
use std::time::Instant;

macro_rules! define_components {
    ($($name:ident),*) => {
        $(
            #[derive(Default, Debug)]
            struct $name(u32);
            impl Component for $name {}
        )*
    };
}

define_components!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10
    // C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    // C21, C22, C23, C24, C25, C26, C27, C28, C29, C30,
    // C31, C32
);

fn system_heavy_read(query: Query<(&C1, &C2, &C3, &C4)>) {
    let mut _sum: u32 = 0;
    for (c1, c2, c3, c4) in query.into_iter() {
        _sum = _sum.wrapping_add(c1.0).wrapping_add(c2.0).wrapping_sub(c3.0).wrapping_sub(c4.0);
    }
}

fn system_heavy_write(query: Query<(&mut C1, &C2)>) {
    for (mut c1, c2) in query.into_iter() {
        c1.0 += c2.0;
    }
}

fn main() {
    let mut world = World::new();
    let entity_count = 100_000;
    
    println!("--- ECS Stress Test ---");
    println!("Entities: {}", entity_count);

    // 1. Spawning
    let start = Instant::now();
    let mut entities = Vec::with_capacity(entity_count);
    for i in 0..entity_count {
        let (entity, _) = world.spawn_entity((C1(i as u32), C2(i as u32), C3(i as u32)));
        entities.push(entity);
    }
    println!("Spawning {} entities with 3 components: {:?}", entity_count, start.elapsed());

    // 2. Add Component
    let start = Instant::now();
    for &entity in &entities {
        world.add_component(entity, C4(42)).unwrap();
    }
    println!("Adding C4 to all entities: {:?}", start.elapsed());

    // 3. System execution
    world.add_system(system_heavy_read);
    world.add_system(system_heavy_write);
    
    let start = Instant::now();
    for _ in 0..100 {
        world.step();
    }
    println!("Running 100 steps (2 systems): {:?}", start.elapsed());

    // 4. Mass Component swap
    let start = Instant::now();
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            world.add_component(entity, (C5(i as u32), C6(i as u32))).unwrap();
        } else {
            world.remove_component::<C1>(entity).unwrap();
        }
    }
    println!("Mass component add/remove (50% add C5,C6 / 50% remove C1): {:?}", start.elapsed());

    // 5. Query with many components
    println!("Running query with 10 components...");
    let start = Instant::now();
    let mut _count = 0;
    // Note: We need to spawn some entities with 10 components first to match this query
    for i in 0..1000 {
         world.spawn_entity((
            C1(i), C2(i), C3(i), C4(i), C5(i), 
            C6(i), C7(i), C8(i), C9(i), C10(i)
        ));
    }
    
    // Manual query execution to benchmark it
    #[allow(clippy::type_complexity)]
    fn complex_query_system(query: Query<(&C1, &C2, &C3, &C4, &C5, &C6, &C7, &C8, &C9, &C10)>) {
        let mut _sum = 0;
        for (c1, c2, c3, c4, c5, c6, c7, c8, c9, c10) in query.into_iter() {
            _sum += c1.0 + c2.0 + c3.0 + c4.0 + c5.0 + c6.0 + c7.0 + c8.0 + c9.0 + c10.0;
        }
    }
    world.add_system(complex_query_system);
    world.step(); // One step to include the new system
    println!("Complex query step: {:?}", start.elapsed());

    // 6. Removal
    let start = Instant::now();
    for entity in entities {
        let _ = world.remove_entity(entity);
    }
    println!("Removing {} entities: {:?}", entity_count, start.elapsed());
    
    println!("Stress test complete.");
}
