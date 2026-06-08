use std::{cell::Ref, collections::HashMap};

use crate::{query::EntityArgument, system_local::{Local, SystemLocal}, world::FromWorld};

//
//
//
// IMPORTANT: The tests were writtent by A.I.
//
//
//
use super::{
    component::Component, 
    world::World, 
    query::{Query, With, Without},
    system::SystemParam,
    ressource::{Ressource, Res, ResMut}
};

// --- Components ---
#[derive(Clone, PartialEq, Debug)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, PartialEq, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[derive(Clone, PartialEq, Debug)]
struct Health(u32);
impl Component for Health {}

#[derive(Clone, PartialEq, Debug)]
struct Name(String);
impl Component for Name {}

// --- Resources ---
#[derive(Default)]
struct Score(u32);
impl Ressource for Score {}

struct Settings {
    difficulty: f32,
}
impl Ressource for Settings {}

// --- Systems ---
fn movement_system(query: Query<(& mut Position, & Velocity)>) {
    for (mut position, velocity) in query.into_iter() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn health_system(query: Query<&mut Health, With<Position>>) {
    for mut health in query.into_iter() {
        health.0 = health.0.saturating_sub(1); // Reduce health, but not below 0
    }
}

fn name_logger_system(query: Query<(&Name, &Position), Without<Health>>) {
    for (name, position) in query.into_iter() {
        // In a real scenario, this might log to console or a file
        println!("Entity {} is at ({}, {})", name.0, position.x, position.y);
    }
}

fn resource_system(mut score: ResMut<Score>, settings: Res<Settings>) {
    score.0 += (10.0 * settings.difficulty) as u32;
}

fn combined_system(query: Query<&Position>, score: Res<Score>) {
    for _pos in query.into_iter() {
        // Just use the resource to ensure it's accessible
        let _s = score.0;
    }
}

// --- Tests ---
#[test]
fn test_spawn_entity_no_components() {
    let mut world = World::new();
    let entity = world.spawn_entity(()).0; // Spawning with no components
    assert_eq!(entity.id, 0);
    assert_eq!(entity.version, 0);
}

#[test]
fn test_spawn_entity_with_components() {
    let mut world = World::new();
    let entity = world.spawn_entity((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 })).0;
    assert_eq!(entity.id, 0);
    assert_eq!(entity.version, 0);

    // Verify components are present by trying to query
    let query = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new()).unwrap();
    let mut iter = query.into_iter();
    let (pos, vel) = iter.next().expect("Should have one entity with Position and Velocity");
    assert_eq!(*pos, Position { x: 0.0, y: 0.0 });
    assert_eq!(*vel, Velocity { x: 1.0, y: 1.0 });
    assert!(iter.next().is_none());
}

#[test]
fn test_remove_entity() {
    let mut world = World::new();
    let entity1 = world.spawn_entity(Position { x: 0.0, y: 1.0 }).0;
    let _entity2 = world.spawn_entity(Position { x: 1.0, y: 1.0 }).0;

    assert!(world.remove_entity(entity1).is_ok());

    // Entity2 should still exist
    let query = Query::<&Position>::retrieve(&world, &HashMap::new()).unwrap();
    let mut iter = query.into_iter();
    let pos = iter.next().expect("Entity2 should still be present");
    assert_eq!(*pos, Position { x: 1.0, y: 1.0 });
    assert!(iter.next().is_none());
    std::mem::drop(pos);

    // Spawning a new entity should reuse the ID of entity1
    let entity3 = world.spawn_entity(Health(10)).0;
    assert_eq!(entity3.id, entity1.id); // Should reuse id 0
    assert_eq!(entity3.version, entity1.version + 1); // Version should increment
}

