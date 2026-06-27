# Project Status

Last updated: 2026-06-27

## Current Milestone

First sky-island traversal slice.

The project has a Bevy sandbox with a richer self-authored animated player fixture, playable ground movement, camera-relative planar air control, smoothed body yaw/bank response, deployable glider wings with visible authored traversal response, one-launch-per-airtime vertical burst, collectible aerial boost gates, mouse-look camera follow, HUD diagnostics, debug gizmos, Bevy-native atmosphere/fog/bloom lighting, dynamic sun/fog/exposure weather, procedural PBR materials, multi-lobed drifting cloud banks with wisp-card and filament-ribbon detail plus layered high-cirrus clusters, visual `WindField` volumes that drive wind visuals, diagnostics, and bounded horizontal airborne current, paired gameplay updrafts with separate `LiftField` vertical lift, cinematic lift ribbons, marked recovery branch islands, a 20-island floating route with 19 named terrain archetypes, route-aligned ravine/channel incisions, terrace/shelf/basin/ridge/needle/satellite variation, fine microrelief, higher-resolution vertex-colored terrain relief, world-space tiled terrain UVs, quantized terrain material-region identity, sharper terrain-specific procedural PBR textures with smoothed broad material noise, irregular procedural rims shared by terrain and ground containment, stratified generated cliff/underside body meshes, layered color-banded distant island impostors, batched ground-cover/detail props, deterministic multi-ring/root-flared trunk meshes, multi-lobed/detail-card near-LOD tree canopies, collidable trees/rocks/route landmarks, wind-responsive ponds, complete declared glTF fixture coverage across asset residency classes, visible route-anchored authored world fixture scenes with stronger terrain, foliage, rock, water, route-marker, weather, and distant-impostor mesh/material floors, Bevy-native glTF scene/animation readiness hooks, repo-native asset fixture audits, background-safe terrain/export and sim/app eval coverage, and scripted evals for ground taxi control, generated-asset collision contact, mouse camera control, yaw/strafe/turn camera stability, air-control response, baseline traversal, updraft lift, aerial boost collection, branch recovery landing, long-glide visibility, and island launch-to-landing.

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
- `Space` deploys glider wings while airborne, with speed-responsive wing flex, subtle wingtip airflow trails, and visible authored glider banking/braking/dive response.
- `Shift` dives.
- Player posing now derives a shared pose intent for grounded idle/stride, launch, fall, glide, explicit airborne turn, dive, air brake, landing anticipation, and post-touchdown landing recovery so generated fallback poses, authored player nodes, authored clip selection, eval labels, and pose-readability metrics stay aligned; the generated and authored player surfaces now exaggerate dive flattening, air-brake arm spread, sink-scaled landing flare/forward leg tuck/crouch, impact recovery, signed input-responsive airborne turn lean, grounded idle breathing, and subtle glide/dive airflow limb motion, with high-sink descents selecting landing anticipation before the contact frame.
- The sandbox spawns a 20-island floating archipelago with 19 named terrain archetypes, higher-resolution generated visual terrain relief, route-aligned ravine/channel incisions, terrace/shelf/basin/ridge/needle/mist-arch/broken-stair/cloud-gate/satellite variation, fine microrelief, vertex-color surface variation, per-island terrain/detail biome palettes, world-space tiled terrain UVs, encoded terrain material-weight channels, derived material regions, terrain-specific texture-detail bands, irregular island rims shared by terrain mesh generation and ground containment, stratified generated cliff/underside body meshes, layered color-banded distant island impostors, smooth terrain normals, dense curved-blade near-LOD ground cover, organic canopy/cloud detail-card meshes, irregular pond surfaces, stacked route cairns, a crystalized launch beacon, organic landing-garden markers, a launch island, long-glide route, and landing target.
- The camera uses Bevy-native atmosphere, dynamic distance fog, volumetric fog/light, bloom, Aces tonemapping, exposure tuning, and atmosphere-driven environment lighting.
- Terrain, biome-specific ground cover, tree canopies, trunks, irregular rock scatter, water, suit, glider, and markers use generated surface textures with PBR roughness, occlusion, and parallax depth maps; terrain albedo now uses 512px tiled broad material noise plus sharper pitted strata, fissure, mineral-fleck, and micro-grain detail, while non-terrain generated surface textures use 128px maps so large islands and props do not smear one blurred texture over the whole surface; marker and flower materials feed bloom through emissive color.
- Five-layer drifting cloud banks with denser wisp-card edge detail and sinuous filament-ribbon geometry, layered high-cirrus clusters, wind-responsive near-LOD generated trees with multi-ring tapered trunks, branch mass, root flares, lobed canopies, and organic detail cards, and denser advection-driven updraft/crosswind ribbons and motes provide non-debug weather/environment motion layers while shared `WindField` flow adds traveling gusts, crosswind lane shear, and updraft curl; gameplay response from `WindField` stays horizontal and `LiftField` owns vertical lift.
- Near-LOD generated tree trunks, rock scatter, route cairns, launch beacons, recovery masts, target markers, and solid visible authored terrain/foliage/rock/route-marker fixture scenes now spawn simple AABB world-collision proxies; runtime movement resolves the player footprint out of those proxies while eval summaries track proxy count, collision-resolved samples, meaningful contact samples, and max push distance.
- Route-surface contact can land the player on an island and applies landing damping once instead of crushing standing WASD movement every frame.
- Runtime movement is camera-relative from the stable camera follow direction plus explicit mouse orbit, with airborne planar steering, input-aligned planar thrust, explicit horizontal velocity turning toward camera-relative lateral/rear-diagonal headings, targeted counter-steer authority for lateral reversals, pure backward air-brake control that bleeds sideways drift, rear-diagonal lateral steering while air-brake owns rearward slowdown, pure-backward body-facing intent without uncapped reverse velocity, body yaw and body bank smoothed toward intended movement, body-local generated/authored pose lean, error-scaled turn recovery for large lateral input changes, and horizontal velocity as the no-input facing fallback.
- Mouse camera control has player-centered orbit pitch, separate yaw and pitch sensitivity, pitch clamps, click-to-lock cursor capture, right-mouse temporary look, and `Esc` release.
- Camera keeps smoothed mostly-forward follow direction independent from mouse orbit, exposes that stable follow direction as the movement basis, ignores sideways/backward movement for automatic follow-heading changes, avoids tagged obstruction volumes, and stays above the active ground surface.
- HUD reports frame time, camera pitch, camera distance, player framing angle, camera motion, camera orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, altitude, mode, launch state, target distance, visual asset admission/load/preload/scene/animation readiness, authored animation current/desired clip state, visible authored world fixture count, visual wind-field count, active lift-field count, world-collision proxy/resolution/push metrics, environment-motion count/offset, and sky-island count.
- Authored glTF scene readiness and animation linking are split: `SceneInstanceReady` marks scene lifecycle state, then a retryable update path discovers nested `AnimationPlayer`s, validates the declared named clips, and attaches the animation graph once dependencies are present.
- The seven non-player authored world glTF fixture scenes now spawn visibly on route islands after scene readiness, and evals gate their visible fixture-kind count separately from the existing load/spawn/ready scene lifecycle counters.
- `F1` toggles debug gizmos for player vectors, camera line, visual wind/updraft stream fields, and gameplay lift fields.
- Visual `WindField` volumes now drive both readable airflow visuals/diagnostics and bounded horizontal airborne current: crosswinds push laterally, updraft swirl bends horizontal motion, wind guide/ribbon visual motion is checked against shared `WindField::flow_at` vectors, and vertical climb remains separate bounded `LiftField` lift.
- Three authored aerial boost gates are visible as glowing route rings, apply capped forward/upward boosts while airborne, disappear after collection, and report visible/collected/active-effect counters in HUD/eval metrics.
- `sunlit terrace` and `western refuge` are marked as recovery branch islands with visible mast/ring beacons.
- Reusable logic now lives in dedicated modules for asset readiness, movement, environment, camera, route-surface/world math, diagnostics, eval metrics, and richer pose math.
- `ground_taxi_control` eval proves pre-launch camera-relative WASD moves the player across the launch island without leaving grounded mode.
- `world_collision_contact` eval proves a grounded route sustains meaningful pushed contact with a generated launch-mesa obstacle, so collidable asset props must affect movement instead of only existing as counted entities.
- `terrain_rim_collision_contact` eval proves near-LOD islands spawn four invisible terrain-rim collision rails each, a grounded route can be blocked by the launch-mesa rim, and normal `ground_taxi_control` movement does not report false terrain-rim contact.
- `camera_mouse_control` eval proves scripted mouse X/Y deltas exercise yaw and both pitch directions without hiding camera regressions behind player movement.
- `camera_yaw_stability` eval proves a small yaw impulse does not keep rotating after mouse input stops.
- `camera_strafe_stability` eval proves `A`/`D` movement does not auto-orbit the camera by bounding movement-only view-yaw/world-yaw drift while requiring measurable right and left strafe response.
- `camera_turn_stability` eval proves rapid airborne turns and backward air-braking stay within camera step/rotation thresholds.
- `air_control_response` eval proves diagonal/lateral airborne steering, separate right/left/rear-right/rear-left response latency, rear-right/rear-left rearward response, stronger total and planar backward braking, pure-backward body-heading intent, readable right/left airborne turn-pose coverage, readable dive and air-brake pose coverage, visible generated/authored part or authored-clip-aligned pose quality, authored dive/air-brake clip coverage with zero clip mismatches, visible pose part coverage with bounded key-pose rotation/translation deltas, pose torso pitch/arm spread/leg tuck/turn lean/wing airflow readability, visible authored glider response/motion, signed right/left pose lean, key-pose score coverage with zero unreadable key-pose samples, post-brake recovery, desired-heading and aggregate plus right/left/backward-right/backward-left desired-travel alignment, bounded average/p95/max body-heading error, stricter desired-travel and body/travel heading error bounds for omnidirectional air control, non-vacuous right/left and backward-right/backward-left body/travel heading samples with bounded error, bounded max yaw-error step, yaw oscillation count, left/right body-bank response, bounded body-roll step, zero movement-driven camera orbit offset, bounded camera rotation delta, bounded average/p95 camera follow-direction error, and bounded movement-only view-yaw/world-yaw drift.
- `pose_state_coverage` eval proves a compact no-mouse walk/run/launch/fall/glide route emits grounded walk/run samples, readable launch/fall/glide samples, and zero unreadable key-pose samples in both the app and windowless sim harnesses.
- `updraft_route` eval proves a scripted route enters a gameplay lift field, sees a paired visible updraft while lift is active, samples dynamic wind flow with nonzero speed/gust variation/range, applies sustained bounded lateral wind response, gates updraft/crosswind guide and ribbon motion against shared field flow, and gains altitude only through `LiftField` lift.
- `branch_recovery_route` eval proves a scripted route can target and land on the named `sunlit terrace` branch island after using readable lift and late air-braking while gating readable landing anticipation, authored landing clip coverage, landing flare, post-contact landing recovery, landing crouch depth, zero unreadable key-pose samples, and landing-only pose temporal stability.
- `long_glide_visibility` eval proves sustained traversal across the larger archipelago while collecting the three aerial boost gates and preserving content-scale and LOD signals.
- `island_launch_to_landing` eval proves the scripted route reaches and lands on the target island while gating readable landing anticipation, authored landing clip coverage, landing flare, post-contact landing recovery, landing crouch depth, zero unreadable key-pose samples, and landing-only pose temporal stability.
- The HUD and eval samples now track a route objective sequence: main routes point from the near updraft to the landing garden, while branch-target evals add the distant recovery updraft before the named branch landing.
- Metric-only app evals hide the native window by default; `NAU_EVAL_SIM_ONLY=1 ./tools/eval.sh <scenario> <dir>` runs `traversal_sim_eval` without creating a Bevy app or native window, `./tools/eval_sim_suite.sh target/eval/sim_suite` runs simulation-supported scripted scenarios through that path with wind-current and pose-state metrics/gates, one aggregate summary, and one asset-fixture audit. App-only routes such as `world_collision_contact` and `terrain_rim_collision_contact` still use the default app eval path, and screenshot evals remain explicit via `NAU_EVAL_SCREENSHOT=1`.
- `./tools/terrain_export.sh target/terrain_export` writes a manifest, per-island terrain/cliff/underside/impostor OBJ meshes, terrain material-weight CSV sidecars with material-region coverage, and `audit.json` without creating a native window; the audit now requires terrain-archetype diversity, terrain texture-detail and texture-edge floors, terrain/body/impostor silhouette and vertical-mass floors, manifest and OBJ height-band/normal-slope-band floors, and every island's base, transition, highland, and exposed terrain-region coverage above minimum floors.
- `./tools/visual_content_export.sh target/visual_content_export` writes a manifest, generated ground-cover/tree/cloud/landmark OBJ meshes, and `audit.json` without creating a native window; the audit requires artifact presence, OBJ vertex/face agreement, ground-cover blade density and height variance, multi-ring trunk mesh floors, trunk taper, branch reach/count, root-flare count, canopy lobe/detail-card structure, tree height/canopy-radius variation, cloud veil plus lobe/wisp-card/filament-ribbon/depth-span floors, route-cairn/launch-beacon/landing-marker/pond-surface count and mesh/span floors, and terrain/detail palette diversity.
- Screenshot evals run a non-golden visual audit for resolution, exposure, contrast, color variety, edge density, sky/scene balance, center-scene detail, scene detail tile frequency, flat low-detail scene-tile dominance, player visibility, severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, route-marker readability/component identity/hue diversity, distant horizon/impostor component readability/color-bucket/span identity, terrain/material family diversity, terrain material coverage/color/tile spread, foliage coverage/tile spread, cloud-layer coverage/component/span identity, and HUD-text dominance when launched through `tools/eval.sh`; they also emit projection-backed marker sidecars that classify known route beacons, route objectives, uncollected aerial power-ups, and terrain/foliage/cloud/distant-island/wind scene samples as visible, occluded, offscreen, or behind-camera, plus `marker_projection_audit.json` to verify marker-colored pixels near non-occluded projected visible markers and `semantic_scene_audit.json` to verify every visible scene-material family has enough projected material-like pixels per checkpoint, every checkpoint keeps enough distinct visible scene sample kinds, every visible terrain material variant has projected terrain pixels plus per-variant coverage, wind-critical checkpoints keep at least one visible wind guide/ribbon sample, conditional visible wind samples clear wind-like pixel-hit floors, each expected sample kind (`terrain_surface`, `tree_canopy`, `weather_cloud`, and `distant_island`) has visible projected sample hits across the checkpoint sequence, and expected material/kind hits clear aggregate pixel-coverage floors. `terrain_surface` sidecars now include terrain material variant identity, and the semantic scene audit requires aggregate terrain material/biome variant diversity while still using broad terrain-like pixel matching instead of exact per-pixel biome classification. Screenshot runs disable debug gizmos and use opaque window composition so transparent scene effects cannot reveal desktop content.
- Eval summaries now include the scenario target island, route objective progress, movement/camera/pose metrics, authored clip match/mismatch and dive/air-brake/landing coverage metrics, visible authored glider response metrics, global and landing-only pose temporal metrics, dynamic wind-flow field/speed/variation/range metrics, split wind guide/ribbon direction-flow metrics including updraft swirl displacement and short-horizon flow-coherent visual counts/alignment, wind-force field/delta/flow/variation metrics plus sustained meaningful force-sample counts, world-collision and terrain-rim collision metrics, content/asset/streaming metrics, aerial power-up counters, and readable/unreadable/dynamic-readable lift samples so camera/control/content/collision/streaming and wind-current regressions are visible in metrics.

