# Performance E2E report — 2026-07-18

Verdict: **pass with follow-ups**.

The tested candidate has healthy sustained gameplay pacing on the Apple M4 Max test host. No CPU bottleneck, GPU recovery, swap activity, or >33 ms runtime frame was observed in clean foreground runs. Dense freeflight is GPU-pressure dominant, and long-run memory confidence remains incomplete because this automation session could not foreground the game for the planned five-minute soak.

## Test subject

- Commit: `1802a007b661b0571763ab819597b746dd01f257`
- Source state: dirty camera-fix worktree
- Source fingerprint: `d5d2c9efd4f47b031275a13f51181d677052003db731828e9edcb34d84cbdd94`
- Candidate binary SHA-256: `78391ff6b4962f03d4dc77d43fd63fc402cea26453421f998e87b477c1e7797e`
- Host: macOS 26.5.2, Apple M4 Max, 16 CPU cores, 40 GPU cores, 64 GiB unified memory
- Display path: built-in 3024×1964 display; foreground profiles observed the 120 Hz path

## Sustained frame pacing

Startup warmup is excluded from the runtime metrics below.

|Workload|Avg ms|p95 ms|p99 ms|Max ms|>33 ms|Entities|Triangles|
|-|-|-|-|-|-|-|-|
|Freeflight, 60 s|8.458|8.685|16.737|25.164|0|5,058|1,365,030|
|Ground traversal, 60 s|8.334|8.708|8.889|16.225|0|4,620|1,166,422|
|Baseline route|8.759|16.552|16.977|18.777|0|5,005|1,258,550|
|Long glide visibility|8.514|8.660|16.878|18.180|0|5,066|1,307,094|
|Camera mouse control|8.579|8.721|17.112|18.590|0|4,836|1,235,510|
|Great Sky Plateau route|8.442|8.659|16.559|18.935|0|5,059|1,372,710|

The 60-second freeflight profile covered 1,365 m with a focused ratio of 1.0. Its single 25.164 ms hitch did not coincide with a streaming change. Ground traversal covered 659 m with no frame over 16.67 ms.

The debug/release camera gate also passed:

- Debug runtime: 8.629 ms average, 8.646 ms p95, 20.194 ms maximum.
- Release runtime: 8.577 ms average, 8.778 ms p95, 18.803 ms maximum.
- Debug/release average ratio: 1.006.
- Startup maxima were 126–173 ms and are tracked separately from sustained play.

## CPU, memory, and GPU

The lightweight probes ran with the full asset set and did not attach invasive profilers.

|Probe|CPU mean / max|RSS mean / max|Late RSS drift, 20 s+|GPU mean / max|Recovery / split|
|-|-|-|-|-|-|
|Freeflight|1.33 / 1.46 cores|508.57 / 512.34 MiB|1.84 MiB/min|70.94% / 93.00%|0 / 0|
|Ground traversal|1.23 / 1.28 cores|463.74 / 464.59 MiB|1.13 MiB/min|45.94% / 50.00%|0 / 0|

Interpretation:

- CPU is not the limiting resource on this host in the measured workloads.
- Dense freeflight is the pressure case. Its system-wide GPU device counter averaged 71% and briefly reached 93%, versus 46% mean during ground traversal.
- Process RSS rapidly loaded, then approached a plateau. The late-window drift is small but is not sufficient to prove leak-freedom.
- No run-level page faults, swaps, system swap deltas, GPU recovery events, or split scenes occurred in the lightweight probes.
- The invasive freeflight probe measured a 1.7 GiB physical footprint. About 1.2 GiB was graphics-owned unmapped unified memory; process malloc zones held 357 MiB resident and 267 MiB allocated.

The CPU sample did not expose one runaway gameplay system. Recurring optimization candidates were:

- `WindField::flow_at` and trigonometric work.
- wgpu staging-buffer creation and queue writes.
- render-pass encoding.
- render extraction and presentation synchronization.

These are optimization opportunities, not demonstrated blockers at the current scene complexity.

## Visual and content validation

The Great Sky Plateau capture passed all of its visual, marker-projection, semantic-scene, and nine-fixture asset audits. Captures show the terrain, ruins, waterfall, wind guides, route markers, and surrounding islands rendered with the expected authored content.

The long-glide scenario and marker/asset audits passed, but its semantic checkpoint gate recorded only one of three required checkpoint pixel hits and zero wind checkpoint hits. The rendered screenshots contain the expected scene, so this is currently classified as a checkpoint/classifier coverage defect.

![Visual contact sheet](../target/eval/performance_e2e_2026-07-18/renderings/visual_contact_sheet.jpg)

## Findings and required follow-ups

