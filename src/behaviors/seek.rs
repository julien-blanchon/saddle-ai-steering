use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    target: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
) -> LinearIntent {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return LinearIntent::zero();
    }

    let target = plane.align_point(position, target);
    let to_target = plane.project_vector(target - position);
    if to_target.length_squared() <= STEERING_EPSILON {
        return LinearIntent::zero();
    }

    desired_velocity_intent(
        to_target.normalize() * max_speed,
        current_velocity,
        plane,
        max_acceleration,
    )
}
