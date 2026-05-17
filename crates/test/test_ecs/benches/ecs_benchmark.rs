use criterion::{criterion_group, criterion_main, Criterion};
use ecs::{component::Component, world::World, query::{Query, With, Without}};

macro_rules! define_components {
    ($($name:ident),*) => {
        $(
            #[derive(Debug, Default)]
            struct $name(u32);
            impl Component for $name {}
        )*
    };
}

define_components!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10,
    C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27, C28, C29, C30,
    C31, C32
);

fn spawn_varied_entity(world: &mut World, i: u32) -> ecs::entity::Entity {
    let (entity, _) = match i % 10 {
        0 => world.spawn_entity((C1(i), C2(i))),
        1 => world.spawn_entity((C3(i), C4(i), C5(i))),
        2 => world.spawn_entity((C6(i), C7(i), C8(i), C9(i))),
        3 => world.spawn_entity((C10(i), C11(i), C12(i), C13(i), C14(i))),
        4 => world.spawn_entity((C15(i), C16(i), C17(i), C18(i))),
        5 => world.spawn_entity((C19(i), C20(i), C21(i), C22(i), C23(i))),
        6 => world.spawn_entity((C24(i), C25(i), C26(i), C27(i))),
        7 => world.spawn_entity((C28(i), C29(i), C30(i), C31(i), C32(i))),
        8 => world.spawn_entity((C1(i), C5(i), C10(i), C15(i), C20(i), C25(i), C30(i))),
        _ => world.spawn_entity((
            C1(i), C2(i), C3(i), C4(i), C5(i), C6(i), C7(i), C8(i),
            C9(i), C10(i), C11(i), C12(i), C13(i), C14(i), C15(i), C16(i),
            C17(i), C18(i), C19(i), C20(i), C21(i), C22(i), C23(i), C24(i),
            C25(i), C26(i), C27(i), C28(i), C29(i), C30(i), C31(i), C32(i)
        ))
    };
    entity
}

fn system_1(query: Query<(&mut C1, &C2)>) {
    for (mut c1, c2) in query.into_iter() {
        c1.0 += c2.0;
    }
}

fn system_2(query: Query<(&C3, &mut C4), With<C5>>) {
    for (c3, mut c4) in query.into_iter() {
        c4.0 += c3.0;
    }
}

fn system_3(query: Query<(&C28, &C29), Without<C1>>) {
    for (c28, c29) in query.into_iter() {
        let _ = c28.0 + c29.0;
    }
}

fn system_4(query: Query<(
    &C1, &C2, &C3, &C4, &C5, &C6, &C7, &C8,
    &C9, &C10, &C11, &C12, &C13, &C14, &C15, &C16,
    &C17, &C18, &C19, &C20, &C21, &C22, &C23, &C24,
    &C25, &C26, &C27, &C28, &C29, &C30, &C31, &C32
)>) {
    for (c1, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, c32) in query.into_iter() {
        let _ = c1.0 + c32.0;
    }
}

fn bench_ecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ECS Stress Test");
    let entity_counts = [100, 10_000, 100_000];

    for &count in &entity_counts {
        group.bench_function(format!("spawn_entities_varied_{}", count), |b| {
            b.iter_with_setup(World::new, |mut world| {
                for i in 0..count {
                    spawn_varied_entity(&mut world, i);
                }
            });
        });

        group.bench_function(format!("add_component_to_varied_{}", count), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let mut entities = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        entities.push(spawn_varied_entity(&mut world, i));
                    }
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        let _ = world.add_component(entity, C32(42));
                    }
                },
            );
        });

        group.bench_function(format!("query_iteration_mixed_systems_{}", count), |b| {
            let mut world = World::new();
            for i in 0..count {
                spawn_varied_entity(&mut world, i);
            }
            
            world.add_system(system_1);
            world.add_system(system_2);
            world.add_system(system_3);
            world.add_system(system_4);

            b.iter(|| {
                world.step();
            });
        });

        group.bench_function(format!("remove_entities_varied_{}", count), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let mut entities = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        entities.push(spawn_varied_entity(&mut world, i));
                    }
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        world.remove_entity(entity).unwrap();
                    }
                },
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_ecs);
criterion_main!(benches);
