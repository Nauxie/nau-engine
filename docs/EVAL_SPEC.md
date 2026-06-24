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

`updraft_route` is the first traversal power-up eval:

- fixed spawn on the launch island
- scripted launch, glide, and steering into the gameplay lift field
- the route must register active lift samples
- every sampled active lift frame must also register a paired visible updraft field so gameplay lift cannot silently drift away from its readable signal
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

- scripted launch, glide deployment, alternating left/right air turns, and a late backward air-brake segment
- camera step distance and rotation delta must remain under thresholds during rapid heading changes
- the route must keep gliding samples and traversal distance high enough to avoid a no-op pass

`air_control_response` is the airborne movement-feel regression test:

- scripted launch, glide deployment, diagonal air steering, pure right/left air steering, backward braking, a short dive, and forward recovery
- no scripted camera input, so camera orbit yaw offset must remain zero while `A`/`D` move Nau
- actual camera rotation delta must stay small during the movement-only route
- average and p95 camera follow-direction error must stay bounded so a stable camera cannot hide a stale follow target
- camera world-yaw drift must stay bounded so movement input cannot quietly rotate the camera
- average and p95 desired body heading error plus desired-heading velocity alignment must stay within thresholds
- right and left lateral input must each produce measurable response within the response-latency threshold
- backward input must produce measurable air-brake speed drop, and the final forward segment must recover forward alignment
- body-yaw oscillation count must remain bounded so input reversals do not become spin or wobble regressions

`camera_strafe_stability` is the lateral-movement camera regression test:

- fixed spawn on the launch island
- no mouse input, launch, glide, or dive
- scripted `D` and `A` segments exercise lateral ground motion
- camera view yaw must stay near the starting heading so strafe velocity cannot auto-orbit the camera
- max speed and grounded samples must prove the route was not a no-op

The baseline route remains a fast smoke test. The island route is the stronger signal for traversal/content regressions. The ground taxi route guards the pre-launch controls that airborne evals can miss. The updraft route proves the first gameplay power-up remains measurable and visually signaled. The branch recovery route proves a named alternate landing island can be targeted, reached, and validated without changing the primary landing-garden route. The long-glide route guards the first larger-map slice before real despawn, asset streaming, or richer impostor work exists. The mouse-camera route guards the control surface that manual play will feel immediately but movement-only evals miss. The yaw-stability route guards against persistent mouse yaw being fed back into the camera every frame. The strafe-stability route guards against `A`/`D` movement being treated as camera orbit input. The turn-stability route guards rapid airborne direction changes and backward air braking. The air-control route guards the actual flight-feel response that manual play exposed as jank.

## Artifacts

Each run writes to the eval output directory:

- `samples.ndjson`: newline-delimited per-sample telemetry.
- `summary.json`: pass/fail checks, optional named target island, aggregate metrics, artifact paths, and final state.
- `final.png`: final rendered screenshot when screenshot capture is enabled.
- `checkpoints/*.png`: fixed-frame camera screenshots when screenshot capture is enabled.
- `visual_audit.json`: non-golden image audit for screenshot evals run through `tools/eval.sh` unless `NAU_EVAL_VISUAL_AUDIT=0` is set.

The summary is the primary artifact for agents. Screenshots are for visual review and should not be treated as pixel-perfect golden images.
`tools/eval.sh` checks that declared PNG artifacts exist, are large enough, and pass a lightweight visual audit for resolution, nonblack/nonwhite exposure, luma variance, color variety, edge density, per-frame scene coverage, per-frame center detail, per-frame scene detail tile frequency, per-frame flat low-detail scene-tile dominance, per-frame player visibility, per-frame severe border clipping, per-frame HUD-text dominance, sequence-level route-marker readability, sequence-level route-marker component identity, route-marker hue-family telemetry, and sequence-level top-sky coverage across the final screenshot plus fixed checkpoints. The audit catches gross render, composition, and low-frequency scene-detail failures; it does not prove terrain material identity, exact marker semantics, vegetation/cloud mesh quality, or AAA-quality art direction.

## Sample Fields

Every sample includes:

