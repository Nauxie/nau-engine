# Project Status

Last updated: 2026-07-16

## Current Milestone

Playable traversal baseline.

NAU is a working Mac-first Bevy sandbox, not a general-purpose engine or a production game. The current priority is to preserve the enjoyable launch/glide/dive/land loop while adding clearer route purpose, stronger authored presentation, and only the infrastructure that those improvements require.

Ordinary development play is configured to keep Bevy and rendering dependencies optimized while preserving debug assertions. Release play remains the final feel and performance reference:

```sh
cargo run -- --play
cargo run --release -- --play
```

## Current Game

### Traversal

- Ground movement, one-launch-per-airtime vertical assist, airborne steering, glider deployment, diving, air braking, landing, recovery, relaunch, and central-island reset are playable.
- Movement is camera-relative. Body yaw, bank, glide response, dive posture, and landing anticipation/recovery are smoothed and covered by app and simulation evals.
- Ordinary gliding trades altitude for distance. Vertical gain comes from launch, authored `LiftField` updrafts, or capped gate boosts.

### World

- The route contains 41 floating islands spread across 1.68 km of X, 1.02 km of Y, and 3.94 km of Z. Six footprint tiers, seven occupied horizontal sectors, rear launch branches, side arcs, and a colossal plateau create a broader archipelago than the original forward corridor.
- Every route island now has an ordered `IslandArtDirection` profile with its own epithet, story, palette tuple, surface pattern, hero landmark, ecology, geology, ruin inventory, and water story. Art-direction signatures, palette tuples, and aggregate visual signatures are unique; individual grammar families intentionally recur across the route. The accepted export contains 2,732 ground-cover patches, 171 species-varied trees, 282 rocks, 102 dense flora clusters across six families, 46 ruin complexes across five families, 69 geological formations across five families, 56 water-detail clusters across six families, six legacy ruin clusters, 101 surface artifacts, nine river channels, ten ponds, plateau lakes and waterfalls, cave-route structures, 578 exported landmark entries, and distant material-split impostors.
- Runtime surface rendering maps those 41 authored profiles onto bounded palette-family `SurfaceMaterial` sets instead of allocating materials per island. Mipped generated albedo/normal/ORM maps feed a shared Bevy `ExtendedMaterial` shader that adds terrain macro/detail variation and animated water/foam response while preserving the existing material-count budget.
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
- Mouse orbit applies the full current-frame input without frame-time scaling or response smoothing, and movement consumes the resulting camera heading in the same update.
- The play window uses display-synchronized presentation, which preserves the built-in ProMotion display's 120 Hz path on the validated Mac. Pausing or releasing mouse look clears capture history before resume so accumulated desktop motion cannot become delayed camera input.
- Follow, obstruction, floor-clearance, collision, streaming, and reset handoffs share bounded frame-rate-independent continuity with attributed full-rate diagnostics. Broad blockers remain solid; tree-scale local props use a softer camera policy to avoid abrupt framing changes.

### Collision And Debugging

- Grounding follows deterministic island and world-floor surfaces.
- Each island has 16 terrain-rim contour proxies and four broad terrain-body cliff proxies. Trees, rocks, route landmarks, authored solid fixtures, and obstruction spires use tagged AABB push-out proxies.
- This is custom movement and collision math, not rigid-body physics, capsule sweeps, or authored collider import.
- Debug mode exposes runtime metrics and player/camera/wind/lift gizmos. `F1` toggles the main gizmos and `F2` toggles collision proxies.

## Accepted Baseline

The streamed world-floor checkpoint is accepted for the current sandbox. It supports landing, grounded traversal, relaunch, bounded tile residency, and measured stream churn. The compact game UI and twelve-gate objective route are part of the current baseline, not active candidates.

Camera/player continuity, ordinary development-play performance, and the individually authored 41-island surface pass are also accepted baselines. The camera gate covers deterministic 30/60/120/144 Hz behavior, 50/100 ms hitches, native mouse response, obstruction transitions, floor boundaries, collisions, streaming, and resets. The development-play gate compares the same full-content scenario in debug and release and fails if debug frame time exceeds its absolute budget or materially trails release. The island contract requires accepted full-profile signatures, unique art and palette signatures, one matching hero landmark per island, exact feature and tree/rock budgets, correct water presence, large-island authored-feature coverage, and pixel-backed near/mid/traversal review for all 123 island views while preserving route clearance and movement/camera behavior. Hero evidence comes from real landmark triangles, island semantic occlusion uses authored silhouettes, and route-edge waterfalls share their placement with stricter story-aware review framing. The visual and semantic audits now separately protect muted terrain, water, foliage, coherent local water components, waterfall foam, terrain shadow detail, and landscape-vista composition so one broad color classifier cannot mask a missing surface family.

