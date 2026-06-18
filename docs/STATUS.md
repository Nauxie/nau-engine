# Project Status

Last updated: 2026-06-18

## Current Milestone

Glider traversal prototype.

The project has a rough Bevy sandbox with a primitive humanoid, deployable glider wings, one-launch-per-airtime vertical burst, camera follow, HUD readout, and deterministic tests around the most failure-prone traversal math.

## Last Known Good

- Commit: `1d211e2`
- Merged PR: `#2` - Add humanoid glider prototype
- Verification:
  - `cargo fmt --all --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo run` native smoke test opened on Metal without Bevy hierarchy warnings

## Active Work

Use this section for milestone handoffs, not routine worktree changes.

- Active milestone PR: none on `main`
- Active branch: consult `git status --short --branch`
- Open PRs: consult GitHub

## What Works

- Native macOS Bevy app launches on Apple M4 Max through wgpu/Metal.
- Player entity has movement, velocity, flight controller state, animation state, and a primitive child-model hierarchy.
- `E` launches from the ground and is gated to one launch per airtime.
- `Space` deploys glider wings while airborne.
- `Shift` dives.
- Camera uses horizontal follow direction instead of full 3D velocity, avoiding the known vertical launch camera snap.
- Traversal, camera, and pose math live in testable pure functions in `src/lib.rs`.

## Known Issues

- The character is still primitive geometry, not a rigged character asset.
- Limb posing is approximate and not skeletal. It is less glitchy than elapsed-time phase math, but still placeholder animation.
- Camera has no collision sweep, obstruction avoidance, or dedicated pitch metric.
- There is no real terrain, chunk streaming, LOD, wind volume, water, vegetation, or environment asset pipeline yet.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Add runtime debug capture for camera pitch, camera distance, frame time, flight mode, altitude, and speed.
2. Spike a physics integration choice: Rapier vs Avian, with debug visualization.
3. Define the first real island slice: terrain mesh, spawn point, launch route, glide target, and visual constraints.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
