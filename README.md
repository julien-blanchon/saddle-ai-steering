# Saddle AI Steering

Reusable steering behaviors for locomotion intent in Bevy.

The crate focuses on classical steering output, not high-level decision-making and not physics ownership. It lets consumers attach behavior components to entities, resolve points or entity targets, blend or prioritize multiple behaviors, and optionally apply the resulting intent to kinematic velocity, transform motion, and facing.

`steering` is deliberately generic:

- it uses a `Vec3`-first API with `XY`, `XZ`, and free-3D plane handling
- it does not depend on Avian or any other physics/runtime AI crate
- it keeps the math layer separate from ECS orchestration
- it exposes a crate-local lab for BRP and E2E verification instead of relying on project sandboxes

## Quick Start

```toml
[dependencies]
saddle-ai-steering = { git = "https://github.com/julien-blanchon/saddle-ai-steering" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_ai_steering::{
    Seek, SteeringAgent, SteeringAutoApply, SteeringDebugSettings, SteeringPath, SteeringPlane,
    SteeringPlugin, SteeringTarget,
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Active,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<DemoState>()
        .insert_resource(SteeringDebugSettings {
            enabled: true,
            ..default()
        })
        .add_plugins(SteeringPlugin::new(
            OnEnter(DemoState::Active),
            OnExit(DemoState::Active),
            Update,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Seeker"),
        SteeringAgent::new(SteeringPlane::XZ)
            .with_max_speed(5.0)
            .with_max_acceleration(12.0),
        SteeringAutoApply::default(),
        Seek::new(SteeringTarget::point(6.0, 0.0, 4.0)),
        Transform::from_xyz(-6.0, 0.5, -4.0),
        GlobalTransform::default(),
    ));

    let _loop_path = SteeringPath::new([
        Vec3::new(-4.0, 0.5, -4.0),
        Vec3::new(4.0, 0.5, -4.0),
        Vec3::new(4.0, 0.5, 4.0),
        Vec3::new(-4.0, 0.5, 4.0),
    ])
    .looped();
}
```

For apps where steering should stay active for the entire app lifetime, `SteeringPlugin::default()` is the always-on entrypoint.

## Public API

- Plugin: `SteeringPlugin`
- System sets:
  `SteeringSystems::{Gather, EvaluateCustom, Evaluate, Apply, Debug}`
- Core agent components:
  `SteeringAgent`, `SteeringKinematics`, `SteeringOutput`, `SteeringDiagnostics`
- Optional application helpers:
  `SteeringAutoApply`, `SteeringTrackedVelocity`
- Core vocabulary:
  `SteeringTarget`, `SteeringPath`, `SteeringObstacle`, `SteeringObstacleShape`,
  `SteeringPlane`, `SteeringComposition`, `SteeringLayerMask`
- Behavior components:
  `Seek`, `Flee`, `Arrive`, `Pursue`, `Evade`, `Wander`, `ObstacleAvoidance`, `PathFollowing`, `Flocking`, `ReciprocalAvoidance`, `LeaderFollowing`, `Formation`, `Containment`
- Custom behavior support:
  `CustomSteeringBehavior`, `CustomContribution`, `LinearIntent`, `desired_velocity_intent`, `clamp_magnitude`, `predict_target_position`
- Behavior runtime state:
  `WanderState`, `PathFollowingState`
- Debug resources:
  `SteeringDebugSettings`, `SteeringStats`, `SteeringDebugGizmos`

## Behavior Summary

