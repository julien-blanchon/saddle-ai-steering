use steering_example_support as support;

use bevy::prelude::*;
use steering::{PathFollowing, SteeringAgent, SteeringAutoApply, SteeringPath, SteeringPlane};

#[derive(Component)]
struct PathAgent;

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 5.0,
        max_acceleration: 10.0,
        path_lookahead: 2.4,
        path_tolerance: 0.6,
        slowing_radius: 3.0,
        arrival_tolerance: 0.3,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: path following");
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (index, point) in [
        Vec3::new(-5.0, 0.6, -4.0),
        Vec3::new(5.0, 0.6, -4.0),
        Vec3::new(5.0, 0.6, 4.0),
        Vec3::new(-5.0, 0.6, 4.0),
    ]
    .into_iter()
    .enumerate()
    {
        support::spawn_target_marker(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Waypoint {index}"),
            Color::srgb(0.86, 0.44, 0.78),
            Transform::from_translation(point),
        );
    }

    support::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Center Pillar",
        Vec3::new(1.4, 1.8, 1.4),
        Transform::from_xyz(0.0, 0.9, 0.0),
    );

    let path = SteeringPath::new([
        Vec3::new(-5.0, 0.6, -4.0),
        Vec3::new(5.0, 0.6, -4.0),
        Vec3::new(5.0, 0.6, 4.0),
        Vec3::new(-5.0, 0.6, 4.0),
    ])
    .looped()
    .with_lookahead_distance(2.4);

    let agent = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Path Agent",
        Color::srgb(0.28, 0.84, 0.66),
        Transform::from_xyz(-5.0, 0.6, -4.0),
    );
    commands.entity(agent).insert((
        PathAgent,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.0)
            .with_max_acceleration(10.0),
        SteeringAutoApply::default(),
        PathFollowing::new(path),
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut agents: Query<(&mut SteeringAgent, &mut PathFollowing), With<PathAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut path_following) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        path_following.path.lookahead_distance = pane.path_lookahead;
        path_following.path.waypoint_tolerance = pane.path_tolerance;
        path_following.slowing_radius = pane.slowing_radius;
        path_following.arrival_tolerance = pane.arrival_tolerance;
    }
}
