use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use steering::{SteeringAgent, SteeringPlane};

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["smoke_launch", "formation_cycle"]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "formation_cycle" => Some(formation_cycle()),
        _ => None,
    }
}

fn log_formation_diagnostics(label: &'static str) -> Action {
    Action::Custom(Box::new(move |world| {
        let diagnostics = world.resource::<crate::FormationDiagnostics>();
        info!(
            "[e2e:{label}] mode={} followers={} avg_slot_error={:.2} max_slot_error={:.2}",
            diagnostics.mode_label,
            diagnostics.follower_count,
            diagnostics.avg_slot_error,
            diagnostics.max_slot_error,
        );
    }))
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the formation example, verify the leader and six followers spawn on the XZ plane, and capture the default wedge overview.")
        .then(Action::WaitFrames(30))
        .then(assertions::resource_exists::<crate::FormationDiagnostics>(
            "formation diagnostics resource exists",
        ))
        .then(assertions::entity_count::<crate::Follower>(
            "six followers spawned",
            6,
        ))
        .then(assertions::component_where::<SteeringAgent, crate::Leader>(
            "leader uses XZ steering plane",
            |agent| agent.plane == SteeringPlane::XZ,
        ))
        .then(Action::Screenshot("formation_wedge_boot".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn formation_cycle() -> Scenario {
    Scenario::builder("formation_cycle")
        .description("Let the wedge settle, then press F twice to cycle through line and circle formations while verifying the followers keep converging to their slot layout.")
        .then(Action::WaitFrames(210))
        .then(log_formation_diagnostics("wedge"))
        .then(assertions::resource_satisfies::<crate::FormationDiagnostics>(
            "wedge settles with all followers assigned",
            |diagnostics| {
                diagnostics.mode_label == "Wedge"
                    && diagnostics.follower_count == 6
                    && diagnostics.avg_slot_error < 20.0
                    && diagnostics.max_slot_error < 40.0
            },
        ))
        .then(Action::Screenshot("formation_wedge".into()))
        .then(Action::PressKey(KeyCode::KeyF))
        .then(Action::WaitFrames(1))
        .then(Action::ReleaseKey(KeyCode::KeyF))
        .then(Action::WaitUntil {
            label: "line mode selected".into(),
            condition: Box::new(|world| {
                world.resource::<crate::FormationDiagnostics>().mode_label == "Line"
            }),
            max_frames: 30,
        })
        .then(Action::WaitFrames(210))
        .then(log_formation_diagnostics("line"))
        .then(assertions::resource_satisfies::<crate::FormationDiagnostics>(
            "line formation converges after cycling",
            |diagnostics| {
                diagnostics.mode_label == "Line"
                    && diagnostics.follower_count == 6
                    && diagnostics.avg_slot_error < 20.0
                    && diagnostics.max_slot_error < 40.0
            },
        ))
        .then(Action::Screenshot("formation_line".into()))
        .then(Action::PressKey(KeyCode::KeyF))
        .then(Action::WaitFrames(1))
        .then(Action::ReleaseKey(KeyCode::KeyF))
        .then(Action::WaitUntil {
            label: "circle mode selected".into(),
            condition: Box::new(|world| {
                world.resource::<crate::FormationDiagnostics>().mode_label == "Circle"
            }),
            max_frames: 30,
        })
        .then(Action::WaitFrames(210))
        .then(log_formation_diagnostics("circle"))
        .then(assertions::resource_satisfies::<crate::FormationDiagnostics>(
            "circle formation converges after cycling",
            |diagnostics| {
                diagnostics.mode_label == "Circle"
                    && diagnostics.follower_count == 6
                    && diagnostics.avg_slot_error < 20.0
                    && diagnostics.max_slot_error < 40.0
            },
        ))
        .then(Action::Screenshot("formation_circle".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("formation_cycle"))
        .build()
}
