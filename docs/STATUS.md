# Project Status

Last updated: 2026-06-18

## Current Milestone

Traversal diagnostics and visual wind prototype.

The project has a rough Bevy sandbox with a primitive humanoid, deployable glider wings, one-launch-per-airtime vertical burst, camera follow, HUD diagnostics, debug gizmos, authored visual wind/updraft fields, and deterministic tests around the most failure-prone traversal math.

## Last Known Good

- Commit: `f5e8fb3`
- Merged PR: `#3` - Add project docs map
- Verification:
  - `cargo fmt --all --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`

## Active Work

Use this section for milestone handoffs, not routine worktree changes.

- Active milestone PR: `#4` - Add traversal diagnostics and visual wind fields
- Active branch: `abhinav/traversal-diagnostics-wind`
- Open PRs: consult GitHub

## What Works

- Native macOS Bevy app launches on Apple M4 Max through wgpu/Metal.
- Player entity has movement, velocity, flight controller state, animation state, and a primitive child-model hierarchy.
- `E` launches from the ground and is gated to one launch per airtime.
- `Space` deploys glider wings while airborne.
- `Shift` dives.
- Camera uses horizontal follow direction instead of full 3D velocity, avoiding the known vertical launch camera snap.
- HUD reports frame time, camera pitch, camera distance, velocity, altitude, mode, launch state, and visual wind-field count.
- `F1` toggles debug gizmos for player vectors, camera line, and visual wind/updraft stream fields.
- Wind/updraft fields are finite, visible, and visual-only; they do not affect traversal physics.
- Traversal, visual wind-field geometry, camera, diagnostics, and pose math live in testable pure functions in `src/lib.rs`.

## Known Issues

- The character is still primitive geometry, not a rigged character asset.
- Limb posing is approximate and not skeletal. It is less glitchy than elapsed-time phase math, but still placeholder animation.
- Camera has no collision sweep or obstruction avoidance.
- Wind/updraft visuals are debug gizmos only, not yet represented through particles, cloth/glider motion, vegetation, or environment art.
- Gameplay wind/updraft lift is not implemented.
- There is no real terrain, chunk streaming, LOD, water, vegetation, or environment asset pipeline yet.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Build a repeatable manual route that crosses launch, glide, dive, visual wind/updraft, and landing beats.
2. Add camera collision/obstruction handling before larger terrain makes clipping harder to debug.
3. Spike a physics integration choice: Rapier vs Avian, with debug visualization.
4. Define the first real island slice: terrain mesh, spawn point, launch route, glide target, and visual constraints.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
