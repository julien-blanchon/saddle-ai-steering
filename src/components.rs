use crate::math::LinearIntent;
use bevy::prelude::*;

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SteeringPlane {
    XY,
    #[default]
    XZ,
    Free3d,
}

impl SteeringPlane {
    pub fn align_point(self, origin: Vec3, point: Vec3) -> Vec3 {
        match self {
            SteeringPlane::XY => Vec3::new(point.x, point.y, origin.z),
            SteeringPlane::XZ => Vec3::new(point.x, origin.y, point.z),
            SteeringPlane::Free3d => point,
        }
    }

    pub fn project_vector(self, value: Vec3) -> Vec3 {
        match self {
            SteeringPlane::XY => Vec3::new(value.x, value.y, 0.0),
            SteeringPlane::XZ => Vec3::new(value.x, 0.0, value.z),
            SteeringPlane::Free3d => value,
        }
    }

    pub fn clamp_translation(self, current: Vec3, next: Vec3) -> Vec3 {
        match self {
            SteeringPlane::XY => Vec3::new(next.x, next.y, current.z),
            SteeringPlane::XZ => Vec3::new(next.x, current.y, next.z),
            SteeringPlane::Free3d => next,
        }
    }

    pub fn distance(self, a: Vec3, b: Vec3) -> f32 {
        self.project_vector(b - a).length()
    }

    pub fn up(self) -> Vec3 {
        match self {
            SteeringPlane::XY => Vec3::Z,
            SteeringPlane::XZ | SteeringPlane::Free3d => Vec3::Y,
        }
    }

