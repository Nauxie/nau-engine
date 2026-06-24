# Project Status

Last updated: 2026-06-24

## Current Milestone

First sky-island traversal slice.

The project has a Bevy sandbox with a primitive humanoid, playable ground movement, camera-relative planar air control, deployable glider wings, one-launch-per-airtime vertical burst, collectible aerial boost gates, mouse-look camera follow, HUD diagnostics, debug gizmos, Bevy-native atmosphere/fog/bloom lighting, dynamic sun/fog/exposure weather, procedural PBR materials, multi-lobed drifting cloud banks and layered high-cirrus clusters, authored crosswind fields, paired gameplay updrafts with aligned visual wind volumes and cinematic lift ribbons, marked recovery branch islands, a 12-island floating route with higher-resolution vertex-colored terrain relief, irregular procedural rims, stratified generated cliff/underside body meshes, batched ground-cover/detail props, deterministic tapered/multi-lobed near-LOD trees and wind-responsive ponds, Bevy-native glTF scene/animation readiness hooks, and scripted evals for ground taxi control, mouse camera control, yaw/strafe/turn camera stability, air-control response, baseline traversal, updraft lift, aerial boost collection, branch recovery landing, long-glide visibility, and island launch-to-landing.

## Last Known Good

- Commit: `445cb5a`
- Merged PR: `#41` - Add streaming LOD pressure counters
- Verification:
  - `cargo fmt --all --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo naux -- --help`
  - metric-only evals for baseline traversal, ground taxi control, long-glide visibility, and branch recovery traversal, including streaming residency and directional churn gates

## Active Work

Use this section for milestone handoffs, not routine worktree changes.

- Active branch: none on `main`
- Open PRs: consult GitHub

## What Works

- Native macOS Bevy app launches on Apple M4 Max through wgpu/Metal.
- Player entity has movement, velocity, flight controller state, animation state, and a primitive child-model hierarchy.
- `WASD` works on the ground before launch; ground movement has separate acceleration, top speed, and friction from airborne/glider motion.
- `E` launches from the ground and is gated to one launch per airtime.
- `Space` deploys glider wings while airborne, with speed-responsive wing flex and subtle wingtip airflow trails.
- `Shift` dives.
- The sandbox spawns a 12-island floating archipelago with higher-resolution generated visual terrain relief, vertex-color surface variation, irregular island rims, stratified generated cliff/underside body meshes, smooth terrain normals, batched near-LOD ground cover, a launch island, long-glide route, and landing target.
- The camera uses Bevy-native atmosphere, dynamic distance fog, volumetric fog/light, bloom, Aces tonemapping, exposure tuning, and atmosphere-driven environment lighting.
- Terrain, ground cover, props, water, suit, glider, and markers use generated surface textures with PBR roughness, occlusion, and parallax depth maps; marker and flower materials feed bloom through emissive color.
- Multi-lobed drifting cloud banks, layered high-cirrus clusters, and wind-responsive near-LOD generated tree/pond visuals provide non-debug weather/environment motion layers without changing gameplay collision or traversal math.
- Route-surface contact can land the player on an island and applies landing damping once instead of crushing standing WASD movement every frame.
- Runtime movement is camera-relative, with airborne planar steering, targeted counter-steer authority for lateral reversals, backward air-brake control, body yaw smoothed toward intended movement, error-scaled turn recovery for large lateral input changes, and horizontal velocity as the no-input facing fallback.
- Mouse camera control has player-centered orbit pitch, separate yaw and pitch sensitivity, pitch clamps, click-to-lock cursor capture, right-mouse temporary look, and `Esc` release.
- Camera keeps smoothed mostly-forward follow direction independent from mouse orbit, ignores sideways/backward movement for automatic follow-heading changes, avoids tagged obstruction volumes, and stays above the active ground surface.
- HUD reports frame time, camera pitch, camera distance, player framing angle, camera motion, camera orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, altitude, mode, launch state, target distance, visual asset scene/animation readiness, visual wind-field count, active lift-field count, environment-motion count/offset, and sky-island count.
- `F1` toggles debug gizmos for player vectors, camera line, visual wind/updraft stream fields, and gameplay lift fields.
- Crosswind fields remain visual-only; gameplay updrafts are authored as paired visual wind volumes plus bounded `LiftField`s, with faint lift haze, animated spiral airflow ribbons, and small motes in the normal scene.
- Three authored aerial boost gates are visible as glowing route rings, apply capped forward/upward boosts while airborne, disappear after collection, and report visible/collected/active-effect counters in HUD/eval metrics.
- `sunlit terrace` and `western refuge` are marked as recovery branch islands with visible mast/ring beacons.
- Traversal, route-surface geometry, visual wind-field geometry, gameplay lift math, wind-sway visual motion, camera, diagnostics, eval metrics, and richer pose math live in testable pure functions in `src/lib.rs`.
- `ground_taxi_control` eval proves pre-launch camera-relative WASD moves the player across the launch island without leaving grounded mode.
- `camera_mouse_control` eval proves scripted mouse X/Y deltas exercise yaw and both pitch directions without hiding camera regressions behind player movement.
- `camera_yaw_stability` eval proves a small yaw impulse does not keep rotating after mouse input stops.
- `camera_strafe_stability` eval proves `A`/`D` movement does not auto-orbit the camera by bounding movement-only world-yaw drift.
- `camera_turn_stability` eval proves rapid airborne turns and backward air-braking stay within camera step/rotation thresholds.
- `air_control_response` eval proves diagonal/lateral airborne steering, separate right/left response latency, backward braking, post-brake recovery, heading alignment, bounded average/p95/max body-heading error, bounded max yaw-error step, yaw oscillation count, zero movement-driven camera orbit offset, bounded camera rotation delta, bounded average/p95 camera follow-direction error, and bounded movement-only world-yaw drift.
- `updraft_route` eval proves a scripted route enters a gameplay lift field, sees a paired visible updraft while lift is active, and gains altitude beyond the normal route ceiling.
- `branch_recovery_route` eval proves a scripted route can target and land on the named `sunlit terrace` branch island after using readable lift and late air-braking.
- `long_glide_visibility` eval proves sustained traversal across the larger archipelago while collecting the three aerial boost gates and preserving content-scale and LOD signals.
- `island_launch_to_landing` eval proves the scripted route reaches and lands on the target island.
- The HUD and eval samples now track a route objective sequence: main routes point from the near updraft to the landing garden, while branch-target evals add the distant recovery updraft before the named branch landing.
- Metric-only evals hide the native window by default; screenshot evals are explicit via `NAU_EVAL_SCREENSHOT=1`.
- Screenshot evals run a non-golden visual audit for resolution, exposure, contrast, color variety, edge density, sky/scene balance, center-scene detail, scene detail tile frequency, flat low-detail scene-tile dominance, player visibility, severe border clipping, route-marker readability/component identity, route-marker hue telemetry, and HUD-text dominance when launched through `tools/eval.sh`, and disable debug gizmos while using opaque window composition so transparent scene effects cannot reveal desktop content.
- Eval summaries now include the scenario target island, route objective progress, average/p95/max body-heading error, max yaw-error step, desired-heading velocity alignment, lateral response speed/latency, separate right/left air-control response, air-brake recovery metrics, yaw oscillation count, camera surface clearance, camera-to-player framing angle, camera step/rotation deltas, camera orbit alignment, average/p95/max camera follow-direction error, camera view-yaw drift, camera world-yaw drift, obstruction adjustment/hits, camera yaw/pitch offsets, checkpoint screenshot paths, max scene entity count, weather cloud count, environment-motion count/offset, island terrain surface count, terrain mesh vertex floor, terrain vertex-color band floor, terrain relief range, cliff color-band floor, procedural island body count, primitive island body count, island body silhouette complexity, island body mesh vertex count, generated tree/cloud mesh complexity, visual asset slot/load-state/scene-instance/animation readiness counts, aerial power-up inventory/collection/effect counters, readable/unreadable lift samples, hidden/resident/catalog island visual counts, resident visual fraction, and directional stream spawn/despawn churn so camera/control/content/streaming regressions are visible in metrics.

