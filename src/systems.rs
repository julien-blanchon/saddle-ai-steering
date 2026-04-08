use crate::{
    behaviors,
    components::*,
    math::{self, LinearIntent},
    resources::{SteeringRuntimeState, SteeringStats},
};
use bevy::prelude::*;

#[derive(Component, Default)]
pub(crate) struct SteeringHistory {
    pub previous_translation: Option<Vec3>,
}

pub(crate) fn activate_runtime(mut runtime: ResMut<SteeringRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(
    mut runtime: ResMut<SteeringRuntimeState>,
    mut outputs: Query<(&mut SteeringOutput, &mut SteeringDiagnostics)>,
) {
    runtime.active = false;
    for (mut output, mut diagnostics) in &mut outputs {
        *output = SteeringOutput::default();
        *diagnostics = SteeringDiagnostics::default();
    }
}

pub(crate) fn runtime_is_active(runtime: Res<SteeringRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn setup_steering_entities(
    mut commands: Commands,
    agents_missing_helpers: Query<Entity, (With<SteeringAgent>, Without<SteeringKinematics>)>,
    agents_missing_output: Query<Entity, (With<SteeringAgent>, Without<SteeringOutput>)>,
    agents_missing_diagnostics: Query<Entity, (With<SteeringAgent>, Without<SteeringDiagnostics>)>,
    agents_missing_history: Query<Entity, (With<SteeringAgent>, Without<SteeringHistory>)>,
    tracked_missing_helpers: Query<
        Entity,
        (With<SteeringTrackedVelocity>, Without<SteeringKinematics>),
    >,
    tracked_missing_history: Query<
        Entity,
        (With<SteeringTrackedVelocity>, Without<SteeringHistory>),
    >,
    wander_missing_state: Query<(Entity, &Wander), (With<SteeringAgent>, Without<WanderState>)>,
    path_missing_state: Query<
        Entity,
        (
            With<SteeringAgent>,
            With<PathFollowing>,
            Without<PathFollowingState>,
        ),
    >,
) {
    for entity in &agents_missing_helpers {
        commands
            .entity(entity)
            .insert(SteeringKinematics::default());
    }
    for entity in &agents_missing_output {
        commands.entity(entity).insert(SteeringOutput::default());
    }
    for entity in &agents_missing_diagnostics {
        commands
            .entity(entity)
            .insert(SteeringDiagnostics::default());
    }
    for entity in &agents_missing_history {
        commands.entity(entity).insert(SteeringHistory::default());
    }
    for entity in &tracked_missing_helpers {
        commands
            .entity(entity)
            .insert(SteeringKinematics::default());
    }
    for entity in &tracked_missing_history {
        commands.entity(entity).insert(SteeringHistory::default());
    }
    for (entity, wander) in &wander_missing_state {
        commands
            .entity(entity)
            .insert(WanderState::from_seed(wander.seed.max(1)));
    }
    for entity in &path_missing_state {
        commands.entity(entity).insert(PathFollowingState {
            direction: 1,
            ..default()
        });
    }
}

pub(crate) fn refresh_tracked_kinematics(
    time: Res<Time>,
    mut tracked: Query<
        (
            &GlobalTransform,
            Option<&SteeringAgent>,
            Option<&SteeringTrackedVelocity>,
            &mut SteeringKinematics,
            &mut SteeringHistory,
        ),
        Or<(With<SteeringAgent>, With<SteeringTrackedVelocity>)>,
    >,
) {
    let delta_seconds = time.delta_secs().max(STEERING_MIN_DT);
    for (global_transform, agent, explicit_tracker, mut kinematics, mut history) in &mut tracked {
        let current_translation = global_transform.translation();
        let should_track = explicit_tracker.is_some()
            || agent.is_some_and(|agent| {
                matches!(
                    agent.velocity_source,
                    SteeringVelocitySource::TransformDelta
                )
            });

        if should_track {
            if let Some(previous_translation) = history.previous_translation {
                kinematics.linear_velocity =
                    (current_translation - previous_translation) / delta_seconds;
            } else {
                kinematics.linear_velocity = Vec3::ZERO;
            }
        }
        history.previous_translation = Some(current_translation);
    }
}

const STEERING_MIN_DT: f32 = 1.0 / 480.0;

#[allow(clippy::too_many_arguments)]
pub(crate) fn clear_custom_behaviors(mut customs: Query<&mut CustomSteeringBehavior>) {
    for mut custom in &mut customs {
        custom.clear();
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_agents(
    time: Res<Time>,
    mut stats: ResMut<SteeringStats>,
    agents: Query<(
        Entity,
        &SteeringAgent,
        &Transform,
        &GlobalTransform,
        &SteeringKinematics,
        (
            Option<&Seek>,
            Option<&Flee>,
            Option<&Arrive>,
            Option<&Pursue>,
            Option<&Evade>,
            Option<&Wander>,
        ),
        (
            Option<&Flocking>,
            Option<&ObstacleAvoidance>,
            Option<&ReciprocalAvoidance>,
            Option<&PathFollowing>,
            Option<&LeaderFollowing>,
            Option<&Formation>,
            Option<&Containment>,
        ),
        Option<&CustomSteeringBehavior>,
    )>,
    mut wander_states: Query<&mut WanderState>,
    mut path_states: Query<&mut PathFollowingState>,
    mut outputs: Query<(&mut SteeringOutput, &mut SteeringDiagnostics)>,
    targets: Query<(&GlobalTransform, Option<&SteeringKinematics>)>,
    obstacles: Query<(Entity, &GlobalTransform, &SteeringObstacle)>,
) {
    let delta_seconds = time.delta_secs().max(STEERING_MIN_DT);
    stats.evaluated_agents = 0;
    stats.active_behaviors = 0;
    stats.obstacle_tests = 0;
    stats.obstacle_hits = 0;
    stats.flock_neighbors = 0;
    stats.crowd_neighbors = 0;
    stats.crowd_conflicts = 0;

    let crowd_neighbors = agents
        .iter()
        .map(
            |(entity, agent, _transform, global_transform, kinematics, _, _, _)| CrowdNeighbor {
                entity,
                position: global_transform.translation(),
                velocity: kinematics.linear_velocity,
                radius: agent.body_radius,
                layers: agent.crowd_layers,
                plane: agent.plane,
            },
        )
        .collect::<Vec<_>>();

    for (
        entity,
        agent,
        transform,
        global_transform,
        kinematics,
        (seek, flee, arrive, pursue, evade, wander),
        (
            flocking,
            obstacle_avoidance,
            reciprocal_avoidance,
            path_following,
            leader_following,
            formation,
            containment,
        ),
        custom_behavior,
    ) in &agents
    {
        stats.evaluated_agents += 1;

        let Ok((mut output, mut diagnostics)) = outputs.get_mut(entity) else {
            continue;
        };
        let position = global_transform.translation();
        let current_velocity = agent.plane.project_vector(kinematics.linear_velocity);
        let fallback_forward = agent.plane.forward_from_transform(transform);
        let mut contributions = Vec::new();
        let mut primary_target = None;
        let mut path_target = None;
        let mut wander_circle_center = None;
        let mut wander_target = None;
        let mut flock_center = None;
        let mut flock_heading = None;
        let mut flock_neighbor_count = 0_usize;
        let mut probe_end = None;
        let mut hit_point = None;
        let mut hit_normal = None;
        let mut avoidance_obstacle = None;
        let mut crowd_avoidance_velocity = None;
        let mut crowd_neighbor_count = 0_usize;
        let mut formation_slot_position = None;

        if let Some(seek) = seek {
            if seek.tuning.enabled {
                if let Some(target) = resolve_target(seek.target, position, &targets, agent.plane) {
                    primary_target = Some(target.position);
                    let intent = behaviors::seek::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                    );
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Seek,
                        seek.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(flee) = flee {
            if flee.tuning.enabled {
                if let Some(target) = resolve_target(flee.target, position, &targets, agent.plane) {
                    primary_target = Some(target.position);
                    let intent = behaviors::flee::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        flee.panic_distance,
                    );
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Flee,
                        flee.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(arrive) = arrive {
            if arrive.tuning.enabled {
                if let Some(target) = resolve_target(arrive.target, position, &targets, agent.plane)
                {
                    primary_target = Some(target.position);
                    let intent = behaviors::arrive::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        arrive.slowing_radius,
                        arrive.arrival_tolerance,
                        arrive.speed_curve_exponent,
                    );
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Arrive,
                        arrive.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(pursue) = pursue {
            if pursue.tuning.enabled {
                if let Some(target) = resolve_target(pursue.target, position, &targets, agent.plane)
                {
                    let (intent, predicted) = behaviors::pursue::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        target.velocity,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        pursue.lead_scale,
                        pursue.max_prediction_time,
                    );
                    primary_target = Some(predicted);
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Pursue,
                        pursue.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(evade) = evade {
            if evade.tuning.enabled {
                if let Some(target) = resolve_target(evade.target, position, &targets, agent.plane)
                {
                    let (intent, predicted) = behaviors::evade::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        target.velocity,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        evade.lead_scale,
                        evade.max_prediction_time,
                        evade.panic_distance,
                    );
                    primary_target = Some(predicted);
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Evade,
                        evade.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(wander) = wander {
            if wander.tuning.enabled {
                let Ok(mut wander_state) = wander_states.get_mut(entity) else {
                    continue;
                };
                let (intent, debug) = behaviors::wander::evaluate(
                    position,
                    current_velocity,
                    fallback_forward,
                    agent.plane,
                    agent.max_speed,
                    agent.max_acceleration,
                    wander,
                    &mut wander_state,
                    delta_seconds,
                );
                wander_circle_center = Some(debug.circle_center);
                wander_target = Some(debug.target_point);
                push_contribution(
                    &mut contributions,
                    SteeringBehaviorKind::Wander,
                    wander.tuning,
                    intent,
                );
            }
        }

        if let Some(flocking) = flocking {
            if flocking.tuning.enabled {
                let (intent, debug) = behaviors::flocking::evaluate(
                    position,
                    current_velocity,
                    agent.plane,
                    agent,
                    flocking,
                    crowd_neighbors
                        .iter()
                        .filter(|neighbor| {
                            neighbor.entity != entity && neighbor.plane == agent.plane
                        })
                        .map(|neighbor| behaviors::flocking::NeighborSample {
                            position: neighbor.position,
                            velocity: neighbor.velocity,
                            layers: neighbor.layers,
                        }),
                );
                flock_center = debug.center;
                flock_heading = debug.heading;
                flock_neighbor_count = debug.neighbor_count;
                stats.flock_neighbors += debug.neighbor_count;
                push_contribution(
                    &mut contributions,
                    SteeringBehaviorKind::Flocking,
                    flocking.tuning,
                    intent,
                );
            }
        }

        if let Some(path_following) = path_following {
            if path_following.tuning.enabled {
                let Ok(mut path_state) = path_states.get_mut(entity) else {
                    continue;
                };
                let (intent, debug) = behaviors::path_following::evaluate(
                    position,
                    current_velocity,
                    agent.plane,
                    agent.max_speed,
                    agent.max_acceleration,
                    path_following,
                    &mut path_state,
                );
                path_target = debug.lookahead_point;
                push_contribution(
                    &mut contributions,
                    SteeringBehaviorKind::PathFollowing,
                    path_following.tuning,
                    intent,
                );
            }
        }

        if let Some(leader_following) = leader_following {
            if leader_following.tuning.enabled {
                if let Some(target) =
                    resolve_target(leader_following.leader, position, &targets, agent.plane)
                {
                    let (intent, debug) = behaviors::leader_following::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        target.velocity,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        leader_following.behind_distance,
                        leader_following.leader_sight_radius,
                        leader_following.slowing_radius,
                        leader_following.arrival_tolerance,
                    );
                    primary_target = Some(debug.behind_point);
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::LeaderFollowing,
                        leader_following.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(formation) = formation {
            if formation.tuning.enabled {
                if let Some(target) =
                    resolve_target(formation.anchor, position, &targets, agent.plane)
                {
                    let (intent, debug) = behaviors::formation::evaluate(
                        position,
                        current_velocity,
                        target.position,
                        target.velocity,
                        agent.plane,
                        agent.max_speed,
                        agent.max_acceleration,
                        formation.slot_offset,
                        formation.slowing_radius,
                        formation.arrival_tolerance,
                    );
                    formation_slot_position = Some(debug.slot_world_position);
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::Formation,
                        formation.tuning,
                        intent,
                    );
                }
            }
        }

        if let Some(containment) = containment {
            if containment.tuning.enabled {
                let intent = behaviors::containment::evaluate(
                    position,
                    current_velocity,
                    containment.center,
                    containment.radius,
                    containment.margin,
                    agent.plane,
                    agent.max_speed,
                    agent.max_acceleration,
                );
                push_contribution(
                    &mut contributions,
                    SteeringBehaviorKind::Containment,
                    containment.tuning,
                    intent,
                );
            }
        }

        if let Some(custom) = custom_behavior {
            for contribution in &custom.contributions {
                push_contribution(
                    &mut contributions,
                    SteeringBehaviorKind::Custom(contribution.name.clone()),
                    contribution.tuning,
                    contribution.intent,
                );
            }
        }

        let mut preview_contributions = contributions.clone();
        let pre_avoidance_velocity = math::compose_contributions(
            agent.composition,
            agent.max_acceleration,
            &mut preview_contributions,
        )
        .desired_velocity;

        if let Some(reciprocal_avoidance) = reciprocal_avoidance {
            if reciprocal_avoidance.tuning.enabled {
                if let Some((intent, debug)) = behaviors::reciprocal_avoidance::evaluate(
                    position,
                    current_velocity,
                    pre_avoidance_velocity,
                    agent.plane,
                    agent,
                    reciprocal_avoidance,
                    crowd_neighbors
                        .iter()
                        .filter(|neighbor| {
                            neighbor.entity != entity && neighbor.plane == agent.plane
                        })
                        .map(|neighbor| behaviors::reciprocal_avoidance::NeighborSample {
                            position: neighbor.position,
                            velocity: neighbor.velocity,
                            radius: neighbor.radius,
                            layers: neighbor.layers,
                        }),
                ) {
                    crowd_avoidance_velocity = Some(debug.adjusted_velocity);
                    crowd_neighbor_count = debug.neighbor_count;
                    stats.crowd_neighbors += debug.neighbor_count;
                    stats.crowd_conflicts += 1;
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::ReciprocalAvoidance,
                        reciprocal_avoidance.tuning,
                        intent,
                    );
                }
            }
        }

        let mut obstacle_preview = contributions.clone();
        let obstacle_preview_velocity = math::compose_contributions(
            agent.composition,
            agent.max_acceleration,
            &mut obstacle_preview,
        )
        .desired_velocity;

        if let Some(avoidance) = obstacle_avoidance {
            if avoidance.tuning.enabled {
                let obstacle_iter = obstacles
                    .iter()
                    .map(|(entity, transform, obstacle)| (entity, transform.affine(), obstacle));
                if let Some((intent, debug)) = behaviors::obstacle_avoidance::evaluate(
                    position,
                    current_velocity,
                    obstacle_preview_velocity,
                    fallback_forward,
                    agent.plane,
                    agent,
                    avoidance,
                    obstacle_iter,
                ) {
                    probe_end = Some(debug.probe_end);
                    hit_point = debug.hit_point;
                    hit_normal = debug.avoidance_direction;
                    avoidance_obstacle = debug.obstacle;
                    stats.obstacle_tests += debug.tests;
                    stats.obstacle_hits += 1;
                    push_contribution(
                        &mut contributions,
                        SteeringBehaviorKind::ObstacleAvoidance,
                        avoidance.tuning,
                        intent,
                    );
                }
            }
        }

        let mut composed = math::compose_contributions(
            agent.composition,
            agent.max_acceleration,
            &mut contributions,
        );

        let active_behavior_count = contributions
            .iter()
            .filter(|contribution| contribution.applied_acceleration.length_squared() > 0.0)
            .count();
        stats.active_behaviors += active_behavior_count;

        if active_behavior_count == 0 && current_velocity.length_squared() > 0.0 {
            let brake =
                math::braking_intent(current_velocity, agent.plane, agent.braking_acceleration);
            push_contribution(
                &mut contributions,
                SteeringBehaviorKind::Brake,
                BehaviorTuning::new(1.0, u8::MAX),
                brake,
            );
            composed = math::compose_contributions(
                agent.composition,
                agent.max_acceleration,
                &mut contributions,
            );
        }

        *output = SteeringOutput {
            linear_acceleration: agent.plane.project_vector(composed.linear_acceleration),
            desired_velocity: agent.plane.project_vector(math::clamp_magnitude(
                composed.desired_velocity,
                agent.max_speed,
            )),
            desired_facing: math::desired_facing(
                agent,
                composed.desired_velocity,
                current_velocity,
                fallback_forward,
            ),
        };

        *diagnostics = SteeringDiagnostics {
            dominant_behavior: math::dominant_behavior(&contributions),
            contributions,
            primary_target,
            path_target,
            wander_circle_center,
            wander_target,
            flock_center,
            flock_heading,
            flock_neighbor_count,
            probe_end,
            avoidance_hit_point: hit_point,
            avoidance_normal: hit_normal,
            avoidance_obstacle,
            crowd_avoidance_velocity,
            crowd_neighbor_count,
            pre_avoidance_velocity,
            formation_slot_position,
        };
    }
}

pub(crate) fn apply_auto_steering(
    time: Res<Time>,
    mut agents: Query<(
        &SteeringAgent,
        &SteeringAutoApply,
        &SteeringOutput,
        &mut SteeringKinematics,
        &mut Transform,
    )>,
) {
    let delta_seconds = time.delta_secs().max(STEERING_MIN_DT);
    for (agent, auto_apply, output, mut kinematics, mut transform) in &mut agents {
        let new_velocity = agent.plane.project_vector(
            kinematics.linear_velocity + output.linear_acceleration * delta_seconds,
        );
        kinematics.linear_velocity = math::clamp_magnitude(new_velocity, agent.max_speed);

        if auto_apply.apply_translation {
            let translated = transform.translation
                + agent.plane.project_vector(kinematics.linear_velocity) * delta_seconds;
            transform.translation = agent
                .plane
                .clamp_translation(transform.translation, translated);
        }

        if auto_apply.apply_facing {
            let Some(desired_facing) = output.desired_facing else {
                continue;
            };
            let Some(target_rotation) = math::target_rotation(agent.plane, desired_facing) else {
                continue;
            };
            let interpolation =
                (agent.alignment.turn_speed_radians * delta_seconds).clamp(0.0, 1.0);
            transform.rotation = transform.rotation.slerp(target_rotation, interpolation);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ResolvedTarget {
    position: Vec3,
    velocity: Vec3,
}

fn resolve_target(
    target: SteeringTarget,
    origin: Vec3,
    targets: &Query<(&GlobalTransform, Option<&SteeringKinematics>)>,
    plane: SteeringPlane,
) -> Option<ResolvedTarget> {
    match target {
        SteeringTarget::Point(point) => Some(ResolvedTarget {
            position: plane.align_point(origin, point),
            velocity: Vec3::ZERO,
        }),
        SteeringTarget::Entity(entity) => {
            let (transform, kinematics) = targets.get(entity).ok()?;
            Some(ResolvedTarget {
                position: plane.align_point(origin, transform.translation()),
                velocity: plane
                    .project_vector(kinematics.copied().unwrap_or_default().linear_velocity),
            })
        }
    }
}

fn push_contribution(
    contributions: &mut Vec<SteeringContribution>,
    behavior: SteeringBehaviorKind,
    tuning: BehaviorTuning,
    intent: LinearIntent,
) {
    if !tuning.enabled || intent.is_zero() {
        return;
    }

    contributions.push(SteeringContribution {
        behavior,
        priority: tuning.priority,
        weight: tuning.weight.max(0.0),
        requested_acceleration: intent.linear_acceleration,
        applied_acceleration: Vec3::ZERO,
        desired_velocity: intent.desired_velocity,
        suppressed: false,
    });
}

#[derive(Clone, Copy, Debug)]
struct CrowdNeighbor {
    entity: Entity,
    position: Vec3,
    velocity: Vec3,
    radius: f32,
    layers: SteeringLayerMask,
    plane: SteeringPlane,
}
