use arrayvec::ArrayVec;
use glam::{Mat3, Vec3};

use crate::{
    collision::{
        self,
        collision_management::{
            CollisionEntity, ContactManifold, MAX_COLLISION_POINTS, RigidBodyEntity, StaticEntity,
        },
        detection::{DetectedCollision, DetectedCollisions},
    },
    rigidbody::ForceMode,
};

#[derive(Debug, Default, Clone)]
struct PreStepInfo {
    pub ra: ArrayVec<Vec3, MAX_COLLISION_POINTS>,
    pub rb: ArrayVec<Vec3, MAX_COLLISION_POINTS>,

    pub effective_inverse_mass: ArrayVec<f32, MAX_COLLISION_POINTS>,

    pub velocity_bias: ArrayVec<f32, MAX_COLLISION_POINTS>,
}

#[derive(Debug, Default, Clone)]
struct BetweenStepInfo {
    pub accumulated_lambda: ArrayVec<f32, MAX_COLLISION_POINTS>,
}

fn resolve_collision(
    entities: &mut Vec<CollisionEntity<'_>>,
    collision: &mut DetectedCollision,
    pre_step_info: &PreStepInfo,
    mut between_step: BetweenStepInfo,
) -> BetweenStepInfo {
    let (ent_1_mut, ent_2_mut) = {
        let i1 = collision.1;
        let i2 = collision.2;

        let (low, high) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
        let (left, right) = entities.split_at_mut(high);

        if i1 < i2 {
            (&mut left[low], &mut right[0])
        } else {
            (&mut right[0], &mut left[low])
        }
    };

    let CollisionEntity::RigidBody(rb_1) = ent_1_mut else {
        unreachable!("Collision resolution: Entity 1 should always but rigidbody")
    };
    let mut rb_2 = if let CollisionEntity::RigidBody(rb_2) = ent_2_mut {
        Some(rb_2)
    } else {
        None
    };

    let manifold = &collision.0;

    for (i, point) in manifold.points.iter().enumerate() {
        let v_rel = if let Some(rb_2) = &mut rb_2 {
            let v_rel = (rb_1.3.get_velocity_at_point(pre_step_info.ra[i])
                - rb_2.3.get_velocity_at_point(pre_step_info.rb[i]))
            .dot(manifold.normal);

            v_rel
        } else {
            let v_rel = (manifold.normal).dot(rb_1.3.get_velocity_at_point(pre_step_info.ra[i]));
            v_rel
        };

        let delta_lambda =
            -(v_rel + pre_step_info.velocity_bias[i]) / pre_step_info.effective_inverse_mass[i];

        println!("First lambda {delta_lambda}");

        let accumulated_lambda: f32 = between_step
            .accumulated_lambda
            .get(i)
            .cloned()
            .unwrap_or_else(|| {
                between_step.accumulated_lambda.push(0.0);
                0.0
            });

        let delta_target = accumulated_lambda + delta_lambda;
        let delta_clamped = delta_target.max(0.0);
        let delta_lambda = delta_clamped - accumulated_lambda;
        between_step.accumulated_lambda[i] = delta_clamped;

        if delta_lambda > 100.0 {
            panic!()
        }

        println!(
            "Point : {}, coordinate: {}, lambda {}, velocity_rel {}, effective_inverse_mass: {}, velocity_bias: {}",
            i,
            point.world,
            delta_clamped,
            v_rel,
            pre_step_info.effective_inverse_mass[i],
            pre_step_info.velocity_bias[i]
        );

        rb_1.3.apply_force_at_point(
            manifold.normal * delta_lambda,
            point.world,
            ForceMode::Impulse,
        );
        if let Some(rb_2) = &mut rb_2 {
            rb_2.3.apply_force_at_point(
                -manifold.normal * delta_lambda,
                point.world,
                ForceMode::Impulse,
            );
        }
    }
    between_step
}

fn pre_step_info_rb_rb(
    entities: &mut Vec<CollisionEntity<'_>>,
    collision: &DetectedCollision,
    dt_s: f32,
) -> PreStepInfo {
    // TODO: CREATE PHYSICS MATERIAL
    let restitution = 0.3;
    let beta = 0.3;

    let mut pre_step = PreStepInfo::default();

    let (ent_1_mut, ent_2_mut) = {
        let i1 = collision.1;
        let i2 = collision.2;

        let (low, high) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
        let (left, right) = entities.split_at_mut(high);

        if i1 < i2 {
            (&mut left[low], &mut right[0])
        } else {
            (&mut right[0], &mut left[low])
        }
    };
    let CollisionEntity::RigidBody(rb_1) = ent_1_mut else {
        unreachable!("Collision resolution: Entity 1 should always but rigidbody")
    };
    let mut rb_2 = if let CollisionEntity::RigidBody(rb_2) = ent_2_mut {
        Some(rb_2)
    } else {
        None
    };

    let manifold = &collision.0;

    for point in manifold.points.iter() {
        let ra = point.world - rb_1.1.position;
        pre_step.ra.push(ra);

        let ra_cross_n = ra.cross(manifold.normal);

        let angular_a = ra_cross_n.dot(rb_1.3.get_world_inv_inertia_tensor() * ra_cross_n);

        let (k, v_rel) = if let Some(rb_2) = &mut rb_2 {
            let rb = point.world - rb_2.1.position;
            pre_step.rb.push(rb);
            let rb_cross_n = rb.cross(manifold.normal);
            let angular_b = rb_cross_n.dot(rb_2.3.get_world_inv_inertia_tensor() * rb_cross_n);

            let k = 1.0 / rb_1.3.mass + 1.0 / rb_2.3.mass + angular_a + angular_b;

            let v_rel = (rb_1.3.get_velocity_at_point(ra) - rb_2.3.get_velocity_at_point(rb))
                .dot(manifold.normal);

            (k, v_rel)
        } else {
            let k = 1.0 / rb_1.3.mass + angular_a;

            let v_rel = (manifold.normal).dot(rb_1.3.get_velocity_at_point(ra));

            (k, v_rel)
        };
        pre_step.effective_inverse_mass.push(k);

        let bounce_threshold = 0.2;
        let restitution_bias = if v_rel < -bounce_threshold {
            restitution * v_rel
        } else {
            0.0
        };

        let slop = 0.002;
        let penetration = (point.depth - slop).max(0.0);
        let raw_position_bias = -(beta / dt_s) * penetration;
        let max_bias_speed = 3.0;
        let position_bias = raw_position_bias.max(-max_bias_speed);

        let bias = restitution_bias + position_bias;

        pre_step.velocity_bias.push(bias);
    }

    pre_step
}

pub(crate) fn resolve_collisions(
    entities: &mut Vec<CollisionEntity<'_>>,
    mut collisions: DetectedCollisions,
    dt_s: f32,
) {
    let steps = 20;

    let mut collision_data: Vec<_> = collisions
        .iter()
        .map(|collision| pre_step_info_rb_rb(entities, collision, dt_s))
        .map(|pre_step| (pre_step, BetweenStepInfo::default()))
        .collect();

    for _ in 0..steps {
        for (i, collision) in collisions.iter_mut().enumerate() {
            collision_data[i].1 = resolve_collision(
                entities,
                collision,
                &collision_data[i].0,
                collision_data[i].1.clone(),
            )
        }
    }

    if collisions.len() > 0 {
        //panic!();
    }
}