#[test]
fn test_add_component() {
    let mut world = World::new();
    let entity = world.spawn_entity(Position { x: 0.0, y: 0.0 }).0;

    // Initially, no Velocity component
    let query_vel = Query::<&Velocity>::retrieve(&world, &HashMap::new());
    assert!(query_vel.into_iter().flatten().next().is_none());

    // Add Velocity component
    world.add_component(entity, Velocity { x: 1.0, y: 1.0 }).unwrap();

    // Now, entity should have Position and Velocity
    let query_pos_vel = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new()).unwrap();
    let (pos, vel) = query_pos_vel.into_iter().next().expect("Entity should have Position and Velocity");
    assert_eq!(*pos, Position { x: 0.0, y: 0.0 });
    assert_eq!(*vel, Velocity { x: 1.0, y: 1.0 });
    std::mem::drop(pos);
    std::mem::drop(vel);

    // Ensure it's not present in archetypes without Velocity
    let query_pos_only = Query::<&Position, Without<Velocity>>::retrieve(&world, &HashMap::new()).unwrap();
    assert!(query_pos_only.into_iter().next().is_none());
}

#[test]
fn test_remove_component() {
    let mut world = World::new();
    let entity = world.spawn_entity((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 })).0;

    // Initially, entity has both Position and Velocity
    let query_pos_vel = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new()).unwrap();
    assert!(query_pos_vel.into_iter().next().is_some());

    // Remove Velocity component
    world.remove_component::<Velocity>(entity).unwrap();

    // Now, entity should only have Position
    let query_pos_only = Query::<&Position>::retrieve(&world, &HashMap::new()).unwrap();
    let mut query_pos_only_iter = query_pos_only.into_iter();
    let pos = query_pos_only_iter.next().expect("Entity should only have Position");
    assert_eq!(*pos, Position { x: 0.0, y: 0.0 });
    assert!(query_pos_only_iter.next().is_none());
    std::mem::drop(pos);

    // Should not be in queries for Velocity
    let query_vel_only = Query::<&Velocity>::retrieve(&world, &HashMap::new());
    assert!(query_vel_only.into_iter().flatten().next().is_none());
}

#[test]
fn test_add_remove_sequence() {
    let mut world = World::new();
    let entity = world.spawn_entity(Position { x: 0.0, y: 0.0 }).0;

    // Add Velocity
    world.add_component(entity, Velocity { x: 1.0, y: 1.0 }).unwrap();
    let query1 = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new()).unwrap();
    assert!(query1.into_iter().next().is_some());

    // Remove Velocity
    world.remove_component::<Velocity>(entity).unwrap();
    let query2 = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new());
    assert!(query2.into_iter().flatten().next().is_none());
    let query3 = Query::<&Position>::retrieve(&world, &HashMap::new()).unwrap();
    assert!(query3.into_iter().next().is_some());

    // Add Health
    world.add_component(entity, Health(100)).unwrap();
    let query4 = Query::<(&Position, &Health)>::retrieve(&world, &HashMap::new()).unwrap();
    assert!(query4.into_iter().next().is_some());
    let query5 = Query::<(&Position, &Velocity, &Health)>::retrieve(&world, &HashMap::new());
    assert!(query5.into_iter().flatten().next().is_none());
}

#[test]
fn test_movement_system() {
    let mut world = World::new();
    world.add_system(movement_system);

    let _entity1 = world.spawn_entity((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.5 }));
    let _entity2 = world.spawn_entity((Position { x: 10.0, y: 5.0 }, Velocity { x: -2.0, y: 1.0 }));
    let _entity3_no_velocity = world.spawn_entity(Position { x: 100.0, y: 200.0 }); // Should not move

    world.step(); // Run the system

    let query = Query::<(&Position, &Velocity)>::retrieve(&world, &HashMap::new()).unwrap();
    let mut results: Vec<(Ref<Position>, Ref<Velocity>)> = query.into_iter().collect();
    results.sort_by(|a, b| a.0.x.partial_cmp(&b.0.x).unwrap()); // Sort to ensure consistent order

    assert_eq!(*results[0].0, Position { x: 1.0, y: 0.5 }); 
    assert_eq!(*results[1].0, Position { x: 8.0, y: 6.0 }); 

    // Verify entity3_no_velocity didn't move
    let query_no_vel = Query::<&Position, Without<Velocity>>::retrieve(&world, &HashMap::new()).unwrap();
    let pos_no_vel = query_no_vel.into_iter().next().expect("Entity without velocity should still exist");
    assert_eq!(*pos_no_vel, Position { x: 100.0, y: 200.0 });
}

