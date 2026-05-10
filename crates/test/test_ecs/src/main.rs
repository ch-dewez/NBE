use ecs::{component::Component, query::{Query, With, Without}, world::World};

#[derive(Clone)]
struct Velocity{
    x: f32,
    y: f32
}
impl Component for Velocity{}
#[derive(Clone)]
struct Position{
    x: f32,
    y: f32
}
impl Component for Position{}

#[derive(Clone)]
struct MovementPrinter{}
impl Component for MovementPrinter{}

fn hello_world(){
    println!("Hello world!");
}

fn movement_printer(query: Query<&Position, With<MovementPrinter>>){
    for position in query.into_iter(){
        println!("The posittion of this entity is : x:{} y:{}, btw it's moveming printer", position.x, position.y);
    }
}

fn printer(query: Query<&Position, Without<MovementPrinter>>){
    for position in query.into_iter(){
        println!("The posittion of this entity is : x:{} y:{}, btw it's NOT moveming printer", position.x, position.y);
    }
}

fn movement(query: Query<(&mut Position, &Velocity)>){
    for (position, velocity) in query.into_iter(){
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn main() {
    let mut world = World::new();
    world.add_system(hello_world)
        .add_system(movement_printer)
        .add_system(printer)
        .add_system(movement);

    let entity3 = world
        .spawn_entity(&(Position {x:0.0, y:0.0}, Velocity{x: 1.0, y:1.0}, MovementPrinter{})).1
        .spawn_entity(&(Position {x:-1.0, y:0.0})).1
        .spawn_entity(&(Position {x:1.0, y:1.0}, MovementPrinter{})).0;

    let _ = world.add_component(entity3, Velocity{x:-1.0, y:1.0});

    for _ in 0..1_000_000 {
        world.step();
    }
}
