# Architecture

The NAU Engine is a Mac-first Bevy project. The current goal is a traversal sandbox, not a general-purpose engine.

## Stack

- Language: Rust
- App/game framework: Bevy
- Renderer path: Bevy renderer -> wgpu -> Metal on macOS
- Asset target: glTF for real 3D models and scenes
- Current physics: custom testable traversal math
- Candidate physics layer: Rapier or Avian, not chosen yet

## Current Code Shape

- `src/main.rs` owns Bevy app setup, scene spawning, generated island surface meshes, island stream-window visibility, island detail LOD, input mapping, ECS queries, HUD sampling, and visual wiring.
- `src/lib.rs` owns reusable and testable logic.
- `movement` owns flight state, input state, tuning, launch/glide/dive integration, floor clamp, velocity limits, and facing smoothing.
- `environment` owns finite visual wind/updraft field definitions, gameplay `LiftField` updraft volumes, lift application, and deterministic stream placement.
- `world` owns collision-aware route surfaces, sky-island definitions, deterministic island relief, landing target queries, active chunk counters, stream-window classification, and near/mid/far LOD band classification.
- `camera` owns camera follow math, orbit yaw/pitch control math, horizontal follow direction, obstruction avoidance, and ground-clearance helpers.
- `diagnostics` owns pure helpers for frame-time and runtime metric formatting inputs.
- `animation` owns primitive character part pose math, wing visibility state, and animation phase progression.

## Frame Flow

1. Bevy keyboard input is read in `fly_player`; runtime mouse input is read in `update_camera_control`.
2. Input is mapped into `movement::FlightInput`.
3. Movement uses the camera's horizontal forward/right vectors when available.
4. `movement::step_flight` produces the next position, velocity, and controller state.
5. Gameplay lift fields apply bounded upward acceleration when the player is airborne inside an active `LiftField`.
6. Player orientation is smoothed toward horizontal velocity.
7. Character pose phase advances from delta time.
8. `animation::part_pose` maps flight mode and velocity into visible body/glider poses.
9. Mouse deltas update `CameraControlState` yaw/pitch when the cursor is locked or right mouse is held.
10. `camera::update_follow_direction_state` keeps smoothed travel direction independent from mouse orbit, and `camera::step_camera_with_direction` applies yaw/pitch orbit offsets once before smoothing position and rotation.
11. The camera avoids tagged obstruction volumes and is lifted above the active collision terrain surface when needed.
12. Island terrain is visible inside the active stream window, inactive chunks show cheap distant impostors, route beacons remain visible for readability, and nonessential island detail is hidden outside the near LOD band.
13. HUD text reports frame time, mode, speed, altitude, camera pitch/distance/framing/motion/orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, visual wind-field count, lift-field count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, and detail counts, resident island visual count, stream visibility churn, route beacons, cooldown, and launch readiness.
14. Debug gizmos draw player vectors, the camera line, visual wind/updraft streams, and gameplay lift-field bounds.
15. The default world is a 12-island archipelago with varied primitive terrain colors, route cairns, and near/far gameplay updrafts.

Eval samples include camera distance, camera surface clearance, camera-to-player framing angle, per-frame camera step and rotation deltas, camera orbit alignment, camera view yaw, obstruction adjustment/hits, camera yaw/pitch offsets, `active_lift_fields`, sky-island count, active chunk count, active island count, near/mid/far LOD island counts, visible/hidden island terrain counts, visible/hidden island impostor counts, visible/hidden island detail counts, visible route beacon count, resident island visual count, stream visibility churn, and entity count. Eval summaries include frame-time avg/p95/p99/max telemetry, `lifted_samples`, camera-control/framing/orbit/view-yaw/obstruction/jerk checks, checkpoint screenshot artifact paths, sky-island/content-scale checks, streaming/LOD planning checks, stream-visibility checks, resident visual/churn checks, visible-detail/beacon checks, and scene entity-count checks; `updraft_route` verifies gameplay lift, `long_glide_visibility` verifies the larger archipelago route and distant lift traversal, `camera_mouse_control` verifies mouse X/Y behavior, `camera_yaw_stability` verifies that small yaw input does not drift after input stops, `camera_strafe_stability` verifies that `A`/`D` movement does not auto-orbit the camera, and `camera_turn_stability` verifies rapid airborne turn and air-brake camera stability.

## Core Invariants

- Movement math must stay testable outside a Bevy window.
- `E` launch is ground-gated unless a future launch-source mechanic explicitly changes that.
- Glider traversal descends without wind/updraft/launch-source help.
- Visual `WindField` volumes do not directly move the player.
- Gameplay `LiftField` volumes can move the player upward, but only through explicit lift application rules.
- If crosswind ever affects movement, force application rules belong in reusable/testable code, not directly in ECS systems.
- Camera follow direction should use horizontal travel direction, not full 3D velocity.
- Runtime movement should stay camera-relative unless a scenario deliberately requests character-relative controls.
- Camera orbit input should keep yaw and pitch independently measurable in evals while keeping the player focus near the camera centerline.
- Camera should stay above the active route surface and avoid tagged obstruction volumes between the player focus and camera boom.
- Sky-island collision queries and visible terrain meshes should use the same deterministic relief function, with launch and landing centers anchored to their authored route heights.
- Active chunk counters drive terrain/impostor visibility, and stream diagnostics record hidden/resident visual counts plus visibility churn until a future branch adds despawn or asset streaming.
- Camera, animation, and HUD should run after movement.
- `src/main.rs` should stay mostly wiring. Avoid burying gameplay rules directly in ECS systems.
- Do not add raw Metal code until profiling proves a specific renderer hotspot.
- Do not add a broad engine abstraction until at least two real systems need the same boundary.

## Scale Strategy

The route to Zelda-scale is incremental:

1. One stable island slice.
2. Multiple adjacent streamed chunks.
3. Distant impostors and terrain LOD.
4. Floating-origin or origin-rebase strategy.
5. World composition tools and asset rules.

Large-world work should not begin by making a huge map. It should begin by making a small route that exercises streaming, visibility, and traversal distance.

## Asset Strategy

Short term:

- Keep primitives for fast iteration.
- Add debug visuals before polish.
- Use glTF for real character and environment imports.

Medium term:

- Introduce a rigged humanoid character.
- Define animation states for grounded, launch, glide, dive, turn/bank, land, and recover.
- Define island environment asset conventions.

Long term:

- Add LOD variants.
- Add authoring rules for collision, nav/traversal volumes, gameplay lift zones, wind visuals, and visual-only geometry.

## Physics Strategy

The project should choose physics deliberately.

Questions for the physics spike:

- Does the character controller need rigid body dynamics or custom kinematic movement?
- How cleanly can we query terrain beneath/around a fast gliding player?
- How good is debug visualization?
- How hard is Bevy integration?
- How costly is simulation if the world is streamed?

Until that choice is made, keep traversal math pure and tested.
