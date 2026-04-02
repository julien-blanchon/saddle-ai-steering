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
steering = { path = "shared/ai/steering" }
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
  `SteeringSystems::{Gather, Evaluate, Apply, Debug}`
- Core agent components:
  `SteeringAgent`, `SteeringKinematics`, `SteeringOutput`, `SteeringDiagnostics`
- Optional application helpers:
  `SteeringAutoApply`, `SteeringTrackedVelocity`
- Core vocabulary:
  `SteeringTarget`, `SteeringPath`, `SteeringObstacle`, `SteeringObstacleShape`,
  `SteeringPlane`, `SteeringComposition`, `SteeringLayerMask`
- Behavior components:
  `Seek`, `Flee`, `Arrive`, `Pursue`, `Evade`, `Wander`, `ObstacleAvoidance`, `PathFollowing`
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

## Design Decisions

- **Output representation**: `SteeringOutput` carries both `linear_acceleration` and `desired_velocity`.
- **Default arbitration**: `SteeringAgent::default()` uses `SteeringComposition::PrioritizedAccumulation` so obstacle avoidance can dominate without special-case branching. Weighted blending is still available.
- **Obstacle avoidance model**: local probe sampling against explicit `SteeringObstacle` entities. It generates its own contribution after the non-avoid behaviors are preview-composed, so the probe heading can follow seek, arrive, pursue, or path-follow output.
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
| `basic` | One agent seeking a fixed target | `cargo run -p steering --example basic` |
| `arrive` | Slowing-radius settle at a target | `cargo run -p steering --example arrive` |
| `wander` | Deterministic wandering agent | `cargo run -p steering --example wander` |
| `obstacle_avoidance` | Seek through static obstacles with probe debug | `cargo run -p steering --example obstacle_avoidance` |
| `path_following` | Looping waypoint follow with lookahead | `cargo run -p steering --example path_following` |
| `blended` | Pursue + avoid + path follow in one scene | `cargo run -p steering --example blended` |
| `kinematic_2d` | Top-down `XY` usage with sprites | `cargo run -p steering --example kinematic_2d` |
| `steering_lab` | Crate-local BRP/E2E showcase | `cargo run -p steering_lab` |

## Crate-Local Lab

`shared/ai/steering/examples/lab` is the verification app for this crate.

```bash
cargo run -p steering_lab
```

E2E commands:

```bash
cargo run -p steering_lab --features e2e -- smoke_launch
cargo run -p steering_lab --features e2e -- steering_smoke
cargo run -p steering_lab --features e2e -- steering_path_following
cargo run -p steering_lab --features e2e -- steering_avoidance
```

## Limitations

- v0.1 obstacle avoidance is classic local steering, not crowd simulation and not reciprocal avoidance. It is designed for agents versus explicit static obstacles, not dense crowds.
- Built-in obstacle evaluation is `O(agents * obstacles)` each update. This is documented and expected in v0.1.
- Debug gizmos intentionally reflect the actual arrival targets, configured wander radius, and chosen avoidance direction so BRP and screenshot-based debugging stay trustworthy.
- `Free3d` orientation aligns yaw and pitch only. It does not solve roll or banking.
- AABB avoidance uses analytic segment-vs-box tests. If you need collider-accurate avoidance, feed a higher-quality world adapter into your own movement layer and keep `steering` as the locomotion-intent stage.