## Known Issues

- The character is still primitive geometry, not a rigged character asset.
- Limb posing now has grounded stride, airborne banking, glide posture, and speed-responsive wing flex, but it is still approximate non-skeletal animation.
- Camera obstruction avoidance uses simple tagged AABBs, not a full physics sweep.
- Crosswind stream fields are still debug gizmos; updrafts, glider wings, generated trees, clouds, and ponds now have cinematic primitive motion cues, but there are no particles, cloth simulation, or authored environment art assets yet.
- Sky-island collision follows deterministic terrain relief, but it is still a route-surface clamp rather than full rigid-body physics.
- Gameplay lift and power-ups are still first rough authored routes; there is no crosswind force, launch-source chain, inventory UI, or authored recovery-route design beyond two marked primitive branch islands.
- There is no asynchronous asset streaming policy, authored water, authored vegetation, or complete environment asset pipeline yet. The declared glTF slots are measurable, player/glider files can spawn as Bevy `SceneRoot` children when present, the player scene can attach a named-clip `AnimationGraph`/`AnimationTransitions` pair when compatible clips exist, and primitives remain the fallback until those scene instances report ready. Current stream-window terrain residency, detail LOD, procedural island bodies, procedural materials, ground cover, generated tapered/lobed trees, wind-responsive ponds, stones, beacon, multi-lobed cloud layers, and landing markers are deterministic generated systems.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Replace the primitive character/environment with authored or compatible generated glTF assets that satisfy the declared scene and player animation clip readiness metrics.
2. Add richer impostors on top of the resident island visual catalog.
3. Add richer terrain-material identity, vegetation-shape, cloud-depth, and exact route-marker semantic checks to the screenshot audit.
4. Add a simulation-only eval binary if native-window metric runs become a scaling bottleneck.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
