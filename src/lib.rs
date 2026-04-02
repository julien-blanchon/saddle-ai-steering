#![doc = include_str!("../README.md")]

mod behaviors;
mod components;
mod debug;
mod math;
mod resources;
mod systems;

pub use crate::components::{
    Arrive, BehaviorTuning, Evade, Flee, ObstacleAvoidance, PathFollowing, PathFollowingState,
    Pursue, Seek, SteeringAgent, SteeringAlignment, SteeringAutoApply, SteeringBehaviorKind,
    SteeringComposition, SteeringContribution, SteeringDebugAgent, SteeringDiagnostics,
    SteeringFacingMode, SteeringKinematics, SteeringLayerMask, SteeringObstacle,
    SteeringObstacleShape, SteeringOutput, SteeringPath, SteeringPathMode, SteeringPlane,
    SteeringTarget, SteeringTrackedVelocity, SteeringVelocitySource, Wander, WanderState,
};
pub use crate::debug::SteeringDebugGizmos;
pub use crate::resources::{SteeringDebugSettings, SteeringStats};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SteeringSystems {
    Gather,
    Evaluate,
    Apply,
    Debug,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct SteeringPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl SteeringPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for SteeringPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<resources::SteeringDebugSettings>()
            .init_resource::<resources::SteeringStats>()
            .init_resource::<resources::SteeringRuntimeState>()
            .init_gizmo_group::<debug::SteeringDebugGizmos>()
            .register_type::<Arrive>()
            .register_type::<BehaviorTuning>()
            .register_type::<Evade>()
            .register_type::<Flee>()
            .register_type::<ObstacleAvoidance>()
            .register_type::<PathFollowing>()
            .register_type::<PathFollowingState>()
            .register_type::<Pursue>()
            .register_type::<Seek>()
            .register_type::<SteeringAgent>()
            .register_type::<SteeringAlignment>()
            .register_type::<SteeringAutoApply>()
            .register_type::<SteeringBehaviorKind>()
            .register_type::<SteeringComposition>()
            .register_type::<SteeringContribution>()
            .register_type::<SteeringDebugAgent>()
            .register_type::<SteeringDiagnostics>()
            .register_type::<SteeringDebugSettings>()
            .register_type::<SteeringFacingMode>()
            .register_type::<SteeringKinematics>()
            .register_type::<SteeringLayerMask>()
            .register_type::<SteeringObstacle>()
            .register_type::<SteeringObstacleShape>()
            .register_type::<SteeringOutput>()
            .register_type::<SteeringPath>()
            .register_type::<SteeringPathMode>()
            .register_type::<SteeringPlane>()
            .register_type::<SteeringStats>()
            .register_type::<SteeringTarget>()
            .register_type::<SteeringTrackedVelocity>()
            .register_type::<SteeringVelocitySource>()
            .register_type::<Wander>()
            .register_type::<WanderState>()
            .configure_sets(
                self.update_schedule,
                (
                    SteeringSystems::Gather,
                    SteeringSystems::Evaluate,
                    SteeringSystems::Apply,
                    SteeringSystems::Debug,
                )
                    .chain(),
            )
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .add_systems(
                self.update_schedule,
                (
                    systems::setup_steering_entities,
                    systems::refresh_tracked_kinematics,
                )
                    .chain()
                    .in_set(SteeringSystems::Gather)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::evaluate_agents
                    .in_set(SteeringSystems::Evaluate)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::apply_auto_steering
                    .in_set(SteeringSystems::Apply)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                debug::draw_steering_debug
                    .in_set(SteeringSystems::Debug)
                    .run_if(systems::runtime_is_active)
                    .run_if(debug::debug_enabled),
            );
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
