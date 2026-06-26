# Project Status

Last updated: 2026-06-26

## Current Milestone

First sky-island traversal slice.

The project has a Bevy sandbox with a richer self-authored animated player fixture, playable ground movement, camera-relative planar air control, smoothed body yaw/bank response, deployable glider wings, one-launch-per-airtime vertical burst, collectible aerial boost gates, mouse-look camera follow, HUD diagnostics, debug gizmos, Bevy-native atmosphere/fog/bloom lighting, dynamic sun/fog/exposure weather, procedural PBR materials, multi-lobed drifting cloud banks with wisp-card and filament-ribbon detail plus layered high-cirrus clusters, authored crosswind fields, paired gameplay updrafts with aligned visual wind volumes and cinematic lift ribbons, marked recovery branch islands, a 12-island floating route with higher-resolution vertex-colored terrain relief, world-space tiled terrain UVs, quantized terrain material-region identity, sharper terrain-specific procedural PBR textures with smoothed broad material noise, irregular procedural rims, stratified generated cliff/underside body meshes, layered color-banded distant island impostors, batched ground-cover/detail props, deterministic multi-ring/root-flared trunk meshes, multi-lobed/detail-card near-LOD tree canopies, wind-responsive ponds, complete declared glTF fixture coverage across asset residency classes, visible route-anchored authored world fixture scenes with stronger terrain, foliage, rock, water, route-marker, weather, and distant-impostor mesh/material floors, Bevy-native glTF scene/animation readiness hooks, repo-native asset fixture audits, background-safe terrain export/audit artifacts, and scripted evals for ground taxi control, mouse camera control, yaw/strafe/turn camera stability, air-control response, baseline traversal, updraft lift, aerial boost collection, branch recovery landing, long-glide visibility, and island launch-to-landing.

## Last Known Good

