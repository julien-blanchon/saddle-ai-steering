use bevy::prelude::*;
use steering::{SteeringDebugSettings, SteeringObstacle, SteeringPlugin};

pub fn configure_3d_app(app: &mut App, title: &str) {
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.08)));
    app.insert_resource(SteeringDebugSettings {
        enabled: true,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (1280, 820).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(SteeringPlugin::default());
    app.add_systems(Startup, setup_3d_scene);
}

pub fn configure_2d_app(app: &mut App, title: &str) {
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.055)));
    app.insert_resource(SteeringDebugSettings {
        enabled: true,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (1100, 760).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(SteeringPlugin::default());
    app.add_systems(Startup, setup_2d_scene);
}

fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 120.0,
        ..default()
    });

    commands.spawn((
        Name::new("Example Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 18.0).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Key Light"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.8, 0.0)),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.16, 0.20),
            perceptual_roughness: 0.9,
            ..default()
        })),
    ));
}

fn setup_2d_scene(mut commands: Commands) {
    commands.spawn((Name::new("Example Camera"), Camera2d));
}

pub fn spawn_capsule_agent(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    color: Color,
    transform: Transform,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.7))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.75,
                ..default()
            })),
            transform,
        ))
        .id()
}

pub fn spawn_target_marker(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    color: Color,
    transform: Transform,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            Mesh3d(meshes.add(Sphere::new(0.28))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 0.4,
                ..default()
            })),
            transform,
        ))
        .id()
}

pub fn spawn_box_obstacle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    size: Vec3,
    transform: Transform,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            SteeringObstacle::aabb(size * 0.5),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.20, 0.16),
                perceptual_roughness: 0.85,
                ..default()
            })),
            transform,
        ))
        .id()
}