## Known Issues

- The character now has a self-authored animated glTF fixture with faceted body meshes, readable face/eye lenses, belt hardware, gauntlet cuffs, knee guards, boots, shoulder guards, scarf pieces, and named clip coverage, but it is still not a rigged production character.
- Limb posing has grounded walk/run stride, readable launch/fall, airborne banking, glide, dive, air-brake, landing-anticipation posture, post-touchdown recovery crouch, landing crouch, turn-readable lean, and speed-responsive wing flex for generated fallback and named authored player nodes; the authored fixture now proves retryable named-clip validation, graph readiness, runtime clip transitions, and procedural pose parity, but it is still approximate non-skeletal animation.
- Camera obstruction avoidance uses simple tagged AABBs, not a full physics sweep.
- Crosswind and updraft airflow still use generated guide gizmos/ribbons/motes rather than production particles, cloth simulation, or authored environment effects.
- Sky-island collision follows deterministic terrain relief and the same playable silhouette used by terrain mesh generation, while asset collision is simple horizontal push-out against tagged AABB proxies. This makes trees, rocks, and landmarks stop being ghost props, but it is still not full rigid-body physics, slope response, or authored capsule/collider import.
- Gameplay lift, wind current, and power-ups are still first rough authored routes; there is no launch-source chain, inventory UI, or authored recovery-route design beyond two marked primitive branch islands.
- There is now a deterministic visual-asset load admission policy, but no full asynchronous distance-streaming implementation or production environment asset library yet. The default policy admits every current declared fixture and evals fail if any current fixture is deferred. Every declared glTF slot is now measurable: self-authored player, multi-part glider, terrain, foliage, rock, water, route-marker, weather-layer, and distant-impostor fixtures load/spawn/report ready as real `SceneRoot` assets across always, stream-window, near-LOD, weather, and far-LOD residency classes. The solid non-player terrain/foliage/rock/route-marker fixture scenes now attach simple collision/camera-obstruction proxies, while water, weather, and distant-impostor fixtures remain visible non-authoritative art probes; their generators include extra authored signals such as terrain terrace ledges, erosion gullies, path stones, root flares, fern/moss cards, wildflower accents, pond depth/ripple/lily/specular layers, fracture/quartz/rust/shale rock detail, route pennants/glyphs/pebbles, feathered cloud wisps, haze pockets, filament curls, distant tree silhouettes, waterfall veils, and broken far-cliff shelves. `asset_fixture_audit` gates fixture provenance, registry-aligned `extras.nau` schema/kind/label/residency/license metadata, semantic component names including player face/eye/belt/gauntlet/knee pieces and the richer world-fixture fragments, mesh/material/vertex/triangle floors, normals, UVs, transparent-material expectations, and declared player clip names. The player fixture attaches a named-clip `AnimationGraph`/`AnimationTransitions` pair and switches clips from movement state, and generated gameplay visuals remain the fallback until production-authored scene instances replace them deliberately. Current stream-window terrain residency, detail LOD, procedural island bodies, procedural materials, per-island biome terrain/detail palettes, ground cover, generated multi-ring/root-flared/branched trunks, lobed canopies, wind-responsive ponds, stones, beacon, denser multi-lobed cloud layers, and landing markers are deterministic generated systems.
- Physics is still custom movement math, not a real collision/rigid body integration.

## Next Tasks

1. Replace the deepened temporary visible environment fixture scenes and faceted player fixture with authored or compatible production-quality glTF assets that satisfy the declared scene, visible world-fixture, and player animation clip readiness metrics.
2. Add per-biome terrain/material screenshot checks beyond the current broad terrain mask.
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
- `src/world_collision_runtime.rs`
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
