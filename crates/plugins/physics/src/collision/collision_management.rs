use std::cell::Ref;

use core_components::transform::Transform;
use ecs::{
    entity::Entity,
    query::{EntityArgument, Query, Without},
};

use crate::{
    collision::{colliders::Colliders, detection::detect_collision, resolution::resolve_collision},
    rigidbody::RigidBody,
};

pub fn manage_collision(
    rb_collider: Query<(EntityArgument, &Transform, &Colliders, &RigidBody)>,
    static_collider: Query<(EntityArgument, &Transform, &Colliders), Without<RigidBody>>,
) {
    let entities_with_rb: Vec<_> = rb_collider.into_iter().collect();
    let static_entities: Vec<(Entity, Ref<Transform>, Ref<Colliders>)> =
        static_collider.into_iter().collect();

    let collisions = detect_collision(entities_with_rb, static_entities);

    //resolve_collision(collisions);
}
