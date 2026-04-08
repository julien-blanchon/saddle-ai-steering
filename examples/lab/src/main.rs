#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_pane::prelude::*;
use steering::{
    Arrive, BehaviorTuning, CustomSteeringBehavior, Flocking, ObstacleAvoidance, PathFollowing,
    PathFollowingState, Pursue, ReciprocalAvoidance, Seek, SteeringAgent, SteeringAutoApply,
    SteeringDebugSettings, SteeringDiagnostics, SteeringKinematics, SteeringObstacle,
    SteeringObstacleShape, SteeringPath, SteeringPlane, SteeringSystems, SteeringTarget,
    SteeringTrackedVelocity, Wander, desired_velocity_intent,
};
use steering_example_support as support;

#[cfg(feature = "dev")]
const DEFAULT_BRP_PORT: u16 = 15_736;

#[derive(Component)]
struct PathAgent;

#[derive(Component)]
struct AvoidanceAgent;

#[derive(Component)]
struct PursuitAgent;

#[derive(Component)]
struct WanderAgent;

#[derive(Component)]
struct OrbitTarget;

#[derive(Component)]
struct CrowdAgent;

#[derive(Component)]
struct CustomOrbitAgent;

#[derive(Component)]
struct MainObstacle;

#[derive(Component)]
struct OverlayText;

