#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use steering_example_support as support;

use bevy::prelude::*;
use steering::{
    Arrive, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringSystems, SteeringTarget,
};

#[derive(Component)]
struct TargetMarker;

#[derive(Component)]
struct TwoDAgent;

#[derive(Resource)]
struct Kinematic2dDiagnostics {
    agent_position: Vec3,
    target_position: Vec3,
    distance_to_target: f32,
}

impl Default for Kinematic2dDiagnostics {
    fn default() -> Self {
        Self {
            agent_position: Vec3::ZERO,
            target_position: Vec3::ZERO,
            distance_to_target: f32::MAX,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(support::SteeringExamplePane {
        max_speed: 220.0,
        max_acceleration: 420.0,
        target_x_2d: 260.0,
        target_y_2d: 120.0,
        slowing_radius: 180.0,
        arrival_tolerance: 10.0,
        ..default()
    });
    support::configure_2d_app(&mut app, "steering: kinematic 2d");
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::Kinematic2dE2EPlugin);
    app.init_resource::<Kinematic2dDiagnostics>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, sync_pane);
    app.add_systems(
        Update,
        update_kinematic_diagnostics.after(SteeringSystems::Apply),
    );
    app.run();
}

fn setup(mut commands: Commands) {
    let target = Vec3::new(260.0, 120.0, 0.0);
    commands.spawn((
        Name::new("2D Target"),
        TargetMarker,
        Sprite {
            color: Color::srgb(0.98, 0.80, 0.24),
            custom_size: Some(Vec2::splat(28.0)),
            ..default()
        },
        Transform::from_translation(target),
    ));

    let mut arrive = Arrive::new(SteeringTarget::Point(target));
    arrive.slowing_radius = 180.0;
    arrive.arrival_tolerance = 10.0;

    commands.spawn((
        Name::new("2D Agent"),
        TwoDAgent,
        SteeringAgent::new(SteeringPlane::XY)
            .with_max_speed(220.0)
            .with_max_acceleration(420.0),
        SteeringAutoApply::default(),
        arrive,
        Sprite {
            color: Color::srgb(0.26, 0.86, 0.60),
            custom_size: Some(Vec2::new(44.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-340.0, -180.0, 0.0),
        GlobalTransform::default(),
    ));
}

fn sync_pane(
    pane: Res<support::SteeringExamplePane>,
    mut targets: Query<&mut Transform, With<TargetMarker>>,
    mut agents: Query<(&mut SteeringAgent, &mut Arrive), With<TwoDAgent>>,
) {
    if !pane.is_changed() {
        return;
    }

    let target = support::pane_target_translation_2d(&pane);
    for mut transform in &mut targets {
        transform.translation = target;
    }
    for (mut agent, mut arrive) in &mut agents {
        support::apply_agent_tuning(&mut agent, &pane);
        arrive.target = SteeringTarget::Point(target);
        arrive.slowing_radius = pane.slowing_radius;
        arrive.arrival_tolerance = pane.arrival_tolerance;
    }
}

fn update_kinematic_diagnostics(
    agent: Query<&Transform, With<TwoDAgent>>,
    target: Query<&Transform, (With<TargetMarker>, Without<TwoDAgent>)>,
    mut diagnostics: ResMut<Kinematic2dDiagnostics>,
) {
    let Ok(agent_transform) = agent.single() else {
        return;
    };
    let Ok(target_transform) = target.single() else {
        return;
    };

    diagnostics.agent_position = agent_transform.translation;
    diagnostics.target_position = target_transform.translation;
    diagnostics.distance_to_target = agent_transform
        .translation
        .distance(target_transform.translation);
}
