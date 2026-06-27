# Eval Harness

The eval harness exists to make traversal work measurable before the world gets larger. It should answer three questions on every iteration:

- Did the playable route still run end to end?
- What changed in movement, camera, visibility, and runtime scale?
- Which artifacts should a human or agent inspect next?

The harness is repo-native. The game owns deterministic input, state collection, screenshot capture, and pass/fail checks. Shell scripts and future agent orchestration should only run the app, collect artifacts, and interpret the report.

## Current Command

Run the baseline route as a background metric eval:

```sh
./tools/eval.sh
```

Run the island launch-to-landing route:

```sh
./tools/eval.sh island_launch_to_landing target/eval/island_launch_to_landing
```

Run the ground-control route:

```sh
./tools/eval.sh ground_taxi_control target/eval/ground_taxi_control
```

Run the app-only world-collision contact route:

```sh
./tools/eval.sh world_collision_contact target/eval/world_collision_contact
```

Run the app-only terrain-rim collision contact route:

```sh
./tools/eval.sh terrain_rim_collision_contact target/eval/terrain_rim_collision_contact
```

Run the gameplay updraft route:

```sh
./tools/eval.sh updraft_route target/eval/updraft_route
```

Run the branch recovery landing route:

```sh
./tools/eval.sh branch_recovery_route target/eval/branch_recovery_route
```

Run the long-glide archipelago route:

```sh
./tools/eval.sh long_glide_visibility target/eval/long_glide_visibility
```

Run the mouse-camera control route:

```sh
./tools/eval.sh camera_mouse_control target/eval/camera_mouse_control
```

Run the small-yaw no-drift camera route:

```sh
./tools/eval.sh camera_yaw_stability target/eval/camera_yaw_stability
```

Run the airborne turn and air-brake camera stability route:

```sh
./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
```

Run the airborne planar air-control response route:

```sh
./tools/eval.sh air_control_response target/eval/air_control_response
```

Run the lateral strafe camera stability route:

```sh
./tools/eval.sh camera_strafe_stability target/eval/camera_strafe_stability
```

Run every simulation-supported scripted scenario through the no-window simulation path:

```sh
./tools/eval_sim_suite.sh target/eval/sim_suite
```

Request screenshot artifacts explicitly:

```sh
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
```

Screenshot artifact validation in `tools/eval.sh` requires `jq` so the script can read artifact paths from `summary.json`.

Collect screenshots without running the image audit:

```sh
NAU_EVAL_SCREENSHOT=1 NAU_EVAL_VISUAL_AUDIT=0 ./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
```

Run the app directly with screenshot capture:

```sh
cargo run -- --eval baseline_route --eval-output target/eval/baseline_route
```

Run the app directly without screenshot capture. This hides the native window so metric-only loops do not steal focus:

```sh
cargo run -- --eval baseline_route --eval-output target/eval/baseline_route --eval-no-screenshot
```

Export the generated island substrate for offline terrain/material inspection without creating a native window:

```sh
./tools/terrain_export.sh target/terrain_export
```

The export writes `manifest.json`, per-island terrain/cliff/underside/impostor OBJ meshes, `*_terrain_material_weights.csv` sidecars, and `audit.json`. The audit validates schema, mesh/material/texture-detail/texture-edge/impostor floors, artifact presence, OBJ vertex/face/color counts, terrain-archetype diversity, terrain/body silhouette radius variation, island-body vertical range, impostor vertical range and horizontal radius variation, terrain material-weight CSV rows/bands/channels, derived material-region coverage, per-island base/transition/highland/exposed presence floors, and aggregate archipelago material-region distribution. This is still an offline structural gate rather than a final art-quality score.

Export the generated vegetation/cloud/detail substrate for offline shape inspection without creating a native window:

```sh
./tools/visual_content_export.sh target/visual_content_export
```

The export writes `manifest.json`, generated ground-cover/tree/cloud/landmark OBJ meshes, and `audit.json`. The audit validates schema, artifact presence, OBJ vertex/face counts, total generated mesh scale, ground-cover patch/blade density and blade-height variance, multi-ring trunk mesh floors, trunk taper, branch reach/count, root-flare count, canopy lobe/detail-card structure, tree height/canopy-radius variation, cloud veil count, cloud lobe/wisp-card/filament-ribbon counts, cloud-bank depth/span, route-cairn/launch-beacon/landing-marker/pond-surface count and mesh/span floors, obstruction-spire count/mesh/vertical-span and height/radius/normal-slope band floors, and terrain/detail palette diversity. This complements screenshot audit coverage by making primitive vegetation/cloud/route-prop/camera-blocker regressions fail in a deterministic background-safe path.

## Current Scenarios

`baseline_route` is a deterministic scripted traversal smoke test:

- fixed `1 / 60` movement timestep
- fixed spawn on the launch sky island
- launch on frame 1
- forward input after startup
- glider deployment after launch
- one short dive segment
- left and right steering segments
- metrics sampled every 10 frames and at the final frame
- summary written after 420 frames
- optional final screenshot written as `final.png` when screenshots are requested
- optional fixed camera checkpoint screenshots written under `checkpoints/` when screenshots are requested

`island_launch_to_landing` is the first actual route-completion eval:

- fixed spawn on the launch island
- scripted launch, glide, turn, and dive
- route passes multiple visible sky islands
- the target island must be reached
- final target distance must stay under threshold
- at least one sampled frame must be grounded on the landing target

`ground_taxi_control` is the manual-control regression test:

- fixed spawn on the launch island
- no launch, glide, or dive input
- scripted `W`, `A`/`D`, and `S`-style ground movement
- the player must stay grounded
- final horizontal displacement and max speed must prove WASD motion is not being damped away

`world_collision_contact` is the app-only generated-asset collision regression test:

- fixed spawn on the launch island
- no launch, glide, or dive input
- scripted grounded reverse taxi into the launch-mesa tree proxy
- the player must stay grounded
- at least 10 sampled frames must resolve world collision, with at least 0.04 m max push, so generated obstacle proxies must affect player movement instead of only existing as counted entities

`terrain_rim_collision_contact` is the app-only terrain-rim collision regression test:

- fixed spawn on the launch island
- no launch, glide, or dive input
- scripted grounded forward taxi into the launch-mesa rim proxy
- the player must stay grounded
- at least 10 sampled frames must resolve terrain-rim collision, with at least 0.04 m max rim push, so near-LOD island rims cannot disappear or become ghost edges

`updraft_route` is the first traversal power-up eval:

- fixed spawn on the launch island
- scripted launch, glide, and steering into the gameplay lift field
- the route must register active lift samples
- every sampled active lift frame must also register a paired visible updraft field so gameplay lift cannot silently drift away from its readable signal
- the route must sample bounded horizontal wind response from the visual `WindField` while vertical climb remains `LiftField`-driven
- neutral gliding frames inside crosswind load must produce readable body/glider reaction, measured by normalized lateral load, pose lean, and authored glider response gates
- max altitude must exceed the normal route ceiling

`branch_recovery_route` is the first alternate-route recovery eval:

- fixed spawn on the launch island
- scripted launch, glide, steering, late dive, and air-brake into `sunlit terrace`
- the scenario tracks `sunlit terrace` as its target island instead of the primary landing garden
- the route must use readable gameplay lift, preserve larger-world content/LOD signals, and end grounded on the branch island
- final target distance and grounded target landing samples must pass against the named branch island

`long_glide_visibility` is the larger-archipelago traversal eval:

- fixed spawn on the launch island
- scripted launch, glide, and wide steering across the expanded island chain
- the route must cover hundreds of meters without leaving camera thresholds
- the route must keep altitude high enough for a sustained glide across distant islands
- sky-island, active chunk, LOD bucket, and entity-count thresholds must catch accidental content or scale-signal collapse

`camera_mouse_control` is the camera-input regression test:

- fixed spawn on the launch island
- no movement input, so camera regressions are not hidden by traversal
- scripted mouse X input must produce yaw offset
- scripted mouse Y input must produce both upward and downward pitch offsets
- final input settles closer to level so the screenshot artifact remains useful
- camera surface clearance must stay above the active ground surface
- camera-to-player framing angle must stay below threshold so pitch cannot push the player out of frame
- launch-side obstruction avoidance must produce a measurable camera adjustment
- camera orbit alignment must stay below threshold so yaw is not reapplied every frame

