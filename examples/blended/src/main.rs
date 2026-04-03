use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    ObstacleAvoidance, Pursue, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringTarget,
    SteeringTrackedVelocity,
};

#[derive(Component)]
struct OrbitTarget {
    center: Vec3,
    radius: f32,
    speed: f32,
}

#[derive(Component)]
struct PursuitAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 6.0,
        max_acceleration: 13.0,
        orbit_radius: 5.2,
        orbit_speed: 0.75,
        obstacle_min_lookahead: 1.5,
        obstacle_max_lookahead: 4.5,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: blended");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.add_systems(Update, orbit_target);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    support::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Obstacle A",
        Vec3::new(2.2, 1.8, 1.6),
        Transform::from_xyz(0.0, 0.9, 0.0),
    );
    support::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Obstacle B",
        Vec3::new(1.2, 1.8, 1.2),
        Transform::from_xyz(-2.0, 0.9, 2.5),
    );

    let target = support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Orbit Target",
        Color::srgb(0.95, 0.84, 0.24),
        Transform::from_xyz(0.0, 0.6, 4.8),
    );
    commands.entity(target).insert((
        SteeringTrackedVelocity,
        OrbitTarget {
            center: Vec3::new(0.0, 0.6, 0.0),
            radius: 5.2,
            speed: 0.75,
        },
    ));

    let pursuer = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Pursuit Agent",
        Color::srgb(0.14, 0.76, 0.96),
        Transform::from_xyz(-7.0, 0.6, -5.0),
    );
    commands.entity(pursuer).insert((
        PursuitAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(6.0)
            .with_max_acceleration(13.0),
        SteeringAutoApply::default(),
        Pursue::new(SteeringTarget::Entity(target)),
        ObstacleAvoidance::default(),
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut targets: Query<&mut OrbitTarget>,
    mut agents: Query<(&mut SteeringAgent, &mut ObstacleAvoidance), With<PursuitAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    for mut orbit in &mut targets {
        orbit.radius = pane.orbit_radius;
        orbit.speed = pane.orbit_speed;
    }
    for (mut agent, mut avoidance) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        avoidance.min_lookahead = pane.obstacle_min_lookahead;
        avoidance.max_lookahead = pane.obstacle_max_lookahead;
        avoidance.probe_radius = pane.obstacle_probe_radius;
    }
}

fn orbit_target(time: Res<Time>, mut targets: Query<(&OrbitTarget, &mut Transform)>) {
    for (orbit, mut transform) in &mut targets {
        let angle = time.elapsed_secs() * orbit.speed;
        transform.translation =
            orbit.center + Vec3::new(angle.cos() * orbit.radius, 0.0, angle.sin() * orbit.radius);
    }
}
