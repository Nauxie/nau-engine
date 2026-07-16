# NAU Engine

Mac-first Rust/Bevy flight traversal sandbox built around one playable loop: launch from the terrain, deploy the glider, read wind and lift, collect a route of one-shot air gates, land on an island or the world floor, then relaunch or reset.

The current world combines **41 floating islands** with a streamed playable world floor. Twelve non-overlapping air gates form a three-gate main corridor plus optional low and high-altitude branches; the top-right HUD tracks unique collections, and `Esc` opens a compact pause, controls, and quit menu.

<p align="center">
  <img src="docs/showcase/air_gate_route_midflight.png" alt="Nau gliding through the twelve-gate floating-island route" width="100%">
</p>

<p align="center">
  <img src="docs/showcase/world_floor_far_route.png" alt="The air route seen above the streamed playable world floor" width="49%">
  <img src="docs/showcase/world_floor_distant_islands.png" alt="Distant floating islands above the playable world floor" width="49%">
</p>

## Current Playable State

- Launch, glide, steer, dive, air-brake, collect gates, land, recover, and relaunch across floating-island and world-floor terrain.
- Traverse authored updrafts, crosswind lanes, visible airflow cues, and gust-varied wind that affects both player motion and glider response.
- Fly a self-authored player/glider fixture with grounded, launch, fall, bank, glide, dive, brake, landing-anticipation, and recovery animation states.
- Collide with island terrain, cliff bodies, trees, rocks, landmarks, and world-floor terrain; use the central-island reset when a route breaks down.
- Read distinct island ecologies and histories through species-varied trees, dense flora clusters, ruin precincts, geological formations, and water-linked shoreline and waterfall detail.
- Review movement, camera, collision, route progress, streaming, visual readability, fixture integrity, and release performance through measured sim/app evals, screenshot audits, exports, and scripted play profiles.

See the [Showcase](SHOWCASE.md) for gameplay, world scale, UI, and player/glider review captures. The [island-surface content contract](docs/ISLAND_SURFACE_CONTENT.md) documents the generated ecology, ruins, geology, water detail, budgets, and regression coverage. Detailed world-floor acceptance mechanics and evidence live in the [decision record](docs/DECISIONS/0002-world-floor-perf-first.md), [requirements audit](docs/world-floor-requirements-audit.md), and [acceptance report](docs/world-floor-acceptance-report.md).

## Run

Install Rust with `rustup`. Ordinary development play keeps Bevy and rendering dependencies optimized while retaining debug assertions:

```sh
cargo run -- --play
```

Use release mode for final play-feel and performance review:

```sh
cargo run --release -- --play
```

The first development build is slower because it compiles optimized dependencies; later builds remain incremental. Debug mode enables the diagnostic readout and gizmos:

```sh
cargo run -- --debug
```

Repo-local alias:

```sh
cargo naux
```

## Controls

|Input|Action|
|-|-|
|`W` / `S`|Accelerate forward/back|
|`A` / `D`|Strafe/steer|
|Mouse|Look while locked or while right mouse is held|
|Left click|Lock and hide the mouse cursor|
|Right mouse|Temporarily enable mouse look|
|`Esc`|Open the pause menu; return from controls|
|`Space`|Deploy the glider while airborne|
|`E`|Launch upward from the ground|
|`Shift`|Dive|
|`R`|Reset to the central playtest island|
|`F1`|Toggle debug gizmos in `--debug` mode|
|`F2`|Toggle collision proxies in `--debug` mode|

Air gates score once per session and disappear after collection. The HUD and pause menu show the same authoritative total.

## Verification

Core Rust checks:

```sh
cargo check --all-targets
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Representative measured tooling (`jq` is required by the aggregate scripts):

```sh
./tools/dev_play_performance_gate.sh target/eval/dev_play_performance
./tools/camera_continuity_gate.sh target/eval/camera_continuity
./tools/manual_play_profile.sh target/eval/play_profile/manual.json
NAU_PLAY_PROFILE_DURATION_SECS=300 NAU_PLAY_PROFILE_SCRIPT=freeflight \
  ./tools/manual_play_profile.sh target/eval/play_profile/freeflight-5min.json
./tools/eval_sim_suite.sh target/eval/sim_suite
cargo run -- --eval long_glide_visibility --eval-output target/eval/long_glide_visibility
./tools/eval.sh great_sky_plateau_vistas target/eval/great_sky_plateau_vistas
NAU_EVAL_SCREENSHOT=1 NAU_EVAL_ASSET_AUDIT=0 \
  ./tools/eval.sh island_surface_review target/eval/island_surface_review
./tools/player_pose_preview.sh target/player_pose_preview
./tools/world_content_gate.sh target/world_content_gate
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
./tools/wind_visual_export.sh target/wind_visual_export
```

The development-play gate runs the same full-content native scenario in debug and release, enforces frame-time ceilings, and rejects debug builds that drift materially behind release. The camera-continuity gate covers deterministic 30/60/120/144 Hz and hitch contracts plus native mouse, reset, collision, obstruction, and floor-boundary scenarios. Both are required by the macOS CI workflow.

Release comparison and world-floor regression gates remain available through `./tools/perf_baseline.sh`, `./tools/world_floor_full_gate.sh`, and `./tools/world_floor_candidate_gate.sh`. Human release play remains part of acceptance after behavioral changes; the linked acceptance documents define the full contract.

Resolved diagnostic note: a host-specific Apple Silicon presentation hitch was isolated below NAU gameplay and required no game workaround; the probe and evidence live in [`tools/frame_hitch_probe`](tools/frame_hitch_probe/README.md).

## Docs

- [Architecture](docs/ARCHITECTURE.md)
- [Project Status](docs/STATUS.md)
- [Eval Spec](docs/EVAL_SPEC.md)
- [Island Surface Content](docs/ISLAND_SURFACE_CONTENT.md)
- [Flight Mechanics](docs/MECHANICS/flight.md)
- [Roadmap](docs/ROADMAP.md)
- [Showcase](SHOWCASE.md)

## Principles

- Keep the playable loop small and measurable.
- Add debug visualization before complex behavior.
- Prefer Bevy-native systems until a measured limitation appears.
- Keep traversal constants visible and easy to tune.
- Avoid raw Metal unless profiling proves it is needed behind a clear boundary.
