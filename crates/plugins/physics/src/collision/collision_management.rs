use std::cell::Ref;

use arrayvec::ArrayVec;
use core_components::transform::Transform;
use ecs::{
    entity::Entity,
    query::{EntityArgument, Query, Without},
};
use glam::Vec3;

use crate::{
    collision::{colliders::Colliders, detection::detect_collision},
    rigidbody::Velocity,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ContactPoint {
    pub world: Vec3,
    pub depth: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct ContactManifold {
    pub points: ArrayVec<ContactPoint, 8>,
    pub mtv: Vec3,
}

pub fn manage_collision(
    rb_collider: Query<(EntityArgument, &Transform, &Colliders, &Velocity)>,
    static_collider: Query<(EntityArgument, &Transform, &Colliders), Without<Velocity>>,
) {
    let entities_with_rb: Vec<_> = rb_collider.into_iter().collect();
    let static_entities: Vec<(Entity, Ref<Transform>, Ref<Colliders>)> =
        static_collider.into_iter().collect();

    let collisions = detect_collision(entities_with_rb, static_entities);

    //resolve_collision(collisions);
}
