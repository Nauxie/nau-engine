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

- `src/main.rs` owns Bevy app resource/plugin setup, app system scheduling, and CLI action dispatch.
- `src/app_tests.rs` owns binary-level app integration tests for CLI parsing, eval window setup, exported content artifacts, generated content mesh quality, and app-visible helper behavior.
- `src/scene_setup_runtime.rs` owns startup scene setup orchestration, authored scene registration, follow camera spawn, and stable startup constants.
- `src/scene_setup_runtime/materials.rs` owns startup PBR material and procedural texture-handle preparation for player, terrain, clouds, updrafts, water, and detail assets.
- `src/scene_setup_runtime/world.rs` owns startup world spawning: sun, ground plane, island catalog seeding, camera obstacles, wind/lift volumes, weather layers, power-up guides, and authored world fixtures.
- `src/scene_setup_runtime/player.rs` owns startup player entity spawning, authored player/glider scene children, fallback character parts, glider wings, and airflow trail children.
- `src/scene_setup_runtime/hud.rs` owns startup HUD/debug readout root spawning.
- `src/player_runtime.rs` owns the player marker/resource types, keyboard flight input mapping, player movement ECS systems, route objective updates, generated/authored player visibility handoff, and fallback character animation.
- `src/camera_runtime.rs` owns runtime camera resources, mouse-look capture, follow-camera ECS wiring, camera obstruction components, camera spawn/render-stack setup, and camera diagnostics.
- `src/authored_assets.rs` owns the authored-asset runtime facade and stable re-exports.
- `src/authored_assets/types.rs` owns authored visual asset registry, slot, diagnostic resource, scene marker, fixture marker, and generated-placeholder types.
- `src/authored_assets/registry.rs` owns visual asset load admission, Bevy glTF/scene handle registration, scene readiness state, and animation-readiness state.
- `src/authored_assets/fixtures.rs` owns visible authored world-fixture handle lookup, sky-island placement transforms, and scene-ready observer wiring.
- `src/authored_assets/animation.rs` owns named glTF animation clip resolution, animation graph linking, authored player clip selection, and animation playback transitions.
- `src/authored_assets/diagnostics.rs` owns runtime visual asset diagnostics, load-state buckets, dependency preload readiness, scene-instance readiness, and visible fixture counts.
- `src/content_diagnostics.rs` owns runtime content-quality metric accumulation for generated island terrain, island bodies, ground cover, trees, rocks, clouds, and biome detail palettes.
- `src/content_export.rs` owns the background-safe export module surface and shared re-exports.
- `src/content_export/terrain.rs` owns terrain export reports, terrain OBJ/material-weight CSV writing, and terrain manifest metric aggregation.
- `src/content_export/visual.rs` owns the visual-content export facade and stable re-export for generated ground cover, trees, clouds, and biome palettes.
- `src/content_export/visual/` owns visual-content report serialization, export orchestration, mesh metrics, vegetation metrics, cloud artifact writing, and biome-palette summaries.
- `src/content_export/shared.rs` owns shared OBJ writing, mesh attribute inspection, slugging, and JSON formatting helpers used by export and screenshot metadata.
- `src/debug_readout_runtime.rs` owns the live HUD/debug readout component, query surface, and diagnostic text formatting.
- `src/debug_visuals.rs` owns F1 debug-visual toggling and Bevy gizmo drawing for player vectors, camera links, visual wind fields, and gameplay lift fields.
- `src/environment_visuals.rs` owns cinematic weather/light/fog animation, drifting cloud layers, updraft haze/ribbon/guide motion, wind-responsive prop motion, and fallback glider airflow trails.
- `src/eval_app_runtime.rs` owns the app eval-runtime facade and stable re-exports used by Bevy scheduling and tests.
- `src/eval_app_runtime/scene.rs` owns the `EvalScene` ECS query surface shared by app eval sampling and checkpoint metadata.
- `src/eval_app_runtime/metrics.rs` owns app eval frame-time and per-sample metric collection.
- `src/eval_app_runtime/finish.rs` owns eval frame finalization, summary write, app exit, final screenshot capture, and screenshot readiness waiting.
- `src/eval_app_runtime/semantics/mod.rs` and `src/eval_app_runtime/semantics/` own checkpoint screenshot sidecar writes, semantic route markers, semantic scene samples, viewport projection JSON, and marker/scene occlusion helpers.
- `src/eval_runtime.rs` owns CLI action parsing, eval run artifact paths, eval sample/summary file writing, and temporary output cleanup helpers.
- `src/bin/traversal_sim_eval.rs` owns the background-safe traversal simulation binary wiring and shared thresholds.
- `src/bin/traversal_sim_eval/cli.rs` owns simulation CLI parsing, artifact cleanup, NDJSON sample writes, and summary artifact writes.
- `src/bin/traversal_sim_eval/simulation.rs` owns scripted flight stepping, camera stepping, lift/wind field lookup, power-up collection, and sample capture.
- `src/bin/traversal_sim_eval/sample.rs` owns simulation sample/camera diagnostic structures and JSON serialization.
- `src/bin/traversal_sim_eval/state.rs` owns simulation-only objective and power-up state.
- `src/bin/traversal_sim_eval/metrics.rs` owns simulation-only metric state and constructor defaults.
- `src/bin/traversal_sim_eval/metrics/observe.rs` owns per-sample simulation metric accumulation.
- `src/bin/traversal_sim_eval/metrics/checks.rs` owns simulation-only pass/fail check assembly.
- `src/bin/traversal_sim_eval/metrics/checks/` owns focused simulation check groups for core route/camera gates, camera-strafe gates, and air-control gates.
- `src/bin/traversal_sim_eval/metrics/report.rs` owns simulation-only summary serialization and result packaging.
- `src/bin/traversal_sim_eval/metrics/util.rs` owns shared simulation-metric response, percentile, distance, and rear-diagonal helpers.
- `src/bin/traversal_sim_eval/tests.rs` owns unit coverage for simulation-only route, camera, air-control, and body-roll regression checks.
- `src/bin/terrain_export_audit/main.rs` owns terrain-export audit CLI wiring.
- `src/bin/terrain_export_audit/` owns terrain-export audit manifest validation, OBJ/material-weight artifact parsing, thresholds, check helpers, and unit coverage.
- `src/bin/visual_audit.rs` owns the non-golden screenshot visual-audit CLI wiring used by screenshot eval runs.
- `src/bin/visual_audit/analysis.rs` owns per-image audit sampling and check construction.
- `src/bin/visual_audit/image_metrics.rs` owns reusable image statistics, detail-tile scans, border-clipping math, and component counting.
- `src/bin/visual_audit/pixel_rules.rs` owns pixel/region classification rules for sky, scene, player, route-marker, distant-scene, cloud-layer, HUD, and clipping signals.
- `src/bin/visual_audit/report.rs` owns sequence-level visual-audit checks, including route-marker identity, terrain material coverage, distant-impostor span, foliage tile spread, cloud-layer span, and JSON report serialization.
- `src/bin/visual_audit/thresholds.rs` owns visual-audit thresholds.
- `src/bin/visual_audit/types.rs` owns visual-audit report structs.
- `src/bin/visual_audit/tests.rs` owns unit coverage for synthetic visual-audit image and sequence regressions.
- `src/bin/semantic_scene_audit/main.rs` owns semantic scene audit CLI wiring for screenshot marker sidecars.
- `src/bin/semantic_scene_audit/` owns checkpoint loading, semantic material pixel classification, report JSON/check assembly, thresholds, shared audit types, and unit coverage.
- `src/lib.rs` declares the reusable module surface.
- `src/generated_content.rs` owns the generated-content facade and stable mesh/material/texture re-exports.
- `src/generated_content/materials.rs` owns procedural PBR material construction for terrain, player, clouds, water, updrafts, and generated detail assets.
- `src/generated_content/textures.rs` owns deterministic procedural texture, material-map, occlusion/depth-map, color-mix, and noise helpers.
- `src/generated_content/island_meshes.rs` owns the generated-island mesh facade and stable re-exports shared by runtime spawning, app tests, and export audits.
- `src/generated_content/island_meshes/constants.rs` owns island mesh topology, biome-count, ground-cover-density, and test gate constants.
- `src/generated_content/island_meshes/shape.rs` owns deterministic island silhouette and surface-position helpers.
- `src/generated_content/island_meshes/palette.rs` owns island biome palettes, terrain material weights, terrain/rock vertex colors, terrain UVs, and generated detail material palette construction.
- `src/generated_content/island_meshes/terrain.rs` owns top-surface terrain mesh generation.
- `src/generated_content/island_meshes/body.rs` owns cliff, underside, and distant-impostor mesh generation.
- `src/generated_content/island_meshes/ground_cover.rs` owns generated ground-cover blade meshes.
- `src/generated_content/island_meshes/metrics.rs` owns generated island mesh inspection helpers used by tests and content diagnostics.
- `src/generated_content/island_meshes/normals.rs` owns smooth normal reconstruction for generated island meshes.
- `src/generated_content/detail_meshes.rs` owns the generated detail-mesh facade and stable re-exports for rocks, trees, clouds, route/landing/launch/pond landmarks, updraft ribbons, and glider airflow trails.
- `src/generated_content/detail_meshes/` owns the focused generated detail mesh domains: rock scatter, multi-ring tree trunks with branches/root flares, canopy detail cards, cloud lobes/wisps/filaments, route cairns, launch beacon crystals, landing markers, pond surfaces, visual effect ribbons, and shared card/lobe mesh helpers.
- `src/island_visuals.rs` owns the island-visual runtime facade and stable re-exports used by app setup and eval diagnostics.
- `src/island_visuals/types.rs` owns island visual catalog entries, stream state, LOD count metrics, and stream diagnostics.
- `src/island_visuals/queue.rs` owns terrain/body/impostor/ridge/beacon catalog construction for each sky island.
- `src/island_visuals/details.rs` owns generated ground cover, trees, stones, ponds, route cairns, launch/target decorations, and wind-responsive detail placement.
- `src/island_visuals/streaming.rs` owns island visual residency decisions, initial spawning, stream-window spawn/despawn, and stream diagnostics updates.
- `src/power_up_runtime.rs` owns aerial power-up collection state, visual guide spawning/animation, and one-time boost application.
- `src/asset_pipeline.rs` owns the visual asset pipeline facade and stable re-exports used by runtime, evals, and tests.
- `src/asset_pipeline/types.rs` owns visual asset kinds, residency classes, load/admission/preload/scene/animation state types, specs, policies, and metrics structs.
- `src/asset_pipeline/specs.rs` owns the declared glTF visual asset inventory, expected player animation clip names, residency slot counts, readiness floors, and default load policy while generated primitives remain the fallback.
- `src/asset_pipeline/policy.rs` owns deterministic visual asset load-admission ordering.
- `src/asset_pipeline/metrics.rs` owns file/load-state readiness metrics, recursive dependency preload metrics, scene-instance readiness metrics, and animation graph/player readiness metrics.
- `src/asset_pipeline/tests.rs` and `src/asset_pipeline/tests/` own asset inventory, policy, and metric bucket unit coverage.
- `src/movement.rs` owns the public movement module facade and stable re-exports used by runtime and eval code.
- `src/movement/types.rs` owns flight state, controller mode, input state, camera-relative facing, velocity, and tuning resources.
- `src/movement/integration.rs` owns launch/glide/dive stepping, ground/air acceleration, backward air braking, floor clamp, velocity limits, and mode transitions.
- `src/movement/orientation.rs` owns desired movement direction, body yaw/roll helpers, facing smoothing, and movement-response metrics.
- `src/movement/math.rs` owns shared movement vector helpers and smoothing math.
- `src/movement/tests.rs` and `src/movement/tests/` own movement fixtures plus integration, orientation, and math unit coverage.
- `src/environment.rs` owns finite visual wind/updraft field definitions, gameplay `LiftField` updraft volumes, collectible aerial power-up route definitions, lift/boost application, deterministic stream placement, and testable wind-sway visual motion math.
- `src/camera.rs` owns the public camera module facade and stable re-exports used by runtime and eval code.
- `src/camera/types.rs` owns deterministic camera data types: follow tuning/state, mouse-control state/input, orbit offsets, frames, and obstruction reports.
- `src/camera/follow.rs` owns camera follow placement, orbit application, movement-stable follow direction, and follow-direction smoothing.
- `src/camera/input.rs` owns mouse-delta to orbit yaw/pitch control math.
- `src/camera/metrics.rs` owns camera distance, clearance, target-angle, orbit-alignment, view-yaw, and pitch telemetry helpers.
- `src/camera/obstruction.rs` owns floor lifting, obstruction avoidance, and AABB segment-intersection helpers.
- `src/camera/math.rs` owns shared camera vector and angle helpers.
- `src/camera/tests.rs` and `src/camera/tests/` own camera follow, input, metric, and obstruction unit coverage.
- `src/world.rs` owns the world module facade, shared route/terrain constants, and stable re-exports.
- `src/world/route.rs` owns the authored sky-island route catalog, collision-aware route surfaces, landing target queries, and ground-contact resolution.
- `src/world/island.rs` owns sky-island data, deterministic terrain relief, footing/mesh-top helpers, horizontal containment, and per-island LOD/stream activation helpers.
- `src/world/streaming.rs` owns active chunk coordinates, stream activation state, and near/mid/far LOD metric types.
- `src/world/objectives.rs` owns route objective definitions, recovery-branch classification, fly-through completion, and landing objective completion.
- `src/world/surface.rs` owns ground-surface query results.
- `src/world/tests.rs` owns route, terrain relief, objective, streaming, LOD, and ground-contact unit coverage.
- `src/world_collision_runtime.rs` owns runtime world-asset collision proxies, player push-out resolution, collision diagnostics, and focused tests for generated prop collisions.
- `src/diagnostics.rs` owns pure helpers for frame-time and runtime metric formatting inputs.
- `src/eval.rs` owns the eval module surface and shared eval JSON helpers.
- `src/eval/accumulator.rs` owns accumulator state and module wiring for eval metric accumulation.
- `src/eval/accumulator/observe.rs` owns per-frame accumulator entry points and routes samples into focused observer groups.
- `src/eval/accumulator/initial_sample.rs` owns first-sample baseline initialization for min/max metrics.
- `src/eval/accumulator/camera.rs` owns camera framing, follow-error, yaw-drift, obstruction, and pitch/yaw input accumulation.
- `src/eval/accumulator/movement.rs` owns movement, body-heading, body-roll, lateral-response, air-brake, and mode-count accumulation.
- `src/eval/accumulator/world.rs` owns world-scale, LOD, visibility, environment-motion, stream-churn, and entity-count accumulation.
- `src/eval/accumulator/content.rs` owns terrain/body/detail/weather content-quality accumulation.
- `src/eval/accumulator/objectives.rs` owns route objective, power-up, landing-target, and lift-readability accumulation.
- `src/eval/accumulator/assets.rs` owns visual asset, scene-instance, preload, authored-fixture, and animation-readiness accumulation.
- `src/eval/accumulator/summary_report.rs` owns accumulator summary orchestration.
- `src/eval/accumulator/summary_report/derived.rs` owns frame-time percentiles, response-latency helpers, and derived summary values.
- `src/eval/accumulator/summary_report/checks.rs` owns pass/fail gate orchestration for baseline traversal, streaming, and camera checks.
- `src/eval/accumulator/summary_report/checks/` owns focused pass/fail gate groups for content substrate, asset/power-up readiness, and scenario-specific control checks.
- `src/eval/accumulator/summary_report/metrics_summary.rs` owns `EvalMetricsSummary` construction from accumulated and derived values.
- `src/eval/sample.rs` owns the eval sample facade and stable re-exports.
- `src/eval/sample/types.rs` owns eval frame sample structures, movement sample metrics, and objective progress state.
- `src/eval/sample/builders.rs` owns eval sample construction defaults and metric enrichment helpers.
- `src/eval/sample/json.rs` owns objective progress and NDJSON sample serialization.
- `src/eval/scenarios.rs` owns the public eval-scenario facade, scenario name constants, aliases, and `EvalScenario`/`EvalCheckpoint` types.
- `src/eval/scenarios/checkpoints.rs` owns checkpoint slices for all scripted eval scenarios.
- `src/eval/scenarios/input.rs` owns scripted movement and camera input timelines.
- `src/eval/scenarios/traversal_scenarios.rs` owns baseline, island launch/landing, updraft, branch recovery, and long-glide scenario definitions.
- `src/eval/scenarios/control_scenarios.rs` owns ground taxi, camera mouse/yaw/turn/strafe, and air-control scenario definitions.
- `src/eval/summary.rs` owns eval artifact, metrics-summary, check-result, and summary JSON serialization types.
- `src/eval/tests.rs` owns shared eval-test fixture builders and module wiring.
- `src/eval/tests/scenario_scripts.rs` owns unit coverage for scripted scenario inputs, camera scripts, route targets, and checkpoint definitions.
- `src/eval/tests/accumulator_controls.rs` owns accumulator coverage for frame-time, camera/movement drift, air-control response, braking, body heading, body roll, and grounded visual foot-gap gates.
- `src/eval/tests/content_gates.rs` owns accumulator coverage for asset readiness, terrain/body/impostor detail, generated vegetation/cloud shape gates, threshold serialization, and current passing baseline content shape.
- `src/eval/thresholds.rs` owns shared eval gate constants and the `EvalThresholds` scenario threshold contract.
- `src/animation.rs` owns primitive character part pose math, wing visibility/airflow state, and animation phase progression.

