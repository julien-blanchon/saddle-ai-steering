use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

use crate::LabDiagnostics;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "steering_smoke",
        "steering_path_following",
        "steering_avoidance",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "steering_smoke" => Some(steering_smoke()),
        "steering_path_following" => Some(steering_path_following()),
        "steering_avoidance" => Some(steering_avoidance()),
        _ => None,
    }
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description(
            "Boot the steering lab, settle one frame of motion, and capture the initial overview.",
        )
        .then(Action::WaitFrames(12))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "all five showcase agents are active at launch",
            |diagnostics| diagnostics.active_agents == 5,
        ))
        .then(Action::Screenshot("overview".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn steering_smoke() -> Scenario {
    Scenario::builder("steering_smoke")
        .description("Verify the main behaviors are active together: the pursuer closes distance, the path agent advances, and the wander agent keeps moving.")
        .then(Action::WaitFrames(90))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "pursuit closes to a reasonable range",
            |diagnostics| diagnostics.pursuit_distance < 12.5,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "wander remains active",
            |diagnostics| diagnostics.wander_speed > 0.1,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "path agent has advanced at least one waypoint",
            |diagnostics| diagnostics.path_waypoint > 0 || diagnostics.path_cycles > 0,
        ))
        .then(Action::Screenshot("smoke_runtime".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_smoke"))
        .build()
}

fn steering_path_following() -> Scenario {
    Scenario::builder("steering_path_following")
        .description(
            "Capture a path follower at the start and midpoints, then assert waypoint progress after a full lap opportunity.",
        )
        .then(Action::WaitFrames(10))
        .then(Action::Screenshot("path_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(240))
        .then(Action::Screenshot("path_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(240))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "path follower advanced through multiple waypoints",
            |diagnostics| diagnostics.path_waypoint >= 2 || diagnostics.path_cycles >= 1,
        ))
        .then(Action::Screenshot("path_loop".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_path_following"))
        .build()
}

fn steering_avoidance() -> Scenario {
    Scenario::builder("steering_avoidance")
        .description("Track the avoidance agent as it approaches a blocking obstacle, bends around it, and ends up past the obstacle with positive clearance.")
        .then(Action::WaitFrames(12))
        .then(Action::Screenshot("avoidance_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(140))
        .then(Action::Screenshot("avoidance_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "avoidance agent clears the obstacle with positive clearance",
            |diagnostics| {
                diagnostics.avoidance_passed_obstacle && diagnostics.avoidance_min_clearance > 0.05
            },
        ))
        .then(Action::Screenshot("avoidance_clear".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_avoidance"))
        .build()
}
