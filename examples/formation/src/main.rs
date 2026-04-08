#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Formation, ObstacleAvoidance, ReciprocalAvoidance, SteeringAgent, SteeringAutoApply,
    SteeringKinematics, SteeringPlane, SteeringSystems, SteeringTarget, Wander,
};

#[derive(Component)]
struct Leader;

#[derive(Component)]
struct Follower;

#[derive(Resource, Default)]
struct FormationDiagnostics {
    mode_label: String,
    follower_count: usize,
    avg_slot_error: f32,
    max_slot_error: f32,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 4.5,
        max_acceleration: 9.0,
        wander_radius: 2.5,
        wander_distance: 3.5,
        wander_jitter: 0.8,
        ..default()
    });
    support::configure_3d_app(&mut app, "steering: formation");
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::FormationE2EPlugin);
    app.init_resource::<FormationDiagnostics>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    #[cfg(feature = "e2e")]
    app.add_systems(Update, cycle_formation.after(saddle_bevy_e2e::E2ESet));
    #[cfg(not(feature = "e2e"))]
    app.add_systems(Update, cycle_formation);
    app.add_systems(
        Update,
        update_formation_diagnostics.after(SteeringSystems::Apply),
    );
    app.insert_resource(FormationMode::Wedge);
    app.run();
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum FormationMode {
    Wedge,
    Line,
    Circle,
}

impl FormationMode {
    fn next(self) -> Self {
        match self {
            FormationMode::Wedge => FormationMode::Line,
            FormationMode::Line => FormationMode::Circle,
            FormationMode::Circle => FormationMode::Wedge,
        }
    }

    fn label(self) -> &'static str {
        match self {
            FormationMode::Wedge => "Wedge",
            FormationMode::Line => "Line",
            FormationMode::Circle => "Circle",
        }
    }

    fn slot_offsets(self, count: usize) -> Vec<Vec3> {
        match self {
            FormationMode::Wedge => {
                let mut offsets = Vec::new();
                for i in 0..count {
                    let row = (i / 2) as f32 + 1.0;
                    let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                    offsets.push(Vec3::new(side * row * 1.5, 0.0, -row * 2.0));
                }
                offsets
            }
            FormationMode::Line => (0..count)
                .map(|i| {
                    let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                    let col = i.div_ceil(2) as f32;
                    Vec3::new(side * col * 2.0, 0.0, 0.0)
                })
                .collect(),
            FormationMode::Circle => {
                let radius = 2.5;
                (0..count)
                    .map(|i| {
                        let angle = i as f32 / count as f32 * std::f32::consts::TAU;
                        Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius - 2.5)
                    })
                    .collect()
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mode: Res<FormationMode>,
) {
    for (name, size, translation) in [
        (
            "Obstacle A",
            Vec3::new(2.0, 2.0, 1.4),
            Vec3::new(5.0, 1.0, -3.0),
        ),
        (
            "Obstacle B",
            Vec3::new(1.6, 2.2, 1.6),
            Vec3::new(-6.0, 1.1, 4.0),
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

    let leader = support::spawn_capsule_agent(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Formation Leader",
        Color::srgb(0.96, 0.84, 0.22),
        Transform::from_xyz(0.0, 0.6, 0.0),
    );
    commands.entity(leader).insert((
        Leader,
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(3.5)
            .with_max_acceleration(7.0),
        SteeringAutoApply::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(0.0, 0.0, 1.5),
        },
        Wander {
            seed: 7,
            radius: 2.5,
            distance: 3.5,
            jitter_radians_per_second: 0.8,
            vertical_jitter: 0.0,
            ..default()
        },
        ObstacleAvoidance::default(),
        steering::Containment::new(Vec3::new(0.0, 0.0, 0.0), 9.0).with_margin(2.5),
    ));

    let follower_count = 6;
    let offsets = mode.slot_offsets(follower_count);
    let colors = [
        Color::srgb(0.18, 0.75, 0.92),
        Color::srgb(0.28, 0.82, 0.72),
        Color::srgb(0.42, 0.68, 0.96),
        Color::srgb(0.55, 0.58, 0.95),
        Color::srgb(0.68, 0.52, 0.92),
        Color::srgb(0.82, 0.48, 0.85),
    ];

    for (index, offset) in offsets.iter().enumerate() {
        let color = colors[index % colors.len()];
        let start_pos = Vec3::new(offset.x + (index as f32 - 2.5) * 1.2, 0.6, offset.z - 3.0);
        let follower = support::spawn_capsule_agent(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Follower {index}"),
            color,
            Transform::from_translation(start_pos),
        );
        commands.entity(follower).insert((
            Follower,
            SteeringAgent::new(SteeringPlane::XZ)
                .with_max_speed(4.5)
                .with_max_acceleration(9.0),
            SteeringAutoApply::default(),
            Formation::new(SteeringTarget::Entity(leader), *offset),
            ReciprocalAvoidance {
                neighbor_distance: 2.5,
                time_horizon: 0.8,
                ..default()
            },
            ObstacleAvoidance::default(),
        ));
    }

    commands.spawn((
        Name::new("Instructions"),
        Text::new(format!("Formation: {} | Press F to cycle", mode.label())),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextFont {
            font_size: 20.0,
            ..default()
        },
    ));
}

fn cycle_formation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<FormationMode>,
    mut followers: Query<&mut Formation, With<Follower>>,
    mut label: Query<&mut Text>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    *mode = mode.next();
    let offsets = mode.slot_offsets(followers.iter().count());
    for (index, mut formation) in followers.iter_mut().enumerate() {
        if let Some(offset) = offsets.get(index) {
            formation.slot_offset = *offset;
        }
    }
    for mut text in &mut label {
        **text = format!("Formation: {} | Press F to cycle", mode.label());
    }
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut leaders: Query<(&mut SteeringAgent, &mut Wander), With<Leader>>,
    mut followers: Query<&mut SteeringAgent, (With<Follower>, Without<Leader>)>,
) {
    if !pane.is_changed() {
        return;
    }

    for (mut agent, mut wander) in &mut leaders {
        agent.max_speed = pane.max_speed * 0.78;
        agent.max_acceleration = pane.max_acceleration * 0.78;
        wander.radius = pane.wander_radius;
        wander.distance = pane.wander_distance;
        wander.jitter_radians_per_second = pane.wander_jitter;
    }
    for mut agent in &mut followers {
        agent.max_speed = pane.max_speed;
        agent.max_acceleration = pane.max_acceleration;
    }
}

fn update_formation_diagnostics(
    mode: Res<FormationMode>,
    leader: Query<&Transform, With<Leader>>,
    followers: Query<(&Transform, &Formation), With<Follower>>,
    mut diagnostics: ResMut<FormationDiagnostics>,
) {
    let Ok(leader_transform) = leader.single() else {
        return;
    };

    let mut total_error = 0.0_f32;
    let mut max_error = 0.0_f32;
    let mut follower_count = 0;

    for (transform, formation) in &followers {
        let slot_world_position = leader_transform.translation + formation.slot_offset;
        let error = transform.translation.distance(slot_world_position);
        total_error += error;
        max_error = max_error.max(error);
        follower_count += 1;
    }

    diagnostics.mode_label = mode.label().into();
    diagnostics.follower_count = follower_count;
    diagnostics.avg_slot_error = if follower_count > 0 {
        total_error / follower_count as f32
    } else {
        0.0
    };
    diagnostics.max_slot_error = max_error;
}
