use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Arrive, SteeringAgent, SteeringAutoApply, SteeringKinematics, SteeringPlane, SteeringTarget,
};

#[derive(Component)]
struct ArrivalGoal;

#[derive(Component)]
struct ArriveAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 7.0,
        max_acceleration: 14.0,
        target_x: 0.0,
        target_z: 0.0,
        slowing_radius: 4.5,
        arrival_tolerance: 0.25,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: arrive");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_position = Vec3::new(0.0, 0.6, 0.0);
    let goal = support::spawn_target_marker(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Arrival Goal",
        Color::srgb(0.98, 0.74, 0.25),
        Transform::from_translation(target_position),
    );
    commands.entity(goal).insert(ArrivalGoal);

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
        ArriveAgent,
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

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut goals: Query<&mut Transform, With<ArrivalGoal>>,
    mut agents: Query<(&mut SteeringAgent, &mut Arrive), With<ArriveAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    let target = support::pane_target_translation_3d(&pane);
    for mut transform in &mut goals {
        transform.translation = target;
    }
    for (mut agent, mut arrive) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        arrive.target = SteeringTarget::Point(target);
        arrive.slowing_radius = pane.slowing_radius;
        arrive.arrival_tolerance = pane.arrival_tolerance;
    }
}
