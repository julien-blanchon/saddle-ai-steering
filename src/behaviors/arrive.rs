use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    target: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    slowing_radius: f32,
    arrival_tolerance: f32,
    speed_curve_exponent: f32,
) -> LinearIntent {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return LinearIntent::zero();
    }

    let target = plane.align_point(position, target);
    let to_target = plane.project_vector(target - position);
    let distance = to_target.length();
    if distance <= arrival_tolerance.max(0.0) {
        return LinearIntent::zero();
    }

    let direction = to_target.normalize();
    let desired_speed = if slowing_radius <= arrival_tolerance || distance >= slowing_radius {
        max_speed
    } else {
        let t = ((distance - arrival_tolerance).max(0.0)
            / (slowing_radius - arrival_tolerance).max(STEERING_EPSILON))
        .clamp(0.0, 1.0);
        max_speed * t.powf(speed_curve_exponent.max(0.1))
    };

    desired_velocity_intent(
        direction * desired_speed,
        current_velocity,
        plane,
        max_acceleration,
    )
}