The July 2026 severe camera-feel regression was render pacing rather than camera interpolation: per-island surface palette allocation increased live materials from 100 to 244 and raised the measured full-content average frame time from 9.77 ms to 15.63 ms. Palette-family reuse restored the full 1.23-million-triangle scene to about 8.6 ms and 116 materials. A follow-up `AutoNoVsync` experiment was rejected because it degraded that same fixed scene to 14-16 ms on Metal/ProMotion; display-synchronized presentation restored the 120 Hz path. The current surface-shader pass measures 8.37/9.29/11.19/19.19 ms debug and 8.36/9.03/9.63/18.66 ms release average/p95/p99/max across 171 steady runtime frames, with 115 total materials. Initial deferred stream reconciliation around frame 9 can still produce a large startup-only frame, so the eval now records those first 30 frames in all-frame evidence without misclassifying them as steady play. Local performance acceptance includes warmup, quiet-host preflight, runtime average/p95/max/slow-frame ceilings, full-frame diagnostics, and a material-count budget; pull requests also compare the candidate with its base revision on the same runner.

World-floor contract and evidence:

- `docs/DECISIONS/0002-world-floor-perf-first.md`
- `docs/world-floor-requirements-audit.md`
- `docs/world-floor-acceptance-report.md`

Island-surface contract and evidence:

- `docs/ISLAND_ART_DIRECTION.md`
- `docs/ISLAND_SURFACE_CONTENT.md`
- `tools/island_art_direction_gate.sh`
- `tools/island_hero_gallery_gate.sh`
- `island_surface_review`
- `tools/world_content_gate.sh`

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
- `baseline_route`, `branch_recovery_route`, `long_glide_visibility`, `great_sky_plateau_route`, `great_sky_plateau_vistas`, and `island_surface_review`
- terrain, visual-content, island-art-direction, 123-view hero-gallery, wind-visual, player-pose, asset-fixture, screenshot, marker-projection, and semantic-scene audits
- `tools/camera_continuity_gate.sh` and `tools/dev_play_performance_gate.sh` on macOS
- release app baselines, scripted/manual play profiles, and world-floor comparison gates

## Known Limits

- The player-facing objective is gate collection; route beats, recovery branches, lift sequencing, and landing targets are richer in authored data and evals than in the game UI.
- The character and environment are self-authored prototype assets backed by deterministic generated content. They are measurable and readable, but not production art.
- Collision is terrain sampling plus AABB proxies. There is no selected Rapier/Avian integration, dynamic rigid-body layer, slope-aware capsule controller, or imported collider pipeline.
- Island/world-floor residency is bounded and observable, but full asynchronous asset streaming, floating-origin support, and per-chunk physics activation are not implemented.
- Atmosphere, volumetric fog/light, bloom, shadows, clouds, wind visuals, and island detail are measurable performance costs and are not optimized for laptop power draw.
- The development-play gate protects the ordinary `cargo run` path and same-host debug/release parity, but it is not a substitute for GPU timestamps, Metal counters, or longer foreground release profiles.
- Human release play remains required after changes to movement, camera, content density, rendering, collision, or streaming.

## Active Priorities

1. Turn the existing route data, lift network, recovery branches, landmarks, and gates into a clearer player-facing expedition with meaningful choices and completion feedback.
2. Preserve all 41 accepted island identities, hero landmarks, exact inventories, 123 review views, route clearance, and frame pacing through the required world, camera, and performance gates plus human release play.
3. Improve player/glider animation fidelity without losing current pose readability, attachment integrity, or traversal feel.
4. Replace prototype collision pieces only when a concrete gameplay need justifies a physics/query layer.
5. Keep streaming and rendering work evidence-driven; prefer reducing cosmetic cost or residency before adding architecture.
6. Preserve the current movement and camera baseline unless a reproducible regression requires retuning.

## Read First

- `README.md`
- `docs/ROADMAP.md`
- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/EVAL_SPEC.md`
- `docs/ISLAND_ART_DIRECTION.md`
- `docs/ISLAND_SURFACE_CONTENT.md`

Update this file when the playable baseline, accepted limitations, or active priorities change. Do not use it as a branch log or PR diary.
