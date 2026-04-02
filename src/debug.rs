use crate::{components::*, resources::SteeringDebugSettings};
use bevy::{
    color::palettes::css::{AQUA, CRIMSON, GOLD, GREEN, HOT_PINK, ORANGE, WHITE, YELLOW},
    gizmos::config::GizmoConfigGroup,
    prelude::*,
};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct SteeringDebugGizmos;

pub(crate) fn debug_enabled(settings: Res<SteeringDebugSettings>) -> bool {
    settings.enabled
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_steering_debug(
    settings: Res<SteeringDebugSettings>,
    mut gizmos: Gizmos<SteeringDebugGizmos>,
    agents: Query<(
        &GlobalTransform,
        &SteeringAgent,
        &SteeringKinematics,
        &SteeringOutput,
        &SteeringDiagnostics,
        Option<&Arrive>,
        Option<&Wander>,
        Option<&PathFollowing>,
        Option<&SteeringDebugAgent>,
    )>,
    obstacles: Query<(&GlobalTransform, &SteeringObstacle)>,
) {
    for (
        global_transform,
        agent,
        kinematics,
        output,
        diagnostics,
        arrive,
        wander,
        path_following,
        debug_agent,
    ) in &agents
    {
        if debug_agent.is_some_and(|agent| !agent.enabled) {
            continue;
        }

        let position = global_transform.translation();
        if settings.draw_velocity {
            gizmos.arrow(
                position,
                position + agent.plane.project_vector(kinematics.linear_velocity),
                GREEN,
            );
        }
        if settings.draw_output {
            gizmos.arrow(
                position,
                position + agent.plane.project_vector(output.linear_acceleration),
                CRIMSON,
            );
            gizmos.arrow(
                position,
                position + agent.plane.project_vector(output.desired_velocity),
                AQUA,
            );
        }
        if settings.draw_targets {
            if let Some(target) = diagnostics.primary_target {
                gizmos.cross(target, 0.25, GOLD);
                gizmos.line(position, target, GOLD);
            }
            if let Some(target) = diagnostics.path_target {
                gizmos.cross(target, 0.2, HOT_PINK);
                gizmos.line(position, target, HOT_PINK);
            }
        }
        if settings.draw_arrival_radii {
            if let Some(arrive) = arrive {
                if let Some(target) = diagnostics.primary_target {
                    draw_plane_circle(
                        &mut gizmos,
                        agent.plane,
                        target,
                        arrive.slowing_radius,
                        ORANGE,
                    );
                }
            } else if let Some(path_following) = path_following {
                if matches!(path_following.path.mode, SteeringPathMode::Once) {
                    if let Some(target) = path_following.path.points.last().copied() {
                        draw_plane_circle(
                            &mut gizmos,
                            agent.plane,
                            target,
                            path_following.slowing_radius,
                            ORANGE,
                        );
                    }
                }
            }
        }
        if settings.draw_wander {
            if let (Some(circle_center), Some(wander)) = (diagnostics.wander_circle_center, wander)
            {
                draw_plane_circle(
                    &mut gizmos,
                    agent.plane,
                    circle_center,
                    wander.radius,
                    WHITE,
                );
                if let Some(target) = diagnostics.wander_target {
                    gizmos.cross(target, 0.18, WHITE);
                    gizmos.line(circle_center, target, WHITE);
                }
            }
        }
        if settings.draw_probes {
            if let Some(probe_end) = diagnostics.probe_end {
                gizmos.line(position, probe_end, YELLOW);
            }
            if let Some(hit_point) = diagnostics.avoidance_hit_point {
                gizmos.cross(hit_point, 0.18, CRIMSON);
            }
            if let (Some(hit_point), Some(normal)) = (
                diagnostics.avoidance_hit_point,
                diagnostics.avoidance_normal,
            ) {
                gizmos.arrow(hit_point, hit_point + normal, CRIMSON);
            }
        }
        if settings.draw_paths {
            if let Some(path_following) = path_following {
                for window in path_following.path.points.windows(2) {
                    gizmos.line(window[0], window[1], HOT_PINK);
                }
                if matches!(path_following.path.mode, SteeringPathMode::Loop)
                    && path_following.path.points.len() > 2
                {
                    let first = path_following
                        .path
                        .points
                        .first()
                        .copied()
                        .unwrap_or_default();
                    let last = path_following
                        .path
                        .points
                        .last()
                        .copied()
                        .unwrap_or_default();
                    gizmos.line(last, first, HOT_PINK);
                }
                for point in &path_following.path.points {
                    gizmos.cross(*point, 0.12, HOT_PINK);
                }
            }
        }
    }

    if settings.draw_obstacles {
        for (global_transform, obstacle) in &obstacles {
            match obstacle.shape {
                SteeringObstacleShape::Sphere { radius } => {
                    gizmos.sphere(global_transform.translation(), radius, WHITE);
                }
                SteeringObstacleShape::Aabb { half_extents } => {
                    gizmos.cube(
                        Transform::from_translation(global_transform.translation())
                            .with_rotation(global_transform.compute_transform().rotation)
                            .with_scale(half_extents * 2.0),
                        WHITE,
                    );
                }
            }
        }
    }
}

fn draw_plane_circle(
    gizmos: &mut Gizmos<SteeringDebugGizmos>,
    plane: SteeringPlane,
    center: Vec3,
    radius: f32,
    color: impl Into<Color>,
) {
    let color = color.into();
    match plane {
        SteeringPlane::XY => {
            gizmos.circle(Isometry3d::new(center, Quat::IDENTITY), radius, color);
        }
        SteeringPlane::XZ => {
            gizmos.circle(
                Isometry3d::new(center, Quat::from_rotation_arc(Vec3::Z, Vec3::Y)),
                radius,
                color,
            );
        }
        SteeringPlane::Free3d => {
            gizmos.sphere(center, radius, color);
        }
    }
}
