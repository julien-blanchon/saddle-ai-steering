use bevy::prelude::*;
use saddle_pane::prelude::*;
use steering::{SteeringDebugSettings, SteeringObstacle, SteeringPlugin};

#[derive(Resource, Debug, Clone, Pane)]
#[pane(title = "Steering Demo", position = "top-right")]
pub struct SteeringExamplePane {
    #[pane]
    pub debug_enabled: bool,
    #[pane(slider, min = 1.0, max = 14.0, step = 0.1)]
    pub max_speed: f32,
    #[pane(slider, min = 2.0, max = 28.0, step = 0.2)]
    pub max_acceleration: f32,
    #[pane(slider, min = -10.0, max = 10.0, step = 0.1)]
    pub target_x: f32,
    #[pane(slider, min = 0.3, max = 2.0, step = 0.05)]
    pub target_y: f32,
    #[pane(slider, min = -10.0, max = 10.0, step = 0.1)]
    pub target_z: f32,
    #[pane(slider, min = 40.0, max = 420.0, step = 5.0)]
    pub target_x_2d: f32,
    #[pane(slider, min = -260.0, max = 220.0, step = 5.0)]
    pub target_y_2d: f32,
    #[pane(slider, min = 1.0, max = 10.0, step = 0.1)]
    pub slowing_radius: f32,
    #[pane(slider, min = 0.05, max = 1.2, step = 0.01)]
    pub arrival_tolerance: f32,
    #[pane(slider, min = 0.6, max = 5.0, step = 0.1)]
    pub path_lookahead: f32,
    #[pane(slider, min = 0.1, max = 2.0, step = 0.05)]
    pub path_tolerance: f32,
    #[pane(slider, min = 0.6, max = 4.0, step = 0.05)]
    pub wander_radius: f32,
    #[pane(slider, min = 0.8, max = 5.0, step = 0.05)]
    pub wander_distance: f32,
    #[pane(slider, min = 0.1, max = 3.0, step = 0.05)]
    pub wander_jitter: f32,
    #[pane(slider, min = 0.8, max = 6.0, step = 0.1)]
    pub obstacle_min_lookahead: f32,
    #[pane(slider, min = 1.0, max = 8.0, step = 0.1)]
    pub obstacle_max_lookahead: f32,
    #[pane(slider, min = 0.05, max = 1.0, step = 0.01)]
    pub obstacle_probe_radius: f32,
    #[pane(slider, min = 2.0, max = 8.0, step = 0.1)]
    pub orbit_radius: f32,
    #[pane(slider, min = 0.1, max = 2.2, step = 0.05)]
    pub orbit_speed: f32,
    #[pane(slider, min = 0.2, max = 3.0, step = 0.05)]
    pub flock_separation_weight: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub flock_alignment_weight: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub flock_cohesion_weight: f32,
    #[pane(slider, min = 1.0, max = 6.0, step = 0.1)]
    pub crowd_neighbor_distance: f32,
    #[pane(slider, min = 0.4, max = 2.5, step = 0.05)]
    pub crowd_time_horizon: f32,
}

impl Default for SteeringExamplePane {
    fn default() -> Self {
        Self {
            debug_enabled: true,
            max_speed: 5.5,
            max_acceleration: 11.0,
            target_x: 5.5,
            target_y: 0.6,
            target_z: 4.0,
            target_x_2d: 260.0,
            target_y_2d: 120.0,
            slowing_radius: 4.5,
            arrival_tolerance: 0.25,
            path_lookahead: 2.4,
            path_tolerance: 0.6,
            wander_radius: 2.3,
            wander_distance: 2.7,
            wander_jitter: 1.4,
            obstacle_min_lookahead: 2.0,
            obstacle_max_lookahead: 5.0,
            obstacle_probe_radius: 0.25,
            orbit_radius: 5.2,
            orbit_speed: 0.75,
            flock_separation_weight: 1.6,
            flock_alignment_weight: 0.95,
            flock_cohesion_weight: 0.85,
            crowd_neighbor_distance: 3.6,
            crowd_time_horizon: 1.2,
        }
    }
}

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
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<SteeringExamplePane>();
    app.add_plugins(SteeringPlugin::default());
    app.add_systems(Startup, setup_3d_scene);
    app.add_systems(Update, sync_debug_settings);
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
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<SteeringExamplePane>();
    app.add_plugins(SteeringPlugin::default());
    app.add_systems(Startup, setup_2d_scene);
    app.add_systems(Update, sync_debug_settings);
}

fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 150.0,
        ..default()
    });

    commands.spawn((
        Name::new("Example Camera"),
        Camera3d::default(),
        Transform::from_xyz(-1.5, 15.0, 19.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Key Light"),
        DirectionalLight {
            illuminance: 22_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.8, 0.0)),
    ));
    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 5_000_000.0,
            range: 24.0,
            shadows_enabled: true,
            color: Color::srgb(0.48, 0.58, 0.80),
            ..default()
        },
        Transform::from_xyz(-7.5, 7.0, -5.5),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.11, 0.14),
            metallic: 0.06,
            perceptual_roughness: 0.94,
            ..default()
        })),
    ));
    for (name, translation, size, color) in [
        (
            "Outer Ring",
            Vec3::new(0.0, 0.08, 0.0),
            Vec3::new(18.0, 0.12, 18.0),
            Color::srgb(0.16, 0.19, 0.24),
        ),
        (
            "Runway Strip",
            Vec3::new(0.0, 0.05, 0.0),
            Vec3::new(4.0, 0.06, 22.0),
            Color::srgb(0.23, 0.24, 0.28),
        ),
    ] {
        commands.spawn((
            Name::new(name),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.98,
                ..default()
            })),
            Transform::from_translation(translation),
        ));
    }
    for (index, x) in [-8.5_f32, -3.0, 3.0, 8.5].into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Beacon {index}")),
            Mesh3d(meshes.add(Cylinder::new(0.18, 1.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: Color::srgb(0.15, 0.32, 0.44).into(),
                base_color: Color::srgb(0.18, 0.22, 0.26),
                metallic: 0.08,
                ..default()
            })),
            Transform::from_xyz(x, 0.65, -8.0),
        ));
    }
}

fn setup_2d_scene(mut commands: Commands) {
    commands.spawn((Name::new("Example Camera"), Camera2d));
    commands.spawn((
        Name::new("2D Arena Backdrop"),
        Sprite::from_color(Color::srgb(0.05, 0.07, 0.10), Vec2::new(1100.0, 760.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    commands.spawn((
        Name::new("2D Lane"),
        Sprite::from_color(Color::srgb(0.10, 0.13, 0.18), Vec2::new(860.0, 180.0)),
        Transform::from_xyz(0.0, -40.0, -5.0),
    ));
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

pub fn pane_target_translation_3d(pane: &SteeringExamplePane) -> Vec3 {
    Vec3::new(pane.target_x, pane.target_y, pane.target_z)
}

pub fn pane_target_translation_2d(pane: &SteeringExamplePane) -> Vec3 {
    Vec3::new(pane.target_x_2d, pane.target_y_2d, 0.0)
}

pub fn apply_agent_tuning(agent: &mut steering::SteeringAgent, pane: &SteeringExamplePane) {
    agent.max_speed = pane.max_speed;
    agent.max_acceleration = pane.max_acceleration;
}

fn sync_debug_settings(pane: Res<SteeringExamplePane>, mut debug: ResMut<SteeringDebugSettings>) {
    if !pane.is_changed() {
        return;
    }

    debug.enabled = pane.debug_enabled;
}
