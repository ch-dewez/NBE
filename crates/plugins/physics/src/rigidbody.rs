use core_components::transform::Transform;
use ecs::{
    component::{Component, ComponentTupple},
    query::Query,
    ressource::Res,
};
use engine::application::DeltaTimeS;
use glam::{Mat3, Quat, Vec3};

use crate::collision::colliders::Colliders;

#[derive(Debug)]
pub struct RigidBody {
    pub linear: Vec3,
    pub inv_inertia_tensor_local: Mat3,
    pub mass: f32,
    pub local_center_of_mass: Vec3,

    // non-pub fields: managed by struct
    current_inv_world_inertia_tensor: Mat3,
    angular_momentum: Vec3,
    angular_velocity: Vec3,

    force_accum: Vec3,
    torque_accum: Vec3,

    world_center_of_mass: Vec3,
}
impl Component for RigidBody {}

/// FORCE : directly add to m_force -> is affected by mass and dt
/// IMPULSE : adding to m_acceleration but divided my mass -> is affected by mass
/// but not dt
/// ACCELERATION : adding to m_force but multiplied by mass -> mass
/// canceled out so is affected by dt but not mass
/// VELOCITYCHANGE: adding
/// directly to m_acceleration -> is affected by nothing
///
/// Which one to chose ?
/// normal force : force which happened in a period (not a single moment) and is affected by mass
/// Impulse: if the force happened in a difined moment in time (me hitting a ball or a ball bouncing of somehting)
/// Acceleration : if something continiously and is not affected by mass -> gravity
/// (either you say the gravity force is multiplied by mass and you use Force or don't multiply it and use this)
/// VELOCITY CHANGE : if you want to directly change the velocity
#[derive(Debug, Clone, Copy)]
pub enum ForceMode {
    Force,
    Impulse,
    Acceleration,
    VelocityChange,
}

impl RigidBody {
    pub fn new_from_linear_angular_momentum(
        linear_vel: Vec3,
        angular_momentum: Vec3,
        colliders: &Colliders,
        transform: &Transform,
    ) -> Self {
        let rb_info = colliders.get_rigid_body_info_inv(transform.scale);
        let mut rb = Self {
            linear: linear_vel,
            angular_momentum,

            inv_inertia_tensor_local: rb_info.2,
            mass: rb_info.0,
            local_center_of_mass: rb_info.1,

            current_inv_world_inertia_tensor: Mat3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accum: Vec3::ZERO,
            torque_accum: Vec3::ZERO,
            world_center_of_mass: Vec3::ZERO,
        };

        rb.recompute_world_inertia_tensor(transform);
        rb.recompute_angular_velocity();
        rb.recompute_world_center_of_mass(transform);

        rb
    }

    pub fn new_from_linear_angular_vel(
        linear_vel: Vec3,
        angular_vel: Vec3,
        colliders: &Colliders,
        transform: &Transform,
    ) -> Self {
        let rb_info = colliders.get_rigid_body_info_inv(transform.scale);
        let mut rb = Self {
            linear: linear_vel,
            angular_momentum: Vec3::ZERO,

            inv_inertia_tensor_local: rb_info.2,
            mass: rb_info.0,
            local_center_of_mass: rb_info.1,

            angular_velocity: Vec3::ZERO,
            force_accum: Vec3::ZERO,
            torque_accum: Vec3::ZERO,
            world_center_of_mass: Vec3::ZERO,

            current_inv_world_inertia_tensor: Mat3::ZERO,
        };

        rb.recompute_world_inertia_tensor(transform);
        rb.set_angular_velocity(angular_vel);
        rb.recompute_world_center_of_mass(transform);

        rb
    }

    pub fn set_angular_momentum(&mut self, momentum: Vec3) {
        self.angular_momentum = momentum;
        self.recompute_angular_velocity();
    }

    /// set the angular momentum and velocity from angular velocity
    pub fn set_angular_velocity(&mut self, vel: Vec3) {
        self.angular_momentum = self.current_inv_world_inertia_tensor.inverse() * vel;
        self.angular_velocity = vel;
    }

    pub fn get_current_angular_velocity(&self) -> Vec3 {
        self.angular_velocity
    }

    pub fn get_world_inv_inertia_tensor(&self) -> Mat3 {
        self.current_inv_world_inertia_tensor
    }