- Commit: `0eebcc8`
- Merged PR: `#91` - Add richer island impostor eval gates
- Verification:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `./tools/terrain_export.sh target/terrain_material_granularity_export`
  - metric-only evals for baseline traversal and air-control response
  - screenshot baseline eval with visual audit, opaque PNG alpha, and foreign-canvas checks

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
- The sandbox spawns a 12-island floating archipelago with higher-resolution generated visual terrain relief, vertex-color surface variation, per-island terrain/detail biome palettes, world-space tiled terrain UVs, encoded terrain material-weight channels, derived material regions, terrain-specific texture-detail bands, irregular island rims, stratified generated cliff/underside body meshes, layered color-banded distant island impostors, smooth terrain normals, dense curved-blade near-LOD ground cover, organic canopy/cloud detail-card meshes, irregular pond surfaces, stacked route cairns, a crystalized launch beacon, organic landing-garden markers, a launch island, long-glide route, and landing target.
- The camera uses Bevy-native atmosphere, dynamic distance fog, volumetric fog/light, bloom, Aces tonemapping, exposure tuning, and atmosphere-driven environment lighting.
- Terrain, biome-specific ground cover, tree canopies, trunks, irregular rock scatter, water, suit, glider, and markers use generated surface textures with PBR roughness, occlusion, and parallax depth maps; terrain albedo uses smoothed broad material noise plus fine mineral flecks so large islands do not stretch one blurred texture over the whole surface; marker and flower materials feed bloom through emissive color.
- Vertically layered drifting cloud banks with wisp-card edge detail and sinuous filament-ribbon geometry, layered high-cirrus clusters, and wind-responsive near-LOD generated trees with multi-ring tapered trunks, branch mass, root flares, lobed canopies, and organic detail cards provide non-debug weather/environment motion layers without changing gameplay collision or traversal math.
- Route-surface contact can land the player on an island and applies landing damping once instead of crushing standing WASD movement every frame.
- Runtime movement is camera-relative, with airborne planar steering, targeted counter-steer authority for lateral reversals, pure backward air-brake control that bleeds sideways drift, rear-diagonal glide steering, body yaw and body bank smoothed toward intended movement, body-local generated pose lean, error-scaled turn recovery for large lateral input changes, and horizontal velocity as the no-input facing fallback.
- Mouse camera control has player-centered orbit pitch, separate yaw and pitch sensitivity, pitch clamps, click-to-lock cursor capture, right-mouse temporary look, and `Esc` release.
- Camera keeps smoothed mostly-forward follow direction independent from mouse orbit, ignores sideways/backward movement for automatic follow-heading changes, avoids tagged obstruction volumes, and stays above the active ground surface.
- HUD reports frame time, camera pitch, camera distance, player framing angle, camera motion, camera orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, altitude, mode, launch state, target distance, visual asset admission/load/preload/scene/animation readiness, visible authored world fixture count, visual wind-field count, active lift-field count, environment-motion count/offset, and sky-island count.
- Authored glTF scene readiness and animation linking are split: `SceneInstanceReady` marks scene lifecycle state, then a retryable update path discovers nested `AnimationPlayer`s, validates the declared named clips, and attaches the animation graph once dependencies are present.
- The seven non-player authored world glTF fixture scenes now spawn visibly on route islands after scene readiness, and evals gate their visible fixture-kind count separately from the existing load/spawn/ready scene lifecycle counters.
- `F1` toggles debug gizmos for player vectors, camera line, visual wind/updraft stream fields, and gameplay lift fields.
- Crosswind fields remain visual-only; gameplay updrafts are authored as paired visual wind volumes plus bounded `LiftField`s, with faint lift haze, animated spiral airflow ribbons, and small motes in the normal scene.
- Three authored aerial boost gates are visible as glowing route rings, apply capped forward/upward boosts while airborne, disappear after collection, and report visible/collected/active-effect counters in HUD/eval metrics.
- `sunlit terrace` and `western refuge` are marked as recovery branch islands with visible mast/ring beacons.
- Reusable logic now lives in dedicated modules for asset readiness, movement, environment, camera, route-surface/world math, diagnostics, eval metrics, and richer pose math.
- `ground_taxi_control` eval proves pre-launch camera-relative WASD moves the player across the launch island without leaving grounded mode.
- `camera_mouse_control` eval proves scripted mouse X/Y deltas exercise yaw and both pitch directions without hiding camera regressions behind player movement.
- `camera_yaw_stability` eval proves a small yaw impulse does not keep rotating after mouse input stops.
- `camera_strafe_stability` eval proves `A`/`D` movement does not auto-orbit the camera by bounding movement-only view-yaw/world-yaw drift while requiring measurable right and left strafe response.
- `camera_turn_stability` eval proves rapid airborne turns and backward air-braking stay within camera step/rotation thresholds.
- `air_control_response` eval proves diagonal/lateral airborne steering, separate right/left/rear-right/rear-left response latency, rear-right/rear-left rearward response, stronger total and planar backward braking, post-brake recovery, heading alignment, bounded average/p95/max body-heading error, bounded max yaw-error step, yaw oscillation count, left/right body-bank response, bounded body-roll step, zero movement-driven camera orbit offset, bounded camera rotation delta, bounded average/p95 camera follow-direction error, and bounded movement-only view-yaw/world-yaw drift.
- `updraft_route` eval proves a scripted route enters a gameplay lift field, sees a paired visible updraft while lift is active, and gains altitude beyond the normal route ceiling.
- `branch_recovery_route` eval proves a scripted route can target and land on the named `sunlit terrace` branch island after using readable lift and late air-braking.
- `long_glide_visibility` eval proves sustained traversal across the larger archipelago while collecting the three aerial boost gates and preserving content-scale and LOD signals.
- `island_launch_to_landing` eval proves the scripted route reaches and lands on the target island.
- The HUD and eval samples now track a route objective sequence: main routes point from the near updraft to the landing garden, while branch-target evals add the distant recovery updraft before the named branch landing.
- Metric-only app evals hide the native window by default; `NAU_EVAL_SIM_ONLY=1 ./tools/eval.sh <scenario> <dir>` runs `traversal_sim_eval` without creating a Bevy app or native window, `./tools/eval_sim_suite.sh target/eval/sim_suite` runs all scripted scenarios through that simulation path with one aggregate summary and one asset-fixture audit, and screenshot evals remain explicit via `NAU_EVAL_SCREENSHOT=1`.
- `./tools/terrain_export.sh target/terrain_export` writes a manifest, per-island terrain/cliff/underside/impostor OBJ meshes, terrain material-weight CSV sidecars with material-region coverage, and `audit.json` without creating a native window; the audit now requires terrain texture-detail and texture-edge floors, terrain/body/impostor silhouette and vertical-mass floors, and every island's base, transition, highland, and exposed terrain-region coverage above minimum floors.
- `./tools/visual_content_export.sh target/visual_content_export` writes a manifest, generated ground-cover/tree/cloud/landmark OBJ meshes, and `audit.json` without creating a native window; the audit requires artifact presence, OBJ vertex/face agreement, ground-cover blade density and height variance, multi-ring trunk mesh floors, trunk taper, branch reach/count, root-flare count, canopy lobe/detail-card structure, tree height/canopy-radius variation, cloud veil plus lobe/wisp-card/filament-ribbon/depth-span floors, route-cairn/launch-beacon/landing-marker/pond-surface count and mesh/span floors, and terrain/detail palette diversity.
- Screenshot evals run a non-golden visual audit for resolution, exposure, contrast, color variety, edge density, sky/scene balance, center-scene detail, scene detail tile frequency, flat low-detail scene-tile dominance, player visibility, severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, route-marker readability/component identity/hue diversity, distant horizon/impostor component readability and color-bucket identity, terrain/material family diversity, foliage coverage, cloud-layer coverage/component identity, and HUD-text dominance when launched through `tools/eval.sh`; they also emit projection-backed marker sidecars that classify known route beacons, route objectives, uncollected aerial power-ups, and terrain/foliage/cloud/distant-island scene samples as visible, occluded, offscreen, or behind-camera, plus `marker_projection_audit.json` to verify marker-colored pixels near non-occluded projected visible markers and `semantic_scene_audit.json` to verify every visible scene-material family has enough projected material-like pixels per checkpoint, every checkpoint keeps enough distinct visible scene sample kinds, and each expected sample kind (`terrain_surface`, `tree_canopy`, `weather_cloud`, and `distant_island`) has at least one visible projected sample/hit across the checkpoint sequence. Screenshot runs disable debug gizmos and use opaque window composition so transparent scene effects cannot reveal desktop content.
- Eval summaries now include the scenario target island, route objective progress, average/p95/max body-heading error, max yaw-error step, desired-heading velocity alignment, body-roll step, left/right body-bank response, lateral response speed/latency, separate right/left/rear-right/rear-left air-control response, rear-right/rear-left rearward response, total and planar air-brake recovery metrics, yaw oscillation count, camera surface clearance, camera-to-player framing angle, camera step/rotation deltas, camera orbit alignment, average/p95/max camera follow-direction error, camera view-yaw drift, camera world-yaw drift, obstruction adjustment/hits, camera yaw/pitch offsets, checkpoint screenshot and marker-metadata paths, max scene entity count, weather cloud count, weather cloud-bank count/depth, weather cloud lobe/mesh/filament-ribbon floors, environment-motion count/offset, island terrain surface count, terrain mesh vertex floor, terrain vertex-color band floor, terrain material-weight band/channel/region floors, terrain texture-detail floor, terrain relief range, cliff color-band floor, distant island impostor mesh/color-band floors, procedural island body count, primitive island body count, island body silhouette complexity, island body mesh vertex floor and max, generated ground-cover density/mesh complexity, generated tree/cloud/rock mesh complexity, generated detail biome-palette count, visual asset slot/deferred-load/load-state/dependency-preload/scene-instance/animation readiness counts, visible authored world fixture count, aerial power-up inventory/collection/effect counters, readable/unreadable lift samples, hidden/resident/catalog island visual counts, resident visual fraction, and directional stream spawn/despawn churn so camera/control/content/streaming regressions are visible in metrics.