- `frame`
- `time_secs`
- `position`
- `velocity`
- `speed_mps`
- `altitude_m`
- `mode`
- `desired_body_yaw_error_degrees`
- `desired_body_heading_error_degrees`
- `desired_heading_alignment_mps`
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
- `island_terrain_surface_count`
- `min_island_terrain_mesh_vertices`
- `min_island_terrain_color_bands`
- `min_island_terrain_relief_range_m`
- `min_island_cliff_color_bands`
- `procedural_island_body_count`
- `primitive_island_body_count`
- `min_island_body_silhouette_segments`
- `avg_island_body_silhouette_segments`
- `max_island_body_mesh_vertices`
- `generated_tree_trunk_count`
- `generated_tree_canopy_count`
- `min_tree_trunk_mesh_vertices`
- `min_tree_canopy_mesh_vertices`
- `generated_weather_cloud_count`
- `min_weather_cloud_lobe_count`
- `max_weather_cloud_lobe_count`
- `min_weather_cloud_mesh_vertices`
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
- `queued_visual_asset_scene_count`
- `loading_visual_asset_scene_count`
- `loaded_visual_asset_scene_count`
- `failed_visual_asset_scene_count`
- `spawned_visual_asset_scene_count`
- `ready_visual_asset_scene_count`
- `always_visual_asset_slot_count`
- `stream_window_visual_asset_slot_count`
- `near_lod_visual_asset_slot_count`
- `far_lod_visual_asset_slot_count`
- `weather_visual_asset_slot_count`
- `declared_animation_clip_count`
- `ready_animation_clip_count`
- `animation_player_count`
- `animation_graph_count`
- `power_up_count`
- `visible_power_up_count`
- `collected_power_up_count`
- `active_power_up_effects`
- `total_power_up_activations`

Add fields here before adding them to code. New fields should be cheap to collect, stable across runs, and useful for deciding what to fix.

The island terrain/detail/impostor hidden counts are catalog entries that are not currently resident. `catalog_island_visual_count`, `hidden_island_visual_count`, and `resident_island_visual_fraction` report stream pressure directly so future optimization work does not have to infer it from per-layer fields. The `stream_visibility_*` names are retained for artifact compatibility, but now report resident island visual spawn/despawn churn rather than `Visibility` flag flips; the `stream_spawned_*` and `stream_despawned_*` fields split that churn into directional budget signals.
The visual asset fields report the declared glTF scene inventory, how many slots are loaded and ready, how many are still using generated placeholders, how many Bevy scene handles are queued/loading/loaded/failed, how many optional scene instances have spawned/reported ready, how many named player animation clips are declared/ready, whether nested animation players were linked, whether animation graphs were prepared, and how slots divide across always-loaded, stream-window, near-LOD, far-LOD, and weather residency classes. Missing files and failed loads count as placeholder-backed; queued/loading files are not counted as ready until Bevy reports them loaded. These are readiness signals for replacing primitives with real assets; they do not prove final art quality yet.
The power-up fields report authored aerial boost gates, how many remain visible, how many have been collected, whether an effect is currently active, and the total activation count. They are route-readiness signals for the simple power-up slice, not final ability design.
The environment-motion fields report how many resident near-LOD visuals are wind-responsive and the largest sampled transform offset from their base placement. They prove the visual motion layer exists and is active; they do not evaluate final animation quality.
The island-terrain fields report generated terrain surface count, minimum terrain mesh vertex count, minimum vertex-color band count, minimum sampled terrain relief range, and minimum cliff/underside color-band count. They are structural signals for denser terrain and stratified rock detail; they do not replace screenshot or human review for final material quality.
The island-body fields report whether the catalogued route island bodies are generated procedural meshes or registered primitive/fallback body placeholders, plus the minimum silhouette segment count and body mesh vertex count signal. They are a structural signal for replacing cylinder-like islands; they do not prove final terrain art quality or texture fidelity.

## Summary Metrics

The summary aggregates:

