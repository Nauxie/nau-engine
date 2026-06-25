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

- `src/main.rs` owns Bevy app setup, scene spawning, high-resolution vertex-colored generated island surface meshes, island stream-window visibility, island detail LOD, wind-responsive environment visual components, procedural material/weather setup, Bevy render-stack wiring, input mapping, ECS queries, and HUD sampling.
- `src/lib.rs` owns reusable and testable logic.
- `movement` owns flight state, input state, tuning, launch/glide/dive integration, floor clamp, velocity limits, and facing smoothing.
- `environment` owns finite visual wind/updraft field definitions, gameplay `LiftField` updraft volumes, collectible aerial power-up route definitions, lift/boost application, deterministic stream placement, and testable wind-sway visual motion math.
- `world` owns collision-aware route surfaces, sky-island definitions, deterministic island relief, landing target queries, active chunk counters, stream-window classification, and near/mid/far LOD band classification.
- `camera` owns camera follow math, orbit yaw/pitch control math, movement-stable horizontal follow direction, obstruction avoidance, and ground-clearance helpers.
- `diagnostics` owns pure helpers for frame-time and runtime metric formatting inputs.
- `animation` owns primitive character part pose math, wing visibility/airflow state, and animation phase progression.
- `asset_pipeline` owns the declared glTF visual asset inventory, expected player animation clip names, residency classes, file/load-state readiness metrics, recursive dependency preload metrics, scene-instance readiness metrics, visible authored world-fixture metrics, and animation graph/player readiness metrics while generated primitives remain the fallback.

## Frame Flow

1. Bevy keyboard input is read in `fly_player`; runtime mouse input is read in `update_camera_control`.
2. Input is mapped into `movement::FlightInput`.
3. Movement uses the camera's horizontal forward/right vectors when available.
4. `movement::step_flight` produces the next position, velocity, and controller state.
5. Gameplay lift fields apply bounded upward acceleration when the player is airborne inside an active `LiftField`.
6. One-time aerial power-up gates apply capped forward/upward boosts when the airborne player intersects their authored route volumes.
7. Player orientation is smoothed toward desired camera-relative planar movement while airborne, with horizontal velocity as the fallback when no steering input is active.
8. Character pose phase advances from delta time.
9. `animation::part_pose` maps flight mode and velocity into visible body/glider poses, including speed-responsive wing flex.
10. Runtime wingtip airflow trails reuse the same pure wing-airflow strength helper so visual pressure follows gliding speed without changing gameplay forces.
11. Mouse deltas update `CameraControlState` yaw/pitch when the cursor is locked or right mouse is held.
12. `camera::movement_input_stable_follow_direction` ignores sideways/backward movement input when choosing the camera follow heading, `camera::update_follow_direction_state` smooths that heading independently from mouse orbit, and `camera::step_camera_with_direction` applies yaw/pitch orbit offsets once before smoothing position and rotation.
13. The camera avoids tagged obstruction volumes and is lifted above the active collision terrain surface when needed.
14. Island terrain is resident inside the active stream window, inactive or non-near chunks show layered distant impostors with measurable mesh and vertex-color complexity, route beacons remain resident for readability, and nonessential island detail, including dense curved-blade ground-cover meshes, is despawned outside the near LOD band.
15. Resident near-LOD tree and pond visuals apply wind-sway transforms from pure environment motion math. Ground cover uses deterministic curved blade clusters, trees use deterministic tapered trunk meshes and overlapping canopy lobes with organic detail cards, and generated detail materials are selected from the same per-island biome family as the terrain, but this motion is visual-only and does not alter route surfaces, camera obstacles, or gameplay forces.
16. HUD text reports frame time, mode, speed, altitude, camera pitch/distance/framing/motion/orbit alignment, obstruction adjustment, mouse yaw/pitch offsets, velocity, power-up visible/collected/active counts, visual asset slots, missing/queued/loading/loaded/failed load-state buckets, recursive dependency preload buckets, spawned/ready scene-instance buckets, visible authored world-fixture count, declared/ready animation clip counts, linked animation players, ready animation graphs, asset residency classes, visual wind-field count, lift-field count, terrain surface vertex/color/material-weight/material-region/texture-detail/relief/cliff-band metrics, distant island impostor mesh/color-band metrics, procedural-vs-primitive island body counts, island body silhouette and mesh min/max complexity, ground-cover patch/blade/vertex metrics, generated detail biome-palette count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, detail counts, environment-motion count/offset, resident island visual count, stream entity churn, route beacons, cooldown, and launch readiness.
17. Debug gizmos draw player vectors, the camera line, visual wind/updraft streams, and gameplay lift-field bounds.
18. The render stack uses Bevy-native atmosphere, dynamic sun/fog/exposure weather, volumetric fog/light, bloom, filmic tonemapping, procedural PBR texture maps, reflective/transmissive water, emissive markers, irregular generated terrain rims, per-island biome vertex-color palettes and matching generated detail-material palettes, encoded terrain material-weight channels with derived material-region tinting, terrain-specific high-frequency albedo/roughness/occlusion/depth maps, stratified generated cliff/underside island body meshes, smooth generated island normals, layered color-banded distant island impostors, cinematic updraft haze/ribbons, wingtip airflow trails, vertically layered drifting cloud banks with wisp-card edge detail, layered high-cirrus cloud clusters, and wind-responsive near-LOD environment motion.
19. The default world is a 12-island archipelago with varied procedural terrain materials, procedural island bodies, far-LOD layered island impostors, route cairns, near/far gameplay updrafts, batched near-LOD ground cover, and a three-gate aerial boost route.
20. The `--export-terrain` CLI path runs without creating a Bevy window and writes per-island terrain/cliff/underside/impostor OBJ meshes, terrain material-weight CSV sidecars, and a manifest of mesh/material/texture-detail/texture-edge/impostor floors for offline inspection; `terrain_export_audit` validates that manifest, OBJ topology/color counts, material-weight sidecars, terrain texture-detail and texture-edge floors, terrain/body/impostor silhouette and vertical-mass floors, derived material-region coverage, and minimum base/transition/highland/exposed region distribution agree.
21. The `--export-visual-content` CLI path also runs without creating a Bevy window and writes generated ground-cover/tree/cloud OBJ meshes plus a manifest of vegetation, cloud, and detail-palette structural metrics; `visual_content_audit` validates artifact presence, OBJ vertex/face counts, blade density and height variance, trunk taper, branch reach, canopy lobe/detail-card structure, cloud lobe/wisp-card/depth floors, and terrain/detail palette diversity.