## Frame Flow

1. Bevy keyboard input is read in `player_runtime::fly_player`; runtime mouse input is read in `update_camera_control`.
2. Input is mapped into `movement::FlightInput`.
3. Movement uses the camera's horizontal forward/right vectors when available.
4. `movement::step_flight` produces the next position, velocity, and controller state.
5. Gameplay lift fields apply bounded upward acceleration when the player is airborne inside an active `LiftField`.
6. One-time aerial power-up gates apply capped forward/upward boosts when the airborne player intersects their authored route volumes.
7. Route ground contact resolves against the deterministic island surface, then `world_collision_runtime::resolve_world_collisions` pushes the player footprint horizontally out of spawned prop proxies and clears inward velocity.
8. Player orientation is smoothed toward desired camera-relative planar movement while airborne, with horizontal velocity as the fallback when no steering input is active.
9. Character pose phase advances from delta time.
10. `animation::part_pose` maps flight mode and velocity into visible body/glider poses, including speed-responsive wing flex.
11. Runtime wingtip airflow trails reuse the same pure wing-airflow strength helper so visual pressure follows gliding speed without changing gameplay forces.
12. Mouse deltas update `CameraControlState` yaw/pitch when the cursor is locked or right mouse is held.
13. `camera::movement_input_stable_follow_direction` ignores sideways/backward movement input when choosing the camera follow heading, `camera::update_follow_direction_state` smooths that heading independently from mouse orbit, and `camera::step_camera_with_direction` applies yaw/pitch orbit offsets once before smoothing position and rotation.
14. The camera avoids tagged obstruction volumes and is lifted above the active collision terrain surface when needed.
15. Island terrain is resident inside the active stream window, inactive or non-near chunks show layered distant impostors with measurable mesh and vertex-color complexity, route beacons remain resident for readability, and nonessential island detail, including dense curved-blade ground-cover meshes, is despawned outside the near LOD band.
16. Resident near-LOD tree and pond visuals apply wind-sway transforms from pure environment motion math. Ground cover uses deterministic curved blade clusters, trees use deterministic multi-ring tapered trunk meshes with branch mass/root flares plus overlapping canopy lobes with organic detail cards, and generated detail materials are selected from the same per-island biome family as the terrain, but this motion is visual-only and does not alter route surfaces, camera obstacles, or gameplay forces.
17. HUD text reports frame time, mode, speed, altitude, camera pitch/distance/framing/motion/orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, power-up visible/collected/active counts, visual asset slots, missing/deferred/queued/loading/loaded/failed load-state buckets, recursive dependency preload buckets, spawned/ready scene-instance buckets, visible authored world-fixture count, declared/ready animation clip counts, linked animation players, ready animation graphs, asset residency classes, visual wind-field count, lift-field count, world-collision proxy/resolution/push metrics, terrain surface vertex/color/material-weight/material-region/texture-detail/relief/cliff-band metrics, distant island impostor mesh/color-band metrics, procedural-vs-primitive island body counts, island body silhouette and mesh min/max complexity, ground-cover patch/blade/vertex metrics, generated detail biome-palette count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, detail counts, environment-motion count/offset, resident island visual count, stream entity churn, route beacons, cooldown, and launch readiness.
18. Debug gizmos draw player vectors, the camera line, visual wind/updraft streams, and gameplay lift-field bounds.
19. The render stack uses Bevy-native atmosphere, dynamic sun/fog/exposure weather, volumetric fog/light, bloom, filmic tonemapping, procedural PBR texture maps, reflective/transmissive water, emissive markers, irregular generated terrain rims, route-aligned ravine/channel incisions, terrace/shelf variation, fine microrelief, per-island biome vertex-color palettes and matching generated detail-material palettes, encoded terrain material-weight channels with derived material-region tinting, terrain-specific high-frequency albedo/roughness/occlusion/depth maps, stratified generated cliff/underside island body meshes, smooth generated island normals, layered color-banded distant island impostors, cinematic updraft haze/ribbons, wingtip airflow trails, five-layer drifting cloud banks with denser wisp-card edge detail and filament ribbons, layered high-cirrus cloud clusters, and wind-responsive near-LOD environment motion.
20. The default world is a 12-island archipelago with varied procedural terrain materials, bounded generated terrain incisions/shelves/microrelief, procedural island bodies, far-LOD layered island impostors, collidable route landmarks/trees/rocks, near/far gameplay updrafts, batched near-LOD ground cover, and a three-gate aerial boost route.
21. The `--export-terrain` CLI path runs without creating a Bevy window and writes per-island terrain/cliff/underside/impostor OBJ meshes, terrain material-weight CSV sidecars, and a manifest of mesh/material/texture-detail/texture-edge/height-band/normal-slope-band/impostor floors for offline inspection; `terrain_export_audit` is a thin directory-backed binary whose manifest, artifact, threshold, and check modules validate that manifest, OBJ topology/color counts, OBJ-derived height-band and normal-slope-band counts, material-weight sidecars, terrain texture-detail and texture-edge floors, terrain/body/impostor silhouette and vertical-mass floors, derived material-region coverage, and minimum base/transition/highland/exposed region distribution agree.
22. The `--export-visual-content` CLI path also runs without creating a Bevy window and writes generated ground-cover/tree/cloud/landmark OBJ meshes plus a manifest of vegetation, cloud, landmark, and detail-palette structural metrics; the visual export facade delegates report serialization, export orchestration, mesh metrics, vegetation metrics, cloud artifact writing, landmark artifact writing, and palette summaries to separate modules, and `visual_content_audit` validates artifact presence, OBJ vertex/face counts, blade density and height variance, multi-ring trunk mesh floors, trunk taper, branch reach/count, root-flare count, canopy lobe/detail-card structure, tree height/canopy-radius variation, cloud veil plus lobe/wisp-card/filament-ribbon/depth-span floors, route-cairn/launch-beacon/landing-marker/pond-surface count and mesh/span floors, and terrain/detail palette diversity.

