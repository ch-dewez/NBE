use core_components::transform::Transform;

use crate::collision::{
    colliders::{Collider, CubeCollider},
    collision_management::{CollisionEntity, ContactManifold},
    sat::apply_sat_and_clipping,
};

pub type DetectedCollision = (ContactManifold, usize, usize);
pub type DetectedCollisions<'a> = Vec<DetectedCollision>;

type DetectionArg<'a, T> = (&'a Transform, &'a T);
type DetectionResult = Option<ContactManifold>;

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

pub fn detect_collision<'a>(entities: &'a Vec<CollisionEntity<'a>>) -> DetectedCollisions<'a> {
    let mut detected_collision: DetectedCollisions<'a> = DetectedCollisions::new();

    for (i, entity1) in entities.iter().enumerate() {
        for collider1 in entity1.get_colliders().colliders.iter() {
            for (j, entity2) in entities.iter().enumerate().skip(i + 1) {
                if i == j {
                    continue;
                } else if !entity1.has_rb() && !entity2.has_rb() {
                    // Both entities are static, no need to check collision we can't resolve them
                    continue;
                }
                for collider2 in entity2.get_colliders().colliders.iter() {
                    let result = call_detection(
                        (entity1.get_transform(), collider1),
                        (entity2.get_transform(), collider2),
                    );
                    if let Some(mut manifold) = result {
                        let collision = match (entity1, entity2) {
                            (CollisionEntity::RigidBody(_), CollisionEntity::RigidBody(_))
                            | (CollisionEntity::RigidBody(_), CollisionEntity::StaticEntity(_)) => {
                                (manifold, i, j)
                            }
                            (CollisionEntity::StaticEntity(_), CollisionEntity::RigidBody(_)) => {
                                manifold.mtv *= -1.0;
                                manifold.normal *= -1.0;
                                (manifold, j, i)
                            }
                            (
                                CollisionEntity::StaticEntity(_),
                                CollisionEntity::StaticEntity(_),
                            ) => {
                                unreachable!("Static Static ent collision response, can't happen")
                            }
                        };
                        detected_collision.push(collision);
                    }
                }
            }
        }
    }

    detected_collision
}

fn cube_cube_detection(
    entity1: DetectionArg<CubeCollider>,
    entity2: DetectionArg<CubeCollider>,
) -> DetectionResult {
    let cube1 = entity1.1.get_world_space(entity1.0);
    let cube2 = entity2.1.get_world_space(entity2.0);

    apply_sat_and_clipping(cube1, cube2)
}
