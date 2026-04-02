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

fn main() {
    let mut app = App::new();
    support::configure_3d_app(&mut app, "steering: blended");
    app.add_systems(Startup, setup);
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
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(6.0)
            .with_max_acceleration(13.0),
        SteeringAutoApply::default(),
        Pursue::new(SteeringTarget::Entity(target)),
        ObstacleAvoidance::default(),
    ));
}

fn orbit_target(time: Res<Time>, mut targets: Query<(&OrbitTarget, &mut Transform)>) {
    for (orbit, mut transform) in &mut targets {
        let angle = time.elapsed_secs() * orbit.speed;
        transform.translation =
            orbit.center + Vec3::new(angle.cos() * orbit.radius, 0.0, angle.sin() * orbit.radius);
    }
}
