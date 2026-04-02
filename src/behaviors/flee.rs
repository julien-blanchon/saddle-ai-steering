use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    threat: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    panic_distance: Option<f32>,
) -> LinearIntent {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return LinearIntent::zero();
    }

    let threat = plane.align_point(position, threat);
    let away = plane.project_vector(position - threat);
    let distance = away.length();
    if distance <= STEERING_EPSILON {
        return LinearIntent::zero();
    }

    if panic_distance.is_some_and(|limit| distance > limit) {
        return LinearIntent::zero();
    }

    desired_velocity_intent(
        away.normalize() * max_speed,
        current_velocity,
        plane,
        max_acceleration,
    )
}
