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
- steering on `WASD`
- third-person follow camera
- basic terrain and obstacle markers
- live debug readout for speed, altitude, and velocity
- deterministic unit tests for movement, glider, camera, and animation-state math

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
```

## Controls

|Input|Action|
|-|-|
|`W` / `S`|Accelerate forward/back|
|`A` / `D`|Strafe/steer|
|`Space`|Deploy glider while airborne|
|`E`|Launch upward from the ground|
|`Shift`|Dive|

## Near-Term Roadmap

1. Replace the primitive humanoid with a real rigged character asset.
2. Add manual runtime capture/debug tooling for camera pitch, collision, and frame pacing.
3. Add debug gizmos for velocity, forward vector, glide state, launch cooldown, and camera target.
4. Introduce chunked terrain loading with deliberately tiny chunks before making the world large.
5. Add LOD and culling experiments once flight visibility makes distant terrain matter.

## Development Principles

- Tune movement before adding content.
- Instrument behavior before making it more complex.
- Prefer Bevy-native APIs until the project has a measured reason to go lower-level.
- Keep raw Metal out of the codebase unless it is isolated behind a clear renderer boundary and justified by profiling.
