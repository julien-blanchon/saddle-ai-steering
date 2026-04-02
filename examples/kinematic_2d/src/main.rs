use steering_example_support as support;

use bevy::prelude::*;
use steering::{Arrive, SteeringAgent, SteeringAutoApply, SteeringPlane, SteeringTarget};

fn main() {
    let mut app = App::new();
    support::configure_2d_app(&mut app, "steering: kinematic 2d");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let target = Vec3::new(260.0, 120.0, 0.0);
    commands.spawn((
        Name::new("2D Target"),
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