#[derive(Component, Clone, Copy, Debug)]
struct OrbitMotion {
    center: Vec3,
    radius: f32,
    speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct CustomOrbitBehavior {
    center: Vec3,
    radius: f32,
    speed: f32,
    correction_strength: f32,
    tuning: BehaviorTuning,
}

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct LabDiagnostics {
    pub active_agents: usize,
    pub path_waypoint: usize,
    pub path_cycles: u32,
    pub avoidance_position: Vec3,
    pub avoidance_min_clearance: f32,
    pub avoidance_passed_obstacle: bool,
    pub pursuit_distance: f32,
    pub wander_speed: f32,
    pub crowd_peak_flock_neighbors: usize,
    pub crowd_peak_neighbors: usize,
    pub crowd_conflict_frames: u32,
    pub crowd_min_separation: f32,
    pub custom_orbit_speed: f32,
    pub custom_orbit_distance_to_center: f32,
    pub custom_orbit_min_radius_error: f32,
    pub custom_orbit_max_radius_error: f32,
}

impl Default for LabDiagnostics {
    fn default() -> Self {
        Self {
            active_agents: 0,
            path_waypoint: 0,
            path_cycles: 0,
            avoidance_position: Vec3::ZERO,
            avoidance_min_clearance: f32::MAX,
            avoidance_passed_obstacle: false,
            pursuit_distance: 0.0,
            wander_speed: 0.0,
            crowd_peak_flock_neighbors: 0,
            crowd_peak_neighbors: 0,
            crowd_conflict_frames: 0,
            crowd_min_separation: f32::MAX,
            custom_orbit_speed: 0.0,
            custom_orbit_distance_to_center: 0.0,
            custom_orbit_min_radius_error: f32::MAX,
            custom_orbit_max_radius_error: 0.0,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));
    app.insert_resource(SteeringDebugSettings {
        enabled: true,
        ..default()
    });
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 7.0,
        max_acceleration: 16.0,
        path_lookahead: 2.6,
        obstacle_min_lookahead: 3.0,
        obstacle_max_lookahead: 7.5,
        obstacle_probe_radius: 0.38,
        orbit_radius: 5.2,
        orbit_speed: 0.85,
        wander_radius: 2.0,
        wander_distance: 2.8,
        wander_jitter: 1.35,
        ..default()
    });
    app.init_resource::<LabDiagnostics>();
    app.register_type::<LabDiagnostics>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "steering crate-local lab".into(),
            resolution: (1420, 900).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::SteeringLabE2EPlugin);
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<support::SteeringExamplePane>();
    app.add_plugins(steering::SteeringPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        evaluate_custom_orbit.in_set(SteeringSystems::EvaluateCustom),
    );
    app.add_systems(
        Update,
        (
            orbit_targets,
            sync_pane,
            toggle_debug,
            update_lab_diagnostics,
            update_overlay,
        ),
    );
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 140.0,
        ..default()
    });

    commands.spawn((
        Name::new("Lab Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 18.0, 19.0).looking_at(Vec3::new(0.0, 0.7, 0.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Directional Light"),
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.7, 0.0)),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(28.0, 28.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.14, 0.18),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));
    commands.spawn((
        Name::new("Overlay"),
        OverlayText,
        Text::new("steering lab"),
        Node {
            position_type: PositionType::Absolute,
            top: px(14.0),
            left: px(14.0),
            ..default()
        },
    ));

    spawn_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Center Block",
        Vec3::new(2.8, 2.0, 2.8),
        Transform::from_xyz(0.0, 1.0, 0.0),
        true,
    );
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Path Pillar A",
        Vec3::new(1.4, 1.8, 1.4),
        Transform::from_xyz(-3.4, 0.9, 3.0),
        false,
    );
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Path Pillar B",
        Vec3::new(1.2, 1.8, 1.2),
        Transform::from_xyz(3.6, 0.9, -2.6),
        false,
    );

    let avoidance_goal = Vec3::new(8.0, 0.6, 0.0);
    spawn_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Avoidance Goal",
        Color::srgb(0.96, 0.84, 0.22),
        avoidance_goal,
    );
    spawn_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Orbit Target",
        Color::srgb(0.98, 0.77, 0.32),
        Vec3::new(0.0, 0.6, 5.0),
    );

    for (index, point) in [
        Vec3::new(-5.5, 0.6, -4.5),
        Vec3::new(5.5, 0.6, -4.5),
        Vec3::new(5.5, 0.6, 4.5),
        Vec3::new(-5.5, 0.6, 4.5),
    ]
    .into_iter()
    .enumerate()
    {
        spawn_marker(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Waypoint {index}"),
            Color::srgb(0.90, 0.40, 0.78),
            point,
        );
    }

    let path_agent = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Path Agent",
        Color::srgb(0.20, 0.82, 0.56),
        Transform::from_xyz(-5.5, 0.6, -4.5),
    );
    commands.entity(path_agent).insert((
        PathAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(7.0)
            .with_max_acceleration(16.0),
        SteeringAutoApply::default(),
        PathFollowing::new(
            SteeringPath::new([
                Vec3::new(-5.5, 0.6, -4.5),
                Vec3::new(5.5, 0.6, -4.5),
                Vec3::new(5.5, 0.6, 4.5),
                Vec3::new(-5.5, 0.6, 4.5),
            ])
            .looped()
            .with_lookahead_distance(2.6),
        ),
    ));

    let avoidance_agent = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Avoidance Agent",
        Color::srgb(0.18, 0.76, 0.94),
        Transform::from_xyz(-8.0, 0.6, 0.0),
    );
    commands.entity(avoidance_agent).insert((
        AvoidanceAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(6.0)
            .with_max_acceleration(13.0),
        SteeringAutoApply::default(),
        Seek::new(SteeringTarget::Point(avoidance_goal)),
        ObstacleAvoidance {
            min_lookahead: 3.0,
            max_lookahead: 7.5,
            probe_radius: 0.38,
            lateral_weight: 1.7,
            braking_weight: 0.35,
            ..default()
        },
    ));

    let orbit_target = commands
        .spawn((
            Name::new("Orbit Target Driver"),
            OrbitTarget,
            SteeringTrackedVelocity,
            SteeringAutoApply {
                apply_translation: false,
                apply_facing: false,
            },
            Transform::from_xyz(0.0, 0.6, 5.2),
            GlobalTransform::default(),
            OrbitMotion {
                center: Vec3::new(0.0, 0.6, 0.0),
                radius: 5.2,
                speed: 0.85,
            },
        ))
        .id();

    let pursue_agent = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pursuit Agent",
        Color::srgb(0.38, 0.64, 0.96),
        Transform::from_xyz(-6.5, 0.6, -6.0),
    );
    commands.entity(pursue_agent).insert((
        PursuitAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(6.4)
            .with_max_acceleration(14.0),
        SteeringAutoApply::default(),
        Pursue::new(SteeringTarget::Entity(orbit_target)),
        ObstacleAvoidance::default(),
    ));

    let wander_agent = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Wander Agent",
        Color::srgb(0.88, 0.54, 0.20),
        Transform::from_xyz(0.0, 0.6, 7.0),
    );
    commands.entity(wander_agent).insert((
        WanderAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(4.6)
            .with_max_acceleration(9.0),
        SteeringAutoApply::default(),
        Wander {
            seed: 11,
            radius: 2.0,
            distance: 2.8,
            jitter_radians_per_second: 1.35,
            ..default()
        },
    ));

    let arrive_agent = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Arrive Agent",
        Color::srgb(0.76, 0.30, 0.86),
        Transform::from_xyz(8.0, 0.6, 6.0),
    );
    let mut arrive = Arrive::new(SteeringTarget::Point(Vec3::new(2.5, 0.6, 7.0)));
    arrive.slowing_radius = 3.2;
    commands.entity(arrive_agent).insert((
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.0)
            .with_max_acceleration(12.0),
        SteeringAutoApply::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(-1.0, 0.0, 0.0),
        },
        arrive,
    ));

    for (index, (start, goal, color)) in [
        (
            Vec3::new(-5.0, 0.6, -1.4),
            Vec3::new(5.0, 0.6, -0.6),
            Color::srgb(0.84, 0.64, 0.94),
        ),
        (
            Vec3::new(-5.2, 0.6, 1.2),
            Vec3::new(5.1, 0.6, 0.8),
            Color::srgb(0.98, 0.66, 0.84),
        ),
        (
            Vec3::new(5.0, 0.6, -0.8),
            Vec3::new(-5.0, 0.6, -1.3),
            Color::srgb(0.54, 0.82, 0.98),
        ),
        (
            Vec3::new(5.3, 0.6, 1.1),
            Vec3::new(-5.1, 0.6, 1.4),
            Color::srgb(0.62, 0.92, 0.82),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let entity = spawn_agent(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Crowd Agent {index}"),
            color,
            Transform::from_translation(start),
        );
        commands.entity(entity).insert((
            CrowdAgent,
            SteeringAgent::new(SteeringPlane::XZ)
                .with_max_speed(5.6)
                .with_max_acceleration(14.0),
            SteeringAutoApply::default(),
            Seek::new(SteeringTarget::Point(goal)),
            Flocking::default(),
            ReciprocalAvoidance::default(),
        ));
    }

    let orbit_center = Vec3::new(6.0, 0.6, 6.0);
    spawn_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Custom Orbit Center",
        Color::srgb(0.52, 0.96, 0.62),
        orbit_center,
    );
    let custom_orbit = spawn_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Custom Orbit Agent",
        Color::srgb(0.42, 0.96, 0.52),
        Transform::from_xyz(9.5, 0.6, 6.0),
    );
    commands.entity(custom_orbit).insert((
        CustomOrbitAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(4.5)
            .with_max_acceleration(11.0),
        SteeringAutoApply::default(),
        CustomSteeringBehavior::default(),
        CustomOrbitBehavior {
            center: orbit_center,
            radius: 3.5,
            speed: 3.8,
            correction_strength: 2.5,
            tuning: BehaviorTuning::new(1.0, 40),
        },
    ));
}