`camera_yaw_stability` is the isolated no-drift mouse regression test:

- fixed spawn on the launch island
- no movement input, so yaw drift cannot be confused with player heading changes
- a small scripted mouse X impulse must produce a measurable yaw offset
- input then stops for multiple seconds
- camera orbit alignment must remain below threshold after the yaw impulse

`camera_turn_stability` is the airborne camera-feel regression test:

- scripted launch, glide deployment, forward travel through alternating left/right air turns, and a late backward air-brake segment
- camera step distance and rotation delta must remain under thresholds during rapid heading changes
- the route must keep gliding samples and traversal distance high enough to avoid a no-op pass

`air_control_response` is the airborne movement-feel regression test:

- scripted launch, glide deployment, diagonal air steering, pure right/left air steering, backward-right and backward-left glide steering, backward braking, a short dive, and forward recovery
- no scripted camera input, so camera orbit yaw offset must remain zero while `A`/`D` move Nau
- actual camera rotation delta must stay small during the movement-only route
- average and p95 camera follow-direction error must stay bounded so a stable camera cannot hide a stale follow target
- camera view-yaw and world-yaw drift must stay bounded so movement input cannot quietly rotate the camera
- average, p95, and max desired body heading error plus desired-heading velocity alignment must stay within thresholds
- desired-travel heading error must collect enough finite samples in aggregate, right, left, backward-right, and backward-left buckets and stay bounded so lateral/rear-diagonal input cannot pass by accelerating sideways while the intended travel vector points elsewhere
- lateral and backward-diagonal body/travel heading error must collect finite right, left, backward-right, and backward-left samples and stay bounded after each direction-specific response window and current-frame lateral response threshold so Nau cannot visibly fly sideways while presenting the wrong body direction; the sample field is finite only for airborne/gliding lateral-input samples at or above the 6.0 m/s planar speed floor
- right, left, rear-right, and rear-left lateral input must each produce measurable response within the response-latency threshold
- rear-right and rear-left samples must also build a rearward component, exposed as `max_backward_right_rear_response_mps` and `max_backward_left_rear_response_mps`, so pure sideways drift cannot satisfy the diagonal-control gate
- backward input must produce measurable total and planar air-brake speed drop, and the final forward segment must recover forward alignment
- max body-yaw error step and same-intent oscillation count must remain bounded so steering hold does not become spin or wobble while deliberate input reversals are measured as fresh intents
- body roll must bank in both lateral directions and stay smooth across sampled input transitions
- lateral airborne input must also produce explicit readable `air_turn` key-pose samples so turn readability cannot pass accidentally through dive or air-brake coverage
- signed pose lean must agree with input direction, so right input must produce readable right lean and left input must produce readable left lean instead of one good unsigned lean sample satisfying both directions
- visible generated/authored pose parts must produce at least one finite temporal sample during key poses, and key-pose part rotation/translation deltas must stay bounded so readable pose coverage cannot hide a one-frame limb snap; focused pose tests additionally cover pressure-scaled dive flattening and leg trail so shallow and high-sink dives do not collapse to the same fixed key pose; during smooth authored pose transitions, readability may score against the immediately previous key-pose shape, and a brief in-between key-pose sample window of at most five frames may be accepted only when it is already at least 0.65 readable and pose-part deltas stay within 60 degrees/0.15 m
- current gates require lateral response within 0.20 seconds after reaching at least 6 m/s, at least 22 m/s directional right/left lateral response, at least 10 m/s rear-right and rear-left lateral response, at least 10 m/s rear-right and rear-left rearward response, at least 12 m/s of total and planar air-brake speed drop, at least 14 m/s of post-brake forward recovery, at least 8 degrees of left and right body-bank response, at least 8 degrees of signed right and left pose lean, at least four readable `air_turn` pose samples with at least four right and four left `air_turn` samples, at least one visible pose temporal sample, pose-part rotation/translation deltas at or under 120 degrees/0.55 m, sampled body-roll steps at or under 12 degrees, camera view-yaw and world-yaw drift at or under 2 degrees, p95 body-heading error at or under 20 degrees, at least eight desired-travel heading samples with p95/max error at or under 8/32 degrees, at least four right, left, backward-right, and backward-left desired-travel samples, at least four right and left lateral body/travel heading samples plus at least four backward-right and backward-left diagonal body/travel heading samples, lateral body/travel heading p95/max error at or under 8/20 degrees, backward-diagonal body/travel heading p95/max error at or under 8/12 degrees, max body-heading error at or under 34 degrees, max same-intent yaw-step error at or under 28 degrees, and zero same-intent yaw oscillations

`camera_strafe_stability` is the lateral-movement camera regression test:

- fixed spawn on the launch island
- no mouse input, launch, glide, or dive
- scripted `D` and `A` segments exercise lateral ground motion
- right and left lateral movement must each reach at least 8 m/s so camera-stability gates cannot pass on a no-op strafe
- camera view yaw must stay near the starting heading so strafe velocity cannot auto-orbit the camera
- max speed and grounded samples must prove the route was not a no-op

The baseline route remains a fast smoke test. The island route is the stronger signal for traversal/content regressions. The ground taxi route guards the pre-launch controls that airborne evals can miss. The updraft route proves the first gameplay power-up remains measurable and visually signaled. The branch recovery route proves a named alternate landing island can be targeted, reached, and validated without changing the primary landing-garden route. The long-glide route guards the first larger-map slice before real despawn, asset streaming, or richer impostor work exists. The mouse-camera route guards the control surface that manual play will feel immediately but movement-only evals miss. The yaw-stability route guards against persistent mouse yaw being fed back into the camera every frame. The strafe-stability route guards against `A`/`D` movement being treated as camera orbit input. The turn-stability route guards rapid airborne direction changes and backward air braking. The air-control route guards the actual flight-feel response that manual play exposed as jank. The pose-state route guards a compact full-pose chain from launch through landing plus post-touchdown walk/run so traversal-pose coverage cannot pass only because the larger air-control or landing routes happened to exercise a few key poses.

## Artifacts

Each run writes to the eval output directory:

- `samples.ndjson`: newline-delimited per-sample telemetry.
- `summary.json`: pass/fail checks, optional named target island, aggregate metrics, artifact paths, and final state.
- `final.png`: final rendered screenshot when screenshot capture is enabled.
- `checkpoints/*.png`: fixed-frame camera screenshots when screenshot capture is enabled.
- `checkpoints/*.markers.json`: fixed-frame semantic route-beacon/objective/power-up marker visibility classifications plus terrain/foliage/cloud/distant-island/wind scene-sample projections and visibility classifications when screenshot capture is enabled.
- `marker_projection_audit.json`: projected visible-marker pixel audit for screenshot evals that emit marker sidecars.
- `semantic_scene_audit.json`: projected terrain/foliage/cloud/distant-island material-family and expected scene-kind hit, conditional wind visual pixel-hit checks, terrain material/biome variant diversity, visible terrain material-variant hit coverage, and aggregate pixel-coverage audit for screenshot evals that emit marker sidecars.
- `visual_audit.json`: non-golden image audit for screenshot evals run through `tools/eval.sh` unless `NAU_EVAL_VISUAL_AUDIT=0` is set.
- `asset_fixture_audit.json`: structural glTF fixture audit run through `tools/eval.sh` unless `NAU_EVAL_ASSET_AUDIT=0` is set.