## Known Issues

- The character now has a self-authored animated glTF fixture with faceted body meshes, readable face/eye lenses, belt hardware, gauntlet cuffs, knee guards, boots, shoulder guards, scarf pieces, and named clip coverage, but it is still not a rigged production character.
- Limb posing has grounded stride, airborne banking, glide posture, and speed-responsive wing flex for generated fallback geometry; the authored fixture now proves retryable named-clip validation, graph readiness, and runtime clip transitions, but it is still approximate non-skeletal animation.
- Camera obstruction avoidance uses simple tagged AABBs, not a full physics sweep.
- Crosswind stream fields are still debug gizmos; updrafts, generated trees, clouds, and ponds now have cinematic generated motion/mesh cues, and the visible glider has a multi-part authored glTF fixture, but there are no particles, cloth simulation, or production environment art assets yet.
- Sky-island collision follows deterministic terrain relief, but it is still a route-surface clamp rather than full rigid-body physics.
- Gameplay lift and power-ups are still first rough authored routes; there is no crosswind force, launch-source chain, inventory UI, or authored recovery-route design beyond two marked primitive branch islands.
- There is now a deterministic visual-asset load admission policy, but no full asynchronous distance-streaming implementation or production environment asset library yet. The default policy admits every current declared fixture and evals fail if any current fixture is deferred. Every declared glTF slot is now measurable: self-authored player, multi-part glider, terrain, foliage, rock, water, route-marker, weather-layer, and distant-impostor fixtures load/spawn/report ready as real `SceneRoot` assets across always, stream-window, near-LOD, weather, and far-LOD residency classes. The seven non-player world fixtures are visible decorative placements, not collision-authoritative terrain, foliage, water, marker, weather, or impostor production art; their generators now include extra authored signals such as terrain terrace ledges, canopy detail cards, wildflower accents, pond depth/ripple layers, fracture/quartz rock detail, route pennants, feathered cloud wisps, and distant tree silhouettes. `asset_fixture_audit` gates fixture provenance, registry-aligned `extras.nau` schema/kind/label/residency/license metadata, semantic component names including player face/eye/belt/gauntlet/knee pieces, mesh/material/vertex/triangle floors, normals, UVs, transparent-material expectations, and declared player clip names. The player fixture attaches a named-clip `AnimationGraph`/`AnimationTransitions` pair and switches clips from movement state, and generated gameplay visuals remain the fallback until production-authored scene instances replace them deliberately. Current stream-window terrain residency, detail LOD, procedural island bodies, procedural materials, per-island biome terrain/detail palettes, ground cover, generated multi-ring/root-flared/branched trunks, lobed canopies, wind-responsive ponds, stones, beacon, denser multi-lobed cloud layers, and landing markers are deterministic generated systems.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Replace or deepen the temporary visible environment fixture scenes and faceted player fixture with authored or compatible production-quality glTF assets that satisfy the declared scene, visible world-fixture, and player animation clip readiness metrics.
2. Deepen screenshot terrain/material, vegetation/cloud, and distant-impostor gates from projected semantic-kind presence into stronger quality and composition checks.
3. Add asynchronous asset-loading policy and budget checks once real imported scenes exist.

