# Project Status

Last updated: 2026-07-14

## Current Milestone

Playable traversal baseline.

NAU is a working Mac-first Bevy sandbox, not a general-purpose engine or a production game. The current priority is to preserve the enjoyable launch/glide/dive/land loop while adding clearer route purpose, stronger authored presentation, and only the infrastructure that those improvements require.

Release play is the default feel and performance reference:

```sh
cargo run --release -- --play
```

## Current Game

### Traversal

- Ground movement, one-launch-per-airtime vertical assist, airborne steering, glider deployment, diving, air braking, landing, recovery, relaunch, and central-island reset are playable.
- Movement is camera-relative. Body yaw, bank, glide response, dive posture, and landing anticipation/recovery are smoothed and covered by app and simulation evals.
- Ordinary gliding trades altitude for distance. Vertical gain comes from launch, authored `LiftField` updrafts, or capped gate boosts.

### World

- The route contains 41 floating islands using 20 terrain archetypes, deterministic relief, irregular playable contours, generated cliff/underside bodies, vegetation, rocks, ponds, landmarks, and distant material-split impostors.
- Island visuals use active chunk windows plus near/mid/far LOD residency. Terrain, cliff, and underside meshes are created from cached recipes as islands become resident; detail preparation is still mostly synchronous.
- A playable biome-colored world floor streams a player-centered `3x3` visible tile window from a pool capped at 25 tiles. The same terrain sampler drives rendering and gameplay grounding, while island surfaces remain authoritative where they overlap.

### Wind And Objectives

- Eighteen authored lift routes pair gameplay `LiftField` volumes with visible updraft flow.
- Twenty crosswind fields plus the 18 updraft visuals produce 38 measured wind fields. Shared gust-cell flow drives visual motion, horizontal airborne response, diagnostics, and eval checks; vertical climb remains explicit lift behavior.
- Twelve one-shot aerial gates provide capped boosts. Three sit on the main glide corridor and nine reward low, thermal, and high-altitude branches.
- The top-right HUD and pause menu share the authoritative collected-gate total. `Esc` pauses virtual time, releases cursor capture, and exposes resume, controls, and quit actions.

### Player And Camera

- The self-authored player and glider glTF fixtures support named idle, walk, run, launch, fall, bank, glide, dive, air-brake, and land states with procedural pose refinement and attachment/readability audits.
- The character is still an approximate non-skeletal prototype, not a production rig.
- Mouse orbit, stable movement-facing direction, obstruction avoidance, boom limits, ground clearance, and reset handoff are implemented. Broad blockers remain solid; tree-scale local props use a softer camera policy to avoid abrupt framing changes.

### Collision And Debugging

- Grounding follows deterministic island and world-floor surfaces.
- Each island has 16 terrain-rim contour proxies and four broad terrain-body cliff proxies. Trees, rocks, route landmarks, authored solid fixtures, and obstruction spires use tagged AABB push-out proxies.
- This is custom movement and collision math, not rigid-body physics, capsule sweeps, or authored collider import.
- Debug mode exposes runtime metrics and player/camera/wind/lift gizmos. `F1` toggles the main gizmos and `F2` toggles collision proxies.

## Accepted Baseline

The streamed world-floor checkpoint is accepted for the current sandbox. It supports landing, grounded traversal, relaunch, bounded tile residency, and measured stream churn. The compact game UI and twelve-gate objective route are part of the current baseline, not active candidates.

World-floor contract and evidence:

- `docs/DECISIONS/0002-world-floor-perf-first.md`
- `docs/world-floor-requirements-audit.md`
- `docs/world-floor-acceptance-report.md`

## Verification Surface

Core repository gates:

```sh
cargo fmt --check
cargo check --all-targets
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Representative behavior and content gates:

- `ground_taxi_control`, `playtest_reset`, and `island_launch_to_landing`
- `air_control_response`, `pose_state_coverage`, and `updraft_route`
- `camera_mouse_control`, `camera_turn_stability`, `camera_strafe_stability`, and `underbridge_under_route`
- `world_collision_contact`, `terrain_rim_collision_contact`, and `terrain_body_collision_contact`
- `baseline_route`, `branch_recovery_route`, `long_glide_visibility`, and `great_sky_plateau_route`
- terrain, visual-content, wind-visual, player-pose, asset-fixture, screenshot, marker-projection, and semantic-scene audits
- release app baselines, scripted/manual play profiles, and world-floor comparison gates

## Known Limits

- The player-facing objective is gate collection; route beats, recovery branches, lift sequencing, and landing targets are richer in authored data and evals than in the game UI.
- The character and environment are self-authored prototype assets backed by deterministic generated content. They are measurable and readable, but not production art.
- Collision is terrain sampling plus AABB proxies. There is no selected Rapier/Avian integration, dynamic rigid-body layer, slope-aware capsule controller, or imported collider pipeline.
- Island/world-floor residency is bounded and observable, but full asynchronous asset streaming, floating-origin support, and per-chunk physics activation are not implemented.
- Atmosphere, volumetric fog/light, bloom, shadows, clouds, wind visuals, and island detail are measurable performance costs and are not optimized for laptop power draw.
- Human release play remains required after changes to movement, camera, content density, rendering, collision, or streaming.

## Active Priorities

1. Turn the existing route data, lift network, recovery branches, landmarks, and gates into a clearer player-facing expedition with meaningful choices and completion feedback.
2. Improve player/glider animation fidelity without losing current pose readability, attachment integrity, or traversal feel.
3. Replace prototype collision pieces only when a concrete gameplay need justifies a physics/query layer.
4. Keep streaming and rendering work evidence-driven; prefer reducing cosmetic cost or residency before adding architecture.
5. Preserve the current movement and camera baseline unless a reproducible regression requires retuning.

## Read First

- `README.md`
- `docs/ROADMAP.md`
- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/EVAL_SPEC.md`

Update this file when the playable baseline, accepted limitations, or active priorities change. Do not use it as a branch log or PR diary.
