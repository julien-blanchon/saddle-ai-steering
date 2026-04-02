use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Arrive, SteeringAgent, SteeringAutoApply, SteeringKinematics, SteeringPlane, SteeringTarget,
};

fn main() {
    let mut app = App::new();
    support::configure_3d_app(&mut app, "steering: arrive");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_position = Vec3::new(0.0, 0.6, 0.0);
    support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Arrival Goal",
        Color::srgb(0.98, 0.74, 0.25),
        Transform::from_translation(target_position),
    );

    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Arrive Agent",
        Color::srgb(0.24, 0.85, 0.52),
        Transform::from_xyz(-8.0, 0.6, -1.5),
    );

    let mut arrive = Arrive::new(SteeringTarget::Point(target_position));
    arrive.slowing_radius = 4.5;
    arrive.arrival_tolerance = 0.25;
    commands.entity(agent).insert((
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(7.0)
            .with_max_acceleration(14.0),
        SteeringAutoApply::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(4.0, 0.0, 0.0),
        },
        arrive,
    ));
}
