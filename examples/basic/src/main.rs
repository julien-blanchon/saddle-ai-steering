use steering_example_support as support;

use bevy::prelude::*;
use steering::{Seek, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringTarget};

#[derive(Component)]
struct TargetMarker;

#[derive(Component)]
struct SeekAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 5.0,
        max_acceleration: 10.0,
        target_x: 5.5,
        target_z: 4.0,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: basic seek");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_position = Vec3::new(5.5, 0.6, 4.0);
    let marker = support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Target",
        Color::srgb(0.96, 0.84, 0.22),
        Transform::from_translation(target_position),
    );
    commands.entity(marker).insert(TargetMarker);

    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Seek Agent",
        Color::srgb(0.18, 0.75, 0.92),
        Transform::from_xyz(-6.0, 0.6, -4.5),
    );
    commands.entity(agent).insert((
        SeekAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.0)
            .with_max_acceleration(10.0),
        SteeringAutoApply::default(),
        Seek::new(SteeringTarget::Point(target_position)),
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut markers: Query<&mut Transform, With<TargetMarker>>,
    mut agents: Query<(&mut SteeringAgent, &mut Seek), With<SeekAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    let target = support::pane_target_translation_3d(&pane);
    for mut transform in &mut markers {
        transform.translation = target;
    }
    for (mut agent, mut seek) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        seek.target = SteeringTarget::Point(target);
    }
}