The summary is the primary artifact for agents. `tools/eval.sh` exits nonzero when `summary.json` reports `passed: false` and prints failed checks when `jq` is available, so shell loops cannot silently continue after a failed scenario. Screenshots are for visual review and should not be treated as pixel-perfect golden images.
`NAU_EVAL_SIM_ONLY=1 ./tools/eval.sh <scenario> <dir>` runs the `traversal_sim_eval` binary instead of the app. It writes the same `summary.json` and `samples.ndjson` artifact names with schema `nau_traversal_sim_eval.v2`, reports `mode: "simulation_only"` and `native_window_created: false`, and covers scripted input, flight integration, route objectives, lift fields, bounded horizontal wind response, aerial power-ups, LOD route math, camera follow, shared route-spire obstruction avoidance, and movement/camera checks without creating a Bevy app or native window. The v2 summary adds direction-split desired-travel, AirTurn pose coverage fields, and explicit grounded-walk, grounded-run, launching, falling, and gliding pose coverage fields. The `air_control_response` sim gates right/left lateral response and latency, backward-right/backward-left diagonal response and latency, aggregate plus right/left/backward-right/backward-left desired-travel heading samples/error, explicit right/left `air_turn` pose samples, body-heading error/oscillation, directional body/travel heading sample coverage and alignment, body-roll/bank response, air-brake speed drop, post-brake recovery, and movement-only camera yaw/rotation drift. The `camera_mouse_control` sim gates yaw/pitch input and nonzero camera-obstruction adjustment from the same route-integrated spires used by the app. The `pose_state_coverage` sim gates grounded walk/run samples, walk/run stride foot travel, walk/run leg opposition, readable launch/fall/glide/air-turn/air-brake/gliding-dive samples, landing anticipation/recovery samples, landing crouch, feet-forward flare, recovery flip, and zero unreadable key poses on a compact no-mouse route; the app path also gates matching authored `jog` and `fall` clip samples. Wind-current scenarios gate active wind-force samples, wind-force delta, source flow speed/variation, and crosswind-specific delta where relevant, so background loops can catch the core flight-control regressions before screenshot/app evals. `./tools/eval_sim_suite.sh <dir>` runs all simulation-supported scripted scenarios through this path, disables per-scenario asset audits, runs `asset_fixture_audit` once, includes the pose-state counters in its aggregate summary, and emits `<dir>/summary.json` with schema `nau_sim_suite.v1` so long background loops can prove those routes stayed windowless. App-only routes such as `world_collision_contact` and `terrain_rim_collision_contact` depend on runtime-spawned Bevy collision proxies and should be run through the default app eval path. Simulation-only evals deliberately do not produce screenshots, marker sidecars, render/content image audits, frame-time telemetry, asset-server lifecycle checks, or generated-asset and terrain-rim collision contact checks.
`tools/eval.sh` checks that declared PNG artifacts exist, are large enough, that declared marker sidecars exist and pass their projection checks, that projected non-occluded visible markers have marker-colored pixels near their screen-space positions, that projected and non-occluded terrain/foliage/cloud/distant-island/wind scene samples have matching material-like pixels near their screen-space positions, and that screenshots pass a lightweight visual audit for resolution, nonblack/nonwhite exposure, luma variance, color variety, edge density, per-frame scene coverage, per-frame center detail, scene detail tile frequency, flat low-detail scene-tile dominance, dominant low-detail scene-component dominance, player visibility, severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, HUD-text dominance, route-marker readability/component identity/hue diversity, distant horizon/impostor component readability/color-bucket/span identity, terrain/material family diversity plus terrain material coverage/color/tile spread, foliage coverage/tile spread, cloud-layer coverage/component/span identity, and top-sky coverage across the final screenshot plus fixed checkpoints. The audit catches gross render, composition, transparency, baked desktop/window canvas, low-frequency scene-detail failures, large smooth primitive-like scene surfaces, collapsed marker hue identity, missing or flat distant horizon silhouettes, one-family or missing/flat terrain material screenshots, missing or overly clustered readable foliage coverage, and missing or collapsed cloud-layer image readability. Marker sidecars add exact known-marker viewport projection plus approximate terrain-occlusion classification for route beacons, route objectives, uncollected aerial power-ups, and semantic terrain/foliage/cloud/distant-island/wind sample points; `terrain_surface` samples also carry terrain material variant identity for projected diversity checks, and the marker and semantic scene pixel audits ignore samples classified as occluded/offscreen/behind-camera. Wind-critical checkpoint sidecars must retain at least one visible wind sample so wind-current screenshots cannot pass with the airflow cues offscreen, while late non-wind checkpoints can still be validated by the aggregate semantic scene audit. The semantic scene audit reports per-checkpoint in-viewport, occluded, and visible scene-sample counts, per-checkpoint visible material families, per-checkpoint visible scene sample kinds, per-family and per-kind visible sample counts, terrain material/biome variant counts for visible `terrain_surface` samples, per-family and per-kind visible sample counts and pixel-hit counts, minimum hit floors, hit ratios, per-visible-variant pixel-coverage floors, conditional wind visual hit checks, and aggregate pixel-coverage checks for each expected material family and scene kind; it fails when any visible material family lacks enough material-like pixels, when a checkpoint has too few distinct visible scene sample kinds, when terrain material/biome variants do not clear aggregate diversity floors, when too few visible terrain material variants have matching variant-class terrain pixels, when matching variant hits collapse into tiny projected specks, when any expected material family or sample kind has no visible sample/hit across the checkpoint sequence, or when terrain/foliage/cloud/distant-island projected hits are too tiny to clear their aggregate coverage floors. These audits use tolerant per-variant terrain color classifiers for `terrain_surface` samples rather than exact per-pixel material ownership; they are projected-sample diversity gates, not production material classifiers, and do not prove exact 3D occlusion, exact per-pixel world semantics, or AAA-quality art direction.
`asset_fixture_audit` checks every declared glTF fixture for self-authored provenance, `extras.nau` registry metadata (`nau_visual_asset_fixture.v1`, asset kind, label, residency, and self-authored license contract), semantic component names, mesh/material/vertex/triangle floors, normals, UVs, blend-material expectations for transparent fixtures, and the player named animation clip inventory. It also rejects reused authored bank or fall clip motion so directional banking and falling cannot pass by aliasing the glide, air-brake, or land tracks. The current world-fixture contract also requires richer semantic fragments for terrain erosion/path detail, foliage root/fern/moss detail, water lily/specular detail, rock rust/shale detail, route glyph/pebble detail, cloud haze/filament detail, and distant waterfall/broken-cliff impostor detail. It is still a structural fixture gate, not a substitute for production art review.

## Sample Fields

Every sample includes:

