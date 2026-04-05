# Behaviors

## `Seek`

Moves directly toward a point or entity at full desired speed.

- Needs:
  `SteeringTarget`
- Composes well with:
  `ObstacleAvoidance`, `PathFollowing`
- Failure modes:
  overshoot when used alone on a final goal
- Typical fix:
  pair with `Arrive` near the endpoint or switch to `PathFollowing` for authored routes

## `Flee`

Moves directly away from a threat.

- Needs:
  `SteeringTarget`
- Optional:
  `panic_distance`
- Failure modes:
  agents can run indefinitely or oscillate if the threat is close and another behavior keeps pulling them back
- Typical fix:
  use a higher flee priority or cap activation with `panic_distance`

## `Arrive`

Like seek, but slows inside a configurable radius and settles inside a tolerance.

- Needs:
  `SteeringTarget`
- Key tuning:
  `slowing_radius`, `arrival_tolerance`, `speed_curve_exponent`
- Failure modes:
  too-small slowing radius causes overshoot
  too-large arrival tolerance causes visibly early stopping

## `Pursue`

Predicts a moving target and then seeks that predicted position.

- Needs:
  target position
  target velocity
- Works when target velocity is missing:
  yes, but prediction collapses to a simple seek
- Failure modes:
  prediction becomes too aggressive for far-away fast targets
- Typical fix:
  lower `lead_scale` or clamp `max_prediction_time`

## `Evade`

Predicts a moving threat and flees from the predicted position.

- Needs:
  target position
  target velocity
- Typical use:
  escape drones, dodging animals, defensive civilians
- Failure modes:
  can fight high-priority path or seek behaviors if priorities are poorly tuned

## `Wander`

Uses a deterministic seeded drift to move a wander-circle target over time.

- Needs:
  `Wander`
  `WanderState` is auto-created by the plugin
- Determinism:
  seeded xorshift state per entity
- Failure modes:
  high jitter looks noisy
  very low acceleration makes the wander target drift faster than the body can follow

## `ObstacleAvoidance`

Evaluates the local probe corridor against explicit `SteeringObstacle` entities.

- Needs:
  `ObstacleAvoidance`
  nearby `SteeringObstacle` entities
- Strengths:
  easy to debug
  no physics dependency
  good fit for authored static obstacles
- Limits:
  not reciprocal avoidance
  does not solve dense crowd behavior
  scales linearly with obstacle count

## `PathFollowing`

Follows a waypoint path using waypoint tolerance and lookahead.

- Needs:
  `SteeringPath`
  `PathFollowingState` is auto-created by the plugin
- Supports:
  `Once`
  `Loop`
  `PingPong`
- Internals:
  seek-style motion on regular segments
  arrive-style settle near a terminal once-path endpoint
- Failure modes:
  too-small lookahead can pin agents to corners
  too-large lookahead can skip intended tight authored turns

## `Flocking`

Boids-style local group motion: separation, alignment, and cohesion around nearby agents.

- Needs:
  `Flocking` component, other `SteeringAgent` entities nearby
- Key tuning:
  `separation_weight`, `alignment_weight`, `cohesion_weight`, `max_neighbors`
- Failure modes:
  too-strong separation can fragment a group
  too-strong cohesion can collapse agents into a single point
- Typical use:
  bird flocks, fish schools, drone swarms, ambient crowd life

## `ReciprocalAvoidance`

Lightweight agent-agent collision avoidance that deflects velocity based on predicted time-to-collision.

- Needs:
  `ReciprocalAvoidance` component, other `SteeringAgent` entities nearby
- Key tuning:
  `time_horizon`, `comfort_distance`, `side_bias`
- Limits:
  not a full ORCA solver; suitable for medium-density crowds
- Composes well with:
  `PathFollowing`, `Flocking`

## `LeaderFollowing`

Follow behind a leader entity. Arrives at a point `behind_distance` behind the leader's velocity direction. If the agent is ahead of the leader within `leader_sight_radius`, it evades sideways to clear the leader's path.

- Needs:
  `SteeringTarget` for the leader
- Key tuning:
  `behind_distance`, `leader_sight_radius`
- Composes well with:
  `ReciprocalAvoidance` (to avoid other followers), `ObstacleAvoidance`
- Failure modes:
  leader with no velocity (stationary) defaults to following behind along the Z axis

## `Formation`

Hold a slot position relative to an anchor entity. The `slot_offset` is defined in anchor-local space and rotated by the anchor's velocity direction each frame. Uses arrive behavior internally.

- Needs:
  `SteeringTarget` for the anchor, `slot_offset: Vec3`
- Typical use:
  military units, escort missions, NPC groups
- Composes well with:
  `ReciprocalAvoidance`, `ObstacleAvoidance`
- Failure modes:
  anchor with no velocity causes the slot orientation to default to the Z axis

## `Containment`

Keep agents within a bounding sphere. The behavior activates when the agent enters the margin zone and steers back toward the center with force proportional to proximity to the boundary.

- Needs:
  `center: Vec3`, `radius: f32`
- Key tuning:
  `margin` controls how early the behavior activates
- Typical use:
  arena boundaries, world edges, prevent agents from wandering off
- Composes well with:
  any behavior that might drive agents out of bounds (`Wander`, `Flee`, `Flocking`)

## Composition Guidance

Recommended defaults:

- Seek target with static obstacles:
  `Seek` + `ObstacleAvoidance`
- Smooth stop at a goal:
  `Arrive`
- Moving target chase:
  `Pursue` + `ObstacleAvoidance`
- Patrol:
  `PathFollowing`
- Ambient life:
  `Wander`
- Predator-prey:
  `Pursue` + `ObstacleAvoidance` + `Containment` for predators, `Evade` + `Wander` + `ObstacleAvoidance` + `Containment` for prey
- Formation movement:
  `Formation` + `ReciprocalAvoidance` + `ObstacleAvoidance` for followers, `Wander` + `ObstacleAvoidance` + `Containment` for the leader
- Crowd navigation:
  `PathFollowing` + `ReciprocalAvoidance` + `ObstacleAvoidance` + `Containment`

When multiple behaviors are active:

- use `PrioritizedAccumulation` if avoidance or evade must dominate
- use `WeightedBlend` only when softer compromises are acceptable
