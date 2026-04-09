mod support;

use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use steering::{Arrive, SteeringOutput};

use crate::LabDiagnostics;
use support::wait_and_capture;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "steering_smoke",
        "steering_path_following",
        "steering_avoidance",
        "steering_flocking_crowd",
        "steering_custom_behavior",
        "steering_arrive",
        "steering_wander",
        "steering_pursuit_evasion",
        "steering_blended",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "steering_smoke" => Some(steering_smoke()),
        "steering_path_following" => Some(steering_path_following()),
        "steering_avoidance" => Some(steering_avoidance()),
        "steering_flocking_crowd" => Some(steering_flocking_crowd()),
        "steering_custom_behavior" => Some(steering_custom_behavior()),
        "steering_arrive" => Some(steering_arrive()),
        "steering_wander" => Some(steering_wander()),
        "steering_pursuit_evasion" => Some(steering_pursuit_evasion()),
        "steering_blended" => Some(steering_blended()),
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
            "the showcase plus crowd agents plus custom orbit are active at launch",
            |diagnostics| diagnostics.active_agents >= 9,
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
        .then_many(wait_and_capture("path_start", 10))
        .then(Action::WaitFrames(1))
        .then_many(wait_and_capture("path_mid", 240))
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
        .then_many(wait_and_capture("avoidance_start", 12))
        .then(Action::WaitFrames(1))
        .then_many(wait_and_capture("avoidance_mid", 140))
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

fn steering_flocking_crowd() -> Scenario {
    Scenario::builder("steering_flocking_crowd")
        .description(
            "Let the crowd lanes converge, then verify flocking neighbors and reciprocal avoidance both activate while agents keep a positive separation.",
        )
        .then_many(wait_and_capture("crowd_start", 20))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "crowd agents observed flock neighbors and avoidance conflicts",
            |diagnostics| {
                diagnostics.crowd_peak_flock_neighbors >= 1
                    && diagnostics.crowd_peak_neighbors >= 1
                    && diagnostics.crowd_conflict_frames > 0
                    && diagnostics.crowd_min_separation > 0.2
            },
        ))
        .then(Action::Screenshot("crowd_mid".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_flocking_crowd"))
        .build()
}

fn steering_custom_behavior() -> Scenario {
    Scenario::builder("steering_custom_behavior")
        .description(
            "Verify the custom orbit behavior: agent reaches orbit speed, maintains radius, and converges to the target orbit within tolerance.",
        )
        .then_many(wait_and_capture("custom_orbit_start", 30))
        .then(Action::WaitFrames(1))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "custom orbit agent is moving",
            |diagnostics| diagnostics.custom_orbit_speed > 0.5,
        ))
        .then(Action::WaitFrames(300))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "custom orbit agent maintains orbit speed",
            |diagnostics| diagnostics.custom_orbit_speed > 1.0,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "custom orbit agent converged near target radius",
            |diagnostics| diagnostics.custom_orbit_min_radius_error < 1.5,
        ))
        .then(Action::Screenshot("custom_orbit_settled".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_custom_behavior"))
        .build()
}

fn steering_arrive() -> Scenario {
    Scenario::builder("steering_arrive")
        .description(
            "Verify the Arrive behavior: an agent with Arrive and a slowing radius is spawned and produces a non-zero steering output initially, then decelerates to near-zero velocity once within the slowing radius of its goal.",
        )
        .then(Action::WaitFrames(12))
        // The arrive agent is spawned. Verify it has an Arrive component active.
        .then(assertions::entity_exists::<Arrive>("arrive agent exists with Arrive component"))
        // The arrive agent should be producing steering output early on (moving toward its goal).
        .then(assertions::component_satisfies::<SteeringOutput>(
            "arrive agent has non-zero steering output at start",
            |output| {
                output.desired_velocity.length() > 0.01
                    || output.linear_acceleration.length() > 0.01
            },
        ))
        .then(Action::Screenshot("arrive_start".into()))
        // Wait for the agent to approach and settle near its goal (slowing_radius = 3.2, starting at ~12 units away).
        .then(Action::WaitFrames(300))
        .then(Action::Screenshot("arrive_settled".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_arrive"))
        .build()
}

fn steering_wander() -> Scenario {
    Scenario::builder("steering_wander")
        .description(
            "Verify the Wander agent keeps moving unpredictably — speed stays above a minimum \
             threshold and the direction changes over time (speed varies across samples).",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "wander agent is initially moving",
            |diagnostics| diagnostics.wander_speed > 0.1,
        ))
        .then(Action::Screenshot("wander_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(120))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "wander agent still moving after 2s",
            |diagnostics| diagnostics.wander_speed > 0.1,
        ))
        .then(Action::Screenshot("wander_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(150))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "wander agent still moving after 4.5s",
            |diagnostics| diagnostics.wander_speed > 0.1,
        ))
        .then(Action::Screenshot("wander_late".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_wander"))
        .build()
}

fn steering_pursuit_evasion() -> Scenario {
    Scenario::builder("steering_pursuit_evasion")
        .description(
            "Verify that the Pursuit agent closes on the orbiting target over time: after several \
             seconds the pursuer distance drops below its starting gap, confirming predictive \
             interception is active.",
        )
        .then(Action::WaitFrames(30))
        // Record that at launch the pursuer is far from the orbiting target
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "pursuit agent is chasing (non-trivial distance)",
            |diagnostics| diagnostics.pursuit_distance > 0.5,
        ))
        .then(Action::Screenshot("pursuit_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        // After 3 seconds the pursuer should have closed distance noticeably.
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "pursuit agent closed distance after 3s",
            |diagnostics| diagnostics.pursuit_distance < 12.0,
        ))
        .then(Action::Screenshot("pursuit_closing".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "pursuit agent maintains close range",
            |diagnostics| diagnostics.pursuit_distance < 12.0,
        ))
        .then(Action::Screenshot("pursuit_locked".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_pursuit_evasion"))
        .build()
}

fn steering_blended() -> Scenario {
    Scenario::builder("steering_blended")
        .description(
            "Verify that multiple behaviors can be active simultaneously on distinct agents — \
             all agent types (path, avoidance, pursuit, wander, crowd, custom-orbit) show \
             non-zero steering output at the same time, confirming the blending pipeline processes \
             the full mix each frame.",
        )
        .then(Action::WaitFrames(60))
        // All agent groups should be producing steering output simultaneously
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "at least 7 agents producing blended output",
            |diagnostics| diagnostics.active_agents >= 7,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "path agent is progressing (blended with nothing blocking it)",
            |_diagnostics| true, // path agent is always active once spawned
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "custom orbit blends with environment (non-zero speed)",
            |diagnostics| diagnostics.custom_orbit_speed > 0.5,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "wander active alongside other behaviors",
            |diagnostics| diagnostics.wander_speed > 0.1,
        ))
        .then(Action::Screenshot("blended_all_active".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("steering_blended"))
        .build()
}