- `frame`
- `time_secs`
- `position`
- `velocity`
- `speed_mps`
- `altitude_m`
- `visual_foot_gap_m`
- `mode`
- `pose_intent`
- `pose_torso_pitch_degrees`
- `pose_arm_spread_degrees`
- `pose_leg_tuck_degrees`
- `pose_lateral_lean_degrees`
- `pose_signed_lateral_lean_degrees`
- `pose_grounded_stride_foot_travel_m`
- `pose_grounded_stride_leg_opposition_degrees`
- `pose_landing_crouch_m`
- `pose_landing_foot_forward_m`
- `pose_wing_airflow_strength`
- `key_pose_readability_score`
- `key_pose_transition_grace`
- `visible_pose_part_count`
- `max_pose_part_rotation_delta_degrees`
- `max_pose_part_translation_delta_m`
- `desired_body_yaw_error_degrees`
- `desired_body_heading_error_degrees`
- `body_travel_heading_error_degrees`
- `body_roll_degrees`
- `desired_heading_alignment_mps`
- `desired_travel_heading_error_degrees`
- `lateral_response_mps`
- `lateral_input_active`
- `movement_input_lateral_axis`
- `movement_input_forward_axis`
- `camera_distance_m`
- `camera_surface_clearance_m`
- `camera_player_angle_degrees`
- `camera_pitch_degrees`
- `camera_yaw_offset_degrees`
- `camera_pitch_offset_degrees`
- `camera_step_distance_m`
- `camera_rotation_delta_degrees`
- `camera_orbit_alignment_degrees`
- `camera_follow_direction_error_degrees`
- `camera_view_yaw_degrees`
- `camera_world_yaw_degrees`
- `camera_obstruction_adjustment_m`
- `camera_obstruction_hits`
- `visible_wind_fields`
- `wind_field_count`
- `dynamic_wind_flow_fields`
- `max_wind_flow_speed_mps`
- `max_wind_flow_variation`
- `active_wind_force_fields`
- `crosswind_force_fields`
- `updraft_swirl_force_fields`
- `max_wind_force_delta_mps`
- `max_crosswind_force_delta_mps`
- `max_updraft_swirl_force_delta_mps`
- `max_wind_force_flow_speed_mps`
- `max_wind_force_variation`
- `max_wind_force_flow_alignment`
- `max_crosswind_force_flow_alignment`
- `max_updraft_swirl_force_flow_alignment`
- `max_wind_force_aligned_delta_mps`
- `max_crosswind_force_aligned_delta_mps`
- `max_updraft_swirl_force_aligned_delta_mps`
- `active_lift_fields`
- `readable_lift_fields`
- `lift_field_count`
- `target_distance_m`, measured against the scenario target island
- `on_landing_target`, measured against the scenario target island
- `objective`, containing completed count, total count, current step, current label, current distance, and complete state
- `sky_island_count`
- `active_chunk_count`
- `active_island_count`
- `near_lod_islands`
- `mid_lod_islands`
- `far_lod_islands`
- `visible_island_terrain_count`
- `hidden_island_terrain_count`
- `visible_island_impostor_count`
- `hidden_island_impostor_count`
- `visible_island_detail_count`
- `hidden_island_detail_count`
- `visible_route_beacon_count`
- `weather_cloud_count`
- `environment_motion_visual_count`
- `max_environment_motion_offset_m`
- `updraft_guide_visual_count`
- `updraft_ribbon_visual_count`
- `crosswind_guide_visual_count`
- `crosswind_ribbon_visual_count`
- `max_updraft_visual_motion_m`
- `max_updraft_visual_rise_m`
- `max_updraft_visual_swirl_displacement_m`
- `max_updraft_visual_depth_span_m`
- `max_updraft_visual_scale_pulse`
- `max_crosswind_visual_motion_m`
- `max_crosswind_guide_flow_displacement_m`
- `max_crosswind_ribbon_flow_displacement_m`
- `max_crosswind_visual_lane_depth_span_m`
- `max_crosswind_visual_scale_pulse`
- `updraft_flow_coherent_visual_count`
- `crosswind_flow_coherent_visual_count`
- `max_updraft_visual_flow_alignment`
- `max_crosswind_visual_flow_alignment`
- `world_collision_proxy_count`
- `terrain_rim_collision_proxy_count`
- `solid_world_collision_proxy_count`
- `tree_world_collision_proxy_count`
- `rock_world_collision_proxy_count`
- `landmark_world_collision_proxy_count`
- `world_collision_resolved_count`
- `terrain_rim_collision_resolved_count`
- `max_world_collision_push_m`
- `max_terrain_rim_collision_push_m`
- `island_terrain_surface_count`
- `min_island_terrain_mesh_vertices`
- `min_island_terrain_color_bands`
- `min_island_terrain_material_weight_bands`
- `min_island_terrain_material_channels`
- `min_island_terrain_material_regions`
- `min_island_terrain_texture_detail_bands`
- `min_island_terrain_relief_range_m`
- `island_terrain_archetype_count`
- `min_island_cliff_color_bands`
- `procedural_island_body_count`
- `primitive_island_body_count`
- `min_island_body_silhouette_segments`
- `avg_island_body_silhouette_segments`
- `min_island_body_mesh_vertices`
- `max_island_body_mesh_vertices`
- `generated_ground_cover_patch_count`
- `min_ground_cover_blade_count`
- `min_ground_cover_mesh_vertices`
- `generated_tree_trunk_count`
- `generated_tree_canopy_count`
- `min_tree_trunk_mesh_vertices`
- `min_tree_canopy_mesh_vertices`
- `detail_biome_palette_count`
- `generated_rock_count`
- `min_rock_mesh_vertices`
- `generated_weather_cloud_count`
- `generated_weather_cloud_bank_count`
- `min_weather_cloud_bank_depth_m`
- `min_weather_cloud_lobe_count`
- `max_weather_cloud_lobe_count`
- `min_weather_cloud_mesh_vertices`
- `min_weather_cloud_filament_ribbon_detail_count`
- `resident_island_visual_count`
- `stream_visibility_changes_this_frame`
- `max_stream_visibility_changes_per_frame`
- `total_stream_visibility_changes`
- `catalog_island_visual_count`
- `hidden_island_visual_count`
- `resident_island_visual_fraction`
- `stream_spawned_visuals_this_frame`
- `stream_despawned_visuals_this_frame`
- `max_stream_spawned_visuals_per_frame`
- `max_stream_despawned_visuals_per_frame`
- `total_stream_spawned_visuals`
- `total_stream_despawned_visuals`
- `entity_count`
- `visual_asset_slot_count`
- `gltf_scene_asset_slot_count`
- `ready_visual_asset_slot_count`
- `placeholder_visual_asset_slot_count`
- `streaming_visual_asset_slot_count`
- `missing_visual_asset_slot_count`
- `deferred_visual_asset_scene_count`
- `queued_visual_asset_scene_count`
- `loading_visual_asset_scene_count`
- `loaded_visual_asset_scene_count`
- `dependency_loaded_visual_asset_scene_count`
- `preload_ready_visual_asset_scene_count`
- `failed_visual_asset_scene_count`
- `spawned_visual_asset_scene_count`
- `ready_visual_asset_scene_count`
- `visible_authored_world_fixture_count`
- `always_visual_asset_slot_count`
- `stream_window_visual_asset_slot_count`
- `near_lod_visual_asset_slot_count`
- `far_lod_visual_asset_slot_count`
- `weather_visual_asset_slot_count`
- `always_preload_ready_visual_asset_slot_count`
- `streaming_preload_ready_visual_asset_slot_count`
- `declared_animation_clip_count`
- `ready_animation_clip_count`
- `animation_player_count`
- `animation_graph_count`
- `authored_player_current_clip_label`
- `authored_player_desired_clip_label`
- `authored_player_count`
- `authored_transition_duration_ms`
- `authored_glider_response_degrees`
- `authored_glider_motion_m`
- `power_up_count`
- `visible_power_up_count`
- `collected_power_up_count`
- `active_power_up_effects`
- `total_power_up_activations`

Add fields here before adding them to code. New fields should be cheap to collect, stable across runs, and useful for deciding what to fix.

