# NAU Engine

Mac-first Rust/Bevy flight sandbox for tuning traversal feel before growing into a broader engine.

The current focus is narrow: make glide, dive, lift, camera framing, authored player motion, and route readability measurable enough that changes can be reviewed by eye and by eval output.

<p align="center">
  <img src="docs/showcase/great_sky_plateau_high_crossing.png" alt="Glider crossing the high archipelago route" width="100%">
</p>

<p align="center">
  <img src="docs/showcase/updraft_high_glide.png" alt="Glider approaching visible updraft columns and route markers" width="49%">
  <img src="docs/showcase/player_rig_stress_review_sheet.png" alt="Player rig stress review sheet" width="49%">
</p>

## What Exists

- A small third-person flight playground with launch, glide, dive, air-brake, landing anticipation, and landing recovery states.
- A self-authored player/glider fixture with pose-intent animation, connector checks, silhouette review, and visual pose sheets.
- Floating-island traversal routes with generated terrain, water, clouds, route markers, updraft ribbons, crosswind cues, and debug overlays.
- Scripted evals for movement, camera stability, route progress, collision, fixture readiness, visual readability, and screenshot audits.

## Visual Review

These are checked-in copies of generated eval artifacts. The source outputs live under `target/eval/`, `target/player_pose_preview/`, and export-specific `target/` folders after running the relevant scripts.

See [Showcase](SHOWCASE.md) for the fuller image set.

<p align="center">
  <img src="docs/showcase/long_glide_archipelago_midroute.png" alt="Long glide archipelago midroute screenshot" width="49%">
  <img src="docs/showcase/great_sky_plateau_summit_climb.png" alt="Great sky plateau summit climb screenshot" width="49%">
</p>

<p align="center">
  <img src="docs/showcase/player_glider_attachment_sheet.png" alt="Player and glider attachment pose sheet" width="49%">
  <img src="docs/showcase/player_motion_integrity_review_sheet.png" alt="Player motion integrity review sheet" width="49%">
</p>

## Run

Install Rust with `rustup`, then:

```sh
cargo run --release -- --play
```

Use release mode for judging play feel and performance. Debug mode keeps the readout and gizmos available for development:

```sh
cargo run -- --debug
```

Repo-local alias:

```sh
cargo naux
```

## Useful Checks

```sh
cargo check
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Release performance baseline:

```sh
./tools/perf_baseline.sh
```

The perf baseline writes per-scenario app eval artifacts plus `perf_summary.json`, `perf_summary.tsv`, and interpretation notes under `target/eval/perf_baseline/`.

Visual and fixture artifacts:

```sh
./tools/player_pose_preview.sh target/player_pose_preview
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh long_glide_visibility target/eval/long_glide_visibility
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh updraft_route target/eval/updraft_route
./tools/eval_sim_suite.sh target/eval/sim_suite
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
./tools/wind_visual_export.sh target/wind_visual_export
```

## Controls

|Input|Action|
|-|-|
|`W` / `S`|Accelerate forward/back|
|`A` / `D`|Strafe/steer|
|Mouse|Look while locked or while right mouse is held|
|Left click|Lock and hide the mouse cursor|
|Esc|Release the mouse cursor|
|`Space`|Deploy glider while airborne|
|`E`|Launch upward from the ground|
|`Shift`|Dive|
|`R`|Reset to the central playtest island|
|`F1`|Toggle debug gizmos in `--debug` mode|

## Docs

- [Architecture](docs/ARCHITECTURE.md)
- [Eval Spec](docs/EVAL_SPEC.md)
- [Flight Mechanics](docs/MECHANICS/flight.md)
- [Roadmap](docs/ROADMAP.md)
- [Showcase](SHOWCASE.md)

## Principles

- Keep the playable loop small and measurable.
- Add debug visualization before complex behavior.
- Prefer Bevy-native systems until a measured limitation appears.
- Keep traversal constants visible and easy to tune.
- Avoid raw Metal unless profiling proves it is needed behind a clear boundary.
