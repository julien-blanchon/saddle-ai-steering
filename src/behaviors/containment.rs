use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    center: Vec3,
    radius: f32,
    margin: f32,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
) -> LinearIntent {
    if max_speed <= 0.0 || max_acceleration <= 0.0 || radius <= 0.0 {
        return LinearIntent::zero();
    }

    let center = plane.align_point(position, center);
    let offset = plane.project_vector(position - center);
    let distance = offset.length();
    let safe_radius = (radius - margin.max(0.0)).max(0.0);

    if distance <= safe_radius {
        return LinearIntent::zero();
    }

    let urgency = if radius > safe_radius {
        ((distance - safe_radius) / (radius - safe_radius)).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let desired_velocity = if distance > STEERING_EPSILON {
        -offset.normalize() * max_speed * urgency
    } else {
        Vec3::ZERO
    };

    desired_velocity_intent(desired_velocity, current_velocity, plane, max_acceleration)
}
