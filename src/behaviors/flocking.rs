use crate::{
    components::{Flocking, SteeringAgent, SteeringLayerMask, SteeringPlane},
    math::{LinearIntent, STEERING_EPSILON, clamp_magnitude, desired_velocity_intent},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NeighborSample {
    pub position: Vec3,
    pub velocity: Vec3,
    pub layers: SteeringLayerMask,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlockingDebug {
    pub center: Option<Vec3>,
    pub heading: Option<Vec3>,
    pub neighbor_count: usize,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    plane: SteeringPlane,
    agent: &SteeringAgent,
    config: &Flocking,
    neighbors: impl IntoIterator<Item = NeighborSample>,
) -> (LinearIntent, FlockingDebug) {
    let mut separation = Vec3::ZERO;
    let mut alignment = Vec3::ZERO;
    let mut cohesion_sum = Vec3::ZERO;
    let mut cohesion_count = 0_usize;
    let mut alignment_count = 0_usize;
    let mut neighbor_count = 0_usize;

    let max_neighbors = config.max_neighbors.max(1);
    let separation_radius = config.separation_radius.max(0.0);
    let alignment_radius = config.alignment_radius.max(separation_radius);
    let cohesion_radius = config.cohesion_radius.max(alignment_radius);

    for neighbor in neighbors {
        if neighbor_count >= max_neighbors {
            break;
        }
        if !config.layers.contains(neighbor.layers) {
            continue;
        }

        let offset = plane.project_vector(neighbor.position - position);
        let distance = offset.length();
        if distance <= STEERING_EPSILON || distance > cohesion_radius {
            continue;
        }

        neighbor_count += 1;

        if distance <= separation_radius {
            separation -= offset / distance.max(STEERING_EPSILON).powi(2);
        }

        if distance <= alignment_radius {
            let velocity = plane.project_vector(neighbor.velocity);
            if velocity.length_squared() > STEERING_EPSILON {
                alignment += velocity.normalize();
                alignment_count += 1;
            }
        }

        cohesion_sum += neighbor.position;
        cohesion_count += 1;
    }

    let center = (cohesion_count > 0).then_some(cohesion_sum / cohesion_count as f32);
    let alignment_heading = if alignment_count > 0 {
        Some((alignment / alignment_count as f32).normalize_or_zero())
    } else {
        None
    };

    if neighbor_count == 0 {
        return (
            LinearIntent::zero(),
            FlockingDebug {
                center: None,
                heading: None,
                neighbor_count: 0,
            },
        );
    }

    let mut desired = Vec3::ZERO;
    if separation.length_squared() > STEERING_EPSILON {
        desired += separation.normalize() * agent.max_speed * config.separation_weight.max(0.0);
    }
    if let Some(alignment_heading) = alignment_heading {
        desired += alignment_heading * agent.max_speed * config.alignment_weight.max(0.0);
    }
    if let Some(center) = center {
        let cohesion = plane.project_vector(center - position).normalize_or_zero();
        desired += cohesion * agent.max_speed * config.cohesion_weight.max(0.0);
    }

    let desired_velocity = clamp_magnitude(plane.project_vector(desired), agent.max_speed);

    (
        desired_velocity_intent(
            desired_velocity,
            current_velocity,
            plane,
            agent.max_acceleration,
        ),
        FlockingDebug {
            center,
            heading: alignment_heading,
            neighbor_count,
        },
    )
}