#[test]
fn test_health_system_with_filter() {
    let mut world = World::new();
    world.add_system(health_system);

    let _entity1 = world.spawn_entity((Health(10), Position { x: 0.0, y: 0.0 })); // Has Position, will be processed
    let _entity2 = world.spawn_entity(Health(5)); // No Position, should not be processed
    let _entity3 = world.spawn_entity((Health(1), Position { x: 1.0, y: 1.0 }));

    world.step(); // Run the system

    let query_health_pos = Query::<&Health, With<Position>>::retrieve(&world, &HashMap::new()).unwrap();
    let mut health_values_with_pos: Vec<u32> = query_health_pos.into_iter().map(|h| h.0).collect();
    health_values_with_pos.sort();
    assert_eq!(health_values_with_pos, vec![0, 9]);

    let query_health_only = Query::<&Health, Without<Position>>::retrieve(&world, &HashMap::new()).unwrap();
    let health_only = query_health_only.into_iter().next().expect("Entity2 health should be unchanged");
    assert_eq!(health_only.0, 5);
}

#[test]
fn test_name_logger_system_without_filter() {
    let mut world = World::new();
    world.add_system(name_logger_system);

    let _entity1 = world.spawn_entity((Name("Alice".to_string()), Position { x: 1.0, y: 2.0 })); // Will be logged
    let _entity2 = world.spawn_entity((Name("Bob".to_string()), Position { x: 3.0, y: 4.0 }, Health(10))); // Has Health, will NOT be logged

    // The system prints, we can't assert output directly without capturing stdout,
    // but we can ensure it runs without panicking.
    world.step();

    // To properly test, one would usually capture stdout or mock the logging.
    // For now, we'll assume no panic means it ran correctly.
    let query_logged = Query::<(&Name, &Position), Without<Health>>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(query_logged.into_iter().count(), 1); // Only Alice should match
}

#[test]
fn test_resource_management() {
    let mut world = World::new();

    // Add resources
    world.add_ressource(Score(100));
    world.add_ressource(Settings { difficulty: 1.5 });

    // Retrieve resources
    {
        let score = world.get_ressource::<Score>().expect("Score resource should exist");
        assert_eq!(score.0, 100);

        let settings = world.get_ressource::<Settings>().expect("Settings resource should exist");
        assert_eq!(settings.difficulty, 1.5);
    }

    // Modify resource
    {
        let mut score = world.get_ressource_mut::<Score>().expect("Score resource should exist");
        score.0 += 50;
    }

    // Verify modification
    {
        let score = world.get_ressource::<Score>().unwrap();
        assert_eq!(score.0, 150);
    }

    // Remove resource
    world.remove_ressource::<Settings>();
    assert!(world.get_ressource::<Settings>().is_none());
    assert!(world.get_ressource::<Score>().is_some());
}

#[test]
fn test_resource_system() {
    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_ressource(Settings { difficulty: 2.0 });
    world.add_system(resource_system);

    world.step();

    let score = world.get_ressource::<Score>().unwrap();
    assert_eq!(score.0, 20); // 10 * 2.0 = 20
}

#[test]
fn test_combined_query_resource_system() {
    let mut world = World::new();
    world.add_ressource(Score(100));
    world.spawn_entity(Position { x: 0.0, y: 0.0 });
    world.spawn_entity(Position { x: 1.0, y: 1.0 });

    world.add_system(combined_system);

    // Should run without panicking
    world.step();

    let score = world.get_ressource::<Score>().unwrap();
    assert_eq!(score.0, 100);
}

