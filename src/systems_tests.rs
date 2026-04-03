use super::*;

#[derive(Resource, Default)]
struct OrderLog(Vec<&'static str>);

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Active,
    Inactive,
}

fn make_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
        bevy::gizmos::GizmoPlugin,
    ));
    app
}

#[test]
fn plugin_registers_resources_and_produces_output() {
    let mut app = make_test_app();
    app.init_state::<DemoState>()
        .add_plugins(SteeringPlugin::new(
            OnEnter(DemoState::Active),
            OnExit(DemoState::Active),
            Update,
        ));

    let entity = app
        .world_mut()
        .spawn((
            SteeringAgent::default(),
            SteeringAutoApply::default(),
            Seek::new(SteeringTarget::point(4.0, 0.0, 0.0)),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.update();

    assert!(app.world().contains_resource::<SteeringDebugSettings>());
    assert!(app.world().contains_resource::<SteeringStats>());
    let output = app
        .world()
        .entity(entity)
        .get::<SteeringOutput>()
        .expect("steering output should exist for the spawned agent");
    assert!(output.linear_acceleration.length() > 0.0);
}

#[test]
fn steering_sets_chain_in_expected_order() {
    let mut app = make_test_app();
    app.add_plugins(SteeringPlugin::default())
        .init_resource::<OrderLog>()
        .add_systems(Update, record_gather.in_set(SteeringSystems::Gather))
        .add_systems(Update, record_evaluate.in_set(SteeringSystems::Evaluate))
        .add_systems(Update, record_apply.in_set(SteeringSystems::Apply))
        .add_systems(Update, record_debug.in_set(SteeringSystems::Debug));

    app.update();

    let log = &app.world().resource::<OrderLog>().0;
    assert_eq!(log, &["gather", "evaluate", "apply", "debug"]);
}

#[test]
fn deactivate_schedule_disables_runtime() {
    let mut app = make_test_app();
    app.init_state::<DemoState>()
        .add_plugins(SteeringPlugin::new(
            OnEnter(DemoState::Active),
            OnExit(DemoState::Active),
            Update,
        ));

    app.update();
    app.world_mut()
        .resource_mut::<NextState<DemoState>>()
        .set(DemoState::Inactive);
    app.update();

    let runtime = app.world().resource::<resources::SteeringRuntimeState>();
    assert!(!runtime.active);
}

#[test]
fn deactivation_clears_existing_output() {
    let mut app = make_test_app();
    app.init_state::<DemoState>()
        .add_plugins(SteeringPlugin::new(
            OnEnter(DemoState::Active),
            OnExit(DemoState::Active),
            Update,
        ));

    let entity = app
        .world_mut()
        .spawn((
            SteeringAgent::default(),
            SteeringAutoApply::default(),
            Seek::new(SteeringTarget::point(4.0, 0.0, 0.0)),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.update();
    let active_output = app
        .world()
        .entity(entity)
        .get::<SteeringOutput>()
        .expect("output should be present after activation")
        .clone();
    assert!(active_output.linear_acceleration.length() > 0.0);

    app.world_mut()
        .resource_mut::<NextState<DemoState>>()
        .set(DemoState::Inactive);
    app.update();

    let cleared_output = app
        .world()
        .entity(entity)
        .get::<SteeringOutput>()
        .expect("output should still be present after deactivation");
    let cleared_diagnostics = app
        .world()
        .entity(entity)
        .get::<SteeringDiagnostics>()
        .expect("diagnostics should still be present after deactivation");

    assert_eq!(*cleared_output, SteeringOutput::default());
    assert_eq!(*cleared_diagnostics, SteeringDiagnostics::default());
}

#[test]
fn missing_target_entity_is_ignored_without_panic() {
    let mut app = make_test_app();
    app.add_plugins(SteeringPlugin::default());

    let entity = app
        .world_mut()
        .spawn((
            SteeringAgent::default(),
            SteeringAutoApply::default(),
            Pursue::new(SteeringTarget::Entity(Entity::from_bits(999))),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.update();

    let output = app
        .world()
        .entity(entity)
        .get::<SteeringOutput>()
        .expect("steering output should still exist");
    assert_eq!(output.linear_acceleration, Vec3::ZERO);
    assert_eq!(output.desired_velocity, Vec3::ZERO);
}

#[test]
fn empty_world_does_not_panic() {
    let mut app = make_test_app();
    app.add_plugins(SteeringPlugin::default());
    app.update();
}

#[test]
fn flocking_aligns_with_neighbor_heading() {
    let mut app = make_test_app();
    app.add_plugins(SteeringPlugin::default());

    app.world_mut().spawn((
        SteeringAgent::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(0.0, 0.0, 2.0),
        },
        Transform::from_xyz(-1.0, 0.0, 0.0),
        GlobalTransform::from(Transform::from_xyz(-1.0, 0.0, 0.0)),
    ));
    app.world_mut().spawn((
        SteeringAgent::default(),
        SteeringKinematics {
            linear_velocity: Vec3::new(0.0, 0.0, 2.0),
        },
        Transform::from_xyz(1.0, 0.0, 0.0),
        GlobalTransform::from(Transform::from_xyz(1.0, 0.0, 0.0)),
    ));

    let entity = app
        .world_mut()
        .spawn((
            SteeringAgent::default(),
            Flocking {
                separation_weight: 0.0,
                cohesion_weight: 0.0,
                alignment_weight: 1.5,
                ..default()
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.update();

    let output = app
        .world()
        .entity(entity)
        .get::<SteeringOutput>()
        .expect("flocking output should exist");
    let diagnostics = app
        .world()
        .entity(entity)
        .get::<SteeringDiagnostics>()
        .expect("flocking diagnostics should exist");

    assert!(output.desired_velocity.z > 0.5);
    assert_eq!(diagnostics.flock_neighbor_count, 2);
}

#[test]
fn reciprocal_avoidance_deflects_head_on_agents() {
    let mut app = make_test_app();
    app.add_plugins(SteeringPlugin::default());

    let left = app
        .world_mut()
        .spawn((
            SteeringAgent::default(),
            Seek::new(SteeringTarget::point(4.0, 0.0, 0.0)),
            ReciprocalAvoidance::default(),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();
    app.world_mut().spawn((
        SteeringAgent::default(),
        Seek::new(SteeringTarget::point(-4.0, 0.0, 0.0)),
        ReciprocalAvoidance::default(),
        Transform::from_xyz(1.2, 0.0, 0.0),
        GlobalTransform::from(Transform::from_xyz(1.2, 0.0, 0.0)),
    ));

    app.update();

    let output = app
        .world()
        .entity(left)
        .get::<SteeringOutput>()
        .expect("reciprocal avoidance output should exist");
    let diagnostics = app
        .world()
        .entity(left)
        .get::<SteeringDiagnostics>()
        .expect("reciprocal avoidance diagnostics should exist");

    assert!(diagnostics.crowd_neighbor_count > 0);
    assert!(output.desired_velocity.z.abs() > 0.1);
}

fn record_gather(mut log: ResMut<OrderLog>) {
    log.0.push("gather");
}

fn record_evaluate(mut log: ResMut<OrderLog>) {
    log.0.push("evaluate");
}

fn record_apply(mut log: ResMut<OrderLog>) {
    log.0.push("apply");
}

fn record_debug(mut log: ResMut<OrderLog>) {
    log.0.push("debug");
}
