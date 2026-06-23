# The NAU Engine

The NAU Engine is a Mac-first Rust/Bevy sandbox for flight traversal experiments. The project starts with a small, measurable playground rather than a giant world: first make glide, dive, lift, and camera feel good; then scale the world around those mechanics.

## Why This Stack

- **Rust** for performance, explicit systems programming, and a strong open-source package ecosystem.
- **Bevy** for a transparent Rust game engine layer with ECS, rendering, input, assets, cameras, and app structure.
- **wgpu** through Bevy for portable GPU access. On macOS this routes to Metal, without tying the whole project to Apple-only rendering code.
- **Mac-first, not Mac-only** as the default posture. The M-series hardware is the main development target, but the code should stay portable until a measured hotspot proves otherwise.

## Current Sandbox

The first executable is a simple 3D flight testbed:

- primitive humanoid character with separate head, torso, limbs, and visible flight poses
- deployable glider wing panels on `Space`
- one-launch-per-airtime vertical burst on `E`
- dive on `Shift`
- camera-relative grounded and airborne steering on `WASD`, with separate ground friction so walking is playable before launch
- mouse-look third-person follow camera with player-centered orbit pitch, separate yaw/pitch tuning, click-to-lock cursor capture, and surface-clearance clamping
- a small floating sky-island route with launch, midpoint, landing, and distant reference islands
- deterministic island detail props: trees, ponds, stones, launch beacon, and landing-garden markers
- simple route-surface landing detection with one-shot landing friction
- live debug readout for frame time, speed, altitude, target distance, camera pitch/distance/framing angle/yaw offset, velocity, visual wind-field count, lift-field count, and sky-island count
- visible debug gizmos for player velocity, facing, camera line, visual wind/updraft fields, and gameplay lift fields
- authored visual wind fields plus a separate gameplay updraft lift field
- deterministic unit tests for movement, ground control, glider, world route, visual wind fields, gameplay lift, camera, diagnostics, eval metrics, and animation-state math
- scripted eval runs for ground taxi control, mouse camera control, baseline traversal, updraft lift, and island launch-to-landing

This is intentionally not a full physics simulation yet. The first job is to create a place where movement constants can be tuned quickly.

## Getting Started

Install Rust through `rustup`, then run:

```sh
cargo run
```

Useful development checks:

```sh
cargo check
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
./tools/eval.sh ground_taxi_control target/eval/ground_taxi_control
./tools/eval.sh camera_mouse_control target/eval/camera_mouse_control
./tools/eval.sh updraft_route target/eval/updraft_route
./tools/eval.sh island_launch_to_landing target/eval/island_launch_to_landing
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
|`F1`|Toggle debug gizmos|

## Near-Term Roadmap

1. Replace primitive island slabs with authored or generated terrain meshes.
2. Add fixed camera checkpoint screenshots to the island and updraft evals.
3. Add camera collision/obstruction handling.
4. Tune gameplay updraft placement, readability, and recovery routes.
5. Introduce chunked terrain loading with deliberately tiny chunks before making the world large.

## Development Principles

- Tune movement before adding content.
- Instrument behavior before making it more complex.
- Prefer Bevy-native APIs until the project has a measured reason to go lower-level.
- Keep raw Metal out of the codebase unless it is isolated behind a clear renderer boundary and justified by profiling.
