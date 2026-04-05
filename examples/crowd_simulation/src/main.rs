use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Containment, ObstacleAvoidance, PathFollowing, ReciprocalAvoidance, SteeringAgent,
    SteeringAutoApply, SteeringKinematics, SteeringPath, SteeringPlane,
};

#[derive(Component)]
struct CrowdAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 4.0,
        max_acceleration: 8.5,
        crowd_neighbor_distance: 3.2,
        crowd_time_horizon: 1.0,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: crowd simulation");
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
            "Central Block",
            Vec3::new(3.0, 3.0, 3.0),
            Vec3::new(0.0, 1.5, 0.0),
        ),
        (
            "Pillar NW",
            Vec3::new(1.2, 2.2, 1.2),
            Vec3::new(-5.0, 1.1, -5.0),
        ),
        (
            "Pillar NE",
            Vec3::new(1.2, 2.2, 1.2),
            Vec3::new(5.0, 1.1, -5.0),
        ),
        (
            "Pillar SE",
            Vec3::new(1.2, 2.2, 1.2),
            Vec3::new(5.0, 1.1, 5.0),
        ),
        (
            "Pillar SW",
            Vec3::new(1.2, 2.2, 1.2),
            Vec3::new(-5.0, 1.1, 5.0),
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

    let arena_radius = 9.0;
    let agent_count = 32;

    for index in 0..agent_count {
        let phase = index as f32 / agent_count as f32 * std::f32::consts::TAU;
        let spawn_radius = 6.0 + (index % 3) as f32 * 0.6;
        let x = phase.cos() * spawn_radius;
        let z = phase.sin() * spawn_radius;

        let opposite_x = -x * 0.9 + ((index * 7) as f32 % 3.0 - 1.5);
        let opposite_z = -z * 0.9 + ((index * 11) as f32 % 3.0 - 1.5);

        let path =
            SteeringPath::new([Vec3::new(x, 0.6, z), Vec3::new(opposite_x, 0.6, opposite_z)])
                .looped()
                .with_waypoint_tolerance(1.5)
                .with_lookahead_distance(2.5);

        let hue = index as f32 / agent_count as f32 * 360.0;
        let color = Color::hsl(hue, 0.65, 0.58);

        let speed_variation = 3.6 + (index % 5) as f32 * 0.4;
        let accel_variation = speed_variation * 2.1;

        let agent = support::spawn_capsule_agent(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Agent {index}"),
            color,
            Transform::from_xyz(x, 0.6, z),
        );
        commands.entity(agent).insert((
            CrowdAgent,
            SteeringAgent::new(SteeringPlane::XZ)
                .with_max_speed(speed_variation)
                .with_max_acceleration(accel_variation),
            SteeringAutoApply::default(),
            SteeringKinematics {
                linear_velocity: Vec3::new(-phase.sin(), 0.0, phase.cos()) * 1.5,
            },
            PathFollowing::new(path),
            ReciprocalAvoidance {
                neighbor_distance: 3.2,
                time_horizon: 1.0,
                comfort_distance: 0.2,
                side_bias: 0.15,
                max_neighbors: 10,
                ..default()
            },
            ObstacleAvoidance::default(),
            Containment::new(Vec3::ZERO, arena_radius).with_margin(2.0),
        ));
    }
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut agents: Query<(&mut SteeringAgent, &mut ReciprocalAvoidance), With<CrowdAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut avoidance) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        avoidance.neighbor_distance = pane.crowd_neighbor_distance;
        avoidance.time_horizon = pane.crowd_time_horizon;
    }
}