fn spawn_agent(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    color: Color,
    transform: Transform,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.7))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.7,
                ..default()
            })),
            transform,
        ))
        .id()
}

fn spawn_marker(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    color: Color,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            Mesh3d(meshes.add(Sphere::new(0.22))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 0.4,
                ..default()
            })),
            Transform::from_translation(position),
        ))
        .id()
}

fn spawn_obstacle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    size: Vec3,
    transform: Transform,
    main: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        SteeringObstacle::aabb(size * 0.5),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.40, 0.19, 0.18),
            perceptual_roughness: 0.85,
            ..default()
        })),
        transform,
    ));
    if main {
        entity.insert(MainObstacle);
    }
    entity.id()
}

fn evaluate_custom_orbit(
    mut agents: Query<(
        &SteeringAgent,
        &GlobalTransform,
        &SteeringKinematics,
        &CustomOrbitBehavior,
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

        let radial = to_center / distance;
        let tangent = Vec3::new(-radial.z, 0.0, radial.x);

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

fn orbit_targets(
    time: Res<Time>,
    mut targets: Query<(&OrbitMotion, &mut Transform), With<OrbitTarget>>,
) {
    for (motion, mut transform) in &mut targets {
        let angle = time.elapsed_secs() * motion.speed;
        transform.translation = motion.center
            + Vec3::new(
                angle.cos() * motion.radius,
                0.0,
                angle.sin() * motion.radius,
            );
    }
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut debug: ResMut<SteeringDebugSettings>,
    mut path_agents: Query<
        (&mut SteeringAgent, &mut PathFollowing),
        (
            With<PathAgent>,
            Without<AvoidanceAgent>,
            Without<PursuitAgent>,
            Without<WanderAgent>,
        ),
    >,
    mut avoidance_agents: Query<
        (&mut SteeringAgent, &mut ObstacleAvoidance),
        (
            With<AvoidanceAgent>,
            Without<PathAgent>,
            Without<PursuitAgent>,
            Without<WanderAgent>,
        ),
    >,
    mut pursuit_agents: Query<
        (&mut SteeringAgent, &mut ObstacleAvoidance),
        (
            With<PursuitAgent>,
            Without<PathAgent>,
            Without<AvoidanceAgent>,
            Without<WanderAgent>,
        ),
    >,
    mut orbit_targets: Query<&mut OrbitMotion, With<OrbitTarget>>,
    mut wander_agents: Query<
        (&mut SteeringAgent, &mut Wander),
        (
            With<WanderAgent>,
            Without<PathAgent>,
            Without<AvoidanceAgent>,
            Without<PursuitAgent>,
        ),
    >,
) {
    if !pane.is_changed() {
        return;
    }

    debug.enabled = pane.debug_enabled;

    for (mut agent, mut path) in &mut path_agents {
        support::apply_agent_tuning(&mut agent, &pane);
        path.path.lookahead_distance = pane.path_lookahead;
        path.path.waypoint_tolerance = pane.path_tolerance;
    }
    for (mut agent, mut avoidance) in &mut avoidance_agents {
        support::apply_agent_tuning(&mut agent, &pane);
        avoidance.min_lookahead = pane.obstacle_min_lookahead;
        avoidance.max_lookahead = pane.obstacle_max_lookahead;
        avoidance.probe_radius = pane.obstacle_probe_radius;
    }
    for (mut agent, mut avoidance) in &mut pursuit_agents {
        support::apply_agent_tuning(&mut agent, &pane);
        avoidance.min_lookahead = pane.obstacle_min_lookahead;
        avoidance.max_lookahead = pane.obstacle_max_lookahead;
        avoidance.probe_radius = pane.obstacle_probe_radius;
    }
    for mut orbit in &mut orbit_targets {
        orbit.radius = pane.orbit_radius;
        orbit.speed = pane.orbit_speed;
    }
    for (mut agent, mut wander) in &mut wander_agents {
        support::apply_agent_tuning(&mut agent, &pane);
        wander.radius = pane.wander_radius;
        wander.distance = pane.wander_distance;
        wander.jitter_radians_per_second = pane.wander_jitter;
    }
}

fn toggle_debug(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<SteeringDebugSettings>) {
    if keyboard.just_pressed(KeyCode::Tab) {
        settings.enabled = !settings.enabled;
    }
}

fn update_lab_diagnostics(
    mut diagnostics: ResMut<LabDiagnostics>,
    outputs: Query<&steering::SteeringOutput, With<SteeringAgent>>,
    path_agent: Query<&PathFollowingState, With<PathAgent>>,
    avoidance_agent: Query<&Transform, With<AvoidanceAgent>>,
    pursuit_agent: Query<&Transform, With<PursuitAgent>>,
    pursuit_target: Query<&Transform, With<OrbitTarget>>,
    wander_agent: Query<&SteeringKinematics, With<WanderAgent>>,
    main_obstacle: Query<(&Transform, &SteeringObstacle), With<MainObstacle>>,
    crowd_agents: Query<(&Transform, &SteeringDiagnostics), With<CrowdAgent>>,
    custom_orbit_agent: Query<
        (&Transform, &SteeringKinematics, &CustomOrbitBehavior),
        With<CustomOrbitAgent>,
    >,
) {
    diagnostics.active_agents = outputs
        .iter()
        .filter(|output| {
            output.desired_velocity.length() > 0.05 || output.linear_acceleration.length() > 0.05
        })
        .count();

    if let Ok(path_state) = path_agent.single() {
        diagnostics.path_waypoint = path_state.current_waypoint;
        diagnostics.path_cycles = path_state.completed_cycles;
    }

    if let (Ok(transform), Ok((obstacle_transform, obstacle))) =
        (avoidance_agent.single(), main_obstacle.single())
    {
        diagnostics.avoidance_position = transform.translation;
        diagnostics.avoidance_passed_obstacle =
            transform.translation.x > obstacle_transform.translation.x;
        if let SteeringObstacleShape::Aabb { half_extents } = obstacle.shape {
            let clearance = aabb_clearance(
                transform.translation,
                obstacle_transform.translation,
                half_extents,
                0.45,
            );
            diagnostics.avoidance_min_clearance =
                diagnostics.avoidance_min_clearance.min(clearance);
        }
    }

    if let (Ok(pursuer), Ok(target)) = (pursuit_agent.single(), pursuit_target.single()) {
        diagnostics.pursuit_distance = pursuer.translation.distance(target.translation);
    }

    if let Ok(wander_velocity) = wander_agent.single() {
        diagnostics.wander_speed = wander_velocity.linear_velocity.length();
    }

    let crowd_samples = crowd_agents
        .iter()
        .map(|(transform, steering)| (transform.translation, steering))
        .collect::<Vec<_>>();

    let frame_peak_flock_neighbors = crowd_samples
        .iter()
        .map(|(_, steering)| steering.flock_neighbor_count)
        .max()
        .unwrap_or(0);
    let frame_peak_neighbors = crowd_samples
        .iter()
        .map(|(_, steering)| steering.crowd_neighbor_count)
        .max()
        .unwrap_or(0);
    let frame_conflicts = crowd_samples
        .iter()
        .filter(|(_, steering)| steering.crowd_avoidance_velocity.is_some())
        .count() as u32;

    diagnostics.crowd_peak_flock_neighbors = diagnostics
        .crowd_peak_flock_neighbors
        .max(frame_peak_flock_neighbors);
    diagnostics.crowd_peak_neighbors = diagnostics.crowd_peak_neighbors.max(frame_peak_neighbors);
    diagnostics.crowd_conflict_frames += frame_conflicts;

    if crowd_samples.len() >= 2 {
        for (index, (position, _)) in crowd_samples.iter().enumerate() {
            for (other_position, _) in crowd_samples.iter().skip(index + 1) {
                diagnostics.crowd_min_separation = diagnostics
                    .crowd_min_separation
                    .min(position.distance(*other_position));
            }
        }
    }

    if let Ok((transform, kinematics, orbit)) = custom_orbit_agent.single() {
        let position = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        let center = Vec3::new(orbit.center.x, 0.0, orbit.center.z);
        let distance = position.distance(center);
        let radius_error = (distance - orbit.radius).abs();
        diagnostics.custom_orbit_speed = kinematics.linear_velocity.length();
        diagnostics.custom_orbit_distance_to_center = distance;
        diagnostics.custom_orbit_min_radius_error =
            diagnostics.custom_orbit_min_radius_error.min(radius_error);
        diagnostics.custom_orbit_max_radius_error =
            diagnostics.custom_orbit_max_radius_error.max(radius_error);
    }
}

fn update_overlay(
    diagnostics: Res<LabDiagnostics>,
    mut overlay: Single<&mut Text, With<OverlayText>>,
    debug: Res<SteeringDebugSettings>,
) {
    if !diagnostics.is_changed() && !debug.is_changed() {
        return;
    }

    **overlay = format!(
        "steering lab\n\
active agents: {}\n\
path waypoint/cycles: {}/{}\n\
avoidance clearance min: {:.2}\n\
avoidance passed obstacle: {}\n\
pursuit distance: {:.2}\n\
wander speed: {:.2}\n\
crowd flock peak: {}\n\
crowd neighbors peak: {}\n\
crowd conflict frames: {}\n\
crowd min separation: {:.2}\n\
custom orbit speed: {:.2}\n\
custom orbit dist: {:.2} (err: {:.2}–{:.2})\n\
debug gizmos: {} (Tab)",
        diagnostics.active_agents,
        diagnostics.path_waypoint,
        diagnostics.path_cycles,
        diagnostics.avoidance_min_clearance,
        diagnostics.avoidance_passed_obstacle,
        diagnostics.pursuit_distance,
        diagnostics.wander_speed,
        diagnostics.crowd_peak_flock_neighbors,
        diagnostics.crowd_peak_neighbors,
        diagnostics.crowd_conflict_frames,
        diagnostics.crowd_min_separation,
        diagnostics.custom_orbit_speed,
        diagnostics.custom_orbit_distance_to_center,
        diagnostics.custom_orbit_min_radius_error,
        diagnostics.custom_orbit_max_radius_error,
        if debug.enabled { "on" } else { "off" },
    )
    .into();
}

fn aabb_clearance(position: Vec3, center: Vec3, half_extents: Vec3, radius: f32) -> f32 {
    let local = position - center;
    let delta = (local.abs() - half_extents).max(Vec3::ZERO);
    delta.length() - radius
}