    pub fn recompute_world_inertia_tensor(&mut self, transform: &Transform) {
        let rot_mat = transform.get_rotation_matrix();
        self.current_inv_world_inertia_tensor =
            rot_mat * self.inv_inertia_tensor_local * rot_mat.transpose();
    }

    pub fn recompute_world_center_of_mass(&mut self, transform: &Transform) {
        let rot_mat = transform.get_rotation_matrix();
        self.world_center_of_mass = transform.position + (rot_mat * self.local_center_of_mass);
    }

    fn recompute_angular_velocity(&mut self) {
        self.angular_velocity = self.get_world_inv_inertia_tensor() * self.angular_momentum;
    }

    pub fn get_velocity_at_point(&self, point: Vec3) -> Vec3 {
        self.linear + self.get_current_angular_velocity().cross(point)
    }

    pub fn apply_force(&mut self, force: Vec3, force_mode: ForceMode) {
        match force_mode {
            ForceMode::Force => self.force_accum += force,
            ForceMode::Impulse => self.linear += force / self.mass,
            ForceMode::Acceleration => self.force_accum += force * self.mass,
            ForceMode::VelocityChange => self.linear += force,
        }
    }

    pub fn apply_torque(&mut self, torque: Vec3, force_mode: ForceMode) {
        match force_mode {
            ForceMode::Force => self.torque_accum += torque,
            ForceMode::Impulse => self.angular_momentum += torque,
            ForceMode::Acceleration => {
                self.torque_accum += self.get_world_inv_inertia_tensor().inverse() * torque
            }
            ForceMode::VelocityChange => {
                self.angular_momentum += self.get_world_inv_inertia_tensor().inverse() * torque
            }
        }

        self.recompute_angular_velocity();
    }

    pub fn apply_force_at_point(&mut self, force: Vec3, point_world: Vec3, force_mode: ForceMode) {
        self.apply_force(force, force_mode);

        let r = point_world - self.world_center_of_mass;

        // 3. Convert off-center motion into actual Force or Angular Impulse
        match force_mode {
            ForceMode::Force => {
                self.apply_torque(r.cross(force), ForceMode::Force);
            }
            ForceMode::Impulse => {
                self.apply_torque(r.cross(force), ForceMode::Impulse);
            }
            ForceMode::Acceleration => {
                // F = m * a  =>  Torque = r x (m * a)
                let actual_force = force * self.mass;
                self.apply_torque(r.cross(actual_force), ForceMode::Force);
            }
            ForceMode::VelocityChange => {
                // J = m * dv =>  Angular Impulse = r x (m * dv)
                let actual_impulse = force * self.mass;
                self.apply_torque(r.cross(actual_impulse), ForceMode::Impulse);
            }
        }
    }
}

pub struct Gravity(Vec3);
impl Component for Gravity {}
impl Default for Gravity {
    fn default() -> Self {
        Gravity(Vec3::new(0.0, -9.81, 0.0))
    }
}

pub(crate) fn gravity_update(query: Query<(&mut RigidBody, &Gravity)>) {
    for (mut rb, grav) in query.into_iter() {
        rb.apply_force(grav.0, ForceMode::Acceleration);
    }
}

pub(crate) fn rb_update(query: Query<(&mut Transform, &mut RigidBody)>, dt: Res<DeltaTimeS>) {
    for (mut transform, mut rb) in query.into_iter() {
        let acceleration = rb.force_accum / rb.mass;
        rb.linear += acceleration * dt.0;
        let torque = rb.torque_accum;
        rb.angular_momentum += torque;
        rb.force_accum = Vec3::ZERO;
        rb.torque_accum = Vec3::ZERO;

        rb.recompute_world_inertia_tensor(&transform);
        rb.recompute_angular_velocity();

        // angular
        let omega = rb.get_current_angular_velocity();
        let speed = omega.length();

        if speed > 1e-6 {
            let axis = omega / speed;
            let angle = speed * dt.0;

            let delta_quat = Quat::from_axis_angle(axis, angle);

            transform.rotation = (delta_quat * transform.rotation).normalize();
        }

        // linear velocity
        transform.position += dt.0 * rb.linear;

        rb.recompute_world_center_of_mass(&transform);
    }
}
