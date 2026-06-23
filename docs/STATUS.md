# Project Status

Last updated: 2026-06-23

## Current Milestone

First sky-island traversal slice.

The project has a rough Bevy sandbox with a primitive humanoid, playable ground movement, deployable glider wings, one-launch-per-airtime vertical burst, mouse-look camera follow, HUD diagnostics, debug gizmos, authored visual wind/updraft fields, a separate gameplay updraft lift field, a small floating sky-island route with deterministic terrain props, and scripted evals for ground taxi control, mouse camera control, baseline traversal, updraft lift, and island launch-to-landing.

## Last Known Good

- Commit: `a70ed1a`
- Merged PR: `#5` - Add traversal eval harness
- Verification:
  - `cargo fmt --all --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `NAU_EVAL_NO_SCREENSHOT=1 ./tools/eval.sh baseline_route target/eval/baseline_route-noscreen`
  - `./tools/eval.sh baseline_route target/eval/baseline_route`

## Active Work

Use this section for milestone handoffs, not routine worktree changes.

- Active branch: `abhinav/sky-island-route`
- Open PRs: consult GitHub

## What Works

- Native macOS Bevy app launches on Apple M4 Max through wgpu/Metal.
- Player entity has movement, velocity, flight controller state, animation state, and a primitive child-model hierarchy.
- `WASD` works on the ground before launch; ground movement has separate acceleration, top speed, and friction from airborne/glider motion.
- `E` launches from the ground and is gated to one launch per airtime.
- `Space` deploys glider wings while airborne.
- `Shift` dives.
- The sandbox spawns five floating sky islands with a launch island and landing target.
- Route-surface contact can land the player on an island and applies landing damping once instead of crushing standing WASD movement every frame.
- Runtime movement is camera-relative, with character facing smoothed toward horizontal velocity.
- Mouse camera control has player-centered orbit pitch, separate yaw and pitch sensitivity, pitch clamps, click-to-lock cursor capture, right-mouse temporary look, and `Esc` release.
- Camera uses horizontal follow direction instead of full 3D velocity, avoids tagged obstruction volumes, and stays above the active ground surface.
- HUD reports frame time, camera pitch, camera distance, player framing angle, obstruction adjustment, mouse yaw/pitch offsets, velocity, altitude, mode, launch state, target distance, visual wind-field count, active lift-field count, and sky-island count.
- `F1` toggles debug gizmos for player vectors, camera line, visual wind/updraft stream fields, and gameplay lift fields.
- Visual wind/updraft fields are finite, visible, and visual-only; gameplay lift uses a separate bounded `LiftField`.
- Traversal, route-surface geometry, visual wind-field geometry, gameplay lift math, camera, diagnostics, eval metrics, and pose math live in testable pure functions in `src/lib.rs`.
- `ground_taxi_control` eval proves pre-launch camera-relative WASD moves the player across the launch island without leaving grounded mode.
- `camera_mouse_control` eval proves scripted mouse X/Y deltas exercise yaw and both pitch directions without hiding camera regressions behind player movement.
- `updraft_route` eval proves a scripted route enters a gameplay lift field and gains altitude beyond the normal route ceiling.
- `island_launch_to_landing` eval proves the scripted route reaches and lands on the target island.
- Eval summaries now include camera surface clearance, camera-to-player framing angle, obstruction adjustment/hits, camera yaw/pitch offsets, checkpoint screenshot paths, and max scene entity count so camera/control/content regressions are visible in metrics.

## Known Issues

- The character is still primitive geometry, not a rigged character asset.
- Limb posing is approximate and not skeletal. It is less glitchy than elapsed-time phase math, but still placeholder animation.
- Camera obstruction avoidance uses simple tagged AABBs, not a full physics sweep.
- Wind/updraft visuals are debug gizmos only, not yet represented through particles, cloth/glider motion, vegetation, or environment art.
- Sky islands are still primitive slab geometry, not authored terrain meshes.
- Gameplay lift is a first rough updraft only; there is no crosswind force, launch-source chain, or recovery route design yet.
- There is no chunk streaming, LOD, authored water, authored vegetation, or environment asset pipeline yet. Current ponds, trees, stones, beacon, and landing markers are deterministic primitive props.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Replace primitive island slabs with authored or generated terrain meshes.
2. Add visual checks for fixed camera checkpoint screenshots.
3. Tune gameplay updraft placement, readability, and route recovery.
4. Add chunk/terrain counters before larger route streaming work.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
