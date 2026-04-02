use crate::components::{
    SteeringAgent, SteeringBehaviorKind, SteeringComposition, SteeringContribution,
    SteeringFacingMode, SteeringPlane,
};
use bevy::prelude::*;

pub(crate) const STEERING_EPSILON: f32 = 1.0e-4;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LinearIntent {
    pub desired_velocity: Vec3,
    pub linear_acceleration: Vec3,
}

impl LinearIntent {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn is_zero(self) -> bool {
        self.desired_velocity.length_squared() <= STEERING_EPSILON
            && self.linear_acceleration.length_squared() <= STEERING_EPSILON
    }
}

pub(crate) fn clamp_magnitude(value: Vec3, max_length: f32) -> Vec3 {
    if max_length <= 0.0 {
        return Vec3::ZERO;
    }
    let length = value.length();
    if length > max_length && length > STEERING_EPSILON {
        value * (max_length / length)
    } else {
        value
    }
}

pub(crate) fn desired_velocity_intent(
    desired_velocity: Vec3,
    current_velocity: Vec3,
    plane: SteeringPlane,
    max_acceleration: f32,
) -> LinearIntent {
    let desired_velocity = plane.project_vector(desired_velocity);
    let linear_acceleration = clamp_magnitude(
        plane.project_vector(desired_velocity - current_velocity),
        max_acceleration,
    );
    LinearIntent {
        desired_velocity,
        linear_acceleration,
    }
}

pub(crate) fn braking_intent(
    current_velocity: Vec3,
    plane: SteeringPlane,
    braking_acceleration: f32,
) -> LinearIntent {
    let projected_velocity = plane.project_vector(current_velocity);
    if projected_velocity.length_squared() <= STEERING_EPSILON || braking_acceleration <= 0.0 {
        return LinearIntent::zero();
    }

    LinearIntent {
        desired_velocity: Vec3::ZERO,
        linear_acceleration: -projected_velocity.normalize() * braking_acceleration,
    }
}

pub(crate) fn predict_target_position(
    agent_position: Vec3,
    target_position: Vec3,
    target_velocity: Vec3,
    plane: SteeringPlane,
    agent_max_speed: f32,
    lead_scale: f32,
    max_prediction_time: f32,
) -> Vec3 {
    let offset = plane.project_vector(target_position - agent_position);
    let distance = offset.length();
    let target_speed = plane.project_vector(target_velocity).length();
    let speed_basis = (agent_max_speed.max(0.0) + target_speed).max(0.1);
    let prediction_time =
        ((distance / speed_basis) * lead_scale.max(0.0)).clamp(0.0, max_prediction_time.max(0.0));
    plane.align_point(
        agent_position,
        target_position + plane.project_vector(target_velocity) * prediction_time,
    )
}

pub(crate) fn compose_contributions(
    mode: SteeringComposition,
    max_acceleration: f32,
    contributions: &mut [SteeringContribution],
) -> LinearIntent {
    match mode {
        SteeringComposition::WeightedBlend => compose_weighted(max_acceleration, contributions),
        SteeringComposition::PrioritizedAccumulation => {
            compose_prioritized(max_acceleration, contributions)
        }
    }
}

fn compose_weighted(
    max_acceleration: f32,
    contributions: &mut [SteeringContribution],
) -> LinearIntent {
    let mut sum_acceleration = Vec3::ZERO;
    let mut sum_velocity = Vec3::ZERO;
    let mut velocity_weight = 0.0;
    for contribution in contributions.iter_mut() {
        let requested = contribution.requested_acceleration * contribution.weight.max(0.0);
        contribution.applied_acceleration = requested;
        contribution.suppressed = requested.length_squared() <= STEERING_EPSILON;
        sum_acceleration += requested;
        let velocity_weight_step = contribution.weight.max(0.0);
        sum_velocity += contribution.desired_velocity * velocity_weight_step;
        velocity_weight += velocity_weight_step;
    }

    let clamped = clamp_magnitude(sum_acceleration, max_acceleration);
    let scale = if sum_acceleration.length_squared() > STEERING_EPSILON {
        clamped.length() / sum_acceleration.length()
    } else {
        1.0
    };
    for contribution in contributions.iter_mut() {
        contribution.applied_acceleration *= scale;
        if contribution.applied_acceleration.length_squared() <= STEERING_EPSILON {
            contribution.suppressed = true;
        }
    }

    LinearIntent {
        desired_velocity: if velocity_weight > 0.0 {
            sum_velocity / velocity_weight
        } else {
            Vec3::ZERO
        },
        linear_acceleration: clamped,
    }
}

