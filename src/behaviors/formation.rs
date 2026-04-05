use crate::{components::SteeringPlane, math::*};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FormationDebug {
    pub slot_world_position: Vec3,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    anchor_position: Vec3,
    anchor_velocity: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    slot_offset: Vec3,
    slowing_radius: f32,
    arrival_tolerance: f32,
) -> (LinearIntent, FormationDebug) {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return (
            LinearIntent::zero(),
            FormationDebug {
                slot_world_position: anchor_position,
            },
        );
    }

    let anchor_forward = {
        let velocity = plane.project_vector(anchor_velocity);
        if velocity.length_squared() > STEERING_EPSILON {
            velocity.normalize()
        } else {
            Vec3::Z
        }
    };

    let rotation = rotation_from_forward(plane, anchor_forward);
    let world_offset = rotation * slot_offset;
    let slot_world_position = plane.align_point(position, anchor_position + world_offset);

    let intent = super::arrive::evaluate(
        position,
        current_velocity,
        slot_world_position,
        plane,
        max_speed,
        max_acceleration,
        slowing_radius,
        arrival_tolerance,
        1.0,
    );

    (
        intent,
        FormationDebug {
            slot_world_position,
        },
    )
}

fn rotation_from_forward(plane: SteeringPlane, forward: Vec3) -> Quat {
    let forward = plane.project_vector(forward).normalize_or_zero();
    if forward.length_squared() <= STEERING_EPSILON {
        return Quat::IDENTITY;
    }

    match plane {
        SteeringPlane::XY => Quat::from_rotation_z(forward.y.atan2(forward.x)),
        SteeringPlane::XZ | SteeringPlane::Free3d => Quat::from_rotation_arc(Vec3::Z, forward),
    }
}
