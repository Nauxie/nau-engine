# Eval Harness

The eval harness exists to make traversal work measurable before the world gets larger. It should answer three questions on every iteration:

- Did the playable route still run end to end?
- What changed in movement, camera, visibility, and runtime scale?
- Which artifacts should a human or agent inspect next?

The harness is repo-native. The game owns deterministic input, state collection, screenshot capture, and pass/fail checks. Shell scripts and future agent orchestration should only run the app, collect artifacts, and interpret the report.

## Current Command

Run the baseline route with screenshot capture:

```sh
./tools/eval.sh
```

Run the app directly:

```sh
cargo run -- --eval baseline_route --eval-output target/eval/baseline_route
```

Run without screenshot capture, useful for faster local checks or environments where the native window cannot render:

```sh
cargo run -- --eval baseline_route --eval-output target/eval/baseline_route --eval-no-screenshot
```

## Current Scenario

`baseline_route` is a deterministic scripted traversal smoke test:

- fixed `1 / 60` movement timestep
- fixed spawn at `PLAYER_START`
- launch on frame 1
- forward input after startup
- glider deployment after launch
- one short dive segment
- left and right steering segments
- metrics sampled every 10 frames and at the final frame
- summary written after 420 frames
- optional final screenshot written as `final.png`

This is not yet a sky-island route completion test. It is the first contract that proves launch, glide, dive, camera follow, debug scene visibility, output writing, and autonomous app exit work.

## Artifacts

Each run writes to the eval output directory:

- `samples.ndjson`: newline-delimited per-sample telemetry.
- `summary.json`: pass/fail checks, aggregate metrics, artifact paths, and final state.
- `final.png`: final rendered screenshot when screenshot capture is enabled.

The summary is the primary artifact for agents. Screenshots are for visual review and should not be treated as pixel-perfect golden images.

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
- `camera_pitch_degrees`
- `visible_wind_fields`
- `wind_field_count`
- `entity_count`

Add fields here before adding them to code. New fields should be cheap to collect, stable across runs, and useful for deciding what to fix.

## Summary Metrics

The summary aggregates:

- sample count
- horizontal distance from first to final sample
- max and min altitude
- max speed
- max camera distance
- min and max camera pitch
- max visible wind-field count
- gliding, launching, and grounded sample counts

The pass/fail checks currently guard:

- enough samples were written
- the route covered enough horizontal distance
- launch produced enough altitude
- the route spent enough sampled frames gliding
- camera distance stayed under a loose maximum

Thresholds should remain loose until the intended route becomes richer. Tight thresholds belong only after a mechanic or route is deliberately locked.

## Scaling Rules

As the world grows, extend the harness in this order:

1. Add scenario-specific scripted routes.
2. Add metrics that explain known failure modes.
3. Add low-cost assertions around those metrics.
4. Add screenshots from fixed camera checkpoints.
5. Add visual comparison or computer-vision checks only when the raw metrics are insufficient.

Do not start with pixel-perfect screenshots. Metal/wgpu/native-window output can shift slightly across machines and driver state. Visual evals should classify obvious failures: blank frame, missing terrain, player not visible, severe clipping, unreadable route, or incoherent composition.

## Future Scenarios

The thin-slice target should eventually have these evals:

- `baseline_route`: current smoke test.
- `island_launch_to_landing`: launch from one floating island and land on another.
- `long_glide_visibility`: verify many distant islands remain visible during high-altitude flight.
- `camera_stress`: fly close to geometry and record camera distance, pitch, and obstruction metrics.
- `streaming_route`: cross chunk boundaries and record active chunks, spawned entities, despawns, and frame time.
- `updraft_route`: verify explicit gameplay lift only after wind/updraft rules are accepted.

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

- The current eval still opens a native Bevy window.
- The current screenshot is a final-frame capture only.
- There is no simulation-only binary yet.
- There is no frame-time percentile summary yet.
- There is no terrain/chunk/island route metric yet.
- `entity_count` is a coarse scale proxy, not a streaming health metric.
- Summary JSON is emitted by small local helpers rather than a JSON serialization crate to keep the harness dependency-free.

These are acceptable for the first harness. The next meaningful upgrade is a real island-to-island scenario with route completion, landing target distance, fixed camera checkpoints, and chunk/terrain counters.
