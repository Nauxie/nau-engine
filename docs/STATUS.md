# Project Status

Last updated: 2026-06-24

## Current Milestone

First sky-island traversal slice.

The project has a Bevy sandbox with a primitive humanoid, playable ground movement, deployable glider wings, one-launch-per-airtime vertical burst, mouse-look camera follow, HUD diagnostics, debug gizmos, Bevy-native atmosphere/fog/bloom lighting, procedural materials, drifting cloud banks, authored visual wind/updraft fields, separate gameplay updraft lift fields, a 12-island floating route with generated terrain relief plus deterministic props, and scripted evals for ground taxi control, mouse camera control, yaw/strafe/turn camera stability, baseline traversal, updraft lift, long-glide visibility, and island launch-to-landing.

## Last Known Good

- Commit: `97e9aca`
- Merged PR: `#17` - Add visual feel atmosphere and materials
- Verification:
  - `cargo fmt --all --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - metric-only eval suite across baseline, ground taxi, camera control/stability, updraft, long-glide, and landing scenarios

## Active Work

Use this section for milestone handoffs, not routine worktree changes.

- Active branch: `abhinav/visual-screenshot-audit`
- Open PRs: consult GitHub

## What Works

- Native macOS Bevy app launches on Apple M4 Max through wgpu/Metal.
- Player entity has movement, velocity, flight controller state, animation state, and a primitive child-model hierarchy.
- `WASD` works on the ground before launch; ground movement has separate acceleration, top speed, and friction from airborne/glider motion.
- `E` launches from the ground and is gated to one launch per airtime.
- `Space` deploys glider wings while airborne.
- `Shift` dives.
- The sandbox spawns a 12-island floating archipelago with generated visual terrain relief, a launch island, long-glide route, and landing target.
- The camera uses Bevy-native atmosphere, distance fog, volumetric fog/light, bloom, Aces tonemapping, and atmosphere-driven environment lighting.
- Terrain, props, water, suit, glider, and markers use generated surface textures with tuned roughness/reflectance; marker and flower materials feed bloom through emissive color.
- Drifting cloud banks provide the first non-debug weather layer without changing gameplay collision or traversal math.
- Route-surface contact can land the player on an island and applies landing damping once instead of crushing standing WASD movement every frame.
- Runtime movement is camera-relative, with character facing smoothed toward horizontal velocity.
- Mouse camera control has player-centered orbit pitch, separate yaw and pitch sensitivity, pitch clamps, click-to-lock cursor capture, right-mouse temporary look, and `Esc` release.
- Camera keeps smoothed horizontal follow direction independent from mouse orbit, avoids tagged obstruction volumes, and stays above the active ground surface.
- HUD reports frame time, camera pitch, camera distance, player framing angle, camera motion, camera orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, altitude, mode, launch state, target distance, visual wind-field count, active lift-field count, and sky-island count.
- `F1` toggles debug gizmos for player vectors, camera line, visual wind/updraft stream fields, and gameplay lift fields.
- Visual wind/updraft fields are finite, visible, and visual-only; gameplay lift uses a separate bounded `LiftField`.
- Traversal, route-surface geometry, visual wind-field geometry, gameplay lift math, camera, diagnostics, eval metrics, and richer pose math live in testable pure functions in `src/lib.rs`.
- `ground_taxi_control` eval proves pre-launch camera-relative WASD moves the player across the launch island without leaving grounded mode.
- `camera_mouse_control` eval proves scripted mouse X/Y deltas exercise yaw and both pitch directions without hiding camera regressions behind player movement.
- `camera_yaw_stability` eval proves a small yaw impulse does not keep rotating after mouse input stops.
- `camera_strafe_stability` eval proves `A`/`D` movement does not auto-orbit the camera.
- `camera_turn_stability` eval proves rapid airborne turns and backward air-braking stay within camera step/rotation thresholds.
- `updraft_route` eval proves a scripted route enters a gameplay lift field and gains altitude beyond the normal route ceiling.
- `long_glide_visibility` eval proves sustained traversal across the larger archipelago while preserving content-scale and LOD signals.
- `island_launch_to_landing` eval proves the scripted route reaches and lands on the target island.
- Metric-only evals hide the native window by default; screenshot evals are explicit via `NAU_EVAL_SCREENSHOT=1`.
- Screenshot evals run a non-golden visual audit for resolution, exposure, contrast, color variety, edge density, sky/scene balance, center-scene detail, and HUD-text dominance when launched through `tools/eval.sh`.
- Eval summaries now include camera surface clearance, camera-to-player framing angle, camera step/rotation deltas, camera orbit alignment, obstruction adjustment/hits, camera yaw/pitch offsets, checkpoint screenshot paths, max scene entity count, weather cloud count, hidden/resident island visual counts, and stream visibility churn so camera/control/content/streaming regressions are visible in metrics.

## Known Issues

- The character is still primitive geometry, not a rigged character asset.
- Limb posing now has grounded stride, airborne banking, and glide posture, but it is still approximate non-skeletal animation.
- Camera obstruction avoidance uses simple tagged AABBs, not a full physics sweep.
- Wind/updraft stream fields are still debug gizmos; weather exists as fog/cloud banks, but there are no particles, cloth/glider reactions, vegetation sway, or authored environment art yet.
- Sky-island collision follows deterministic terrain relief, but it is still a route-surface clamp rather than full rigid-body physics.
- Gameplay lift is a first rough updraft only; there is no crosswind force, launch-source chain, or recovery route design yet.
- There is no real chunk despawn, authored water, authored vegetation, or environment asset pipeline yet. Current stream-window terrain visibility, detail LOD, procedural materials, ponds, trees, stones, beacon, cloud banks, and landing markers are deterministic primitive systems.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Extend screenshot audits toward explicit player visibility, severe clipping, and route-marker readability checks.
2. Tune gameplay updraft placement, readability, and route recovery.
3. Promote stream-window counters into actual terrain despawn and asset streaming.
4. Replace the primitive character/environment with a glTF asset pipeline once the traversal/render targets stop moving.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
