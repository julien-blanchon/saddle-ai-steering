# Debugging

## Debug Config

Enable the crate’s built-in gizmos with:

```rust,no_run
app.insert_resource(steering::SteeringDebugSettings {
    enabled: true,
    ..default()
});
```

Per-agent opt-out:

```rust,no_run
commands.entity(agent).insert(steering::SteeringDebugAgent { enabled: false });
```

## Gizmo Meanings

- green arrow:
  current velocity
- aqua arrow:
  desired velocity
- red arrow:
  steering acceleration
- gold cross/line:
  primary behavior target, with arrive radii centered on the actual target
- hot-pink lines and crosses:
  path segments and lookahead target
- white circle/marker:
  configured wander circle and wander target
- yellow line:
  obstacle probe
- red cross and arrow on the probe:
  hit point and chosen avoidance direction
- white obstacle volume:
  explicit steering obstacle shape

## Common Failure Modes

### Agent does not move

Check:

- `SteeringOutput.linear_acceleration` is non-zero
- at least one behavior component is enabled
- the target entity exists and has a valid transform
- the agent either has `SteeringAutoApply` or a consumer system reading `SteeringOutput`

### Agent oscillates near the goal

Check:

- `Arrive::slowing_radius` is not too small
- `Arrive::arrival_tolerance` is not effectively zero
- `SteeringAgent::braking_acceleration` is high enough to bleed off momentum

### Avoidance overpowers seek

Check:

- `ObstacleAvoidance::tuning.priority`
- `ObstacleAvoidance::lateral_weight`
- `ObstacleAvoidance::braking_weight`
- whether `WeightedBlend` would be a better fit than `PrioritizedAccumulation`

### Wander looks jittery

Check:

- `Wander::jitter_radians_per_second`
- `SteeringAgent::max_acceleration`
- fixed-step or deterministic timestep settings in your app

## BRP Workflow

The crate-local lab is the fastest way to inspect live state:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch steering_lab
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp world query steering::components::SteeringOutput
uv run --project .codex/skills/bevy-brp/script brp world query steering::components::SteeringDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get steering_lab::LabDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/steering_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

If type paths drift, use:

```bash
uv run --project .codex/skills/bevy-brp/script brp world list
uv run --project .codex/skills/bevy-brp/script brp resource list
```

## E2E Scenarios

The crate-local lab ships with:

- `smoke_launch`
- `steering_smoke`
- `steering_path_following`
- `steering_avoidance`

Run them with:

```bash
cargo run -p steering_lab --features e2e -- steering_smoke
```