Eval samples include desired body-yaw/heading error, desired-heading velocity alignment, movement input axes, lateral response speed, lateral input activity, camera distance, camera surface clearance, camera-to-player framing angle, per-frame camera step and rotation deltas, camera orbit alignment, camera follow-direction error, camera view yaw relative to the smoothed follow direction, camera world yaw, obstruction adjustment/hits, camera yaw/pitch offsets, route objective progress, `active_lift_fields`, power-up visibility/collection/effect counters, sky-island count, active chunk count, active island count, near/mid/far LOD island counts, visible/hidden island terrain counts, visible/hidden island impostor counts, minimum island impostor mesh vertex and color-band counts, visible/hidden island detail counts, visible route beacon count, weather cloud count, environment-motion visual count and offset, world-collision proxy count/resolved samples/max push, island terrain surface count, terrain mesh vertex floor, terrain vertex-color band floor, terrain material-weight band/channel/region floors, terrain texture-detail floor, terrain relief range, cliff color-band floor, procedural island body count, primitive island body count, island body silhouette complexity, island body mesh vertex floor and max, generated ground-cover density, generated tree/cloud/rock mesh complexity, generated landmark total/per-kind counts and mesh vertex floor, generated cloud-bank depth, generated cloud filament-ribbon detail count, generated detail biome-palette count, resident/catalog/hidden island visual counts, resident visual fraction, directional stream spawn/despawn churn, declared visual asset slot/readiness counts, Bevy scene load-state buckets including deferred admissions, recursive dependency preload counts, spawned/ready scene-instance buckets, visible authored world-fixture count, declared/ready animation clip counts, animation-player/graph readiness counts, asset residency class counts, and entity count. Eval summaries include frame-time avg/p95/p99/max telemetry, average/p95/max body-heading error, max yaw-error step, lateral response, separate right/left/rear-right/rear-left air-control response, rear-right/rear-left rearward response, total and planar air-brake recovery, yaw-oscillation, `lifted_samples`, power-up inventory/collection/effect checks, objective-progress checks, camera-control/framing/orbit/follow-direction/view-yaw/world-yaw/obstruction/jerk checks, checkpoint screenshot and marker-metadata artifact paths, sky-island/content-scale/world-collision checks, terrain mesh/color/material-weight/material-region/texture-detail/relief/cliff-band checks, distant-impostor mesh/color-band checks, procedural island body, primitive-body, island silhouette-complexity, and island body mesh-floor checks, ground-cover density checks, generated tree/cloud/rock shape checks, generated landmark total/per-kind/mesh checks, cloud-bank depth and filament-ribbon checks, detail biome-palette checks, asset-slot inventory, deferred-load, failed-load and dependency-preload checks, scene-instance readiness, visible authored world-fixture readiness, animation-readiness telemetry, streaming/LOD planning checks, stream-churn checks, resident-pressure checks, weather-cloud checks, environment-motion checks, resident visual/churn checks, visible-detail/beacon checks, and scene entity-count checks. Screenshot evals launched through `tools/eval.sh` add projection-backed checkpoint marker sidecars for known route beacons, route objectives, uncollected aerial power-ups, and terrain/foliage/cloud/distant-island scene samples, a `marker_projection_audit.json` for marker-colored pixels near projected visible markers, a `semantic_scene_audit.json` for material-family, scene-kind, and terrain material/biome variant diversity near projected non-occluded scene samples, and a non-golden `visual_audit.json` for image resolution, exposure, contrast, color variety, edge density, per-frame scene coverage, center-scene detail, scene detail tile frequency, flat low-detail scene-tile dominance, dominant low-detail scene-component dominance, per-frame player visibility, per-frame severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, HUD-text dominance, sequence-level route-marker readability/component identity/hue diversity, sequence-level distant horizon/impostor component readability/color-bucket/span identity, sequence-level terrain/material family diversity and terrain material coverage/color/tile spread, sequence-level foliage coverage/tile spread, sequence-level cloud-layer coverage/component/span identity, and sequence-level sky coverage; `terrain_surface` sidecars carry terrain material variant identity for projected diversity gates, while screenshot pixels are still matched with broad terrain-like rules rather than exact per-pixel biome classification. `updraft_route` verifies gameplay lift and objective progress, `branch_recovery_route` verifies the full branch objective sequence, `long_glide_visibility` verifies larger archipelago traversal, aerial boost collection, and content scale, `camera_mouse_control` verifies mouse X/Y behavior, `camera_yaw_stability` verifies that small yaw input does not drift after input stops, `camera_strafe_stability` verifies that `A`/`D` movement does not auto-orbit the camera by bounding view-yaw/world-yaw drift while proving right and left strafe response, `camera_turn_stability` verifies rapid airborne turn and air-brake camera stability, and `air_control_response` verifies diagonal/lateral/rear-right/rear-left/brake/recovery air control without movement-driven camera orbit or view-yaw drift.

