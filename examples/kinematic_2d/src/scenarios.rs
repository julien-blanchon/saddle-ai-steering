use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use steering::{SteeringAgent, SteeringPlane};
use steering_example_support as support;

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["smoke_launch", "kinematic_2d_retarget"]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "kinematic_2d_retarget" => Some(kinematic_2d_retarget()),
        _ => None,
    }
}

fn move_target(x: f32, y: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        let mut pane = world.resource_mut::<support::SteeringExamplePane>();
        pane.target_x_2d = x;
        pane.target_y_2d = y;
    }))
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the 2D steering example, verify the agent uses the XY plane, and capture the initial top-down arrive setup.")
        .then(Action::WaitFrames(20))
        .then(assertions::resource_exists::<crate::Kinematic2dDiagnostics>(
            "2D diagnostics resource exists",
        ))
        .then(assertions::component_where::<SteeringAgent, crate::TwoDAgent>(
            "agent uses XY steering plane",
            |agent| agent.plane == SteeringPlane::XY,
        ))
        .then(assertions::resource_satisfies::<crate::Kinematic2dDiagnostics>(
            "agent starts far enough from the target to demonstrate arrival",
            |diagnostics| diagnostics.distance_to_target > 300.0,
        ))
        .then(Action::Screenshot("kinematic_2d_boot".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn kinematic_2d_retarget() -> Scenario {
    Scenario::builder("kinematic_2d_retarget")
        .description("Let the 2D agent arrive toward the default target, then retarget it across the map and verify it reorients and closes the new gap.")
        .then(Action::WaitFrames(180))
        .then(assertions::resource_satisfies::<crate::Kinematic2dDiagnostics>(
            "agent closes substantial distance to the initial target",
            |diagnostics| diagnostics.distance_to_target < 260.0,
        ))
        .then(Action::Screenshot("kinematic_2d_first_arrive".into()))
        .then(move_target(-260.0, 140.0))
        .then(Action::WaitFrames(10))
        .then(assertions::resource_satisfies::<crate::Kinematic2dDiagnostics>(
            "retarget moved the destination into the negative X half-plane",
            |diagnostics| diagnostics.target_position.x < 0.0,
        ))
        .then(Action::Screenshot("kinematic_2d_retargeted".into()))
        .then(Action::WaitFrames(180))
        .then(assertions::resource_satisfies::<crate::Kinematic2dDiagnostics>(
            "agent closes the new retargeted distance",
            |diagnostics| {
                diagnostics.target_position.x < 0.0
                    && diagnostics.distance_to_target < 260.0
                    && diagnostics.agent_position.x < 0.0
            },
        ))
        .then(Action::Screenshot("kinematic_2d_retarget_arrived".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("kinematic_2d_retarget"))
        .build()
}
