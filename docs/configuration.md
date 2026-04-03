# Configuration

All units are world units, seconds, and radians.

## `SteeringAgent`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `max_speed` | `f32` | `6.0` | `>= 0.0` | Maximum desired speed emitted by behaviors |
| `max_acceleration` | `f32` | `12.0` | `>= 0.0` | Final steering acceleration clamp |
| `mass` | `f32` | `1.0` | `> 0.0` recommended | Exposed so consumers can convert acceleration intent into force |
| `body_radius` | `f32` | `0.45` | `>= 0.0` | Inflates obstacle avoidance probes |
| `plane` | `SteeringPlane` | `XZ` | `XY`, `XZ`, `Free3d` | Constrains movement math and facing |
| `composition` | `SteeringComposition` | `PrioritizedAccumulation` | enum | Chooses weighted blend vs priority budget accumulation |
| `velocity_source` | `SteeringVelocitySource` | `Kinematics` | enum | Uses `SteeringKinematics` directly or derives velocity from transform delta |
| `braking_acceleration` | `f32` | `10.0` | `>= 0.0` | Applied when no behavior is active and the agent is still moving |
| `alignment` | `SteeringAlignment` | see below | struct | Facing policy for built-in auto-apply |

## `SteeringAlignment`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `mode` | `SteeringFacingMode` | `DesiredVelocity` | enum | Which vector drives facing |
| `turn_speed_radians` | `f32` | `12.0` | `>= 0.0` | Slerp rate used by `SteeringAutoApply` |
| `min_speed` | `f32` | `0.05` | `>= 0.0` | Below this speed, facing falls back to current forward |

## `BehaviorTuning`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | boolean | Turns a behavior on or off without removing the component |
| `weight` | `f32` | `1.0` | `>= 0.0` | Scales contribution strength |
| `priority` | `u8` | `50` | `0..=255` | Lower values are evaluated first in prioritized accumulation |

## `Seek`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target` | `SteeringTarget` | required | Point or entity target |
| `tuning` | `BehaviorTuning` | `weight=1, priority=40` | Standard behavior tuning |

## `Flee`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target` | `SteeringTarget` | required | Threat source |
| `panic_distance` | `Option<f32>` | `None` | Optional flee activation radius |
| `tuning` | `BehaviorTuning` | `weight=1, priority=15` | Standard behavior tuning |

## `Arrive`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `target` | `SteeringTarget` | required | Goal point/entity |
| `slowing_radius` | `f32` | `3.0` | `>= 0.0` | Radius where speed ramps down |
| `arrival_tolerance` | `f32` | `0.25` | `>= 0.0` | Distance treated as “close enough” |
| `speed_curve_exponent` | `f32` | `1.0` | `> 0.0` | `1.0` is linear; larger values bias toward sharper braking near the end |
| `tuning` | `BehaviorTuning` | `weight=1, priority=45` | Standard behavior tuning |

## `Pursue`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target` | `SteeringTarget` | required | Target to chase |
| `lead_scale` | `f32` | `1.0` | Scales prediction horizon |
| `max_prediction_time` | `f32` | `1.25` | Hard cap on prediction time |
| `tuning` | `BehaviorTuning` | `weight=1, priority=35` | Standard behavior tuning |

## `Evade`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target` | `SteeringTarget` | required | Target to escape |
| `lead_scale` | `f32` | `1.0` | Scales prediction horizon |
| `max_prediction_time` | `f32` | `1.25` | Hard cap on prediction time |
| `panic_distance` | `Option<f32>` | `None` | Optional activation radius |
| `tuning` | `BehaviorTuning` | `weight=1, priority=10` | Standard behavior tuning |

## `Wander`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `radius` | `f32` | `2.0` | `>= 0.0` | Wander-circle radius |
| `distance` | `f32` | `3.0` | `>= 0.0` | Circle offset along the current heading |
| `jitter_radians_per_second` | `f32` | `1.8` | `>= 0.0` | How quickly the target drifts |
| `seed` | `u64` | `1` | any | Seed for deterministic drift |
| `vertical_jitter` | `f32` | `0.35` | `>= 0.0` | Extra vertical freedom in `Free3d` mode |
| `tuning` | `BehaviorTuning` | `weight=1, priority=80` | Standard behavior tuning |