Eval samples include desired body-yaw/heading error, desired-heading velocity alignment, movement input axes, lateral response speed, lateral input activity, camera distance, camera surface clearance, camera-to-player framing angle, per-frame camera step and rotation deltas, camera orbit alignment, camera follow-direction error, camera view yaw relative to the smoothed follow direction, camera world yaw, obstruction adjustment/hits, camera yaw/pitch offsets, route objective progress, `active_lift_fields`, power-up visibility/collection/effect counters, sky-island count, active chunk count, active island count, near/mid/far LOD island counts, visible/hidden island terrain counts, visible/hidden island impostor counts, minimum island impostor mesh vertex and color-band counts, visible/hidden island detail counts, visible route beacon count, weather cloud count, environment-motion visual count and offset, island terrain surface count, terrain mesh vertex floor, terrain vertex-color band floor, terrain material-weight band/channel/region floors, terrain texture-detail floor, terrain relief range, cliff color-band floor, procedural island body count, primitive island body count, island body silhouette complexity, island body mesh vertex floor and max, generated ground-cover density, generated tree/cloud/rock mesh complexity, generated cloud-bank depth, generated detail biome-palette count, resident/catalog/hidden island visual counts, resident visual fraction, directional stream spawn/despawn churn, declared visual asset slot/readiness counts, Bevy scene load-state buckets, recursive dependency preload counts, spawned/ready scene-instance buckets, visible authored world-fixture count, declared/ready animation clip counts, animation-player/graph readiness counts, asset residency class counts, and entity count. Eval summaries include frame-time avg/p95/p99/max telemetry, average/p95/max body-heading error, max yaw-error step, lateral response, separate right/left/rear-right/rear-left air-control response, rear-right/rear-left rearward response, total and planar air-brake recovery, yaw-oscillation, `lifted_samples`, power-up inventory/collection/effect checks, objective-progress checks, camera-control/framing/orbit/follow-direction/view-yaw/world-yaw/obstruction/jerk checks, checkpoint screenshot and marker-metadata artifact paths, sky-island/content-scale checks, terrain mesh/color/material-weight/material-region/texture-detail/relief/cliff-band checks, distant-impostor mesh/color-band checks, procedural island body, primitive-body, island silhouette-complexity, and island body mesh-floor checks, ground-cover density checks, generated tree/cloud/rock shape checks, cloud-bank depth checks, detail biome-palette checks, asset-slot inventory, failed-load and dependency-preload checks, scene-instance readiness, visible authored world-fixture readiness, animation-readiness telemetry, streaming/LOD planning checks, stream-churn checks, resident-pressure checks, weather-cloud checks, environment-motion checks, resident visual/churn checks, visible-detail/beacon checks, and scene entity-count checks. Screenshot evals launched through `tools/eval.sh` add projection-backed checkpoint marker sidecars for known route beacons, route objectives, and uncollected aerial power-ups, a `marker_projection_audit.json` for marker-colored pixels near projected visible markers, and a non-golden `visual_audit.json` for image resolution, exposure, contrast, color variety, edge density, per-frame scene coverage, center-scene detail, scene detail tile frequency, flat low-detail scene-tile dominance, per-frame player visibility, per-frame severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, HUD-text dominance, sequence-level route-marker readability/component identity/hue diversity, sequence-level distant horizon/impostor component readability and color-bucket identity, sequence-level terrain/material family diversity, sequence-level foliage coverage, sequence-level cloud-layer coverage/component identity, and sequence-level sky coverage; `updraft_route` verifies gameplay lift and objective progress, `branch_recovery_route` verifies the full branch objective sequence, `long_glide_visibility` verifies larger archipelago traversal, aerial boost collection, and content scale, `camera_mouse_control` verifies mouse X/Y behavior, `camera_yaw_stability` verifies that small yaw input does not drift after input stops, `camera_strafe_stability` verifies that `A`/`D` movement does not auto-orbit the camera by bounding view-yaw/world-yaw drift while proving right and left strafe response, `camera_turn_stability` verifies rapid airborne turn and air-brake camera stability, and `air_control_response` verifies diagonal/lateral/rear-right/rear-left/brake/recovery air control without movement-driven camera orbit or view-yaw drift.

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
- Declared visual asset slots should stay measurable even while missing assets fall back to generated placeholders, any queued Bevy scene handle must report a load state with failed loads gated in evals, recursive dependency preload readiness must be tracked separately from top-level scene load state, scene-instance readiness must be tracked separately from file presence or asset load state, visible authored world fixtures must remain counted separately from mere scene readiness, and named animation clip/graph readiness must be tracked separately from scene readiness.
- Screenshot eval windows must disable debug gizmos and use opaque surface composition so checkpoint artifacts show the normal scene and transparent weather/lift visuals cannot reveal unrelated desktop content.
- Checkpoint marker sidecars must distinguish in-viewport markers that are visible from markers that are occluded, offscreen, or behind the camera before projected-pixel audits decide whether marker-colored pixels are required.
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
- Every declared slot currently has a self-authored fixture under `assets/models/` so Bevy `SceneRoot` load, spawn, and ready lifecycle stays exercised in every eval run across all residency classes. The player fixture is a faceted authored scene with tapered body meshes, rounded head geometry, boots, shoulder guards, scarf/tunic accents, chest focus geometry, the declared named animation clips, and UV-ready mesh primitives, generated by `tools/generate_player_fixture.mjs`; it is not a rigged production character. The visible glider fixture is generated by `tools/generate_glider_fixture.mjs` and uses separate cloth panels, seams, frame rods, tethers, handles, normals, UVs, and multiple PBR materials so the authored glider reads as more than a triangle placeholder while still remaining self-authored. The world fixtures are generated by `tools/generate_world_asset_fixtures.mjs` with normals, UVs, multiple PBR materials, and distinct mesh parts for terrain relief/cliffs, branched/lobed foliage, rocks, pond/ripple/reed kits, route-marker gates, weather layers, and distant island impostors. The player and glider slots spawn `SceneRoot` children under the player if their files exist; non-player fixture scenes now spawn visibly as decorative route-anchored world fixtures once their scene instances are ready, while generated gameplay visuals remain the collision/traversal authority until real assets replace them deliberately. The primitive body/wings remain hidden only after player/glider scene instances report `SceneInstanceReady`; failed or missing files leave generated placeholders visible. `SceneInstanceReady` observers only mark scene readiness; a retryable animation-link system discovers nested `AnimationPlayer`s, validates declared glTF clip names, attaches an `AnimationGraph`/`AnimationTransitions` pair when all clips are present, and drives idle, jog, launch, glide, air-brake, and land clips from Nau's current flight state.
- `asset_fixture_audit` is a repo-native structural gate for this fixture inventory. `tools/eval.sh` writes `asset_fixture_audit.json` by default and fails if any declared fixture loses provenance, required semantic component names, mesh/material/vertex/triangle floors, normals, UVs, required blend materials, or the declared player clip names.
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