- sample count
- average frame time
- p95 frame time
- p99 frame time
- max frame time
- horizontal distance from first to final sample
- max and min altitude
- max speed
- average, p95, and max desired body-heading error
- max body-yaw error step and body-yaw oscillation count
- max desired-heading velocity alignment
- max lateral response speed and first-response latency
- max right and left lateral response speed and response latency
- max air-brake speed drop and max post-brake forward alignment
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
- max visible island detail count
- max hidden island detail count
- max visible route beacon count
- max weather cloud count
- max environment-motion visual count
- max environment-motion offset
- min island terrain surface count
- min island terrain mesh vertex count
- min island terrain vertex-color band count
- min island terrain relief range
- min island cliff color-band count
- min procedural island body count
- max primitive island body count
- min island body silhouette segment count
- max average island body silhouette segment count
- max island body mesh vertex count
- min generated tree trunk/canopy counts
- min generated tree trunk/canopy mesh vertex counts
- min generated weather cloud count
- min/max weather cloud lobe counts
- min weather cloud mesh vertex count
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
- max queued/loading/loaded/failed visual asset scene counts
- max spawned/ready visual asset scene instance counts
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
- gliding, launching, and grounded sample counts

The pass/fail checks currently guard:

- enough samples were written
- the route covered enough horizontal distance
- launch produced enough altitude
- max speed crossed the scenario floor
- the route spent enough sampled frames gliding
- the route spent enough sampled frames grounded
- the route spent enough sampled frames inside gameplay lift when a scenario requires it
- lift-required scenarios spend enough sampled active-lift frames inside a paired visible updraft field
- lift-required scenarios have zero unreadable active-lift samples
- the world has enough sky islands to catch accidental route collapse
- the active chunk window stays inside the scenario budget
- enough islands enter the active chunk window
- near/mid/far LOD island buckets remain populated
- visible island terrain stays under the scenario budget, proving inactive chunks are not drawing full terrain
- hidden island terrain stays populated, proving inactive chunk terrain remains measurable before real despawn work
- visible island impostors stay populated, proving inactive chunks retain distant silhouettes
- visible island detail stays under the scenario budget, proving distance LOD is active
- hidden island detail stays populated, proving distance LOD is actually culling resident detail
- visible route beacons stay populated so distant route readability is not culled away
- objective totals stay populated, and objective-route scenarios complete their required objective count
- declared visual asset slots, glTF scene slots, and stream-managed asset slots stay populated so the real-asset migration surface cannot silently disappear
- failed visual asset scene count remains zero so broken imported assets fail the eval loop once real files are present
- authored aerial power-up counts stay populated, and power-up scenarios collect enough gates with enough sampled active-effect frames
- weather cloud count stays populated so the first non-debug weather layer cannot silently disappear
- environment-motion visual count and offset stay populated so wind-responsive near-LOD trees/ponds cannot silently disappear or freeze
- island terrain surface count, mesh vertex floor, vertex-color band floor, relief-range floor, and cliff color-band floor stay populated so the generated world cannot silently regress to lower-resolution, flatter, or visually single-tone island surfaces
- procedural island body count stays populated throughout the run so sky-island bodies cannot silently disappear or fall below the expected generated catalog
- registered primitive island body count remains zero so explicit fallback body placeholders fail the eval loop
- island body silhouette segment count stays above the scenario floor so the route cannot collapse back to low-resolution round islands
- resident island visuals stay under budget while streaming visibility is still hide/show based
- stream visibility changes per frame stay under budget so chunk/LOD crossings do not churn too many visuals at once
- hidden island visual count stays populated and resident visual fraction stays under budget so the catalog does not collapse into always-resident rendering
- spawned and despawned island visuals each stay under the per-frame churn budget so future asset streaming can distinguish load pressure from unload pressure
- the scene has enough entities to catch accidental content collapse
- camera distance stayed under a loose maximum
- camera stayed above the active ground surface
- camera kept the player focus near the camera centerline
- camera per-frame movement and rotation stayed under scenario jerk thresholds
- camera orbit alignment stayed under threshold
- camera view yaw and world-yaw drift stayed within scenario limits when movement should not rotate the camera
- camera obstruction avoidance was exercised when a scenario requires it
- camera mouse scenarios exercised yaw and both pitch directions
- air-control response latency, right/left lateral response, air-brake speed drop, post-brake forward alignment, desired-heading alignment, average/p95 body-heading error, yaw oscillation count, camera orbit yaw offset, and camera rotation delta stayed inside thresholds
- air-control average and p95 camera follow-direction error stayed inside threshold so movement-only routes cannot pass with a stale follow direction
- air-control movement-only camera world-yaw drift stayed inside threshold
- island-route final scenario-target distance stayed under threshold
- island-route grounded target landing was observed on the configured target island

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
- `updraft_route`: current gameplay lift regression test.
- `branch_recovery_route`: current named branch landing and recovery-route test.
- `long_glide_visibility`: current larger-archipelago traversal, aerial boost-gate, and content-scale test.
- `camera_mouse_control`: current mouse X/Y regression test.
- `camera_yaw_stability`: current small-yaw no-drift regression test.
- `camera_turn_stability`: current rapid air-turn and air-brake camera stability test.
- `camera_strafe_stability`: current `A`/`D` no-auto-orbit camera stability test.
- `air_control_response`: current diagonal/lateral/brake/recovery air-control response test.
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

