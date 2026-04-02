use crate::{
    components::{SteeringPlane, Wander, WanderState},
    math::*,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WanderDebug {
    pub circle_center: Vec3,
    pub target_point: Vec3,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    fallback_forward: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    config: &Wander,
    state: &mut WanderState,
    delta_seconds: f32,
) -> (LinearIntent, WanderDebug) {
    if max_speed <= 0.0 || max_acceleration <= 0.0 {
        return (
            LinearIntent::zero(),
            WanderDebug {
                circle_center: position,
                target_point: position,
            },
        );
    }

    if !state.initialized {
        state.rng_state = config.seed.max(1);
        state.local_target = match plane {
            SteeringPlane::XY => Vec3::X,
            SteeringPlane::XZ => Vec3::X,
            SteeringPlane::Free3d => Vec3::new(1.0, 0.2, 0.0).normalize(),
        };
        state.initialized = true;
    }

    let heading = {
        let from_velocity = plane.project_vector(current_velocity);
        if from_velocity.length_squared() > STEERING_EPSILON {
            from_velocity.normalize()
        } else {
            let forward = plane.project_vector(fallback_forward);
            if forward.length_squared() > STEERING_EPSILON {
                forward.normalize()
            } else {
                match plane {
                    SteeringPlane::XY | SteeringPlane::XZ => Vec3::X,
                    SteeringPlane::Free3d => Vec3::NEG_Z,
                }
            }
        }
    };

    let jitter_scale = config.jitter_radians_per_second.max(0.0) * delta_seconds.max(0.0);
    let random_offset = match plane {
        SteeringPlane::XY => Vec3::new(
            rand_signed(&mut state.rng_state),
            rand_signed(&mut state.rng_state),
            0.0,
        ),
        SteeringPlane::XZ => Vec3::new(
            rand_signed(&mut state.rng_state),
            0.0,
            rand_signed(&mut state.rng_state),
        ),
        SteeringPlane::Free3d => Vec3::new(
            rand_signed(&mut state.rng_state),
            rand_signed(&mut state.rng_state) * config.vertical_jitter.max(0.0),
            rand_signed(&mut state.rng_state),
        ),
    } * jitter_scale.max(0.0);

    let drifted = plane.project_vector(state.local_target + random_offset);
    state.local_target = if drifted.length_squared() > STEERING_EPSILON {
        drifted.normalize()
    } else {
        heading
    };

    let circle_center = position + heading * config.distance.max(0.0);
    let target_point = circle_center + state.local_target * config.radius.max(0.0);
    (
        super::seek::evaluate(
            position,
            current_velocity,
            target_point,
            plane,
            max_speed,
            max_acceleration,
        ),
        WanderDebug {
            circle_center,
            target_point,
        },
    )
}

fn rand_signed(state: &mut u64) -> f32 {
    let value = next_u32(state) as f32 / u32::MAX as f32;
    value * 2.0 - 1.0
}

fn next_u32(state: &mut u64) -> u32 {
    let mut x = (*state).max(1);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    (x >> 32) as u32
}