1. **GPU headroom — medium.** Dense freeflight reaches 93% system-wide GPU utilization on an M4 Max. Add a representative-hardware GPU/content-density trend gate before increasing visible complexity.
2. **Stale entity budget — medium.** `great_sky_plateau_route` is frame-healthy but fails at 5,059 entities because it resets the release limit to 5,000 while the related long-glide release limit is 5,500. Define the intentional release budget once.
3. **Asset-root packaging — medium.** A raw release binary launched without `BEVY_ASSET_ROOT` searches `target/release/assets` and silently renders placeholders. Package required assets beside the binary or fail loudly.
4. **Startup hitches — low.** All-frame startup maxima reached 126–173 ms even though post-warmup runtime maxima stayed below 21 ms. Keep startup and sustained-play gates separate and move avoidable first-frame work later.
5. **Five-minute soak coverage — medium.** Two soak attempts failed closed at the 30-second arming timeout because the automation host retained foreground focus. Raw executable, `.app` bundle, System Events, AppKit, and CoreGraphics activation paths all failed to make the game active. Run the same soak interactively; profiling tools must never take foreground focus automatically.
6. **Visual harness defects — low.** Fix the Bash 3.2 empty-array failure, reframe the camera pitch checkpoint whose markers are all offscreen, and repair the long-glide semantic checkpoint coverage.

## Follow-up — 2026-07-22

The July 18 results above remain historical evidence. A later microstutter investigation found both real avoidable frame work and a correctness hole in the visible play profiler:

- A valid 60-second 120 Hz freeflight profile after the terrain-shader optimization measured 8.477/10.554/16.827/19.768 ms average/p95/p99/max with 1.754% missed-refresh frames. A 30-second ground profile measured 8.335/10.192/11.452/13.305 ms with 0.0595% missed-refresh frames.
- Lowering MSAA to Sample2, disabling shadows, and reducing the directional shadow map to 2048 did not improve the measured path and were rejected.
- A terrain-bypass diagnostic changed from 120 Hz to 60 Hz during collection. The legacy report incorrectly evaluated the whole run against the final 60 Hz cadence, so that result is invalid for comparison. Play-profile schema v5 now reads the native current monitor through winit, records refresh transitions, evaluates pacing against the highest observed refresh rate, and fails any run whose monitor refresh changes or cannot be observed.
- The render audit found globally resident transparent weather/wind geometry and distant island impostors using the full terrain shader. Atmospheric visuals now have explicit fade/cull ranges, and distant impostors use the cheaper biome-matched standard ground material.
- The CPU audit found repeated per-entry island residency/contour work, fresh streaming collections, and stable world-floor reconciliation. Residency is now cached once per island, streaming scratch storage is reused, and settled floor windows skip redundant work.
- Automated play profiling no longer requests or steals foreground focus.

These fixes are structurally testable without launching the game, but their final visual and feel impact still requires an explicit user-controlled manual A/B. That foreground validation was intentionally deferred when testing was paused, so this follow-up does not claim the remaining microstutter is fully resolved.

## Controls and limitations

- Clean-main gating attempts correctly refused to launch while the host was busy. Forced main runs fell onto a 60 Hz host/presentation path, so they are not valid performance A/B evidence.
- Candidate and clean-main scene signatures matched exactly for entities, meshes, materials, triangles, and residency. The camera patch therefore did not alter render-content complexity.
- IORegistry GPU counters are system-wide rather than per-process.
- `powermetrics` requires superuser access and `xctrace` is not installed, so this report does not include power, thermal-energy, or Metal timeline captures.
- `sample`, `vmmap`, and `heap` caused aligned frame hitches; their run is excluded from clean pacing conclusions.

## Verification

- `cargo fmt --check` — passed.
- `cargo check --all-targets --all-features` — passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo test --all-targets --all-features` — passed.
- `bash -n tools/manual_play_profile.sh tools/compare_manual_play_profiles.sh tools/scripted_play_profile.sh tools/camera_continuity_gate.sh` — passed.

Cargo emitted only the existing future-incompatibility notice for `block v0.1.6`. No graphical play or native eval executable was launched during this follow-up verification.

## Artifacts

- [Human-readable report](../target/eval/performance_e2e_2026-07-18/REPORT.md)
- [Machine-readable report](../target/eval/performance_e2e_2026-07-18/report.json)
- [Standalone HTML report](../target/eval/performance_e2e_2026-07-18/report.html)
- [Frame-pacing chart](../target/eval/performance_e2e_2026-07-18/renderings/frame_pacing.png)
- [Resource chart](../target/eval/performance_e2e_2026-07-18/renderings/resources.png)
- [Resource time series](../target/eval/performance_e2e_2026-07-18/renderings/resource_timeseries.png)
- [Scene-complexity chart](../target/eval/performance_e2e_2026-07-18/renderings/scene_complexity.png)
- [Visual contact sheet](../target/eval/performance_e2e_2026-07-18/renderings/visual_contact_sheet.jpg)

Raw profiles, NDJSON samples, host snapshots, screenshots, profiler captures, and failed-control evidence are under `target/eval/performance_e2e_2026-07-18/`.
