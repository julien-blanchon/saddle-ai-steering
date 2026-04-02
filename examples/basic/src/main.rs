use steering_example_support as support;

use bevy::prelude::*;
use steering::{Seek, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringTarget};

fn main() {
    let mut app = App::new();
    support::configure_3d_app(&mut app, "steering: basic seek");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_position = Vec3::new(5.5, 0.6, 4.0);
    support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Target",
        Color::srgb(0.96, 0.84, 0.22),
        Transform::from_translation(target_position),
    );

    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Seek Agent",
        Color::srgb(0.18, 0.75, 0.92),
        Transform::from_xyz(-6.0, 0.6, -4.5),
    );
    commands.entity(agent).insert((
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.0)
            .with_max_acceleration(10.0),
        SteeringAutoApply::default(),
        Seek::new(SteeringTarget::Point(target_position)),
    ));
}
