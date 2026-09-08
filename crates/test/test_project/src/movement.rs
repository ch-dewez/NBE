use core_components::transform::Transform;
use ecs::{component::Component, query::Query, ressource::Res};
use engine::application::DeltaTimeS;
use glam::Vec3;

pub struct Velocity(pub Vec3);
impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}
impl Component for Velocity {}

pub fn move_system(query: Query<(&Velocity, &mut Transform)>, dt: Res<DeltaTimeS>) {
    for (velocity, mut transform) in query.into_iter() {
        transform.position += velocity.0 * dt.0;
    }
}