    pub fn forward_from_transform(self, transform: &Transform) -> Vec3 {
        match self {
            SteeringPlane::XY => self.project_vector(*transform.right()).normalize_or_zero(),
            SteeringPlane::XZ | SteeringPlane::Free3d => self
                .project_vector(*transform.forward())
                .normalize_or_zero(),
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SteeringComposition {
    WeightedBlend,
    #[default]
    PrioritizedAccumulation,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SteeringVelocitySource {
    #[default]
    Kinematics,
    TransformDelta,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SteeringFacingMode {
    None,
    Velocity,
    #[default]
    DesiredVelocity,
    DesiredHeading,
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SteeringBehaviorKind {
    Seek,
    Flee,
    Arrive,
    Pursue,
    Evade,
    Wander,
    Flocking,
    ObstacleAvoidance,
    ReciprocalAvoidance,
    PathFollowing,
    LeaderFollowing,
    Formation,
    Containment,
    Brake,
    Custom(String),
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SteeringPathMode {
    #[default]
    Once,
    Loop,
    PingPong,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct SteeringLayerMask(pub u32);

impl SteeringLayerMask {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u32::MAX);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn from_bit(bit: u8) -> Self {
        Self(1_u32 << bit)
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct SteeringAlignment {
    pub mode: SteeringFacingMode,
    pub turn_speed_radians: f32,
    pub min_speed: f32,
}

impl Default for SteeringAlignment {
    fn default() -> Self {
        Self {
            mode: SteeringFacingMode::DesiredVelocity,
            turn_speed_radians: 12.0,
            min_speed: 0.05,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct BehaviorTuning {
    pub enabled: bool,
    pub weight: f32,
    pub priority: u8,
}

impl Default for BehaviorTuning {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: 1.0,
            priority: 50,
        }
    }
}

impl BehaviorTuning {
    pub fn new(weight: f32, priority: u8) -> Self {
        Self {
            enabled: true,
            weight,
            priority,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub enum SteeringTarget {
    Point(Vec3),
    Entity(Entity),
}

impl SteeringTarget {
    pub fn point(x: f32, y: f32, z: f32) -> Self {
        Self::Point(Vec3::new(x, y, z))
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct SteeringPath {
    pub points: Vec<Vec3>,
    pub mode: SteeringPathMode,
    pub waypoint_tolerance: f32,
    pub lookahead_distance: f32,
}

impl Default for SteeringPath {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            mode: SteeringPathMode::Once,
            waypoint_tolerance: 0.6,
            lookahead_distance: 1.8,
        }
    }
}

impl SteeringPath {
    pub fn new(points: impl IntoIterator<Item = Vec3>) -> Self {
        Self {
            points: points.into_iter().collect(),
            ..default()
        }
    }

    pub fn looped(mut self) -> Self {
        self.mode = SteeringPathMode::Loop;
        self
    }

    pub fn ping_pong(mut self) -> Self {
        self.mode = SteeringPathMode::PingPong;
        self
    }

    pub fn with_lookahead_distance(mut self, lookahead_distance: f32) -> Self {
        self.lookahead_distance = lookahead_distance;
        self
    }

    pub fn with_waypoint_tolerance(mut self, waypoint_tolerance: f32) -> Self {
        self.waypoint_tolerance = waypoint_tolerance;
        self
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct SteeringAgent {
    pub max_speed: f32,
    pub max_acceleration: f32,
    pub mass: f32,
    pub body_radius: f32,
    pub crowd_layers: SteeringLayerMask,
    pub plane: SteeringPlane,
    pub composition: SteeringComposition,
    pub velocity_source: SteeringVelocitySource,
    pub braking_acceleration: f32,
    pub alignment: SteeringAlignment,
}

impl Default for SteeringAgent {
    fn default() -> Self {
        Self {
            max_speed: 6.0,
            max_acceleration: 12.0,
            mass: 1.0,
            body_radius: 0.45,
            crowd_layers: SteeringLayerMask::ALL,
            plane: SteeringPlane::XZ,
            composition: SteeringComposition::PrioritizedAccumulation,
            velocity_source: SteeringVelocitySource::Kinematics,
            braking_acceleration: 10.0,
            alignment: SteeringAlignment::default(),
        }
    }
}

impl SteeringAgent {
    pub fn new(plane: SteeringPlane) -> Self {
        Self { plane, ..default() }
    }

    pub fn with_max_speed(mut self, max_speed: f32) -> Self {
        self.max_speed = max_speed;
        self
    }

    pub fn with_max_acceleration(mut self, max_acceleration: f32) -> Self {
        self.max_acceleration = max_acceleration;
        self
    }

    pub fn with_crowd_layers(mut self, crowd_layers: SteeringLayerMask) -> Self {
        self.crowd_layers = crowd_layers;
        self
    }

    pub fn with_composition(mut self, composition: SteeringComposition) -> Self {
        self.composition = composition;
        self
    }

    pub fn with_velocity_source(mut self, velocity_source: SteeringVelocitySource) -> Self {
        self.velocity_source = velocity_source;
        self
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Default)]
#[reflect(Component)]
pub struct SteeringTrackedVelocity;

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct SteeringAutoApply {
    pub apply_translation: bool,
    pub apply_facing: bool,
}

impl Default for SteeringAutoApply {
    fn default() -> Self {
        Self {
            apply_translation: true,
            apply_facing: true,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Default)]
#[reflect(Component)]
pub struct SteeringKinematics {
    pub linear_velocity: Vec3,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct SteeringOutput {
    pub linear_acceleration: Vec3,
    pub desired_velocity: Vec3,
    pub desired_facing: Option<Vec3>,
}

impl Default for SteeringOutput {
    fn default() -> Self {
        Self {
            linear_acceleration: Vec3::ZERO,
            desired_velocity: Vec3::ZERO,
            desired_facing: None,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct SteeringContribution {
    pub behavior: SteeringBehaviorKind,
    pub priority: u8,
    pub weight: f32,
    pub requested_acceleration: Vec3,
    pub applied_acceleration: Vec3,
    pub desired_velocity: Vec3,
    pub suppressed: bool,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq, Default)]
#[reflect(Component)]
pub struct SteeringDiagnostics {
    pub dominant_behavior: Option<SteeringBehaviorKind>,
    pub contributions: Vec<SteeringContribution>,
    pub primary_target: Option<Vec3>,
    pub path_target: Option<Vec3>,
    pub wander_circle_center: Option<Vec3>,
    pub wander_target: Option<Vec3>,
    pub flock_center: Option<Vec3>,
    pub flock_heading: Option<Vec3>,
    pub flock_neighbor_count: usize,
    pub probe_end: Option<Vec3>,
    pub avoidance_hit_point: Option<Vec3>,
    pub avoidance_normal: Option<Vec3>,
    pub avoidance_obstacle: Option<Entity>,
    pub crowd_avoidance_velocity: Option<Vec3>,
    pub crowd_neighbor_count: usize,
    pub pre_avoidance_velocity: Vec3,
    pub formation_slot_position: Option<Vec3>,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct SteeringDebugAgent {
    pub enabled: bool,
}

impl Default for SteeringDebugAgent {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct SteeringObstacle {
    pub shape: SteeringObstacleShape,
    pub layers: SteeringLayerMask,
}

impl SteeringObstacle {
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: SteeringObstacleShape::Sphere { radius },
            layers: SteeringLayerMask::ALL,
        }
    }

    pub fn aabb(half_extents: Vec3) -> Self {
        Self {
            shape: SteeringObstacleShape::Aabb { half_extents },
            layers: SteeringLayerMask::ALL,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum SteeringObstacleShape {
    Sphere { radius: f32 },
    Aabb { half_extents: Vec3 },
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Seek {
    pub target: SteeringTarget,
    pub tuning: BehaviorTuning,
}

impl Seek {
    pub fn new(target: SteeringTarget) -> Self {
        Self {
            target,
            tuning: BehaviorTuning::new(1.0, 40),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Flee {
    pub target: SteeringTarget,
    pub panic_distance: Option<f32>,
    pub tuning: BehaviorTuning,
}

impl Flee {
    pub fn new(target: SteeringTarget) -> Self {
        Self {
            target,
            panic_distance: None,
            tuning: BehaviorTuning::new(1.0, 15),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Arrive {
    pub target: SteeringTarget,
    pub slowing_radius: f32,
    pub arrival_tolerance: f32,
    pub speed_curve_exponent: f32,
    pub tuning: BehaviorTuning,
}

impl Arrive {
    pub fn new(target: SteeringTarget) -> Self {
        Self {
            target,
            slowing_radius: 3.0,
            arrival_tolerance: 0.25,
            speed_curve_exponent: 1.0,
            tuning: BehaviorTuning::new(1.0, 45),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Pursue {
    pub target: SteeringTarget,
    pub lead_scale: f32,
    pub max_prediction_time: f32,
    pub tuning: BehaviorTuning,
}

impl Pursue {
    pub fn new(target: SteeringTarget) -> Self {
        Self {
            target,
            lead_scale: 1.0,
            max_prediction_time: 1.25,
            tuning: BehaviorTuning::new(1.0, 35),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Evade {
    pub target: SteeringTarget,
    pub lead_scale: f32,
    pub max_prediction_time: f32,
    pub panic_distance: Option<f32>,
    pub tuning: BehaviorTuning,
}

impl Evade {
    pub fn new(target: SteeringTarget) -> Self {
        Self {
            target,
            lead_scale: 1.0,
            max_prediction_time: 1.25,
            panic_distance: None,
            tuning: BehaviorTuning::new(1.0, 10),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Wander {
    pub radius: f32,
    pub distance: f32,
    pub jitter_radians_per_second: f32,
    pub seed: u64,
    pub vertical_jitter: f32,
    pub tuning: BehaviorTuning,
}

impl Default for Wander {
    fn default() -> Self {
        Self {
            radius: 2.0,
            distance: 3.0,
            jitter_radians_per_second: 1.8,
            seed: 1,
            vertical_jitter: 0.35,
            tuning: BehaviorTuning::new(1.0, 80),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct WanderState {
    pub rng_state: u64,
    pub local_target: Vec3,
    pub initialized: bool,
}

impl WanderState {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng_state: seed.max(1),
            local_target: Vec3::X,
            initialized: false,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Flocking {
    pub separation_radius: f32,
    pub alignment_radius: f32,
    pub cohesion_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_neighbors: usize,
    pub layers: SteeringLayerMask,
    pub tuning: BehaviorTuning,
}

impl Default for Flocking {
    fn default() -> Self {
        Self {
            separation_radius: 1.25,
            alignment_radius: 3.0,
            cohesion_radius: 4.0,
            separation_weight: 1.6,
            alignment_weight: 0.95,
            cohesion_weight: 0.85,
            max_neighbors: 12,
            layers: SteeringLayerMask::ALL,
            tuning: BehaviorTuning::new(1.0, 55),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct ObstacleAvoidance {
    pub min_lookahead: f32,
    pub max_lookahead: f32,
    pub probe_radius: f32,
    pub braking_weight: f32,
    pub lateral_weight: f32,
    pub layers: SteeringLayerMask,
    pub tuning: BehaviorTuning,
}

impl Default for ObstacleAvoidance {
    fn default() -> Self {
        Self {
            min_lookahead: 1.5,
            max_lookahead: 4.5,
            probe_radius: 0.2,
            braking_weight: 0.55,
            lateral_weight: 1.0,
            layers: SteeringLayerMask::ALL,
            tuning: BehaviorTuning::new(1.0, 0),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct ReciprocalAvoidance {
    pub neighbor_distance: f32,
    pub time_horizon: f32,
    pub comfort_distance: f32,
    pub side_bias: f32,
    pub max_neighbors: usize,
    pub layers: SteeringLayerMask,
    pub tuning: BehaviorTuning,
}

impl Default for ReciprocalAvoidance {
    fn default() -> Self {
        Self {
            neighbor_distance: 3.6,
            time_horizon: 1.2,
            comfort_distance: 0.15,
            side_bias: 0.1,
            max_neighbors: 8,
            layers: SteeringLayerMask::ALL,
            tuning: BehaviorTuning::new(1.0, 5),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct PathFollowing {
    pub path: SteeringPath,
    pub slowing_radius: f32,
    pub arrival_tolerance: f32,
    pub tuning: BehaviorTuning,
}

impl PathFollowing {
    pub fn new(path: SteeringPath) -> Self {
        Self {
            path,
            slowing_radius: 3.0,
            arrival_tolerance: 0.3,
            tuning: BehaviorTuning::new(1.0, 40),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq, Default)]
#[reflect(Component)]
pub struct PathFollowingState {
    pub current_waypoint: usize,
    pub direction: i8,
    pub completed: bool,
    pub completed_cycles: u32,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct LeaderFollowing {
    pub leader: SteeringTarget,
    pub behind_distance: f32,
    pub leader_sight_radius: f32,
    pub slowing_radius: f32,
    pub arrival_tolerance: f32,
    pub tuning: BehaviorTuning,
}

impl LeaderFollowing {
    pub fn new(leader: SteeringTarget) -> Self {
        Self {
            leader,
            behind_distance: 2.5,
            leader_sight_radius: 2.0,
            slowing_radius: 3.0,
            arrival_tolerance: 0.4,
            tuning: BehaviorTuning::new(1.0, 42),
        }
    }

    pub fn with_behind_distance(mut self, behind_distance: f32) -> Self {
        self.behind_distance = behind_distance;
        self
    }

    pub fn with_leader_sight_radius(mut self, leader_sight_radius: f32) -> Self {
        self.leader_sight_radius = leader_sight_radius;
        self
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Formation {
    pub anchor: SteeringTarget,
    pub slot_offset: Vec3,
    pub slowing_radius: f32,
    pub arrival_tolerance: f32,
    pub tuning: BehaviorTuning,
}

impl Formation {
    pub fn new(anchor: SteeringTarget, slot_offset: Vec3) -> Self {
        Self {
            anchor,
            slot_offset,
            slowing_radius: 2.5,
            arrival_tolerance: 0.3,
            tuning: BehaviorTuning::new(1.0, 42),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
pub struct Containment {
    pub center: Vec3,
    pub radius: f32,
    pub margin: f32,
    pub tuning: BehaviorTuning,
}

impl Containment {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self {
            center,
            radius,
            margin: radius * 0.25,
            tuning: BehaviorTuning::new(1.0, 8),
        }
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
}

/// A single contribution from a user-defined custom steering behavior.
#[derive(Reflect, Clone, Debug)]
pub struct CustomContribution {
    pub name: String,
    pub tuning: BehaviorTuning,
    pub intent: LinearIntent,
}

/// Inbox for custom steering contributions.
///
/// Add this component to any entity with [`SteeringAgent`] to inject forces from
/// user-defined behaviors into the composition pipeline. Write contributions in a
/// system placed in [`SteeringSystems::EvaluateCustom`](crate::SteeringSystems::EvaluateCustom),
/// and they will be blended alongside built-in behaviors.
///
/// The inbox is cleared automatically each frame before custom systems run.
///
/// # Example
///
/// ```rust,ignore
/// fn my_behavior(
///     mut agents: Query<(
///         &SteeringAgent,
///         &GlobalTransform,
///         &SteeringKinematics,
///         &mut CustomSteeringBehavior,
///     )>,
/// ) {
///     for (agent, transform, kinematics, mut custom) in &mut agents {
///         let desired_velocity = Vec3::new(1.0, 0.0, 0.0) * agent.max_speed;
///         let intent = desired_velocity_intent(
///             desired_velocity,
///             kinematics.linear_velocity,
///             agent.plane,
///             agent.max_acceleration,
///         );
///         custom.push("MyBehavior", BehaviorTuning::default(), intent);
///     }
/// }
///
/// // Register in EvaluateCustom so it runs before the built-in evaluate pass:
/// app.add_systems(Update, my_behavior.in_set(SteeringSystems::EvaluateCustom));
/// ```
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct CustomSteeringBehavior {
    pub contributions: Vec<CustomContribution>,
}

impl CustomSteeringBehavior {
    /// Push a custom contribution into the inbox.
    pub fn push(&mut self, name: impl Into<String>, tuning: BehaviorTuning, intent: LinearIntent) {
        self.contributions.push(CustomContribution {
            name: name.into(),
            tuning,
            intent,
        });
    }

    /// Remove all contributions (called automatically at the start of each frame).
    pub fn clear(&mut self) {
        self.contributions.clear();
    }
}
