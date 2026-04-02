use crate::{
    components::{ObstacleAvoidance, SteeringObstacle, SteeringObstacleShape, SteeringPlane},
    math::*,
};
use bevy::{math::Affine3A, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AvoidanceDebug {
    pub probe_end: Vec3,
    pub hit_point: Option<Vec3>,
    pub avoidance_direction: Option<Vec3>,
    pub obstacle: Option<Entity>,
    pub tests: usize,
}

pub(crate) fn evaluate<'a>(
    position: Vec3,
    current_velocity: Vec3,
    preferred_velocity: Vec3,
    fallback_forward: Vec3,
    plane: SteeringPlane,
    agent: &crate::components::SteeringAgent,
    config: &ObstacleAvoidance,
    obstacles: impl IntoIterator<Item = (Entity, Affine3A, &'a SteeringObstacle)>,
) -> Option<(LinearIntent, AvoidanceDebug)> {
    let heading = choose_probe_heading(
        plane,
        preferred_velocity,
        current_velocity,
        fallback_forward,
    )?;
    let speed = plane.project_vector(current_velocity).length();
    let speed_t = if agent.max_speed <= STEERING_EPSILON {
        0.0
    } else {
        (speed / agent.max_speed).clamp(0.0, 1.0)
    };
    let lookahead =
        config.min_lookahead + (config.max_lookahead - config.min_lookahead).max(0.0) * speed_t;
    let probe_end = position + heading * lookahead.max(0.0);
    let inflation = agent.body_radius.max(0.0) + config.probe_radius.max(0.0);

    let mut best_hit: Option<(Entity, f32, Vec3, Vec3)> = None;
    let mut tests = 0_usize;

    for (entity, obstacle_transform, obstacle) in obstacles {
        if !config.layers.contains(obstacle.layers) {
            continue;
        }
        tests += 1;
        let hit = match &obstacle.shape {
            SteeringObstacleShape::Sphere { radius } => sphere_hit(
                position,
                probe_end,
                obstacle_transform.translation.into(),
                radius + inflation,
            ),
            SteeringObstacleShape::Aabb { half_extents } => aabb_hit(
                position,
                probe_end,
                obstacle_transform,
                *half_extents + Vec3::splat(inflation),
            ),
        };

        let Some((distance_ratio, hit_point, normal)) = hit else {
            continue;
        };

        let replace = best_hit.is_none_or(|(_, best_ratio, _, _)| distance_ratio < best_ratio);
        if replace {
            best_hit = Some((
                entity,
                distance_ratio,
                hit_point,
                plane.project_vector(normal),
            ));
        }
    }

    let (obstacle, distance_ratio, hit_point, normal) = best_hit?;

    let threat = (1.0 - distance_ratio).clamp(0.0, 1.0);
    let forward = plane.project_vector(heading);
    let adjusted_normal = if normal.length_squared() > STEERING_EPSILON {
        plane.project_vector(normal).normalize_or_zero()
    } else {
        pick_fallback_normal(plane, forward)
    };
    let avoidance_normal = if forward.length_squared() > STEERING_EPSILON
        && adjusted_normal.length_squared() > STEERING_EPSILON
        && forward.dot(adjusted_normal) < -0.25
    {
        pick_fallback_normal(plane, forward)
    } else {
        adjusted_normal
    };
    let lateral_weight = config.lateral_weight.max(0.0);
    let braking_weight = config.braking_weight.max(0.0);
    let requested = clamp_magnitude(
        (avoidance_normal * lateral_weight - forward * braking_weight)
            * agent.max_acceleration
            * threat,
        agent.max_acceleration,
    );
    let desired_direction = plane
        .project_vector(forward + avoidance_normal * lateral_weight)
        .normalize_or_zero();
    let desired_velocity = desired_direction * agent.max_speed * (1.0 - threat * 0.35);

    Some((
        LinearIntent {
            desired_velocity,
            linear_acceleration: requested,
        },
        AvoidanceDebug {
            probe_end,
            hit_point: Some(hit_point),
            avoidance_direction: Some(avoidance_normal),
            obstacle: Some(obstacle),
            tests,
        },
    ))
}

fn choose_probe_heading(
    plane: SteeringPlane,
    preferred_velocity: Vec3,
    current_velocity: Vec3,
    fallback_forward: Vec3,
) -> Option<Vec3> {
    let preferred_velocity = plane.project_vector(preferred_velocity);
    if preferred_velocity.length_squared() > STEERING_EPSILON {
        return Some(preferred_velocity.normalize());
    }

    let current_velocity = plane.project_vector(current_velocity);
    if current_velocity.length_squared() > STEERING_EPSILON {
        return Some(current_velocity.normalize());
    }

    let fallback_forward = plane.project_vector(fallback_forward);
    if fallback_forward.length_squared() > STEERING_EPSILON {
        Some(fallback_forward.normalize())
    } else {
        None
    }
}

fn pick_fallback_normal(plane: SteeringPlane, heading: Vec3) -> Vec3 {
    match plane {
        SteeringPlane::XY => Vec3::new(-heading.y, heading.x, 0.0).normalize_or_zero(),
        SteeringPlane::XZ => Vec3::new(-heading.z, 0.0, heading.x).normalize_or_zero(),
        SteeringPlane::Free3d => {
            let candidate = heading.cross(Vec3::Y);
            if candidate.length_squared() > STEERING_EPSILON {
                candidate.normalize()
            } else {
                heading.cross(Vec3::X).normalize_or_zero()
            }
        }
    }
}

fn sphere_hit(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> Option<(f32, Vec3, Vec3)> {
    let segment = end - start;
    let to_center = start - center;
    let a = segment.length_squared();
    if a <= STEERING_EPSILON {
        return None;
    }

    let c = to_center.length_squared() - radius * radius;
    if c <= 0.0 {
        let normal = (start - center).normalize_or_zero();
        return Some((0.0, start, normal));
    }

    let b = 2.0 * to_center.dot(segment);
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);
    let t = [t1, t2]
        .into_iter()
        .filter(|value| (0.0..=1.0).contains(value))
        .fold(None, |best, value| {
            best.map_or(Some(value), |current: f32| Some(current.min(value)))
        })?;

    let hit_point = start + segment * t;
    let normal = (hit_point - center).normalize_or_zero();
    Some((t, hit_point, normal))
}

fn aabb_hit(
    start: Vec3,
    end: Vec3,
    transform: Affine3A,
    half_extents: Vec3,
) -> Option<(f32, Vec3, Vec3)> {
    let inverse = transform.inverse();
    let local_start = inverse.transform_point3(start);
    let local_end = inverse.transform_point3(end);
    let direction = local_end - local_start;

    let inside = local_start.abs().cmple(half_extents).all();
    if inside {
        let delta = half_extents - local_start.abs();
        let (axis, sign) = if delta.x <= delta.y && delta.x <= delta.z {
            (Vec3::X, local_start.x.signum())
        } else if delta.y <= delta.z {
            (Vec3::Y, local_start.y.signum())
        } else {
            (Vec3::Z, local_start.z.signum())
        };
        let local_normal = axis * sign.max(1.0);
        let world_normal = transform
            .transform_vector3(local_normal)
            .normalize_or_zero();
        return Some((0.0, start, world_normal));
    }

    let mut t_min = 0.0_f32;
    let mut t_max = 1.0_f32;
    let mut normal = Vec3::ZERO;

    for axis_index in 0..3 {
        let start_axis = local_start[axis_index];
        let direction_axis = direction[axis_index];
        let min_axis = -half_extents[axis_index];
        let max_axis = half_extents[axis_index];

        if direction_axis.abs() <= STEERING_EPSILON {
            if start_axis < min_axis || start_axis > max_axis {
                return None;
            }
            continue;
        }

        let inverse_direction = 1.0 / direction_axis;
        let mut near_t = (min_axis - start_axis) * inverse_direction;
        let mut far_t = (max_axis - start_axis) * inverse_direction;
        let mut near_normal = Vec3::ZERO;
        near_normal[axis_index] = -1.0;
        if near_t > far_t {
            std::mem::swap(&mut near_t, &mut far_t);
            near_normal[axis_index] = 1.0;
        }

        if near_t > t_min {
            t_min = near_t;
            normal = near_normal;
        }
        t_max = t_max.min(far_t);
        if t_min > t_max {
            return None;
        }
    }

    if !(0.0..=1.0).contains(&t_min) {
        return None;
    }

    let hit_point = start + (end - start) * t_min;
    let world_normal = transform.transform_vector3(normal).normalize_or_zero();
    Some((t_min, hit_point, world_normal))
}