#[test]
fn test_system_param_resource_retrieval() {
    let mut world = World::new();
    world.add_ressource(Score(50));

    // Manual retrieval via SystemParam
    let score = Res::<Score>::retrieve(&world, &HashMap::new()).expect("Should retrieve Score via SystemParam");
    assert_eq!(score.0, 50);
    drop(score);

    let mut score_mut = ResMut::<Score>::retrieve(&world, &HashMap::new()).expect("Should retrieve Score mutably via SystemParam");
    score_mut.0 = 75;
    std::mem::drop(score_mut);

    assert_eq!(world.get_ressource::<Score>().unwrap().0, 75);
}

#[test]
fn test_local_persistence_in_system() {
    use crate::world::FromWorld;
    use crate::system_local::{SystemLocal, Local};

    struct MyLocal {
        counter: u32,
    }
    impl SystemLocal for MyLocal {}
    impl FromWorld for MyLocal {
        fn from_world(_world: &World) -> Self {
            MyLocal { counter: 0 }
        }
    }

    fn increment_local_system(mut local: Local<MyLocal>, mut score: ResMut<Score>) {
        local.counter += 1;
        score.0 = local.counter;
    }

    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_system(increment_local_system);

    world.step();
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 1);

    world.step();
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 2);
}

#[test]
fn test_resource_system_with_step() {
    fn update_score_system(mut score: ResMut<Score>) {
        score.0 += 10;
    }

    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_system(update_score_system);

    world.step();
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 10);

    world.step();
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 20);
    
    // Test direct retrieve with empty local storage
    let score = Res::<Score>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(score.0, 20);
}

#[test]
fn test_single_event_with_add_event() {
    use crate::event::{Event, SingleEventWriter, SingleEventReader};

    #[derive(Clone, Debug, PartialEq)]
    struct MySingleEvent(u32);
    impl Event for MySingleEvent {}

    fn writer(mut w: SingleEventWriter<MySingleEvent>) {
        w.write(MySingleEvent(100));
    }

    fn reader(mut r: SingleEventReader<MySingleEvent>, mut score: ResMut<Score>) {
        if let Some(ev) = r.read() {
            score.0 = ev.0;
        } else {
            score.0 = 0;
        }
    }

    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_single_event::<MySingleEvent>();
    
    world.add_system(writer);
    world.add_system(reader);

    world.step();
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 100);

    let mut world2 = World::new();
    world2.add_ressource(Score(0));
    world2.add_single_event::<MySingleEvent>();
    
    world2.add_system(|mut w: SingleEventWriter<MySingleEvent>, score: Res<Score>| {
        if score.0 == 0 {
            w.write(MySingleEvent(42));
        }else if score.0 == 42 {
            w.write(MySingleEvent(1));
        }
    });
    world2.add_system(reader);

    world2.step();
    assert_eq!(world2.get_ressource::<Score>().unwrap().0, 42);

    world2.step();
    // Second step, Score was 42 so no write.
    assert_eq!(world2.get_ressource::<Score>().unwrap().0, 1);

    world2.step();
    // third step, no event this frame, reader is not executed so it's the alst as last frame
    assert_eq!(world2.get_ressource::<Score>().unwrap().0, 1);
}

#[test]
fn test_single_event_reader_before() {
    use crate::event::{Event, SingleEventWriter, SingleEventReader};

    #[derive(Clone, Debug, PartialEq)]
    struct MySingleEvent(u32);
    impl Event for MySingleEvent {}

    fn writer(mut w: SingleEventWriter<MySingleEvent>) {
        w.write(MySingleEvent(100));
    }

    fn reader(mut r: SingleEventReader<MySingleEvent>, mut score: ResMut<Score>) {
        if let Some(ev) = r.read() {
            score.0 = ev.0;
        } else {
            score.0 = 1;
        }
    }

    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_single_event::<MySingleEvent>();
    
    world.add_system(reader);
    world.add_system(writer);

    world.step();
    // the reader shoud not execute so the score is 0
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 0);

    world.step();
    // the reader shoud execute so the score is 100
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 100);
}