| Behavior | Purpose | Key Inputs | Notes |
| --- | --- | --- | --- |
| `Seek` | Move directly toward a point or entity | `target` | Full-speed direction chase |
| `Flee` | Move away from a threat | `target`, optional `panic_distance` | Ignores far-away threats when configured |
| `Arrive` | Slow down and settle at a goal | `target`, `slowing_radius`, `arrival_tolerance` | Uses a configurable speed curve exponent |
| `Pursue` | Chase a moving target by prediction | `target`, `lead_scale`, `max_prediction_time` | Falls back cleanly when target velocity is missing |
| `Evade` | Escape a moving target by prediction | `target`, `lead_scale`, `max_prediction_time` | Optional panic distance |
| `Wander` | Deterministic seeded drift | `seed`, `radius`, `distance`, `jitter_radians_per_second` | Uses a wander-circle style target with seeded drift |
| `ObstacleAvoidance` | Local static-obstacle avoidance | `layers`, `min_lookahead`, `max_lookahead`, `probe_radius` | Probe-based, priority-friendly, no physics dependency |
| `PathFollowing` | Follow waypoint paths with lookahead | `SteeringPath`, `slowing_radius`, `arrival_tolerance` | Supports once, loop, and ping-pong path modes |
| `Flocking` | Separation, alignment, and cohesion around nearby agents | `neighbor_distance`, `separation_weight`, `alignment_weight`, `cohesion_weight` | Classical boids-style local group motion that reads nearby steering agents |
| `ReciprocalAvoidance` | Agent-agent local deflection | `neighbor_distance`, `time_horizon`, `comfort_distance`, `side_bias` | Lightweight reciprocal crowd avoidance without requiring a physics or navmesh dependency |
| `LeaderFollowing` | Follow behind a leader entity | `leader`, `behind_distance`, `leader_sight_radius` | Arrives at a point behind the leader; evades sideways if ahead of the leader's forward cone |
| `Formation` | Hold a slot relative to an anchor | `anchor`, `slot_offset` | Anchor-local offset is rotated by anchor's velocity direction; uses arrive for positioning |
| `Containment` | Stay within a bounding region | `center`, `radius`, `margin` | Steers back toward center when approaching the boundary, with force scaling by proximity |

## Custom Behaviors

You can define your own steering behaviors outside the crate. Custom behaviors participate fully in the composition pipeline (weighted blend or prioritized accumulation) alongside built-in behaviors, and appear in `SteeringDiagnostics`.

### How it works

1. **Define a component** for your behavior's configuration (this lives in your code).
2. **Write an evaluate system** that reads agent state, computes a `LinearIntent`, and pushes it into the `CustomSteeringBehavior` inbox.
3. **Register the system** in `SteeringSystems::EvaluateCustom`.

```rust,ignore
use bevy::prelude::*;
use saddle_ai_steering::{
    BehaviorTuning, CustomSteeringBehavior, SteeringAgent, SteeringAutoApply,
    SteeringKinematics, SteeringPlane, SteeringSystems, desired_velocity_intent,
};

// 1. Your behavior component
#[derive(Component)]
struct OrbitBehavior {
    center: Vec3,
    radius: f32,
    speed: f32,
    tuning: BehaviorTuning,
}

// 2. Your evaluate system
fn evaluate_orbit(
    mut agents: Query<(
        &SteeringAgent,
        &GlobalTransform,
        &SteeringKinematics,
        &OrbitBehavior,
        &mut CustomSteeringBehavior,
    )>,
) {
    for (agent, transform, kinematics, orbit, mut custom) in &mut agents {
        let position = agent.plane.project_vector(transform.translation());
        let to_center = orbit.center - position;
        let distance = to_center.length();
        if distance < 0.01 { continue; }

        let radial = to_center / distance;
        let tangent = Vec3::new(-radial.z, 0.0, radial.x);
        let correction = radial * (distance - orbit.radius) * 2.0;
        let desired_vel = (tangent * orbit.speed + correction).normalize_or_zero() * orbit.speed;

        let intent = desired_velocity_intent(
            desired_vel, kinematics.linear_velocity,
            agent.plane, agent.max_acceleration,
        );
        custom.push("Orbit", orbit.tuning, intent);
    }
}

// 3. Register
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(saddle_ai_steering::SteeringPlugin::default())
        .add_systems(Update, evaluate_orbit.in_set(SteeringSystems::EvaluateCustom))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                SteeringAgent::new(SteeringPlane::XZ).with_max_speed(4.0),
                SteeringAutoApply::default(),
                CustomSteeringBehavior::default(),
                OrbitBehavior { center: Vec3::ZERO, radius: 5.0, speed: 3.5,
                    tuning: BehaviorTuning::new(1.0, 40) },
                Transform::from_xyz(5.0, 0.5, 0.0),
            ));
        })
        .run();
}
```

