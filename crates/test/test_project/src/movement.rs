use core_components::transform::{Transform};
use ecs::{component::Component, query::Query};
use glam::Vec3;


pub struct Velocity (pub Vec3);
impl Component for Velocity {}

pub fn move_system(query: Query<(&Velocity, &mut Transform)>){
    for (velocity, mut transform) in query.into_iter(){
        transform.position += velocity.0;
    }
}

