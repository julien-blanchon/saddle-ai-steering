use super::{
    behaviors,
    components::*,
    math::{self, STEERING_EPSILON},
};
use bevy::math::Affine3A;
use bevy::prelude::*;

#[test]
fn seek_points_toward_target_and_clamps_acceleration() {
    let intent = behaviors::seek::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        SteeringPlane::XZ,
        8.0,
        4.0,
    );
    assert!((intent.desired_velocity.x - 8.0).abs() < 0.001);
    assert!((intent.linear_acceleration.length() - 4.0).abs() < 0.001);
}

#[test]
fn flee_points_away_from_threat() {
    let intent = behaviors::flee::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(5.0, 0.0, 0.0),
        SteeringPlane::XZ,
        6.0,
        10.0,
        None,
    );
    assert!(intent.desired_velocity.x < -5.9);
}

#[test]
fn arrive_slows_inside_radius() {
    let far = behaviors::arrive::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        SteeringPlane::XZ,
        8.0,
        10.0,
        4.0,
        0.1,
        1.0,
    );
    let near = behaviors::arrive::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        SteeringPlane::XZ,
        8.0,
        10.0,
        4.0,
        0.1,
        1.0,
    );
    assert!(far.desired_velocity.length() > near.desired_velocity.length());
}

#[test]
fn pursue_prediction_is_clamped() {
    let predicted = math::predict_target_position(
        Vec3::ZERO,
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        SteeringPlane::XZ,
        4.0,
        1.0,
        0.5,
    );
    assert!(predicted.x <= 70.1);
}

#[test]
fn evade_uses_prediction_then_flee() {
    let (_intent, predicted) = behaviors::evade::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        SteeringPlane::XZ,
        4.0,
        8.0,
        1.0,
        1.0,
        None,
    );
    assert!(predicted.x > 5.0);
}

#[test]
fn wander_is_deterministic_for_same_seed() {
    let config = Wander::default();
    let mut left = WanderState::from_seed(42);
    let mut right = WanderState::from_seed(42);
    let left_step = behaviors::wander::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::X,
        SteeringPlane::XZ,
        4.0,
        8.0,
        &config,
        &mut left,
        1.0 / 60.0,
    );
    let right_step = behaviors::wander::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::X,
        SteeringPlane::XZ,
        4.0,
        8.0,
        &config,
        &mut right,
        1.0 / 60.0,
    );
    assert_eq!(left_step.1, right_step.1);
}

#[test]
fn obstacle_avoidance_prefers_the_nearest_hit() {
    let agent = SteeringAgent::default();
    let config = ObstacleAvoidance::default();
    let near = SteeringObstacle::sphere(0.8);
    let far = SteeringObstacle::sphere(0.8);
    let obstacles = vec![
        (
            Entity::from_bits(1),
            Affine3A::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            &near,
        ),
        (
            Entity::from_bits(2),
            Affine3A::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            &far,
        ),
    ];
    let (_intent, debug) = behaviors::obstacle_avoidance::evaluate(
        Vec3::ZERO,
        Vec3::X,
        Vec3::X,
        Vec3::X,
        SteeringPlane::XZ,
        &agent,
        &config,
        obstacles,
    )
    .expect("an obstacle should be hit");

    assert_eq!(debug.obstacle, Some(Entity::from_bits(1)));
}

#[test]
fn path_following_advances_waypoints() {
    let mut state = PathFollowingState {
        current_waypoint: 1,
        direction: 1,
        ..default()
    };
    let config = PathFollowing::new(
        SteeringPath::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ])
        .with_waypoint_tolerance(0.4),
    );

    let (_intent, debug) = behaviors::path_following::evaluate(
        Vec3::new(1.1, 0.0, 0.0),
        Vec3::ZERO,
        SteeringPlane::XZ,
        4.0,
        8.0,
        &config,
        &mut state,
    );

    assert_eq!(state.current_waypoint, 2);
    assert!(debug.lookahead_point.is_some());
}

#[test]
fn empty_path_completes_without_intent() {
    let mut state = PathFollowingState::default();
    let config = PathFollowing::new(SteeringPath::default());

    let (intent, debug) = behaviors::path_following::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        SteeringPlane::XZ,
        4.0,
        8.0,
        &config,
        &mut state,
    );

    assert_eq!(intent, math::LinearIntent::zero());
    assert_eq!(debug.lookahead_point, None);
    assert!(state.completed);
}