## Core Invariants

- Movement math must stay testable outside a Bevy window.
- `E` launch is ground-gated unless a future launch-source mechanic explicitly changes that.
- Glider traversal descends without wind/updraft/launch-source help.
- Visual `WindField` volumes do not directly move the player.
- Wind-responsive environment motion is visual-only and must not move collision surfaces, camera obstruction bounds, or gameplay lift/power-up volumes.
- Gameplay `LiftField` volumes can move the player upward, but only through explicit lift application rules.
- Aerial power-up gates are one-time boosts; they must not grant repeatable midair launch spam.
- If crosswind ever affects movement, force application rules belong in reusable/testable code, not directly in ECS systems.
- Camera follow direction should use only mostly-forward horizontal travel for automatic follow updates; sideways/backward movement can move and turn Nau, but must not become camera orbit.
- Runtime movement should stay camera-relative unless a scenario deliberately requests character-relative controls.
- Camera orbit input should keep yaw and pitch independently measurable in evals while keeping the player focus near the camera centerline.
- Camera should stay above the active route surface and avoid tagged obstruction volumes between the player focus and camera boom.
- Sky-island collision queries and visible terrain meshes should use the same deterministic relief function, with launch and landing centers anchored to their authored route heights.
- Active chunk counters drive resident terrain/detail/impostor entities, and stream diagnostics record hidden/resident/catalog visual pressure plus directional spawn/despawn churn until a future branch adds asset streaming.
- Declared visual asset slots should stay measurable even while missing or deferred assets fall back to generated placeholders, the default policy must admit every current fixture, any queued Bevy scene handle must report a load state with deferred and failed loads gated in evals, recursive dependency preload readiness must be tracked separately from top-level scene load state, scene-instance readiness must be tracked separately from file presence or asset load state, visible authored world fixtures must remain counted separately from mere scene readiness, `extras.nau` fixture metadata must match the registry kind/label/residency/license contract, and named animation clip/graph readiness must be tracked separately from scene readiness.
- Screenshot eval windows must disable debug gizmos and use opaque surface composition so checkpoint artifacts show the normal scene and transparent weather/lift visuals cannot reveal unrelated desktop content.
- Checkpoint marker sidecars must distinguish in-viewport route markers and semantic scene samples that are visible from projected points that are occluded, offscreen, or behind the camera before projected-pixel audits decide whether marker-colored or material-like pixels are required, and semantic scene audits must preserve material-family coverage, sample-kind coverage, and aggregate `terrain_surface` material/biome variant diversity so foliage-colored terrain, cloud-colored impostors, or single-variant terrain cannot satisfy the wrong world-semantic signal. This is a projected-sample diversity gate, not a production material classifier.
- Camera, animation, and HUD should run after movement.
- Visual polish should prefer Bevy-native rendering components and generated assets before custom shaders, new render passes, or raw platform code.
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

