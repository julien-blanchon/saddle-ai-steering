use steering_example_support as support;

use bevy::prelude::*;
use steering::{SteeringAgent, SteeringAutoApply, SteeringPlane, Wander};

#[derive(Component)]
struct WanderAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 4.5,
        max_acceleration: 8.0,
        wander_radius: 2.3,
        wander_distance: 2.7,
        wander_jitter: 1.4,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: wander");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Wander Agent",
        Color::srgb(0.86, 0.52, 0.18),
        Transform::from_xyz(0.0, 0.6, 0.0),
    );
    commands.entity(agent).insert((
        WanderAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(4.5)
            .with_max_acceleration(8.0),
        SteeringAutoApply::default(),
        Wander {
            seed: 17,
            radius: 2.3,
            distance: 2.7,
            jitter_radians_per_second: 1.4,
            ..default()
        },
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut agents: Query<(&mut SteeringAgent, &mut Wander), With<WanderAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut wander) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        wander.radius = pane.wander_radius;
        wander.distance = pane.wander_distance;
        wander.jitter_radians_per_second = pane.wander_jitter;
    }
}
