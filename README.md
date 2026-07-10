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

The world-floor revamp checkpoint is accepted. It provides playable near-original-level terrain, uses one shared terrain sampler for rendering and gameplay, preserves island-surface precedence, and maintains a streamed `3x3` visible window backed by a pool bounded to `25` tiles.

Automated tooling:

```sh
./tools/world_floor_full_gate.sh
./tools/world_floor_candidate_gate.sh
```

The accepted checkpoint has clean main-vs-candidate app perf results plus both mandatory foreground profiles:

```sh
NAU_PLAY_PROFILE_SCRIPT=freeflight ./tools/scripted_play_profile.sh target/eval/play_profile/candidate_scripted_freeflight.json
NAU_PLAY_PROFILE_SCRIPT=ground_traversal ./tools/scripted_play_profile.sh target/eval/play_profile/candidate_scripted_ground_traversal.json
```

Automation must verify sampler parity, island precedence, visible-window coverage, the `25`-tile pool bound, frame-time and hitch budgets, mesh/material/triangle cost, and stream churn. Screenshots support visual review but cannot accept the feature.

The final gate is a human release playtest on the target Mac: land on the world floor, walk, run, launch, and fly back into traversal while checking collision/grounding, transitions, camera feel, visible coverage, frame pacing, fan, and heat. This manual sequence was completed for the accepted checkpoint and remains mandatory after future behavioral changes; it has no automated waiver. A visual-only plane at `y=-260`, a one-tile floor, or a report generated only from automated evidence is not success.

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
|Esc|Open the pause menu; return from controls|
|`Space`|Deploy glider while airborne|
|`E`|Launch upward from the ground|
|`Shift`|Dive|
|`R`|Reset to the central playtest island|
|`F1`|Toggle debug gizmos in `--debug` mode|

The top-right HUD tracks unique aerial boost gates collected during the current session. Open
the compact pause menu for the same score, the controls reference, resume, and quit actions.

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
