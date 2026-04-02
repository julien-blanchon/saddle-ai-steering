# Steering Lab

Crate-local standalone lab app for validating the shared `steering` crate in a real Bevy scene.

## Purpose

- exercise seek, arrive, pursue, wander, path following, and obstacle avoidance together
- expose stable named entities and a diagnostics resource for BRP and E2E inspection
- provide repeatable screenshot gates for path progress, obstacle avoidance, and general runtime smoke checks

## Status

Working

## Run

```bash
cargo run -p steering_lab
```

## E2E

```bash
cargo run -p steering_lab --features e2e -- smoke_launch
cargo run -p steering_lab --features e2e -- steering_smoke
cargo run -p steering_lab --features e2e -- steering_path_following
cargo run -p steering_lab --features e2e -- steering_avoidance
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch steering_lab
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp world query steering::components::SteeringOutput
uv run --project .codex/skills/bevy-brp/script brp resource get steering_lab::LabDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/steering_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```
