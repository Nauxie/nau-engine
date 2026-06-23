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

Run the lateral strafe camera stability route:

```sh
./tools/eval.sh camera_strafe_stability target/eval/camera_strafe_stability
```

Request screenshot artifacts explicitly:

```sh
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
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
- max altitude must exceed the normal route ceiling
- baseline and island routes must not accidentally hit the lift field

`long_glide_visibility` is the larger-archipelago traversal eval:

- fixed spawn on the launch island
- scripted launch, glide, and wide steering across the expanded island chain
- the route must cover hundreds of meters without leaving camera thresholds
- the distant gameplay updraft must keep altitude high enough for a sustained glide
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

`camera_strafe_stability` is the lateral-movement camera regression test:

- fixed spawn on the launch island
- no mouse input, launch, glide, or dive
- scripted `D` and `A` segments exercise lateral ground motion
- camera view yaw must stay near the starting heading so strafe velocity cannot auto-orbit the camera
- max speed and grounded samples must prove the route was not a no-op

The baseline route remains a fast smoke test. The island route is the stronger signal for traversal/content regressions. The ground taxi route guards the pre-launch controls that airborne evals can miss. The updraft route proves the first gameplay power-up remains measurable and isolated. The long-glide route guards the first larger-map slice before actual streaming, despawn, or impostors exist. The mouse-camera route guards the control surface that manual play will feel immediately but movement-only evals miss. The yaw-stability route guards against persistent mouse yaw being fed back into the camera every frame. The strafe-stability route guards against `A`/`D` movement being treated as camera orbit input. The turn-stability route guards rapid airborne direction changes and backward air braking.

## Artifacts

Each run writes to the eval output directory:

- `samples.ndjson`: newline-delimited per-sample telemetry.
- `summary.json`: pass/fail checks, aggregate metrics, artifact paths, and final state.
- `final.png`: final rendered screenshot when screenshot capture is enabled.
- `checkpoints/*.png`: fixed-frame camera screenshots when screenshot capture is enabled.

The summary is the primary artifact for agents. Screenshots are for visual review and should not be treated as pixel-perfect golden images.
`tools/eval.sh` also checks that declared PNG artifacts exist and are large enough to catch empty or blank early-frame captures.

## Sample Fields

Every sample includes:

- `frame`
- `time_secs`
- `position`
- `velocity`
- `speed_mps`
- `altitude_m`
- `mode`
- `camera_distance_m`
- `camera_surface_clearance_m`
- `camera_player_angle_degrees`
- `camera_pitch_degrees`
- `camera_yaw_offset_degrees`
- `camera_pitch_offset_degrees`
- `camera_step_distance_m`
- `camera_rotation_delta_degrees`
- `camera_orbit_alignment_degrees`
- `camera_view_yaw_degrees`
- `camera_obstruction_adjustment_m`
- `camera_obstruction_hits`
- `visible_wind_fields`
- `wind_field_count`
- `active_lift_fields`
- `lift_field_count`
- `target_distance_m`
- `on_landing_target`
- `sky_island_count`
- `active_chunk_count`
- `active_island_count`
- `near_lod_islands`
- `mid_lod_islands`
- `far_lod_islands`
- `visible_island_terrain_count`
- `visible_island_impostor_count`
- `visible_island_detail_count`
- `visible_route_beacon_count`
- `entity_count`

Add fields here before adding them to code. New fields should be cheap to collect, stable across runs, and useful for deciding what to fix.

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
- max camera distance
- min camera surface clearance
- max camera-to-player framing angle
- max per-frame camera step distance
- max per-frame camera rotation delta
- max camera orbit alignment
- max absolute camera view yaw
- max camera obstruction adjustment
- max camera obstruction hit count
- min and final target distance
- min and max camera pitch
- max absolute camera yaw offset
- min and max camera pitch offset
- max visible wind-field count
- max active lift-field count
- max sky-island count
- max active chunk count
- max active island count
- max near/mid/far LOD island counts
- max visible island terrain count
- max visible island impostor count
- max visible island detail count
- max visible route beacon count
- max scene entity count
- target landing sample count
- active lift sample count
- gliding, launching, and grounded sample counts

The pass/fail checks currently guard:

- enough samples were written
- the route covered enough horizontal distance
- launch produced enough altitude
- max speed crossed the scenario floor
- the route spent enough sampled frames gliding
- the route spent enough sampled frames grounded
- the route spent enough sampled frames inside gameplay lift when a scenario requires it
- the world has enough sky islands to catch accidental route collapse
- the active chunk window stays inside the scenario budget
- enough islands enter the active chunk window
- near/mid/far LOD island buckets remain populated
- visible island terrain stays under the scenario budget, proving inactive chunks are not drawing full terrain
- visible island impostors stay populated, proving inactive chunks retain distant silhouettes
- visible island detail stays under the scenario budget, proving distance LOD is active
- visible route beacons stay populated so distant route readability is not culled away
- the scene has enough entities to catch accidental content collapse
- camera distance stayed under a loose maximum
- camera stayed above the active ground surface
- camera kept the player focus near the camera centerline
- camera per-frame movement and rotation stayed under scenario jerk thresholds
- camera orbit alignment stayed under threshold
- camera view yaw stayed within scenario limits when movement should not rotate the camera
- camera obstruction avoidance was exercised when a scenario requires it
- camera mouse scenarios exercised yaw and both pitch directions
- island-route final target distance stayed under threshold
- island-route grounded target landing was observed

Thresholds should remain loose until the intended route becomes richer. Tight thresholds belong only after a mechanic or route is deliberately locked.

## Scaling Rules

As the world grows, extend the harness in this order:

1. Add scenario-specific scripted routes.
2. Add metrics that explain known failure modes.
3. Add low-cost assertions around those metrics.
4. Add visual comparison or computer-vision checks only when the raw metrics and checkpoint screenshots are insufficient.

Do not start with pixel-perfect screenshots. Metal/wgpu/native-window output can shift slightly across machines and driver state. Visual evals should classify obvious failures: blank frame, missing terrain, player not visible, severe clipping, unreadable route, or incoherent composition.

## Future Scenarios

The thin-slice target should eventually have these evals:

- `baseline_route`: current smoke test.
- `island_launch_to_landing`: current route-completion test.
- `ground_taxi_control`: current pre-launch WASD regression test.
- `updraft_route`: current gameplay lift regression test.
- `long_glide_visibility`: current larger-archipelago traversal and content-scale test.
- `camera_mouse_control`: current mouse X/Y regression test.
- `camera_yaw_stability`: current small-yaw no-drift regression test.
- `camera_turn_stability`: current rapid air-turn and air-brake camera stability test.
- `camera_strafe_stability`: current `A`/`D` no-auto-orbit camera stability test.
- `camera_stress`: fly close to geometry and record camera distance, pitch, and obstruction metrics.
- `streaming_route`: cross chunk boundaries and record active chunks, active islands, spawned entities, despawns, and frame time.

## Agent Loop Contract

A future Codex or orchestrator loop should:

1. Read this spec and `summary.json`.
2. Inspect `samples.ndjson` only for the failing or suspicious interval.
3. Inspect screenshots only when the summary points to a visual, camera, terrain, or visibility issue.
4. Make one narrow change.
5. Run `cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and the relevant eval.
6. Commit the checkpoint with the eval artifacts path in the commit or PR notes when useful.

The repo should remain the durable memory. Do not depend on a past chat session to know what the eval means.

## Known Limitations

- Metric-only evals hide the native Bevy window, but still instantiate the window/rendering stack.
- Screenshot evals still need a visible native Bevy window.
- Screenshot checks still rely on human/agent inspection rather than image classification.
- There is no simulation-only binary yet.
- Frame-time metrics skip the first few warmup frames and are recorded as local native-window runtime telemetry; they are useful for trend spotting, not stable cross-machine pass/fail thresholds.
- Island collision is a simple route surface clamp, not full physics.
- `active_chunk_count` and `active_island_count` drive terrain visibility, but they do not despawn entities or load assets yet.
- LOD buckets drive visible island detail, and inactive chunks swap full terrain for cheap impostors; hidden terrain/detail entities still remain resident.
- `entity_count` is still a coarse scene-scale proxy, not a streaming health metric, because inactive visuals are hidden rather than despawned.
- Summary JSON is emitted by small local helpers rather than a JSON serialization crate to keep the harness dependency-free.

These are acceptable for the current harness. The next meaningful upgrades are frame-time percentiles, real chunk activation/despawn counters, and visual checks for missing or blank island geometry.
