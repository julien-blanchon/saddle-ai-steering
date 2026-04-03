use crate::{
    components::{ReciprocalAvoidance, SteeringAgent, SteeringLayerMask, SteeringPlane},
    math::{clamp_magnitude, desired_velocity_intent, LinearIntent, STEERING_EPSILON},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NeighborSample {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub layers: SteeringLayerMask,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ReciprocalAvoidanceDebug {
    pub adjusted_velocity: Vec3,
    pub neighbor_count: usize,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    preferred_velocity: Vec3,
    plane: SteeringPlane,
    agent: &SteeringAgent,
    config: &ReciprocalAvoidance,
    neighbors: impl IntoIterator<Item = NeighborSample>,
) -> Option<(LinearIntent, ReciprocalAvoidanceDebug)> {
    let preferred_velocity = plane.project_vector(preferred_velocity);
    if preferred_velocity.length_squared() <= STEERING_EPSILON {
        return None;
    }
    let heading = preferred_velocity.normalize();
    let preferred_speed = preferred_velocity.length();

    let mut adjustment = Vec3::ZERO;
    let mut neighbor_count = 0_usize;
    let max_neighbors = config.max_neighbors.max(1);
    let horizon = config.time_horizon.max(0.05);
    let neighbor_distance = config.neighbor_distance.max(0.0);

    for neighbor in neighbors {
        if neighbor_count >= max_neighbors {
            break;
        }
        if !config.layers.contains(neighbor.layers) {
            continue;
        }

        let offset = plane.project_vector(neighbor.position - position);
        let distance = offset.length();
        if distance <= STEERING_EPSILON || distance > neighbor_distance {
            continue;
        }

        let combined_radius = agent.body_radius.max(0.0)
            + neighbor.radius.max(0.0)
            + config.comfort_distance.max(0.0);
        let relative_velocity = preferred_velocity - plane.project_vector(neighbor.velocity);
        let relative_speed_sq = relative_velocity.length_squared();
        let time_to_closest = if relative_speed_sq > STEERING_EPSILON {
            (-offset.dot(relative_velocity) / relative_speed_sq).clamp(0.0, horizon)
        } else {
            0.0
        };
        let closest_offset = offset + relative_velocity * time_to_closest;
        let closest_distance = closest_offset.length();
        let overlap_now = distance < combined_radius;
        let ahead_distance = offset.dot(heading);
        let lateral_offset = plane.project_vector(offset - heading * ahead_distance);
        let ahead_conflict = ahead_distance > 0.0
            && ahead_distance <= preferred_speed * horizon + combined_radius
            && lateral_offset.length() <= combined_radius;
        let will_collide = overlap_now || closest_distance < combined_radius || ahead_conflict;
        if !will_collide {
            continue;
        }

        neighbor_count += 1;
        let away = if overlap_now {
            (-offset).normalize_or_zero()
        } else {
            (-closest_offset).normalize_or_zero()
        };
        let side = perpendicular(plane, preferred_velocity, away);
        let urgency = if overlap_now {
            1.0 + ((combined_radius - distance) / combined_radius.max(0.001)).clamp(0.0, 1.0)
        } else {
            (1.0 - time_to_closest / horizon).clamp(0.0, 1.0)
        };
        let push = away + side * config.side_bias;
        adjustment += push.normalize_or_zero() * agent.max_speed * urgency * 0.5;
    }

    if neighbor_count == 0 {
        return None;
    }

    let adjusted_velocity = clamp_magnitude(preferred_velocity + adjustment, agent.max_speed);

    Some((
        desired_velocity_intent(
            adjusted_velocity,
            current_velocity,
            plane,
            agent.max_acceleration,
        ),
        ReciprocalAvoidanceDebug {
            adjusted_velocity,
            neighbor_count,
        },
    ))
}

fn perpendicular(plane: SteeringPlane, preferred_velocity: Vec3, away: Vec3) -> Vec3 {
    let reference = if preferred_velocity.length_squared() > STEERING_EPSILON {
        preferred_velocity.normalize()
    } else {
        away
    };

    match plane {
        SteeringPlane::XY => Vec3::new(-reference.y, reference.x, 0.0).normalize_or_zero(),
        SteeringPlane::XZ => Vec3::new(-reference.z, 0.0, reference.x).normalize_or_zero(),
        SteeringPlane::Free3d => {
            let candidate = plane.up().cross(reference);
            if candidate.length_squared() > STEERING_EPSILON {
                candidate.normalize()
            } else {
                away.cross(Vec3::Y).normalize_or_zero()
            }
        }
    }
}
