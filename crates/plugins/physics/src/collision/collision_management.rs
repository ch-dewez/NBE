use arrayvec::ArrayVec;
use core_components::transform::Transform;
use ecs::{
    entity::Entity,
    query::{EntityArgument, Query, Without},
    ressource::Res,
};
use engine::application::DeltaTimeS;
use glam::Vec3;

use crate::{
    collision::{
        colliders::Colliders, detection::detect_collision, resolution::resolve_collisions,
    },
    rigidbody::RigidBody,
};

pub(crate) const MAX_COLLISION_POINTS: usize = 8;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ContactPoint {
    pub world: Vec3,
    pub depth: f32,
}

#[derive(Debug)]
pub(crate) struct ContactManifold {
    pub points: ArrayVec<ContactPoint, MAX_COLLISION_POINTS>,
    pub mtv: Vec3,
    pub normal: Vec3,
}

#[derive(Debug)]
pub(crate) enum CollisionEntity<'a> {
    StaticEntity(StaticEntity<'a>),
    RigidBody(RigidBodyEntity<'a>),
}

impl<'a> CollisionEntity<'a> {
    pub fn get_entity(&self) -> Entity {
        match self {
            Self::StaticEntity(entity) => entity.0,
            Self::RigidBody(entity) => entity.0,
        }
    }
    pub fn get_transform<'b>(&'b self) -> &'b Transform {
        match self {
            Self::StaticEntity(entity) => &entity.1,
            Self::RigidBody(entity) => &entity.1,
        }
    }
    pub fn get_colliders<'b>(&'b self) -> &'b Colliders {
        match self {
            Self::StaticEntity(entity) => &entity.2,
            Self::RigidBody(entity) => &entity.2,
        }
    }

    pub fn has_rb(&self) -> bool {
        match self {
            Self::StaticEntity(_) => false,
            Self::RigidBody(_) => true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StaticEntity<'a>(pub Entity, pub &'a Transform, pub &'a Colliders);
#[derive(Debug)]
pub(crate) struct RigidBodyEntity<'a>(
    pub Entity,
    pub &'a Transform,
    pub &'a Colliders,
    pub &'a mut RigidBody,
);

pub fn manage_collision(
    rb_collider: Query<(EntityArgument, &Transform, &Colliders, &mut RigidBody)>,
    static_collider: Query<(EntityArgument, &Transform, &Colliders), Without<RigidBody>>,
    dt_s: Res<DeltaTimeS>,
) {
    let entities_with_rb = rb_collider
        .into_iter()
        .map(|(entity, transform, colliders, rb)| {
            CollisionEntity::RigidBody(RigidBodyEntity(entity, transform, colliders, rb))
        });
    let static_entities = static_collider
        .into_iter()
        .map(|(entity, transform, colliders)| {
            CollisionEntity::StaticEntity(StaticEntity(entity, transform, colliders))
        });
    let mut all_entities: Vec<_> = entities_with_rb.chain(static_entities).collect();

    let collisions = detect_collision(&all_entities);

    resolve_collisions(&mut all_entities, collisions, dt_s.0);
}
