use crate::{
    components::{PathFollowing, PathFollowingState, SteeringPathMode, SteeringPlane},
    math::*,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PathDebug {
    pub lookahead_point: Option<Vec3>,
}

pub(crate) fn evaluate(
    position: Vec3,
    current_velocity: Vec3,
    plane: SteeringPlane,
    max_speed: f32,
    max_acceleration: f32,
    config: &PathFollowing,
    state: &mut PathFollowingState,
) -> (LinearIntent, PathDebug) {
    let path = &config.path;
    let len = path.points.len();
    if len == 0 || max_speed <= 0.0 || max_acceleration <= 0.0 {
        state.completed = true;
        return (
            LinearIntent::zero(),
            PathDebug {
                lookahead_point: None,
            },
        );
    }

    if len == 1 {
        state.current_waypoint = 0;
        state.completed = plane.distance(position, path.points[0]) <= config.arrival_tolerance;
        return (
            super::arrive::evaluate(
                position,
                current_velocity,
                path.points[0],
                plane,
                max_speed,
                max_acceleration,
                config.slowing_radius,
                config.arrival_tolerance,
                1.0,
            ),
            PathDebug {
                lookahead_point: Some(path.points[0]),
            },
        );
    }

    if state.direction == 0 {
        state.direction = 1;
    }
    state.current_waypoint = state.current_waypoint.min(len - 1);

    loop {
        let target = plane.align_point(position, path.points[state.current_waypoint]);
        let reached = plane.distance(position, target)
            <= path.waypoint_tolerance.max(config.arrival_tolerance);
        if !reached || state.completed {
            break;
        }

        if !advance_cursor(state, path.mode, len) {
            state.completed = true;
            break;
        }
    }

    let lookahead_point = compute_lookahead(position, path, state, plane);
    let Some(target) = lookahead_point else {
        state.completed = true;
        return (
            LinearIntent::zero(),
            PathDebug {
                lookahead_point: None,
            },
        );
    };

    let is_terminal = matches!(path.mode, SteeringPathMode::Once)
        && (state.completed || state.current_waypoint == len - 1);
    let intent = if is_terminal {
        super::arrive::evaluate(
            position,
            current_velocity,
            target,
            plane,
            max_speed,
            max_acceleration,
            config.slowing_radius,
            config.arrival_tolerance,
            1.0,
        )
    } else {
        super::seek::evaluate(
            position,
            current_velocity,
            target,
            plane,
            max_speed,
            max_acceleration,
        )
    };

    (
        intent,
        PathDebug {
            lookahead_point: Some(target),
        },
    )
}

fn advance_cursor(state: &mut PathFollowingState, mode: SteeringPathMode, len: usize) -> bool {
    match mode {
        SteeringPathMode::Once => {
            if state.current_waypoint + 1 < len {
                state.current_waypoint += 1;
                true
            } else {
                false
            }
        }
        SteeringPathMode::Loop => {
            let next = (state.current_waypoint + 1) % len;
            if next == 0 {
                state.completed_cycles += 1;
            }
            state.current_waypoint = next;
            true
        }
        SteeringPathMode::PingPong => {
            if state.direction >= 0 {
                if state.current_waypoint + 1 < len {
                    state.current_waypoint += 1;
                } else if len > 1 {
                    state.direction = -1;
                    state.completed_cycles += 1;
                    state.current_waypoint = len - 2;
                }
            } else if state.current_waypoint > 0 {
                state.current_waypoint -= 1;
            } else if len > 1 {
                state.direction = 1;
                state.completed_cycles += 1;
                state.current_waypoint = 1;
            }
            true
        }
    }
}

fn compute_lookahead(
    position: Vec3,
    path: &crate::components::SteeringPath,
    state: &PathFollowingState,
    plane: SteeringPlane,
) -> Option<Vec3> {
    let len = path.points.len();
    if len == 0 {
        return None;
    }

    let mut remaining = path.lookahead_distance.max(0.0);
    let mut anchor = position;
    let mut index = state.current_waypoint.min(len - 1);
    let mut direction = state.direction.max(1);
    let mut target = plane.align_point(position, path.points[index]);

    loop {
        let segment = plane.project_vector(target - anchor);
        let segment_length = segment.length();
        if remaining <= segment_length || segment_length <= STEERING_EPSILON {
            if segment_length > STEERING_EPSILON {
                return Some(anchor + segment.normalize() * remaining.min(segment_length));
            }
            return Some(target);
        }

        remaining -= segment_length;
        anchor = target;
        let Some((next_index, next_direction)) = preview_next(index, direction, path.mode, len)
        else {
            return Some(target);
        };
        index = next_index;
        direction = next_direction;
        target = plane.align_point(position, path.points[index]);
    }
}

fn preview_next(
    index: usize,
    direction: i8,
    mode: SteeringPathMode,
    len: usize,
) -> Option<(usize, i8)> {
    match mode {
        SteeringPathMode::Once => (index + 1 < len).then_some((index + 1, 1)),
        SteeringPathMode::Loop => Some(((index + 1) % len, 1)),
        SteeringPathMode::PingPong => {
            if direction >= 0 {
                if index + 1 < len {
                    Some((index + 1, 1))
                } else if len > 1 {
                    Some((len - 2, -1))
                } else {
                    None
                }
            } else if index > 0 {
                Some((index - 1, -1))
            } else if len > 1 {
                Some((1, 1))
            } else {
                None
            }
        }
    }
}