## Read First

- `docs/ARCHITECTURE.md`
- `docs/MECHANICS/flight.md`
- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/app_tests.rs`
- `src/asset_pipeline.rs`
- `src/movement.rs`
- `src/environment.rs`
- `src/camera.rs`
- `src/world.rs`
- `src/diagnostics.rs`
- `src/eval.rs`
- `src/eval/accumulator.rs`
- `src/eval/sample.rs`
- `src/eval/scenarios.rs`
- `src/eval/summary.rs`
- `src/eval/tests.rs`
- `src/eval/thresholds.rs`
- `src/animation.rs`
- `src/camera_runtime.rs`
- `src/authored_assets.rs`
- `src/content_diagnostics.rs`
- `src/debug_readout_runtime.rs`
- `src/debug_visuals.rs`
- `src/environment_visuals.rs`
- `src/eval_app_runtime.rs`
- `src/generated_content.rs`
- `src/generated_content/detail_meshes.rs`
- `src/generated_content/detail_meshes/`
- `src/generated_content/island_meshes.rs`
- `src/island_visuals.rs`
- `src/player_runtime.rs`
- `src/power_up_runtime.rs`
- `src/scene_setup_runtime.rs`
- `src/content_export.rs`
- `src/content_export/terrain.rs`
- `src/content_export/visual.rs`
- `src/content_export/shared.rs`
- `src/eval_runtime.rs`
- `src/main.rs`

## Status Discipline

Do not update this file for every branch checkout, worktree, or `main naux` operation. Update it when a milestone lands, when a meaningful PR changes project direction, or before handing the project to a future session.