#[test]
fn single_waypoint_path_uses_arrive_behavior() {
    let mut state = PathFollowingState::default();
    let config = PathFollowing::new(SteeringPath::new([Vec3::new(2.0, 0.0, 0.0)]));

    let (intent, debug) = behaviors::path_following::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        SteeringPlane::XZ,
        4.0,
        8.0,
        &config,
        &mut state,
    );

    assert!(intent.desired_velocity.x > 0.0);
    assert_eq!(debug.lookahead_point, Some(Vec3::new(2.0, 0.0, 0.0)));
    assert_eq!(state.current_waypoint, 0);
    assert!(!state.completed);
}

#[test]
fn overlapping_obstacle_returns_immediate_avoidance() {
    let agent = SteeringAgent::default();
    let config = ObstacleAvoidance::default();
    let obstacle = SteeringObstacle::sphere(0.8);
    let obstacles = vec![(
        Entity::from_bits(1),
        Affine3A::from_translation(Vec3::ZERO),
        &obstacle,
    )];

    let (intent, debug) = behaviors::obstacle_avoidance::evaluate(
        Vec3::ZERO,
        Vec3::X,
        Vec3::X,
        Vec3::X,
        SteeringPlane::XZ,
        &agent,
        &config,
        obstacles,
    )
    .expect("overlapping obstacle should still produce avoidance");

    assert_eq!(debug.hit_point, Some(Vec3::ZERO));
    assert!(intent.linear_acceleration.length() > 0.0);
    assert!(debug.avoidance_direction.is_some());
}

#[test]
fn weighted_blend_cancels_conflicting_forces() {
    let mut contributions = vec![
        SteeringContribution {
            behavior: SteeringBehaviorKind::Seek,
            priority: 10,
            weight: 1.0,
            requested_acceleration: Vec3::X * 4.0,
            applied_acceleration: Vec3::ZERO,
            desired_velocity: Vec3::X,
            suppressed: false,
        },
        SteeringContribution {
            behavior: SteeringBehaviorKind::Flee,
            priority: 10,
            weight: 1.0,
            requested_acceleration: Vec3::NEG_X * 4.0,
            applied_acceleration: Vec3::ZERO,
            desired_velocity: Vec3::NEG_X,
            suppressed: false,
        },
    ];

    let result =
        math::compose_contributions(SteeringComposition::WeightedBlend, 8.0, &mut contributions);

    assert!(result.linear_acceleration.length() <= STEERING_EPSILON);
    assert!(result.desired_velocity.length() <= STEERING_EPSILON);
}

#[test]
fn plane_projection_respects_xy_and_xz() {
    let xy = SteeringPlane::XY.project_vector(Vec3::new(1.0, 2.0, 3.0));
    let xz = SteeringPlane::XZ.project_vector(Vec3::new(1.0, 2.0, 3.0));
    assert!((xy.z).abs() < STEERING_EPSILON);
    assert!((xz.y).abs() < STEERING_EPSILON);
}

#[test]
fn prioritized_accumulation_respects_budget() {
    let mut contributions = vec![
        SteeringContribution {
            behavior: SteeringBehaviorKind::ObstacleAvoidance,
            priority: 0,
            weight: 1.0,
            requested_acceleration: Vec3::X * 4.0,
            applied_acceleration: Vec3::ZERO,
            desired_velocity: Vec3::X,
            suppressed: false,
        },
        SteeringContribution {
            behavior: SteeringBehaviorKind::Seek,
            priority: 10,
            weight: 1.0,
            requested_acceleration: Vec3::Y * 4.0,
            applied_acceleration: Vec3::ZERO,
            desired_velocity: Vec3::Y,
            suppressed: false,
        },
    ];
    let result = math::compose_contributions(
        SteeringComposition::PrioritizedAccumulation,
        5.0,
        &mut contributions,
    );

    assert!(result.linear_acceleration.length() <= 5.001);
    assert!(contributions[1].applied_acceleration.length() < 4.0);
}

#[test]
fn zero_values_short_circuit_cleanly() {
    let seek = behaviors::seek::evaluate(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::ZERO,
        SteeringPlane::XZ,
        0.0,
        0.0,
    );
    assert_eq!(seek, math::LinearIntent::zero());
}
