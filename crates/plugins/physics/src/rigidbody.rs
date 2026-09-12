use core_components::transform::Transform;
use ecs::{
    component::{Component, ComponentTupple},
    query::Query,
    ressource::Res,
};
use engine::application::DeltaTimeS;
use glam::{Mat3, Quat, Vec3};

use crate::collision::colliders::Colliders;

pub struct Velocity {
    pub linear: Vec3,
    pub angular_momentum: Vec3, // I know it's not velocity but hey, it's just a name
    pub inv_inertia_tensor_local: Mat3,
}
impl Component for Velocity {}

pub struct Gravity(Vec3);
impl Component for Gravity {}
impl Default for Gravity {
    fn default() -> Self {
        Gravity(Vec3::new(0.0, -9.81, 0.0))
    }
}

pub fn get_rb_components_from_colliders<'a, 'b>(
    colliders: &'a Colliders,
    external_scale: Vec3,
) -> impl ComponentTupple + use<'b> {
    (
        Velocity {
            linear: Vec3::ZERO,
            angular_momentum: Vec3::ZERO,
            inv_inertia_tensor_local: colliders.get_local_inv_inertia_tensor(external_scale),
        },
        Gravity::default(),
    )
}

pub(crate) fn gravity_update(query: Query<(&mut Velocity, &Gravity)>, dt: Res<DeltaTimeS>) {
    for (mut vel, grav) in query.into_iter() {
        vel.linear += grav.0 * dt.0;
    }
}

pub(crate) fn velocity_update(query: Query<(&mut Transform, &Velocity)>, dt: Res<DeltaTimeS>) {
    for (mut transform, vel) in query.into_iter() {
        let r = Mat3::from_quat(transform.rotation);
        // angular
        let l_local = r.transpose() * vel.angular_momentum;
        let omega_local = vel.inv_inertia_tensor_local * l_local;
        let omega = r * omega_local;

        let speed = omega.length();

        if speed > 1e-6 {
            let axis = omega / speed;
            let angle = speed * dt.0;

            let delta_quat = Quat::from_axis_angle(axis, angle);

            transform.rotation = (delta_quat * transform.rotation).normalize();
        }

        // linear velocity
        transform.position += dt.0 * vel.linear;
    }
}