fn compose_prioritized(
    max_acceleration: f32,
    contributions: &mut [SteeringContribution],
) -> LinearIntent {
    contributions.sort_by_key(|contribution| contribution.priority);

    let mut remaining = max_acceleration.max(0.0);
    let mut sum_acceleration = Vec3::ZERO;
    let mut sum_velocity = Vec3::ZERO;
    let mut velocity_weight = 0.0;

    for contribution in contributions.iter_mut() {
        let weighted = contribution.requested_acceleration * contribution.weight.max(0.0);
        let magnitude = weighted.length();
        if magnitude <= STEERING_EPSILON || remaining <= STEERING_EPSILON {
            contribution.applied_acceleration = Vec3::ZERO;
            contribution.suppressed = true;
            continue;
        }

        let applied = if magnitude <= remaining {
            weighted
        } else {
            weighted.normalize() * remaining
        };
        remaining = (remaining - applied.length()).max(0.0);
        sum_acceleration += applied;
        contribution.applied_acceleration = applied;
        contribution.suppressed = applied.length() + STEERING_EPSILON < magnitude;

        let velocity_step = applied.length();
        sum_velocity += contribution.desired_velocity * velocity_step;
        velocity_weight += velocity_step;
    }

    LinearIntent {
        desired_velocity: if velocity_weight > 0.0 {
            sum_velocity / velocity_weight
        } else {
            Vec3::ZERO
        },
        linear_acceleration: sum_acceleration,
    }
}

pub(crate) fn dominant_behavior(
    contributions: &[SteeringContribution],
) -> Option<SteeringBehaviorKind> {
    contributions
        .iter()
        .filter(|contribution| {
            contribution.applied_acceleration.length_squared() > STEERING_EPSILON
        })
        .max_by(|left, right| {
            left.applied_acceleration
                .length_squared()
                .total_cmp(&right.applied_acceleration.length_squared())
                .then_with(|| right.priority.cmp(&left.priority))
        })
        .map(|contribution| contribution.behavior)
}

pub(crate) fn desired_facing(
    agent: &SteeringAgent,
    output_velocity: Vec3,
    current_velocity: Vec3,
    fallback_forward: Vec3,
) -> Option<Vec3> {
    let target = match agent.alignment.mode {
        SteeringFacingMode::None => Vec3::ZERO,
        SteeringFacingMode::Velocity => current_velocity,
        SteeringFacingMode::DesiredVelocity | SteeringFacingMode::DesiredHeading => output_velocity,
    };

    let target = agent.plane.project_vector(target);
    if target.length() >= agent.alignment.min_speed {
        return Some(target.normalize());
    }

    let fallback_forward = agent.plane.project_vector(fallback_forward);
    (fallback_forward.length() > STEERING_EPSILON).then_some(fallback_forward.normalize())
}

pub(crate) fn target_rotation(plane: SteeringPlane, direction: Vec3) -> Option<Quat> {
    if direction.length_squared() <= STEERING_EPSILON {
        return None;
    }

    let direction = plane.project_vector(direction).normalize_or_zero();
    if direction.length_squared() <= STEERING_EPSILON {
        return None;
    }

    Some(match plane {
        SteeringPlane::XY => Quat::from_rotation_z(direction.y.atan2(direction.x)),
        SteeringPlane::XZ | SteeringPlane::Free3d => {
            Transform::from_translation(Vec3::ZERO)
                .looking_to(direction, plane.up())
                .rotation
        }
    })
}