#[test]
fn test_event_with_add_event() {
    use crate::event::{Event, EventWriter, EventReader};

    #[derive(Clone, Debug, PartialEq)]
    struct MyEvent(u32);
    impl Event for MyEvent {}

    struct OneEveryTwoLocal (bool);
    impl SystemLocal for OneEveryTwoLocal {}
    impl FromWorld for OneEveryTwoLocal {
        fn from_world(_world: &World) -> Self {
            OneEveryTwoLocal(true)
        }
    }

    fn writer(mut w: EventWriter<MyEvent>, mut do_it: Local<OneEveryTwoLocal>) {
        if do_it.0 {
            w.write(MyEvent(10));
            w.write(MyEvent(20));
        }
        do_it.0 = !do_it.0;
    }

    fn reader(mut r: EventReader<MyEvent>, mut score: ResMut<Score>) {
        let mut sum = 0;
        for ev in r.iter() {
            sum += ev.0;
        }
        score.0 += sum;
    }

    let mut world = World::new();
    world.add_ressource(Score(0));
    world.add_event::<MyEvent>();
    
    world.add_system(writer);
    world.add_system(reader);


    world.step();
    // Frame 1: previous=[] this=[10, 20] -> sum=30
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 30);

    world.step();
    // Frame 2: 
    // Last frame event already handled, no new event
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 30);
    world.step();
    // Frame 3: 
    // New events
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 60);
    // Frame 4: 
    // No new events
    assert_eq!(world.get_ressource::<Score>().unwrap().0, 60);
}

#[test]
fn test_command_system() {
    use crate::command::{Command, AddEntityCommand, RemoveEntityCommand, AddComponentsCommand, RemoveComponentCommand, CommandHandlerTrait};
    use crate::event::{Event, SingleEventReader, SingleEventWriter};

    #[derive(Clone)]
    struct DoAdd;
    impl Event for DoAdd {}
    #[derive(Clone)]
    struct DoRemove;
    impl Event for DoRemove {}

    struct DummyHandler;
    impl CommandHandlerTrait for DummyHandler {}

    let mut world = World::new();
    world.add_ressource(Command(vec![]));
    world.add_single_event::<DoAdd>();
    world.add_single_event::<DoRemove>();

    let entity = world.spawn_entity(Position { x: 0.0, y: 0.0 }).0;

    // 1. Test AddComponentsCommand and AddEntityCommand
    world.add_system(move |mut reader: SingleEventReader<DoAdd>, mut commands: ResMut<Command>| {
        if reader.read().is_some() {
            commands.0.push(Box::new(AddComponentsCommand::new(entity, Health(100))));
            commands.0.push(Box::new(AddEntityCommand::new_no_callbacks(Velocity { x: 1.0, y: 1.0 })));
        }
    });

    SingleEventWriter::retrieve(&world, &HashMap::new()).unwrap().write(DoAdd);

    world.step();
    world.handle_command(&mut DummyHandler);

    // Verify AddComponentsCommand
    let query_h = Query::<&Health>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(query_h.into_iter().count(), 1);

    // Verify AddEntityCommand
    let query_v = Query::<&Velocity>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(query_v.into_iter().count(), 1);

    // 2. Test RemoveComponentCommand and RemoveEntityCommand
    // We need to find the new entity with Velocity to remove it
    let velocity_entity = {
        let query = Query::<EntityArgument, With<Velocity>>::retrieve(&world, &HashMap::new()).unwrap();
        query.into_iter().next().unwrap()
    };

    world.add_system(move |mut reader: SingleEventReader<DoRemove>, mut commands: ResMut<Command>| {
        if reader.read().is_some() {
            commands.0.push(Box::new(RemoveComponentCommand::<Health>::new(entity)));
            commands.0.push(Box::new(RemoveEntityCommand::new(velocity_entity)));
        }
    });

    SingleEventWriter::retrieve(&world, &HashMap::new()).unwrap().write(DoRemove);

    world.step();
    world.handle_command(&mut DummyHandler);

    // Verify RemoveComponentCommand
    let query_h_after = Query::<&Health>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(query_h_after.into_iter().count(), 0);

    // Verify RemoveEntityCommand
    let query_v_after = Query::<&Velocity>::retrieve(&world, &HashMap::new()).unwrap();
    assert_eq!(query_v_after.into_iter().count(), 0);
}