The island terrain/detail/impostor hidden counts are catalog entries that are not currently resident. `catalog_island_visual_count`, `hidden_island_visual_count`, and `resident_island_visual_fraction` report stream pressure directly so future optimization work does not have to infer it from per-layer fields. Terrain, cliff, underside, and distant-impostor visuals use cached mesh recipes so hidden entries do not allocate their Bevy mesh handles or construct full mesh data until they first become resident; startup content diagnostics use lightweight scans that are unit-checked against the generated mesh metrics, while export/audit paths still generate the full route to preserve offline quality checks. The `stream_visibility_*` names are retained for artifact compatibility, but now report resident island visual spawn/despawn churn rather than `Visibility` flag flips; the `stream_spawned_*` and `stream_despawned_*` fields split that churn into directional budget signals.
The visual asset fields report the declared glTF scene inventory, how many slots are loaded and ready, how many are still using generated placeholders, how many Bevy scene handles are deferred/queued/loading/loaded/failed, how many optional scene instances have spawned/reported ready, how many non-player authored world fixture kinds are visibly placed in the scene, how many named player animation clips are declared/ready, whether nested animation players were linked, whether animation graphs were prepared, and how slots divide across always-loaded, stream-window, near-LOD, far-LOD, and weather residency classes. The authored player animation fields report the runtime current/desired clip labels, linked authored player count, and transition duration so clip-selection regressions are inspectable in sample artifacts instead of only through visible pose scoring; summary metrics also count authored clip matches/mismatches plus bank-left, bank-right, fall, dive, air-brake, and landing clip coverage. Authored glider fields report the largest visible glider rotation response and local motion offset from its base placement, so the app eval can fail if the visible glider stays static while body/pose math passes. Missing files, deferred admissions, and failed loads count as placeholder-backed; deferred slots do not allocate Bevy handles, and queued/loading files are not counted as ready until Bevy reports them loaded. These are readiness signals for replacing primitives with real assets; they now gate every declared fixture slot, zero deferred current fixtures, the seven visible non-player world fixture kinds, the self-authored player fixture's full named clip set including `Fall_Loop` and directional bank clips, distinct authored fall/bank clip motion instead of glide/air-brake/land reuse, one linked `AnimationPlayer`, and one ready `AnimationGraph`. The separate asset fixture audit verifies fixture semantic-name, geometry, and provenance floors, but neither signal proves final art quality yet.
The power-up fields report authored aerial boost gates, how many remain visible, how many have been collected, whether an effect is currently active, and the total activation count. They are route-readiness signals for the simple power-up slice, not final ability design.
The environment-motion fields report how many resident near-LOD visuals are wind-responsive and the largest sampled transform offset from their base placement. Dynamic wind-flow fields report how many visual wind volumes contain the sampled player position, the strongest shared flow speed, and the strongest gust variation sampled from the same shared `WindField` catalog used by updraft and crosswind guide visuals. Shared `WindField::flow_at` currents include soft edge falloff plus spatial gust cells, so adjacent lanes and heights can move with different speed, shear, lift, and swirl while preserving horizontal crosswind flow and upward-biased updraft flow. Wind-force fields report the bounded horizontal airborne response from those visual `WindField`s, including total applied delta, crosswind-only delta, source flow speed/variation, force-to-flow correction alignment against the field-axis direction needed to move current speed toward sampled flow speed, and the delta component aligned with that correction direction; updraft `WindField` swirl can add lateral current, but vertical lift remains separate `LiftField` telemetry. Layered wind-force summary fields count samples where at least two wind fields affect the player at once, samples where crosswind and updraft swirl overlap in the same frame, aligned overlap samples, max layered field count, and layered-only delta/alignment floors; `updraft_route` gates those values so a single isolated updraft or a route that encounters crosswind only outside lift cannot pass. Wind-load response fields count neutral airborne/gliding samples where meaningful crosswind force creates a body-local lateral load, then gate max normalized load, pose lean, and authored glider response so wind cannot affect velocity while leaving Nau and the glider visually inert. The wind guide visual fields separately count updraft motes/ribbons and crosswind motes/ribbons, then report per-field coverage for guide presence, ribbon presence, guide-plus-ribbon pairing, and field-level flow coherence; the summary gates require all declared updraft and crosswind fields to have paired/coherent visuals so one strong field cannot hide a missing neighbor. The same fields also report the largest sampled guide/ribbon offset from baseline placement, updraft guide rise, updraft swirl displacement, updraft vertical depth span, crosswind lane/height depth span, baseline-relative guide/ribbon scale pulse, split guide/ribbon crosswind flow-direction displacement, short-horizon visual-flow alignment against the same `WindField::flow_at` source used by gameplay force sampling, and sustained sample counts for updraft/crosswind/all wind visual flow so a single good frame cannot satisfy the airflow surface. Focused visual tests cover stream-specific crosswind motion variation so guides do not move in lockstep. This keeps missing, static, flat, one-layer-only, lockstep, wrong-axis, one-good-frame, one-good-field, velocity-only, or gameplay-detached airflow cues from hiding behind generic tree/cloud sway or lateral jitter. Screenshot sidecars also project wind guide/ribbon samples and require visible wind samples on wind-critical routes. Summary metrics track the sampled variation range while lift is active/readable, so a nonzero but static wind-flow value cannot satisfy lift-required scenarios. Together they prove the visual motion layer, readable lift flow, first gameplay wind-current response, and first body/glider wind-load reaction are active; they do not evaluate final animation quality.
The island-terrain fields report generated terrain surface count, minimum terrain mesh vertex count, minimum vertex-color band count, minimum encoded material-weight band count, minimum material channel count, minimum derived material-region count, minimum terrain texture-detail band count, minimum sampled terrain relief range, route terrain-archetype diversity, and minimum cliff/underside color-band count. The material weights are currently encoded into `UV_1` as lush/highland and exposed-edge blend channels; material regions quantize those channels into stable base, transition, lush, and exposed-edge identities; texture-detail bands count coarse color bins in the terrain-specific procedural albedo maps, which are 512px, world-space tiled, and include sharper pitted strata, fissure, mineral-fleck, and micro-grain detail instead of being stretched once across an island, so future PBR material blending or glTF export has a measurable substrate. The headless terrain export adds terrain-archetype diversity, texture-edge, height-band, and normal-slope-band floors over manifest and OBJ artifacts so repeated island grammar, smeared, flat, or single-slope terrain fills fail offline even before screenshot review. These are structural signals for denser generated island substrate with ravines, terraces, basins, ridges, and microrelief; they do not replace screenshot or human review for final material quality.
The island-impostor fields report the minimum generated far-LOD impostor mesh vertex count and vertex-color band count. They are structural gates for distant island silhouettes and layered terrain/cliff/underside color variation, not final far-field art quality.
The island-body fields report whether the catalogued route island bodies are generated procedural meshes or registered primitive/fallback body placeholders, plus the minimum silhouette segment count and minimum/maximum body mesh vertex count signals. They are a structural signal for replacing cylinder-like islands; they do not prove final terrain art quality or texture fidelity.
Current substrate gates require 20 generated island terrain surfaces, all 19 declared route terrain archetypes, at least 32 terrain color bands, 24 terrain material-weight bands, 19 terrain height bands, 10 terrain normal-slope bands, 50 terrain texture-detail bands, 590 terrain texture-edge promille in export audit, 140 island-impostor mesh vertices, 18 island-impostor color bands, 1600 island-body mesh vertices, 55 generated trunks/canopies, 450 canopy mesh vertices, 90 generated rocks, 60 generated landmarks, 16 route cairns, 1 launch beacon, 4 landing-garden markers, 20 pond surfaces, 20 route obstruction spires, 39 landmark mesh vertices, 300 obstruction-spire mesh vertices, 500 obstruction-spire triangles, 3.0m obstruction-spire vertical span, 6 obstruction-spire height bands, 5 obstruction-spire radius bands, 5 obstruction-spire normal-slope bands, 40 generated weather clouds, 9 weather-cloud lobes, 36 weather-cloud wisp cards, 1530 weather-cloud mesh vertices, 5.8m generated cloud-bank depth, and 27 weather-cloud filament ribbons.

## Summary Metrics

The summary aggregates:

- sample count
- average frame time
- p95 frame time
- p99 frame time
- max frame time
- horizontal distance from first to final sample
- max and min altitude
- max grounded authored visual foot gap from the rendered terrain surface
- max speed
- average, p95, and max desired body-heading error
- desired-travel heading sample count, right/left/backward-right/backward-left desired-travel sample counts, plus p95 and max desired-travel heading error
- lateral right/left and backward-diagonal right/left body/travel heading sample counts plus p95/max error
- visible pose part count, explicit aggregate and right/left airborne turn-pose sample counts, global visible-pose temporal stability sample count, max global pose-part rotation/translation deltas, landing-only temporal stability sample count, and max landing pose-part rotation/translation deltas
- max body-yaw error step and same-intent body-yaw oscillation count
- max body-roll step plus max right/left body-bank response
- max desired-heading velocity alignment
- max lateral response speed and first-response latency
- max right, left, rear-right, and rear-left lateral response speed and response latency
- max rear-right and rear-left rearward response speed
- max total air-brake speed drop, max planar air-brake speed drop, and max post-brake forward alignment
- max camera distance
- min camera surface clearance
- max camera-to-player framing angle
- max per-frame camera step distance
- max per-frame camera rotation delta
- max camera orbit alignment
- average, p95, and max camera follow-direction error
- max absolute camera view yaw, relative view-yaw drift, and world-yaw drift
- max camera obstruction adjustment
- max camera obstruction hit count
- min and final scenario-target distance
- min and max camera pitch
- max absolute camera yaw offset
- min and max camera pitch offset
- max visible wind-field count
- max dynamic wind-flow field count, wind-flow speed, wind-flow variation, and readable-lift variation range
- wind-force sample counts, sustained meaningful and flow-aligned wind-force sample counts, crosswind/updraft-swirl/layered force sample counts, crosswind-updraft overlap sample counts, max active wind-force field count, crosswind/updraft-swirl/layered force field counts, wind-force delta, crosswind delta, updraft-swirl delta, layered delta, wind-force flow speed, wind-force variation, force-flow alignment, layered force-flow alignment, and flow-aligned delta
- wind-load response sample count plus max body-local lateral load, wind-driven pose lean, and wind-driven glider response
- max updraft/crosswind guide and ribbon visual counts
- max updraft visual motion, rise, swirl displacement, vertical depth span, and baseline-relative scale pulse
- max crosswind visual motion, guide/ribbon flow displacement, lane/height depth span, and baseline-relative scale pulse
- max updraft/crosswind short-horizon flow-coherent visual counts and flow alignment
- max active lift-field count
- max readable lift-field count
- max sky-island count
- max active chunk count
- max active island count
- max near/mid/far LOD island counts
- max visible island terrain count
- max hidden island terrain count
- max visible island impostor count
- max hidden island impostor count
- min island impostor mesh vertex count
- min island impostor vertex-color band count
- max visible island detail count
- max hidden island detail count
- max visible route beacon count
- max weather cloud count
- max environment-motion visual count
- max environment-motion offset
- max world-collision proxy count
- max terrain-rim collision proxy count
- world-collision resolved sample count
- terrain-rim collision resolved sample count
- terrain-rim collision meaningful-contact sample count
- max world-collision push distance
- max terrain-rim collision push distance
- min island terrain surface count
- min island terrain mesh vertex count
- min island terrain vertex-color band count
- min island terrain material-weight band count
- min island terrain material channel count
- min island terrain material-region count
- min island terrain texture-detail band count
- min island terrain relief range
- min island cliff color-band count
- min procedural island body count
- max primitive island body count
- min island body silhouette segment count
- max average island body silhouette segment count
- min island body mesh vertex count
- max island body mesh vertex count
- min generated tree trunk/canopy counts
- min generated tree trunk/canopy mesh vertex counts
- min generated landmark total and route-cairn/launch-beacon/landing-garden/pond-surface/obstruction-spire counts
- min landmark mesh vertex count
- min obstruction-spire mesh vertex, triangle, vertical-span, height-band, radius-band, and normal-slope-band counts
- min generated weather cloud count
- min/max weather cloud lobe counts
- min weather cloud mesh vertex count
- min weather cloud filament-ribbon count
- min generated cloud-bank depth
- max resident island visual count
- max stream visibility changes per frame
- total stream visibility changes
- max catalog island visual count
- max hidden island visual count
- max resident island visual fraction
- max stream spawned/despawned visuals per frame
- total stream spawned/despawned visuals
- max scene entity count
- objective total count
- max and final completed objective count
- min and final objective distance
- objective complete sample count
- max visual asset slot count
- max glTF scene asset slot count
- max ready visual asset slot count
- max placeholder visual asset slot count
- max stream-managed visual asset slot count
- max missing visual asset slot count
- max deferred/queued/loading/loaded/failed visual asset scene counts
- max spawned/ready visual asset scene instance counts
- max visible authored world fixture count
- max declared/ready animation clip counts and animation player/graph counts
- max always-loaded, stream-window, near-LOD, far-LOD, and weather visual asset slot counts
- max power-up count
- min visible power-up count
- max collected power-up count
- power-up effect sample count
- total power-up activation count
- target landing sample count
- active lift sample count
- readable and unreadable active-lift sample counts
- dynamic readable lift sample count plus max sampled wind-flow speed, gust variation, and readable-lift variation range
- pose-intent sample counts for readable gliding, launching, falling, diving, air brake, landing anticipation, and landing recovery; app and simulation summaries also count grounded walk/run pose samples plus deployed-glider dive samples, and app evals score visible generated or authored player part transforms when those nodes are available and still require authored clip alignment when authored player geometry hides generated fallback parts
- pose torso pitch, arm spread, leg tuck, lateral lean, signed right/left lateral lean, landing crouch, landing foot-forward tuck, landing flare, wing-airflow maxima, authored glider response/motion maxima, authored glider dive response/motion maxima, authored bank-left/bank-right/fall/dive/air-brake/landing clip sample counts, key-pose readability min/max, unreadable key-pose sample count, and key-pose transition-grace sample count
- gliding, launching, and grounded sample counts

The pass/fail checks currently guard:

- enough samples were written
- the route covered enough horizontal distance
- launch produced enough altitude
- max speed crossed the scenario floor
- the route spent enough sampled frames gliding
- the route spent enough sampled frames grounded
- air-control scenarios must drive the authored player state machine through directional bank-left/bank-right, deployed-glider dive, and air-brake clips while reporting zero authored clip mismatches
- target-landing scenarios must drive the authored landing clip for both anticipation/recovery coverage while reporting zero authored clip mismatches
- the route spent enough sampled frames inside gameplay lift when a scenario requires it
- lift-required scenarios spend enough sampled active-lift frames inside a paired visible updraft field
- lift-required scenarios have zero unreadable active-lift samples
- lift-required scenarios sample non-static dynamic wind flow with enough speed, gust variation, and variation range while lift is active/readable
- simulation-supported wind-current scenarios register enough active wind-force samples, enough sustained samples with per-source meaningful force delta and source variation, enough force samples whose applied horizontal delta aligns with the field-axis correction direction toward sampled `WindField::flow_at` speed, and separately clear bounded horizontal response, source flow speed, source variation, flow-alignment, and flow-aligned delta floors
- crosswind-current scenarios specifically register crosswind-force samples and clear crosswind delta, flow-alignment, and flow-aligned delta floors
- lift-required scenarios specifically register updraft-swirl force samples and clear horizontal-current delta, flow-alignment, and flow-aligned delta floors while vertical climb remains `LiftField` lift
- `updraft_route` must register neutral crosswind-load reaction samples and clear normalized lateral-load, pose-lean, and glider-response floors so force-only wind does not pass
- the app scene contains enough updraft and crosswind guide/ribbon visuals; the current minimums are 126 updraft guide motes, 12 updraft ribbons, 120 crosswind guide motes, and 14 crosswind ribbons
- wind guide/ribbon visuals animate enough to prove the sampled airflow cues are not static, including minimum updraft vertical depth span, updraft scale pulse, crosswind lane/height depth span, and crosswind scale pulse
- wind guides move coherently with their field direction: updraft motes must rise and visibly curl around the updraft tangent, and both crosswind motes and ribbons must travel along the crosswind direction
- enough wind guide/ribbon visuals must have short-horizon motion aligned with the shared `WindField::flow_at` vector; current floors require 108 coherent updraft visuals, 100 coherent crosswind visuals, and at least 0.55 max alignment for each family
- screenshot wind-current checkpoints keep at least one projected wind guide/ribbon sample visible for wind-critical routes
- the world has enough sky islands to catch accidental route collapse
- the active chunk window stays inside the scenario budget
- enough islands enter the active chunk window
- near/mid/far LOD island buckets remain populated
- visible island terrain stays under the scenario budget, proving inactive chunks are not drawing full terrain
- hidden island terrain stays populated, proving inactive chunk terrain remains measurable before real despawn work
- visible island impostors stay populated, proving inactive chunks retain distant silhouettes
- distant island impostor mesh vertex and color-band floors stay populated so far LOD cannot silently collapse back to a flat low-detail blob
- visible island detail stays under the scenario budget, proving distance LOD is active
- hidden island detail stays populated, proving distance LOD is actually culling resident detail
- visible route beacons stay populated so distant route readability is not culled away
- objective totals stay populated, and objective-route scenarios complete their required objective count
- declared visual asset slots, glTF scene slots, stream-managed asset slots, and every current self-authored fixture scene stay loaded/spawned/ready so the real-asset migration surface cannot silently disappear
- visible authored world fixture count stays at the current seven non-player fixture kinds so world assets cannot regress back into hidden preload-only probes
- missing visual asset slots stay at zero, proving the full declared fixture inventory remains present and loadable
- deferred visual asset scene count stays at zero, proving the default admission policy still admits every current fixture until a future PR deliberately raises the real streaming budget surface
- failed visual asset scene count remains zero so broken imported assets fail the eval loop once real files are present
- authored aerial power-up counts stay populated, and power-up scenarios collect enough gates with enough sampled active-effect frames
- weather cloud count stays populated so the first non-debug weather layer cannot silently disappear
- environment-motion visual count and offset stay populated so wind-responsive near-LOD trees/ponds cannot silently disappear or freeze
- world-collision proxy count stays populated so generated trees, rocks, route cairns, launch beacons, recovery masts, target markers, and obstruction spires cannot regress into purely decorative ghost props
- the island-visual catalog unit audit classifies every generated solid visual name by collision kind, requires four terrain-rim rails per route island, and fails any camera obstacle without collision unless it is explicitly allowlisted as camera-only foliage or cliff-body obstruction
- the `world_collision_contact` app route sustains resolved collision samples above a 5 mm per-sample push floor and reaches a nontrivial peak push while taxiing into a launch-mesa obstacle, so counted generated obstacles must also affect player movement
- terrain-rim collision proxy count stays populated so near-LOD island edge rails cannot silently disappear
- the `terrain_rim_collision_contact` app route sustains terrain-rim resolved samples above a 5 mm per-sample push floor and reaches a nontrivial peak push while taxiing into the launch-mesa rim
- `ground_taxi_control` records zero terrain-rim resolved samples, so normal pre-launch WASD movement cannot pass while hidden rim rails scrape or snag the player
- island terrain surface count, terrain-archetype count, mesh vertex floor, vertex-color band floor, material-weight band/channel/region floors, texture-detail floor, relief-range floor, height-band/normal-slope-band floors, and cliff color-band floor stay populated so the generated world cannot silently regress to repeated island grammar, lower-resolution, blurrier, flatter, single-slope, or visually single-tone island surfaces
- procedural island body count stays populated throughout the run so sky-island bodies cannot silently disappear or fall below the expected generated catalog
- registered primitive island body count remains zero so explicit fallback body placeholders fail the eval loop
- island body silhouette segment count stays above the scenario floor so the route cannot collapse back to low-resolution round islands
- island body mesh vertex count stays above the generated-body floor so low-resolution cylinder-like bodies fail even if their silhouette segment count is high
- resident island visuals stay under budget while streaming visibility is still hide/show based
- stream visibility changes per frame stay under budget so chunk/LOD crossings do not churn too many visuals at once
- hidden island visual count stays populated and resident visual fraction stays under budget so the catalog does not collapse into always-resident rendering
- spawned and despawned island visuals each stay under the per-frame churn budget so future asset streaming can distinguish load pressure from unload pressure
- the scene has enough entities to catch accidental content collapse
- camera distance stayed under a loose maximum
- grounded authored player feet stayed aligned to the rendered terrain surface
- camera stayed above the active ground surface
- camera kept the player focus near the camera centerline
- camera per-frame movement and rotation stayed under scenario jerk thresholds
- camera orbit alignment stayed under threshold
- camera view yaw and world-yaw drift stayed within scenario limits when movement should not rotate the camera
- camera obstruction avoidance was exercised when a scenario requires it; `camera_mouse_control` specifically exercises the shared route obstruction-spire blockers so standalone test cuboids cannot be the only obstruction coverage
- camera mouse scenarios exercised yaw and both pitch directions
- air-control response latency, right/left/rear-right/rear-left lateral response and latency, rear-right/rear-left rearward response, pure-backward body-heading intent, body-bank response, body-roll step, total and planar air-brake speed drop, readable right/left air-turn plus air-brake/deployed-glider dive pose coverage, authored bank-left/bank-right/dive/air-brake clip coverage, pose torso pitch at least 45 degrees, arm spread at least 100 degrees, leg tuck at least 35 degrees, lateral lean at least 8 degrees, signed right/left pose lean at least 8 degrees in the matching input direction, dive key-pose readability whose 0.9 floor corresponds to at least a 60-degree torso pitch and 165-degree arms-out silhouette, wing-airflow strength at least 0.25, visible authored glider response at least 4 degrees, dive-specific authored glider response at least 4 degrees, dive-specific authored glider motion at least 0.04 m, zero key-pose samples below the 0.9 readability floor, global visible-pose temporal samples and bounded global pose-part deltas, post-brake forward alignment, desired-heading and aggregate plus right/left/backward-right/backward-left desired-travel alignment, average/p95/max body-heading error, nonzero right/left/backward-right/backward-left sample counts and p95/max lateral and backward-diagonal body/travel heading error, max yaw-error step, yaw oscillation count, camera orbit yaw offset, camera view-yaw drift, camera world-yaw drift, and camera rotation delta stayed inside thresholds
- pose-state coverage scenarios record grounded walk/run samples, grounded walk/run stride foot travel and leg opposition, readable launch/fall/glide/air-turn/air-brake/gliding-dive samples, landing anticipation/recovery samples, landing crouch, feet-forward flare, recovery flip, and app-authored `jog`/`fall` clip samples while rejecting unreadable key-pose samples
- air-control average and p95 camera follow-direction error stayed inside threshold so movement-only routes cannot pass with a stale follow direction
- air-control movement-only camera world-yaw drift stayed inside threshold
- island-route final scenario-target distance stayed under threshold
- island-route grounded target landing was observed on the configured target island
- landing-required routes exercised readable landing-anticipation pose coverage before contact, readable landing-recovery pose coverage after contact, reached at least 0.05 m of landing crouch, reached at least 0.32 m of feet-forward landing tuck, reached at least 48 degrees of landing flare, produced zero key-pose samples below the 0.9 readability floor after bounded transition grace, recorded landing-only visible-pose temporal samples, and kept landing pose-part rotation/translation deltas within route thresholds

Thresholds should remain loose until the intended route becomes richer. Tight thresholds belong only after a mechanic or route is deliberately locked.

## Scaling Rules

As the world grows, extend the harness in this order:

1. Add scenario-specific scripted routes.
2. Add metrics that explain known failure modes.
3. Add low-cost assertions around those metrics.
4. Extend visual checks only when raw metrics and the current non-golden screenshot audit are insufficient.

Do not start with pixel-perfect screenshots. Metal/wgpu/native-window output can shift slightly across machines and driver state. Visual evals should classify obvious failures: blank frame, missing terrain, player not visible, severe clipping, unreadable route, or incoherent composition.

## Future Scenarios

The thin-slice target should eventually have these evals:

- `baseline_route`: current smoke test.
- `island_launch_to_landing`: current route-completion test.
- `ground_taxi_control`: current pre-launch WASD regression test.
- `world_collision_contact`: current app-only generated-asset collision contact regression test.
- `terrain_rim_collision_contact`: current app-only terrain-rim collision contact regression test.
- `updraft_route`: current gameplay lift regression test.
- `branch_recovery_route`: current named branch landing and recovery-route test.
- `long_glide_visibility`: current larger-archipelago traversal, aerial boost-gate, and content-scale test.
- `camera_mouse_control`: current mouse X/Y regression test.
- `camera_yaw_stability`: current small-yaw no-drift regression test.
- `camera_turn_stability`: current rapid air-turn and air-brake camera stability test.
- `camera_strafe_stability`: current `A`/`D` no-auto-orbit camera stability test.
- `air_control_response`: current diagonal/lateral/rear-right/rear-left/brake/recovery air-control response test.
- `pose_state_coverage`: current full-pose chain coverage for walk/run stride, launch/fall/glide/air-turn/air-brake/gliding-dive, landing anticipation/recovery shape, authored-jog/authored-fall, and zero-unreadable pose-state coverage.
- `camera_stress`: fly close to geometry and record camera distance, pitch, and obstruction metrics.
- `streaming_route`: cross chunk boundaries and record active chunks, active islands, spawned entities, despawns, and frame time.

## Agent Loop Contract

A future Codex or orchestrator loop should:

