use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Containment, Evade, ObstacleAvoidance, Pursue, SteeringAgent, SteeringAutoApply,
    SteeringKinematics, SteeringPlane, SteeringTarget, Wander,
};

#[derive(Component)]
struct Predator;

#[derive(Component)]
struct Prey;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 5.5,
        max_acceleration: 11.0,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: pursuit & evasion");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (name, size, translation) in [
        (
            "Wall North",
            Vec3::new(1.4, 2.0, 1.4),
            Vec3::new(-4.0, 1.0, -5.0),
        ),
        (
            "Wall South",
            Vec3::new(2.0, 2.0, 1.2),
            Vec3::new(3.0, 1.0, 5.0),
        ),
        (
            "Pillar Center",
            Vec3::new(1.6, 2.6, 1.6),
            Vec3::new(0.0, 1.3, 0.0),
        ),
        (
            "Pillar East",
            Vec3::new(1.2, 1.8, 1.2),
            Vec3::new(5.5, 0.9, -3.0),
        ),
    ] {
        support::spawn_box_obstacle(
            &mut commands,
            &mut meshes,
            &mut materials,
            name,
            size,
            Transform::from_translation(translation),
        );
    }

    let prey = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Prey",
        Color::srgb(0.35, 0.92, 0.42),
        Transform::from_xyz(5.0, 0.6, 4.0),
    );
    commands.entity(prey).insert((
        Prey,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.8)
            .with_max_acceleration(13.0),
        SteeringAutoApply::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(-1.0, 0.0, -1.0),
        },
        Wander {
            seed: 42,
            radius: 2.0,
            distance: 2.5,
            jitter_radians_per_second: 2.2,
            vertical_jitter: 0.0,
            ..default()
        },
        ObstacleAvoidance::default(),
        Containment::new(Vec3::new(0.0, 0.0, 0.0), 9.0).with_margin(2.5),
    ));

    let mut first_predator = None;
    for (index, (x, z, color)) in [
        (-6.0_f32, -4.0_f32, Color::srgb(0.96, 0.28, 0.22)),
        (-5.0, 5.0, Color::srgb(0.96, 0.45, 0.18)),
        (6.0, -5.0, Color::srgb(0.85, 0.22, 0.35)),
    ]
    .into_iter()
    .enumerate()
    {
        let predator = support::spawn_capsule_agent(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Predator {index}"),
            color,
            Transform::from_xyz(x, 0.6, z),
        );
        commands.entity(predator).insert((
            Predator,
            SteeringAgent::new(SteeringPlane::XZ)
                .with_max_speed(4.8)
                .with_max_acceleration(10.0),
            SteeringAutoApply::default(),
            SteeringKinematics::default(),
            Pursue::new(SteeringTarget::Entity(prey)),
            ObstacleAvoidance::default(),
            Containment::new(Vec3::new(0.0, 0.0, 0.0), 9.0).with_margin(2.5),
        ));
        if first_predator.is_none() {
            first_predator = Some(predator);
        }
    }

    // Prey evades the nearest predator. In a real game you'd pick dynamically.
    if let Some(predator_entity) = first_predator {
        commands.entity(prey).insert(Evade {
            target: SteeringTarget::Entity(predator_entity),
            lead_scale: 1.0,
            max_prediction_time: 1.5,
            panic_distance: Some(8.0),
            tuning: steering::BehaviorTuning::new(1.0, 10),
        });
    }
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut predators: Query<&mut SteeringAgent, With<Predator>>,
    mut prey: Query<&mut SteeringAgent, (With<Prey>, Without<Predator>)>,
) {
    if !pane.is_changed() {
        return;
    }

    for mut agent in &mut predators {
        agent.max_speed = pane.max_speed * 0.87;
        agent.max_acceleration = pane.max_acceleration * 0.9;
    }
    for mut agent in &mut prey {
        agent.max_speed = pane.max_speed;
        agent.max_acceleration = pane.max_acceleration;
    }
}