### Available helpers

| Function | Purpose |
| --- | --- |
| `desired_velocity_intent(desired_vel, current_vel, plane, max_accel)` | Convert a desired velocity into a `LinearIntent` — the most common pattern |
| `clamp_magnitude(vec, max_length)` | Clamp a vector's magnitude |
| `predict_target_position(...)` | Predict where a moving target will be — useful for pursuit-like custom behaviors |

Custom behaviors compose with built-in ones. You can add `Seek`, `ObstacleAvoidance`, and your custom `Orbit` on the same entity and they will all participate in the agent's `SteeringComposition`.

## Steering vs Navmesh

This crate and `saddle-ai-navmesh` are **complementary**, not overlapping:

- **Navmesh**: global pathfinding — surface baking, corridor queries, waypoint routes
- **Steering**: local locomotion intent — seek, flee, flocking, obstacle avoidance, formation

The typical integration pipeline: navmesh computes a path, steering follows it with `PathFollowing` while handling local avoidance and flocking. See the `steering_integration` example in `saddle-ai-navmesh` for a working demo.

## Design Decisions

- **Output representation**: `SteeringOutput` carries both `linear_acceleration` and `desired_velocity`.
- **Default arbitration**: `SteeringAgent::default()` uses `SteeringComposition::PrioritizedAccumulation` so obstacle avoidance can dominate without special-case branching. Weighted blending is still available.
- **Obstacle avoidance model**: local probe sampling against explicit `SteeringObstacle` entities. It generates its own contribution after the non-avoid behaviors are preview-composed, so the probe heading can follow seek, arrive, pursue, or path-follow output.
- **Crowd behaviors**: flocking and reciprocal avoidance stay component-based like the rest of the crate. They read nearby `SteeringAgent` snapshots and produce ordinary steering contributions, so they compose with path following and obstacle avoidance instead of becoming a separate movement mode.
- **2D and 3D API**: the public API stays `Vec3`-first. `SteeringPlane::{XY, XZ, Free3d}` constrains movement and orientation without duplicating the whole surface area for `Vec2`.
- **Wander determinism**: each `Wander` behavior seeds a per-entity `WanderState`, then advances it with an internal xorshift sequence. Fixed-step or E2E runs remain reproducible.
- **Arrival and corner jitter control**: `Arrive` exposes `slowing_radius`, `arrival_tolerance`, and `speed_curve_exponent`. `PathFollowing` adds waypoint tolerance plus lookahead so agents do not pinball around corners.

## Configuration Summary

`SteeringAgent` is the main tuning surface:

- `max_speed`: top desired speed in units per second
- `max_acceleration`: steering acceleration clamp
- `mass`: exposed so force-based consumers can derive `force = acceleration * mass`
- `body_radius`: radius used by built-in obstacle avoidance inflation
- `plane`: `XY`, `XZ`, or `Free3d`
- `composition`: weighted blend or prioritized accumulation
- `velocity_source`: use `SteeringKinematics` directly or derive velocity from transform delta
- `braking_acceleration`: deceleration used when no behavior is active
- `alignment`: optional facing behavior

Every behavior carries `BehaviorTuning { enabled, weight, priority }`.