1. Read this spec and `summary.json`.
2. Inspect `samples.ndjson` only for the failing or suspicious interval.
3. Inspect `visual_audit.json` and screenshots when the summary points to a visual, camera, terrain, or visibility issue.
4. Make one narrow change.
5. Run `cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and the relevant eval.
6. Commit the checkpoint with the eval artifacts path in the commit or PR notes when useful.

The repo should remain the durable memory. Do not depend on a past chat session to know what the eval means.

## Known Limitations

- Metric-only app evals hide the native Bevy window, but still instantiate the window/rendering stack and report wind-current metrics; use `NAU_EVAL_SIM_ONLY=1` when the run only needs windowless traversal/camera/wind math.
- Screenshot evals still need a visible native Bevy window. Screenshot runs disable debug gizmos and use opaque surface composition so transparent scene effects blend against the game frame rather than desktop content behind the window.
- Screenshot evals now run a lightweight image and scene-composition audit, including basic player visibility, scene detail tile frequency, flat low-detail scene-tile dominance, dominant low-detail scene-component dominance, severe border-clipping, route-marker readability, route-marker component identity, route-marker hue-family diversity, distant horizon/impostor component readability/color-bucket/span identity, terrain/material family diversity, terrain material coverage/color/tile spread, foliage coverage/tile spread, and cloud-layer coverage/component/span identity. Checkpoint marker sidecars classify known route beacons, route objectives, uncollected aerial power-ups, and projected terrain/foliage/cloud/distant-island semantic scene samples as visible, occluded, offscreen, or behind-camera; `terrain_surface` samples carry terrain material variant identity; `marker_projection_audit.json` verifies marker-colored pixels near non-occluded projected visible markers; and `semantic_scene_audit.json` verifies each visible terrain/foliage/cloud/distant-island material family has enough material-like pixels in each checkpoint, verifies enough distinct visible scene sample kinds per checkpoint, requires enough visible terrain material variants to have matching variant-class terrain pixels plus per-hit coverage, requires visible projected samples/hits for `terrain_surface`, `tree_canopy`, `weather_cloud`, and `distant_island` across the checkpoint sequence, and requires aggregate material/kind pixel coverage plus terrain material/biome variant diversity so projected world samples cannot pass as tiny specks or single-variant terrain. The terrain check is now variant-aware for `lush_meadow`, `gold_meadow`, `copper_clay`, `alpine_mist`, and `highland_grass`, but it remains a tolerant projected-sample diversity gate rather than exact per-pixel world semantics or a production material classifier. Exact 3D occlusion, distant-impostor art direction, exact per-pixel material ownership, and final art quality still need human/agent inspection. Headless terrain export audit covers the first terrain-material identity floor by requiring every exported island to retain base, transition, highland, and exposed material-region coverage, plus procedural albedo edge-frequency and manifest/OBJ height-band/normal-slope-band floors for blurry or flattened terrain fills; visual-content export audit covers generated vegetation/cloud/landmark structural quality through blade, multi-ring trunk, branch, root-flare, canopy-card, tree-size variation, cloud veil/depth-span, cloud-wisp, route cairn, launch beacon, landing marker, pond-surface, obstruction-spire count/mesh/shape-band floors, and palette-diversity floors; runtime and export/audit metric gates now cover the first terrain/body/impostor/vegetation/cloud/route-prop/camera-blocker primitive-shape substrate floors. These improve the generated island substrate but still describe generated placeholder art, not production AAA assets.
- The simulation-only binary intentionally skips renderer, screenshots, frame-time, asset-server, generated-asset collision contact, terrain-rim collision contact, and visual content checks, but it does cover the bounded wind-current response gates. Pair it with app/screenshot/export audits before treating a branch as fully verified.
- Frame-time metrics skip the first few warmup frames and are recorded as local native-window runtime telemetry; they are useful for trend spotting, not stable cross-machine pass/fail thresholds.
- Island collision follows deterministic authored terrain relief, and obvious generated plus solid authored-fixture obstacles now use simple AABB world-collision proxies. Near-LOD islands also spawn four invisible cardinal terrain-rim AABB rails that block grounded edge overruns without counting as visible terrain/detail. Runtime evals gate aggregate proxy presence, split solid non-rim proxies by tree/rock/landmark kind so terrain rails cannot mask missing asset blockers, report resolved samples, meaningful contact samples, and push distance, use `world_collision_contact` to prove the player is pushed by a generated launch-mesa obstacle, use `terrain_rim_collision_contact` to prove the player is pushed by a launch-mesa rim, and use `ground_taxi_control` to reject false rim contacts during normal ground control. The island-visual catalog unit audit checks named generated solid visuals before spawning, requires per-island rim proxy parity, and forces camera-only blockers such as foliage canopies and procedural cliff bodies to remain explicit exemptions. The procedural island ridge, recovery ring pieces, solid authored world fixtures, and shared route obstruction spires also carry AABB collision/camera-obstacle coverage. The collision model is still a route-surface clamp plus horizontal proxy push-out rather than full rigid-body physics, continuous terrain collision, or production imported colliders.
- `active_chunk_count` and `active_island_count` drive resident terrain/detail entities, and visual asset slots are declared, counted, split by residency class, and passed through a deterministic load-admission budget. This is an explicit policy surface for future asset streaming, not full asynchronous distance streaming yet.
- Missing glTF files are counted as placeholders and intentionally do not trigger load errors; deferred glTF files are also placeholder-backed but do not allocate Bevy `AssetServer` handles. Only admitted files that exist under `assets/` are queued through Bevy's `AssetServer`, and queued handles then report queued/loading/loaded/failed state. `ready_visual_asset_slot_count` means Bevy has loaded the scene asset, not merely that the file exists. `dependency_loaded_visual_asset_scene_count` and `preload_ready_visual_asset_scene_count` use Bevy's recursive dependency readiness so textures/buffers/subassets cannot lag behind a top-level scene handle without showing in evals; the always/streaming preload-ready counters split that signal by residency class. `spawned_visual_asset_scene_count` and `ready_visual_asset_scene_count` track Bevy scene-instance lifecycle separately from load state. The eval checks require all nine declared slots to be admitted, load, dependency-preload, spawn, and report ready, require all seven non-player world fixture kinds to be visibly placed, and fail if any current fixture disappears or is deferred; the self-authored player, glider, terrain, foliage, rock, water, route-marker, weather-layer, and distant-impostor fixtures are the current minimum viable asset pipeline surface. `asset_fixture_audit.json` now carries registry-aligned `extras.nau` metadata checks, semantic component-name, mesh/material/vertex/triangle, normal/UV, blend-material, provenance, named-player-clip checks including `Fall_Loop`, `Bank_Left`, and `Bank_Right`, distinct fall/bank clip motion checks, and the stricter world-fixture detail fragments for erosion/path, roots/ferns/moss, lily/specular water detail, rust/shale rock detail, route glyphs/pebbles, cloud haze/filaments, and distant waterfall/broken-cliff detail. `declared_animation_clip_count`, `ready_animation_clip_count`, `animation_player_count`, and `animation_graph_count` track the player scene's named clip/graph path separately from scene readiness; the eval checks gate the declared and ready clip inventory so the player animation contract cannot silently disappear.
- LOD buckets drive resident island detail, inactive or non-near chunks use cheap impostors, and hidden/resident/churn counters quantify stream-window pressure.
- The weather-cloud, generated ground-cover/tree/cloud/rock/landmark shape, detail biome palette, and environment-motion counters verify that cloud-layer entities, lobe counts, cloud-bank vertical depth, cloud filament-ribbon detail, ground-cover patch/blade density, mesh vertex floors, per-island generated detail material identity, non-spherical stone scatter, generated route/launch/landing landmarks, terrain-integrated obstruction spires, and wind-responsive near-LOD visual motion exist. Focused unit tests assert that generated ground cover uses dense curved blades, generated trunks use multiple rings, taper, branch mass, and root flares, canopies use overlapping lobes plus detail cards, rocks use flattened irregular silhouettes, generated landmarks replace primitive cylinders/boxes with stacked cairns, crystal shards, organic marker mounds, rippled pond surfaces, and irregular tapered obstruction spires with height/radius/normal-slope variation, generated detail materials vary across biome families, and cloud clusters stack lobes vertically with wisp-card edge geometry plus filament ribbons instead of collapsing into single cylinder/sphere/blob meshes. The visual-content export audit now makes those vegetation/cloud/landmark shape signals available as offline artifacts with pass/fail thresholds. The screenshot audit catches gross visual/composition failure, large low-detail scene surfaces, missing or flat terrain material masks, overly compact distant-impostor/cloud silhouettes, and clustered foliage patches, but none of these gates proves atmosphere, fog, materials, vegetation, clouds, route props, or animation look production-quality.
- `entity_count` is still a coarse scene-scale proxy; streaming health should be read from resident island visual count and stream entity churn.
- Route objectives are HUD/debug state backed by pure route objective helpers and serialized into eval samples, but only updraft and branch-recovery routes currently gate objective completion.
- Aerial power-up gates are primitive glowing route rings with simple one-time collection state; there is no inventory UI, reset flow, audio/particles, or authored ability progression yet.
- Summary JSON is emitted by small local helpers rather than a JSON serialization crate to keep the harness dependency-free.

These are acceptable for the current harness. The next meaningful upgrades are asynchronous asset-loading simulation and stricter terrain screenshot review for exact world semantics, occlusion accuracy, and art direction once the generated substrate or imported assets are stable enough for less tolerant classifiers.
