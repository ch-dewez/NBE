use criterion::{criterion_group, criterion_main, Criterion};
use ecs::{component::Component, world::World, query::Query};

macro_rules! define_components {
    ($($name:ident),*) => {
        $(
            struct $name(u32);
            impl Component for $name {}
        )*
    };
}

define_components!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10,
    C11, C12, C13, C14, C15, C16
);

fn bench_ecs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ECS Stress Test");
    let entity_counts = [10_000, 100_000];

    for &count in &entity_counts {
        group.bench_function(format!("spawn_entities_{}", count), |b| {
            b.iter_with_setup(World::new, |mut world| {
                for i in 0..count {
                    world.spawn_entity((C1(i), C2(i), C3(i)));
                }
            });
        });

        group.bench_function(format!("add_component_{}", count), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let mut entities = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let (entity, _) = world.spawn_entity((C1(i), C2(i), C3(i)));
                        entities.push(entity);
                    }
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        world.add_component(entity, C4(42)).unwrap();
                    }
                },
            );
        });

        group.bench_function(format!("remove_component_{}", count), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let mut entities = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let (entity, _) = world.spawn_entity((C1(i), C2(i), C3(i), C4(i)));
                        entities.push(entity);
                    }
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        world.remove_component::<C4>(entity).unwrap();
                    }
                },
            );
        });

        group.bench_function(format!("query_iteration_{}", count), |b| {
            let mut world = World::new();
            for i in 0..count {
                world.spawn_entity((C1(i), C2(i), C3(i)));
            }
            
            fn bench_system(query: Query<(&C1, &C2, &C3)>) {
                for (c1, c2, c3) in query.into_iter() {
                    let _ = c1.0 + c2.0 + c3.0;
                }
            }
            world.add_system(bench_system);

            b.iter(|| {
                world.step();
            });
        });

        group.bench_function(format!("remove_entities_{}", count), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let mut entities = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let (entity, _) = world.spawn_entity((C1(i), C2(i), C3(i)));
                        entities.push(entity);
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