Full parameter reference:

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Behaviors](docs/behaviors.md)
- [Debugging](docs/debugging.md)

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | One agent seeking a fixed target | `cargo run --manifest-path examples/Cargo.toml -p steering_example_basic` |
| `arrive` | Slowing-radius settle at a target | `cargo run --manifest-path examples/Cargo.toml -p steering_example_arrive` |
| `wander` | Deterministic wandering agent | `cargo run --manifest-path examples/Cargo.toml -p steering_example_wander` |
| `obstacle_avoidance` | Seek through static obstacles with probe debug | `cargo run --manifest-path examples/Cargo.toml -p steering_example_obstacle_avoidance` |
| `path_following` | Looping waypoint follow with lookahead | `cargo run --manifest-path examples/Cargo.toml -p steering_example_path_following` |
| `blended` | Pursue + avoid + path follow in one scene | `cargo run --manifest-path examples/Cargo.toml -p steering_example_blended` |
| `flocking` | Boids-style local crowd motion with reciprocal avoidance | `cargo run --manifest-path examples/Cargo.toml -p steering_example_flocking` |
| `kinematic_2d` | Top-down `XY` usage with sprites | `cargo run --manifest-path examples/Cargo.toml -p steering_example_kinematic_2d` |
| `pursuit_evasion` | Predator-prey chase with obstacle avoidance and containment | `cargo run --manifest-path examples/Cargo.toml -p steering_example_pursuit_evasion` |
| `formation` | Agents following a leader in wedge/line/circle formations (press F to cycle) | `cargo run --manifest-path examples/Cargo.toml -p steering_example_formation` |
| `crowd_simulation` | 32 agents navigating a space with reciprocal avoidance and obstacles | `cargo run --manifest-path examples/Cargo.toml -p steering_example_crowd_simulation` |
| `custom_behavior` | User-defined orbit behavior via `CustomSteeringBehavior` | `cargo run --manifest-path examples/Cargo.toml -p steering_example_custom_behavior` |
| `steering_lab` | Crate-local BRP/E2E showcase | `cargo run -p steering_lab` |

## Crate-Local Lab

`shared/ai/steering/examples/lab` is the verification app for this crate. It exercises all behaviors — including custom behaviors — in one scene with live diagnostics.

```bash
cargo run --manifest-path examples/Cargo.toml -p steering_lab
```

### E2E Scenarios

Each scenario runs deterministic 60fps simulation, asserts behavioral metrics via `LabDiagnostics`, and captures screenshots to `e2e_output/<scenario>/`.

| Scenario | What it verifies | Assertions |
| --- | --- | --- |
| `smoke_launch` | App boots, all agents active | `active_agents >= 9` |
| `steering_smoke` | Core behaviors run together | Pursuit closing, wander moving, path advancing |
| `steering_path_following` | Path follower completes laps | Waypoint progress after 490 frames |
| `steering_avoidance` | Obstacle avoidance clearance | Agent clears obstacle with `clearance > 0.05` |
| `steering_flocking_crowd` | Flocking + reciprocal avoidance | Neighbors detected, conflicts resolved, `separation > 0.2` |
| `steering_custom_behavior` | Custom orbit behavior | Agent moving, maintains orbit speed, converges to target radius |

Run all scenarios:

```bash
cd examples
cargo run -p steering_lab --features e2e -- smoke_launch
cargo run -p steering_lab --features e2e -- steering_smoke
cargo run -p steering_lab --features e2e -- steering_path_following
cargo run -p steering_lab --features e2e -- steering_avoidance
cargo run -p steering_lab --features e2e -- steering_flocking_crowd
cargo run -p steering_lab --features e2e -- steering_custom_behavior
```

Add `--handoff` to keep the window open after the scenario finishes (useful for BRP inspection).

## Limitations

- Reciprocal avoidance is a lightweight local solver, not a full ORCA implementation with linear-program guarantees. It is designed to be easy to debug and good enough for medium-density crowds.
- Built-in crowd and obstacle evaluation are both neighborhood scans over live ECS data. Flocking and reciprocal avoidance are `O(agents^2)` in the worst case; obstacle evaluation is `O(agents * obstacles)`.
- Debug gizmos intentionally reflect the actual arrival targets, configured wander radius, and chosen avoidance direction so BRP and screenshot-based debugging stay trustworthy.
- `Free3d` orientation aligns yaw and pitch only. It does not solve roll or banking.
- AABB avoidance uses analytic segment-vs-box tests. If you need collider-accurate avoidance, feed a higher-quality world adapter into your own movement layer and keep `steering` as the locomotion-intent stage.
