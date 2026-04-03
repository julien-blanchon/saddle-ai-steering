use bevy::prelude::*;

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct SteeringDebugSettings {
    pub enabled: bool,
    pub draw_velocity: bool,
    pub draw_output: bool,
    pub draw_targets: bool,
    pub draw_arrival_radii: bool,
    pub draw_wander: bool,
    pub draw_obstacles: bool,
    pub draw_probes: bool,
    pub draw_paths: bool,
    pub draw_labels: bool,
}

impl Default for SteeringDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_velocity: true,
            draw_output: true,
            draw_targets: true,
            draw_arrival_radii: true,
            draw_wander: true,
            draw_obstacles: true,
            draw_probes: true,
            draw_paths: true,
            draw_labels: false,
        }
    }
}

#[derive(Resource, Reflect, Clone, Debug, Default)]
#[reflect(Resource)]
pub struct SteeringStats {
    pub evaluated_agents: usize,
    pub active_behaviors: usize,
    pub obstacle_tests: usize,
    pub obstacle_hits: usize,
    pub flock_neighbors: usize,
    pub crowd_neighbors: usize,
    pub crowd_conflicts: usize,
}

#[derive(Resource, Default)]
pub(crate) struct SteeringRuntimeState {
    pub active: bool,
}
