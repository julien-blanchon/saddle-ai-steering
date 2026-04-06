//! Demonstrates defining a **custom steering behavior** entirely outside the
//! crate.  Two agents orbit a center point at different radii, using the
//! `CustomSteeringBehavior` inbox and the public `desired_velocity_intent` helper.
//!
//! This shows the full pattern:
//! 1. Define your own behavior component with whatever parameters you need.
//! 2. Write a system that reads agent state, computes a `LinearIntent`, and
//!    pushes it into `CustomSteeringBehavior`.
//! 3. Register the system in `SteeringSystems::EvaluateCustom`.

use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    BehaviorTuning, CustomSteeringBehavior, SteeringAgent, SteeringAutoApply, SteeringKinematics,
    SteeringPlane, SteeringSystems, desired_velocity_intent,
};

// ---------------------------------------------------------------------------
// 1. Custom behavior component — lives in *your* code, not in the crate
// ---------------------------------------------------------------------------

#[derive(Component)]
struct OrbitBehavior {
    center: Vec3,
    radius: f32,
    speed: f32,
    correction_strength: f32,
    tuning: BehaviorTuning,
}

impl OrbitBehavior {
    fn new(center: Vec3, radius: f32, speed: f32) -> Self {
        Self {
            center,
            radius,
            speed,
            correction_strength: 2.0,
            tuning: BehaviorTuning::new(1.0, 40),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Custom evaluate system — pure game code, no crate internals needed
// ---------------------------------------------------------------------------

fn evaluate_orbit(
    mut agents: Query<(
        &SteeringAgent,
        &GlobalTransform,
        &SteeringKinematics,
        &OrbitBehavior,
        &mut CustomSteeringBehavior,
    )>,
) {
    for (agent, global_transform, kinematics, orbit, mut custom) in &mut agents {
        let position = agent.plane.project_vector(global_transform.translation());
        let center = agent.plane.project_vector(orbit.center);
        let to_center = center - position;
        let distance = to_center.length();

        if distance < 0.01 {
            continue;
        }

        // Tangent direction (perpendicular to the radial in the XZ plane)
        let radial = to_center / distance;
        let tangent = Vec3::new(-radial.z, 0.0, radial.x);

        // Proportional correction toward the desired orbit radius
        let radius_error = distance - orbit.radius;
        let correction = radial * radius_error * orbit.correction_strength;

        let desired_velocity =
            (tangent * orbit.speed + correction).normalize_or_zero() * orbit.speed;

        let intent = desired_velocity_intent(
            desired_velocity,
            kinematics.linear_velocity,
            agent.plane,
            agent.max_acceleration,
        );
        custom.push("Orbit", orbit.tuning, intent);
    }
}

// ---------------------------------------------------------------------------
// App setup
// ---------------------------------------------------------------------------

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 4.0,
        max_acceleration: 10.0,
        orbit_radius: 5.0,
        orbit_speed: 1.2,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: custom behavior (orbit)");

    // 3. Register custom evaluate in EvaluateCustom
    app.add_systems(
        Update,
        evaluate_orbit.in_set(SteeringSystems::EvaluateCustom),
    );

    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

#[derive(Component)]
struct OrbitAgent;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center = Vec3::new(0.0, 0.6, 0.0);

    // Visual marker for the orbit center
    support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Orbit Center",
        Color::srgb(0.96, 0.84, 0.22),
        Transform::from_translation(center),
    );

    // Agent A — inner orbit
    let agent_a = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Inner Orbiter",
        Color::srgb(0.18, 0.75, 0.92),
        Transform::from_xyz(3.5, 0.6, 0.0),
    );
    commands.entity(agent_a).insert((
        OrbitAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(4.0)
            .with_max_acceleration(10.0),
        SteeringAutoApply::default(),
        CustomSteeringBehavior::default(),
        OrbitBehavior::new(center, 3.5, 3.5),
    ));

    // Agent B — outer orbit
    let agent_b = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Outer Orbiter",
        Color::srgb(0.92, 0.42, 0.18),
        Transform::from_xyz(-6.0, 0.6, 0.0),
    );
    commands.entity(agent_b).insert((
        OrbitAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.5)
            .with_max_acceleration(10.0),
        SteeringAutoApply::default(),
        CustomSteeringBehavior::default(),
        OrbitBehavior::new(center, 6.0, 5.0),
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut agents: Query<(&mut SteeringAgent, &mut OrbitBehavior), With<OrbitAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut orbit) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        orbit.radius = pane.orbit_radius;
        orbit.speed = pane.orbit_speed * agent.max_speed;
    }
}
