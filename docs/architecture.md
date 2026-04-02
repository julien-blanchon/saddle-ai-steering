# Architecture

## Steering Pipeline

`steering` is split into three layers:

1. **Pure math layer**
   `src/math.rs` plus `src/behaviors/*`
   This layer evaluates steering behavior from positions, velocities, paths, and obstacles without touching ECS.
2. **ECS orchestration layer**
   `src/systems.rs`
   This layer resolves entity targets, path state, tracked velocity, and obstacle queries, then writes `SteeringOutput` and `SteeringDiagnostics`.
3. **Application layer**
   `src/systems.rs`
   The built-in `SteeringAutoApply` system optionally integrates velocity, translation, and facing for purely kinematic agents. Consumers using Avian or custom controllers can ignore it and read `SteeringOutput` directly.

## Output Representation

`SteeringOutput` contains both:

- `desired_velocity`
- `linear_acceleration`

This lets the crate fit multiple integration styles:

- kinematic transform integration
- velocity controllers
- force-based physics, using `force = acceleration * mass`
- animation/facing systems that care about heading but not motion ownership

## Arbitration Strategy

Each behavior emits an independent contribution:

- requested acceleration
- desired velocity
- priority
- weight

The agent then composes those contributions with one of two modes:

- `WeightedBlend`
- `PrioritizedAccumulation`

`PrioritizedAccumulation` is the default because it gives obstacle avoidance a clean way to dominate seek, arrive, pursue, or path follow without hidden special cases.

### How seek, arrive, and path following interact

- `Seek` contributes a full-speed desired direction.
- `Arrive` contributes a slowdown-aware desired direction.
- `PathFollowing` first computes a path lookahead point, then behaves like seek along the path and like arrive near the final endpoint.

They are not hard-coded into a behavior tree. They are just contributions. The chosen composition mode decides how much of each survives the final acceleration budget.

## Obstacle Avoidance Approach

Obstacle avoidance uses explicit `SteeringObstacle` components and a local probe:

- heading source:
  preview-composed non-avoid desired velocity
  else current velocity
  else transform forward
- lookahead:
  scaled between `min_lookahead` and `max_lookahead` by current speed
- probe inflation:
  `agent.body_radius + probe_radius`
- obstacle tests:
  sphere-vs-segment or oriented-AABB-vs-segment
- response:
  lateral push along the hit normal plus optional braking

This is classical local steering, not a crowd solver. It is intentionally easy to debug with gizmos and easy to replace if a consumer later wants ORCA/RVO or a physics-backed query stage.

## Movement Planes

The crate stays `Vec3`-first and constrains through `SteeringPlane`:

- `XY`
  top-down 2D or sprite games
- `XZ`
  3D ground locomotion
- `Free3d`
  flying or unconstrained movement

The plane:

- projects behavior math
- preserves the agent’s fixed axis when resolving target points
- constrains automatic translation
- chooses the up axis for debug drawing and orientation

This avoids separate `Vec2` and `Vec3` APIs while still keeping planar behavior explicit.

## Jitter and Oscillation Control

The crate avoids common classical-steering failure modes with:

- `Arrive::arrival_tolerance`
- `Arrive::slowing_radius`
- `Arrive::speed_curve_exponent`
- `SteeringAgent::braking_acceleration`
- `SteeringPath::waypoint_tolerance`
- `SteeringPath::lookahead_distance`
- final-segment arrive behavior in `PathFollowing`

These controls matter more than overly clever force math. Without them, even correct seek/arrive logic tends to oscillate near small targets and path corners.

## Runtime State

Two public runtime components are maintained for stateful behaviors:

- `WanderState`
- `PathFollowingState`

The plugin auto-inserts them when required. They stay public because they are useful for BRP inspection, deterministic tests, and custom lab tooling.

## Performance Characteristics

### Expected complexity

- seek, flee, arrive, pursue, evade, wander:
  `O(agents)`
- path following:
  `O(agents * path_lookahead_segments)` with small path-local scans
- obstacle avoidance:
  `O(agents * obstacles)`

### v0.1 tradeoff

The obstacle stage is intentionally straightforward and debuggable rather than spatially indexed. For many games this is acceptable because only a subset of agents need local avoidance each frame and the obstacle set is moderate.

If a consumer needs larger-scale obstacle counts, the recommended extension is:

1. cull or partition obstacle queries externally
2. keep `steering` as the final intent stage
3. feed only nearby obstacles into the avoidance step

## Extension Points

The design leaves room for future work without breaking the core API:

- separation/cohesion/alignment flocking behaviors
- context-steering or sampled-direction backends
- obstacle broad-phase adapters
- custom target resolvers
- spline-backed path sampling
- custom application systems for physics or animation pipelines
