use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    target_position: Vec3,
    target_velocity: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    lead_scale: f32,
    max_prediction_time: f32,
    panic_distance: Option<f32>,
) -> (LinearIntent, Vec3) {
    let predicted = predict_target_position(
        position,
        target_position,
        target_velocity,
        plane,
        max_speed,
        lead_scale,
        max_prediction_time,
    );

    (
        super::flee::evaluate(
            position,
            current_velocity,
            predicted,
            plane,
            max_speed,
            max_acceleration,
            panic_distance,
        ),
        predicted,
    )
}
