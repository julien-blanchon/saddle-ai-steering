use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Flocking, ReciprocalAvoidance, SteeringAgent, SteeringAutoApply, SteeringKinematics,
    SteeringPlane, Wander,
};

#[derive(Component)]
struct Boid;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 6.4,
        max_acceleration: 12.8,
        wander_radius: 1.8,
        wander_distance: 2.2,
        wander_jitter: 1.0,
        flock_separation_weight: 1.7,
        flock_alignment_weight: 1.0,
        flock_cohesion_weight: 0.9,
        crowd_neighbor_distance: 3.8,
        crowd_time_horizon: 1.3,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: flocking");
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
            "Central Tower",
            Vec3::new(2.4, 3.8, 2.4),
            Vec3::new(0.0, 1.9, 0.0),
        ),
        (
            "Outer Pillar A",
            Vec3::new(1.4, 2.4, 1.4),
            Vec3::new(-5.5, 1.2, 4.6),
        ),
        (
            "Outer Pillar B",
            Vec3::new(1.2, 2.0, 1.2),
            Vec3::new(6.0, 1.0, -4.5),
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

    for index in 0..18 {
        let phase = index as f32 / 18.0 * std::f32::consts::TAU;
        let radius = 4.2 + (index % 3) as f32 * 0.8;
        let position = Vec3::new(phase.cos() * radius, 0.6, phase.sin() * radius);
        let color = Color::hsl(160.0 + index as f32 * 8.0, 0.72, 0.58);
        let agent = support::spawn_capsule_agent(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Boid {index}"),
            color,
            Transform::from_translation(position),
        );
        commands.entity(agent).insert((
            Boid,
            SteeringAgent::new(SteeringPlane::XZ)
                .with_max_speed(6.4)
                .with_max_acceleration(12.8),
            SteeringAutoApply::default(),
            SteeringKinematics {
                linear_velocity: Vec3::new(-phase.sin(), 0.0, phase.cos()) * 2.2,
            },
            Wander {
                seed: 100 + index as u64,
                radius: 1.8,
                distance: 2.2,
                jitter_radians_per_second: 1.0,
                vertical_jitter: 0.0,
                ..default()
            },
            Flocking::default(),
            ReciprocalAvoidance::default(),
        ));
    }
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut boids: Query<
        (
            &mut SteeringAgent,
            &mut Wander,
            &mut Flocking,
            &mut ReciprocalAvoidance,
        ),
        With<Boid>,
    >,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut wander, mut flocking, mut avoidance) in &mut boids {
        support::apply_agent_tuning(&mut agent, &pane);
        wander.radius = pane.wander_radius;
        wander.distance = pane.wander_distance;
        wander.jitter_radians_per_second = pane.wander_jitter;
        flocking.separation_weight = pane.flock_separation_weight;
        flocking.alignment_weight = pane.flock_alignment_weight;
        flocking.cohesion_weight = pane.flock_cohesion_weight;
        avoidance.neighbor_distance = pane.crowd_neighbor_distance;
        avoidance.time_horizon = pane.crowd_time_horizon;
    }
}