- Metric-only evals hide the native Bevy window, but still instantiate the window/rendering stack.
- Screenshot evals still need a visible native Bevy window. Screenshot runs disable debug gizmos and use opaque surface composition so transparent scene effects blend against the game frame rather than desktop content behind the window.
- Screenshot evals now run a lightweight image and scene-composition audit, including basic player visibility, scene detail tile frequency, flat low-detail scene-tile dominance, severe border-clipping, route-marker readability, route-marker component identity, and route-marker hue-family telemetry, but terrain material identity, exact marker semantics, vegetation/cloud mesh quality, and art quality still need human/agent inspection.
- There is no simulation-only binary yet.
- Frame-time metrics skip the first few warmup frames and are recorded as local native-window runtime telemetry; they are useful for trend spotting, not stable cross-machine pass/fail thresholds.
- Island collision follows deterministic authored terrain relief, but it is still a route-surface clamp rather than full rigid-body physics.
- `active_chunk_count` and `active_island_count` drive resident terrain/detail entities, and visual asset slots are declared, counted, and split by residency class, but there is no asynchronous asset streaming policy yet.
- Missing glTF files are counted as placeholders and intentionally do not trigger load errors; only files that exist under `assets/` are queued through Bevy's `AssetServer`, and queued handles then report queued/loading/loaded/failed state. `ready_visual_asset_slot_count` means Bevy has loaded the scene asset, not merely that the file exists. `spawned_visual_asset_scene_count` and `ready_visual_asset_scene_count` track Bevy scene-instance lifecycle separately from load state. `declared_animation_clip_count`, `ready_animation_clip_count`, `animation_player_count`, and `animation_graph_count` track the player scene's named clip/graph path separately from scene readiness; the eval checks gate the declared clip inventory so the future player asset contract cannot silently disappear.
- LOD buckets drive resident island detail, inactive or non-near chunks use cheap impostors, and hidden/resident/churn counters quantify stream-window pressure.
- The weather-cloud, generated tree/cloud shape, and environment-motion counters verify that cloud-layer entities, lobe counts, mesh vertex floors, and wind-responsive near-LOD visual motion exist. Focused unit tests assert that generated trunks, canopies, and cloud clusters are no longer single cylinder/sphere meshes. The screenshot audit now catches gross visual/composition failure and large low-detail scene surfaces, but neither proves atmosphere, fog, materials, vegetation, clouds, or animation look correct.
- `entity_count` is still a coarse scene-scale proxy; streaming health should be read from resident island visual count and stream entity churn.
- Route objectives are HUD/debug state backed by pure route objective helpers and serialized into eval samples, but only updraft and branch-recovery routes currently gate objective completion.
- Aerial power-up gates are primitive glowing route rings with simple one-time collection state; there is no inventory UI, reset flow, audio/particles, or authored ability progression yet.
- Summary JSON is emitted by small local helpers rather than a JSON serialization crate to keep the harness dependency-free.

These are acceptable for the current harness. The next meaningful upgrades are asynchronous asset-loading simulation, richer terrain/material identity checks, vegetation/cloud-depth image checks, exact route-marker semantic checks, and a simulation-only eval binary if native-window metric runs become a scaling bottleneck.
