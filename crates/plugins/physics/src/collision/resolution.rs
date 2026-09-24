use arrayvec::ArrayVec;
use glam::Vec3;

use crate::{
    collision::{
        collision_management::{CollisionEntity, MAX_COLLISION_POINTS},
        detection::{DetectedCollision, DetectedCollisions},
    },
    rigidbody::ForceMode,
};

#[derive(Debug, Default, Clone)]
struct PreStepInfo {
    pub ra: ArrayVec<Vec3, MAX_COLLISION_POINTS>,
    pub rb: ArrayVec<Vec3, MAX_COLLISION_POINTS>,

    pub effective_mass: ArrayVec<f32, MAX_COLLISION_POINTS>,

    pub velocity_bias: ArrayVec<f32, MAX_COLLISION_POINTS>,

    pub t: [Vec3; 2],
    pub effective_mass_t: [f32; 2],
    pub combined_friction: f32,
}

#[derive(Debug, Default, Clone)]
struct BetweenStepInfo {
    pub accumulated_lambda: [f32; MAX_COLLISION_POINTS],
    pub accumulated_friction_lambda: [f32; 2],
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
            let vel_a = rb_1.3.get_velocity_at_point(pre_step_info.ra[i]);
            let vel_b = rb_2.3.get_velocity_at_point(pre_step_info.rb[i]);
            let diff = vel_a - vel_b;
            diff.dot(manifold.normal)
        } else {
            let vel_a = rb_1.3.get_velocity_at_point(pre_step_info.ra[i]);
            manifold.normal.dot(vel_a)
        };

        let delta_lambda =
            -(v_rel + pre_step_info.velocity_bias[i]) / pre_step_info.effective_mass[i];

        let accumulated_lambda_n: f32 = between_step.accumulated_lambda[i];

        let lambda_target = accumulated_lambda_n + delta_lambda;
        let lambda_clamped = lambda_target.max(0.0);
        let delta_lambda = lambda_clamped - accumulated_lambda_n;
        between_step.accumulated_lambda[i] = lambda_clamped;

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

        for (j, t) in pre_step_info.t.iter().cloned().enumerate() {
            let v_rel_t = if let Some(rb_2) = &mut rb_2 {
                let vel_a = rb_1.3.get_velocity_at_point(pre_step_info.ra[i]);
                let vel_b = rb_2.3.get_velocity_at_point(pre_step_info.rb[i]);
                let diff = vel_a - vel_b;
                diff.dot(t)
            } else {
                let vel_a = rb_1.3.get_velocity_at_point(pre_step_info.ra[i]);
                t.dot(vel_a)
            };

            let delta_lambda_t = -v_rel_t / pre_step_info.effective_mass_t[j];

            let lambda_t_accum = between_step.accumulated_friction_lambda[j];
            let lambda_target_t = lambda_t_accum + delta_lambda_t;

            let friction_coef = pre_step_info.combined_friction;
            let lambda_clamped_t = lambda_target_t.clamp(
                -friction_coef * accumulated_lambda_n,
                friction_coef * accumulated_lambda_n,
            );

            let delta_lambda_t = lambda_clamped_t - lambda_t_accum;

            rb_1.3
                .apply_force_at_point(t * delta_lambda_t, point.world, ForceMode::Impulse);
            if let Some(rb_2) = &mut rb_2 {
                rb_2.3
                    .apply_force_at_point(-t * delta_lambda_t, point.world, ForceMode::Impulse);
            }
        }

        // println!(
        //     "Point : {}, coordinate: {}, lambda {}, velocity_rel {}, effective_inverse_mass: {}, velocity_bias: {}",
        //     i,
        //     point.world,
        //     lambda_clamped,
        //     v_rel,
        //     pre_step_info.effective_mass[i],
        //     pre_step_info.velocity_bias[i]
        // );
    }
    between_step
}

fn pre_step_info(
    entities: &mut Vec<CollisionEntity<'_>>,
    collision: &DetectedCollision,
    dt_s: f32,
) -> PreStepInfo {
    // TODO: CREATE PHYSICS MATERIAL
    let restitution = 0.3;
    let friction = 0.04;
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

    // friction
    let t_1 = manifold.normal.cross(Vec3::X);
    let t_2 = manifold.normal.cross(t_1);
    let ts = [t_1, t_2];
    pre_step.t = ts.clone();

    pre_step.combined_friction = f32::sqrt(friction * friction); // I know it's just friction for now, but there'll be different friciton

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

            let inv_mass_1 = 1.0 / rb_1.3.mass;
            let inv_mass_2 = 1.0 / rb_2.3.mass;

            let k = inv_mass_1 + inv_mass_2 + angular_a + angular_b;

            let v_rel = (rb_1.3.get_velocity_at_point(ra) - rb_2.3.get_velocity_at_point(rb))
                .dot(manifold.normal);

            for (i, t) in ts.into_iter().enumerate() {
                let ra_cross_t = ra.cross(t);
                let t_angular_a =
                    ra_cross_t.dot(rb_1.3.get_world_inv_inertia_tensor() * ra_cross_t);
                let rb_cross_t = rb.cross(t);
                let t_angular_b =
                    rb_cross_t.dot(rb_2.3.get_world_inv_inertia_tensor() * rb_cross_t);
                let k_t = inv_mass_1 + inv_mass_2 + t_angular_a + t_angular_b;

                pre_step.effective_mass_t[i] = k_t;
            }

            (k, v_rel)
        } else {
            let inv_mass_1 = 1.0 / rb_1.3.mass;
            let k = inv_mass_1 + angular_a;

            let v_rel = (manifold.normal).dot(rb_1.3.get_velocity_at_point(ra));

            for (i, t) in ts.into_iter().enumerate() {
                let ra_cross_t = ra.cross(t);
                let t_angular_a =
                    ra_cross_t.dot(rb_1.3.get_world_inv_inertia_tensor() * ra_cross_t);
                let k_t = inv_mass_1 + t_angular_a;

                pre_step.effective_mass_t[i] = k_t;
            }

            (k, v_rel)
        };
        pre_step.effective_mass.push(k);

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
        .map(|collision| pre_step_info(entities, collision, dt_s))
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
