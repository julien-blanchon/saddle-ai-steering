use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LeaderDebug {
    pub behind_point: Vec3,
    pub is_ahead: bool,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    leader_position: Vec3,
    leader_velocity: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    behind_distance: f32,
    leader_sight_radius: f32,
    slowing_radius: f32,
    arrival_tolerance: f32,
) -> (LinearIntent, LeaderDebug) {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return (
            LinearIntent::zero(),
            LeaderDebug {
                behind_point: leader_position,
                is_ahead: false,
            },
        );
    }

    let leader_forward = {
        let velocity = plane.project_vector(leader_velocity);
        if velocity.length_squared() > STEERING_EPSILON {
            velocity.normalize()
        } else {
            Vec3::Z
        }
    };

    let behind_point = plane.align_point(
        position,
        leader_position - leader_forward * behind_distance.max(0.0),
    );

    let to_agent = plane.project_vector(position - leader_position);
    let ahead_dot = to_agent.dot(leader_forward);
    let lateral_distance = (to_agent - leader_forward * ahead_dot).length();
    let is_ahead = ahead_dot > 0.0
        && lateral_distance < leader_sight_radius.max(0.0)
        && ahead_dot < leader_sight_radius.max(0.0);

    let intent = if is_ahead {
        let evade_point = leader_position + leader_forward * behind_distance.max(0.0) * 0.5;
        super::flee::evaluate(
            position,
            current_velocity,
            evade_point,
            plane,
            max_speed,
            max_acceleration,
            None,
        )
    } else {
        super::arrive::evaluate(
            position,
            current_velocity,
            behind_point,
            plane,
            max_speed,
            max_acceleration,
            slowing_radius,
            arrival_tolerance,
            1.0,
        )
    };

    (
        intent,
        LeaderDebug {
            behind_point,
            is_ahead,
        },
    )
}