## `ObstacleAvoidance`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `min_lookahead` | `f32` | `1.5` | `>= 0.0` | Probe length at zero speed |
| `max_lookahead` | `f32` | `4.5` | `>= min_lookahead` | Probe length near top speed |
| `probe_radius` | `f32` | `0.2` | `>= 0.0` | Additional probe inflation beyond body radius |
| `braking_weight` | `f32` | `0.55` | `>= 0.0` | How strongly the response brakes into the threat |
| `lateral_weight` | `f32` | `1.0` | `>= 0.0` | How strongly the response pushes sideways |
| `layers` | `SteeringLayerMask` | `ALL` | bitmask | Which obstacles are considered |
| `tuning` | `BehaviorTuning` | `weight=1, priority=0` | Standard behavior tuning |

## `SteeringPath`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `points` | `Vec<Vec3>` | empty | Waypoint list |
| `mode` | `SteeringPathMode` | `Once` | End-of-path behavior |
| `waypoint_tolerance` | `f32` | `0.6` | Distance required to advance the cursor |
| `lookahead_distance` | `f32` | `1.8` | Distance projected forward along the polyline |

## `PathFollowing`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `path` | `SteeringPath` | required | Waypoint path |
| `slowing_radius` | `f32` | `3.0` | Used only near a terminal once-path endpoint |
| `arrival_tolerance` | `f32` | `0.3` | Final settle distance for once-paths |
| `tuning` | `BehaviorTuning` | `weight=1, priority=40` | Standard behavior tuning |

## `Flocking`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `neighbor_distance` | `f32` | `3.0` | `>= 0.0` | Maximum range used to gather neighbor agents |
| `separation_weight` | `f32` | `1.5` | `>= 0.0` | Strength of short-range push away from neighbors |
| `alignment_weight` | `f32` | `1.0` | `>= 0.0` | Strength of heading matching toward average neighbor velocity |
| `cohesion_weight` | `f32` | `0.75` | `>= 0.0` | Strength of pull toward the local group center |
| `tuning` | `BehaviorTuning` | `weight=1, priority=30` | Standard behavior tuning |

## `ReciprocalAvoidance`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `neighbor_distance` | `f32` | `3.0` | `>= 0.0` | Maximum distance used to gather crowd neighbors |
| `time_horizon` | `f32` | `1.0` | `> 0.0` recommended | Prediction window for relative collision checks |
| `comfort_distance` | `f32` | `0.1` | `>= 0.0` | Extra spacing margin beyond body radii |
| `side_bias` | `f32` | `1.0` | `>= 0.0` | Strength of the lateral sidestep relative to the brake response |
| `max_neighbors` | `usize` | `8` | `>= 1` | Upper bound on crowd neighbors sampled per agent |
| `tuning` | `BehaviorTuning` | `weight=1, priority=5` | Standard behavior tuning |

## `SteeringAutoApply`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `apply_translation` | `bool` | `true` | Integrates velocity into `Transform.translation` |
| `apply_facing` | `bool` | `true` | Rotates `Transform.rotation` toward desired facing |

## `SteeringDebugSettings`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | `bool` | `false` | Master debug toggle |
| `draw_velocity` | `bool` | `true` | Draw current velocity arrows |
| `draw_output` | `bool` | `true` | Draw desired velocity and acceleration arrows |
| `draw_targets` | `bool` | `true` | Draw seek/arrive/pursue/path targets |
| `draw_arrival_radii` | `bool` | `true` | Draw slowing radii |
| `draw_wander` | `bool` | `true` | Draw wander circle and target |
| `draw_obstacles` | `bool` | `true` | Draw obstacle shapes |
| `draw_probes` | `bool` | `true` | Draw avoidance probe, hit point, and normal |
| `draw_paths` | `bool` | `true` | Draw path waypoints and segments |
| `draw_labels` | `bool` | `false` | Reserved for consumer-side overlays; gizmos do not render text |

## Parameter Interactions

- `ObstacleAvoidance::probe_radius` combines with `SteeringAgent::body_radius`.
- `ObstacleAvoidance` works best with `SteeringComposition::PrioritizedAccumulation`; weighted blend is useful only when you want softer, less dominant avoidance.
- `Flocking` and `ReciprocalAvoidance` both read the current steering-agent snapshot. Their neighborhood cost grows quickly with crowd size, so keep `neighbor_distance` conservative and disable the behavior on agents that do not need it.
- `PathFollowing::arrival_tolerance` should usually stay less than or equal to `SteeringPath::waypoint_tolerance`.
- Large `Wander::jitter_radians_per_second` with low `max_acceleration` creates visible lag; high jitter with high acceleration creates noisy motion.
- If `velocity_source` is `TransformDelta`, add `SteeringTrackedVelocity` to moving target entities you want pursue/evade to predict.
