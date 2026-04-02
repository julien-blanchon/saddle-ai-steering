use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    ObstacleAvoidance, Seek, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringTarget,
};

fn main() {
    let mut app = App::new();
    support::configure_3d_app(&mut app, "steering: obstacle avoidance");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_position = Vec3::new(7.0, 0.6, 0.0);
    support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Far Goal",
        Color::srgb(0.95, 0.82, 0.22),
        Transform::from_translation(target_position),
    );
    support::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Central Obstacle",
        Vec3::new(2.5, 2.0, 2.5),
        Transform::from_xyz(0.0, 1.0, 0.0),
    );
    support::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Offset Obstacle",
        Vec3::new(1.2, 1.6, 1.2),
        Transform::from_xyz(2.5, 0.8, 1.8),
    );

    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Avoidance Agent",
        Color::srgb(0.18, 0.76, 0.92),
        Transform::from_xyz(-7.0, 0.6, 0.0),
    );

    commands.entity(agent).insert((
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.5)
            .with_max_acceleration(12.0),
        SteeringAutoApply::default(),
        Seek::new(SteeringTarget::Point(target_position)),
        ObstacleAvoidance {
            min_lookahead: 2.0,
            max_lookahead: 5.0,
            probe_radius: 0.25,
            ..default()
        },
    ));
}