- Keep primitives for fast iteration, but use procedural materials, animated mesh effects, and Bevy-native atmosphere/weather as the interim visual layer.
- Keep the declared glTF asset inventory, Bevy scene load-state buckets, recursive dependency preload counters, spawned/ready scene-instance buckets, and named player animation graph readiness wired into runtime/eval diagnostics before real imported scenes are available.
- Expected drop-in paths are `assets/models/player/player.gltf`, `assets/models/player/glider.gltf`, `assets/models/world/island_terrain.gltf`, `assets/models/world/foliage.gltf`, `assets/models/world/rocks.gltf`, `assets/models/world/water.gltf`, `assets/models/world/route_markers.gltf`, `assets/models/world/weather_layers.gltf`, and `assets/models/world/island_impostors.gltf`.
- Every declared slot currently has a self-authored fixture under `assets/models/` so Bevy `SceneRoot` load, spawn, and ready lifecycle stays exercised in every eval run across all residency classes. The player fixture is a faceted authored scene with tapered body meshes, rounded head geometry, readable face mask and eye lenses, belt hardware, gauntlet cuffs, knee guards, boots, shoulder guards, scarf/tunic accents, chest focus geometry, the declared named animation clips, registry-aligned `extras.nau` metadata, and UV-ready mesh primitives, generated by `tools/generate_player_fixture.mjs`; it is not a rigged production character. The visible glider fixture is generated by `tools/generate_glider_fixture.mjs` and uses separate cloth panels, seams, frame rods, tethers, handles, normals, UVs, registry metadata, and multiple PBR materials so the authored glider reads as more than a triangle placeholder while still remaining self-authored. The world fixtures are generated by `tools/generate_world_asset_fixtures.mjs` with normals, UVs, registry metadata, multiple PBR materials, and distinct mesh parts for terrain relief/cliffs/erosion/path stones, rooted/branched/lobed foliage with fern and moss cards, rocks with fracture/quartz/rust/shale details, pond/ripple/reed/lily/specular kits, route-marker gates/glyphs/pennants, weather layers with haze pockets and filament wisps, and distant island impostors with tree silhouettes, waterfall veils, and broken cliff shelves. The player and glider slots spawn `SceneRoot` children under the player if their files exist; non-player fixture scenes now spawn visibly as decorative route-anchored world fixtures once their scene instances are ready, while generated gameplay visuals remain the collision/traversal authority until real assets replace them deliberately. The primitive body/wings remain hidden only after player/glider scene instances report `SceneInstanceReady`; failed or missing files leave generated placeholders visible. `SceneInstanceReady` observers only mark scene readiness; a retryable animation-link system discovers nested `AnimationPlayer`s, validates declared glTF clip names, attaches an `AnimationGraph`/`AnimationTransitions` pair when all clips are present, and drives idle, jog, launch, glide, air-brake, and land clips from Nau's current flight state.
- `asset_fixture_audit` is a repo-native structural gate for this fixture inventory. `tools/eval.sh` writes `asset_fixture_audit.json` by default and fails if any declared fixture loses provenance, registry-aligned `extras.nau` schema/kind/label/residency/license metadata, required semantic component names, mesh/material/vertex/triangle floors, normals, UVs, required blend materials, or the declared player clip names. The world-fixture floor now explicitly requires the richer terrain erosion/path, foliage root/fern/moss, water lily/specular, rock rust/shale, route glyph/pebble, cloud haze/filament, and distant waterfall/broken-cliff fragments.
- Add debug visuals before complex behavior.
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
