use std::cell::Ref;

use core_components::transform::Transform;
use ecs::entity::Entity;
use glam::Vec3;

use crate::{
    collision::{
        colliders::{Collider, Colliders, CubeCollider},
        sat::{SATResult, apply_sat_and_clipping, sat},
    },
    rigidbody::RigidBody,
};

pub type DetectedCollision = Vec<(Entity)>;
pub type EntityWithRb<'a> = (
    Entity,
    Ref<'a, Transform>,
    Ref<'a, Colliders>,
    Ref<'a, RigidBody>,
);
pub type StaticEntity<'a> = (Entity, Ref<'a, Transform>, Ref<'a, Colliders>);

type DetectionArg<'a, T> = (&'a Transform, &'a T);
type DetectionResult = Option<ContactManifold>;

#[derive(Default)]
pub(crate) struct ContactManifold {
    pub points: Vec<Vec3>,
    pub normal: Vec3,
    pub penetration: f32,
}

fn call_detection<'a>(
    collider1: DetectionArg<Collider>,
    collider2: DetectionArg<Collider>,
) -> DetectionResult {
    match (collider1.1, collider2.1) {
        (Collider::CubeCollider(cube1), Collider::CubeCollider(cube2)) => {
            cube_cube_detection((collider1.0, cube1), (collider2.0, cube2))
        }
    }
}

pub fn detect_collision(
    entities_with_rb: Vec<EntityWithRb>,
    static_entities: Vec<StaticEntity>,
) -> DetectedCollision {
    let static_entities_iter = static_entities
        .iter()
        .map(|(entity, transform, collider)| (*entity, &**transform, &**collider, None));
    let rb_entities_iter = entities_with_rb
        .iter()
        .map(|(entity, transform, collider, rb)| (*entity, &**transform, &**collider, Some(&**rb)));

    let all_entities_iter = rb_entities_iter.chain(static_entities_iter);
    let all_entities: Vec<_> = all_entities_iter.collect();

    println!("nb entities: {}", all_entities.len());

    let detected_collision: DetectedCollision = DetectedCollision::new();

    for (entity1, transform1, colliders1, _rb1) in all_entities.iter() {
        'outer: for collider1 in colliders1.colliders.iter() {
            for (entity2, transform2, colliders2, _rb2) in all_entities.iter() {
                if *entity1 == *entity2 {
                    break 'outer;
                }
                for collider2 in colliders2.colliders.iter() {
                    let result = call_detection((transform1, collider1), (transform2, collider2));
                }
            }
        }
    }

    Default::default()

    //unimplemented!()
}

fn cube_cube_detection(
    entity1: DetectionArg<CubeCollider>,
    entity2: DetectionArg<CubeCollider>,
) -> DetectionResult {
    let cube1 = entity1.1.get_world_space(entity1.0);
    let cube2 = entity2.1.get_world_space(entity2.0);

    apply_sat_and_clipping(cube1, cube2)
}
