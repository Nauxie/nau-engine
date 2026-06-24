# The NAU Engine

The NAU Engine is a Mac-first Rust/Bevy sandbox for flight traversal experiments. The project starts with a small, measurable playground rather than a giant world: first make glide, dive, lift, and camera feel good; then scale the world around those mechanics.

## Why This Stack

- **Rust** for performance, explicit systems programming, and a strong open-source package ecosystem.
- **Bevy** for a transparent Rust game engine layer with ECS, rendering, input, assets, cameras, and app structure.
- **wgpu** through Bevy for portable GPU access. On macOS this routes to Metal, without tying the whole project to Apple-only rendering code.
- **Mac-first, not Mac-only** as the default posture. The M-series hardware is the main development target, but the code should stay portable until a measured hotspot proves otherwise.

## Current Sandbox

The first executable is a simple 3D flight testbed:

- primitive humanoid character with separate head, torso, limbs, grounded stride poses, airborne banking, speed-responsive glider wing flex, and visible flight poses
- deployable glider wing panels with subtle wingtip airflow trails on `Space`
- one-launch-per-airtime vertical burst on `E`
- dive on `Shift`
- camera-relative grounded and airborne steering on `WASD`, with planar air-control response, smoothed body yaw toward intended movement, airborne banking, and separate ground friction so walking is playable before launch
- mouse-look third-person follow camera with player-centered orbit pitch, separate yaw/pitch tuning, click-to-lock cursor capture, obstruction avoidance, and surface-clearance clamping
- a 12-island floating archipelago with launch, midpoint, landing, high-altitude, and distant reference islands
- deterministic collision-aware island relief with smoother generated terrain normals, stream-windowed terrain, low-poly distant impostors, and distance-managed detail props: varied generated terrain colors, batched ground-cover blades, wind-responsive trees and ponds, stones, route cairns, launch beacon, and landing-garden markers
- declared glTF visual asset slots for future player, glider, island, route-marker, weather, and impostor assets, with runtime missing/queued/loading/loaded/failed and residency-split diagnostics while primitives remain the active implementation
- Bevy-native atmosphere, dynamic sun/fog/exposure weather, volumetric fog/light, bloom, filmic tonemapping, procedural PBR surface maps, reflective/transmissive water, emissive markers, drifting cloud banks, high cirrus veils, and wind-responsive near-LOD environment motion
- simple terrain-surface landing detection with one-shot landing friction
- live debug readout for frame time, speed, altitude, target distance, current route objective, camera pitch/distance/framing angle/motion/obstruction/yaw offset, velocity, aerial power-up visibility/collection/effect state, visual asset slot/load-state/LOD-residency readiness, visual wind-field count, lift-field count, sky-island count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, detail counts, environment-motion count/offset, resident/catalog/hidden island visual pressure, and stream spawn/despawn churn
- visible debug gizmos for player velocity, facing, camera line, visual wind/updraft fields, and gameplay lift fields
- authored crosswind fields, a paired gameplay updraft route with aligned visual wind volumes, collectible aerial boost gates with glowing route-ring markers, cinematic lift haze/ribbon/mote cues, and marked recovery branch islands
- deterministic unit tests for movement, ground control, glider, world route, visual wind fields, gameplay lift, camera, diagnostics, eval metrics, and animation-state/pose/airflow math
- scripted eval runs for ground taxi control, mouse camera control, camera yaw/strafe/turn stability, air-control response, baseline traversal, long-glide visibility, updraft lift, branch recovery landing, and island launch-to-landing with traversal, camera, movement-heading/response, objective-progress, aerial power-up collection/effect, frame-time, content-scale, asset-slot/load-state readiness, streaming/LOD, spawn/despawn churn, resident pressure, weather-cloud, environment-motion, resident visual, entity-churn, and visible-detail summary metrics plus fixed camera checkpoint screenshots

This is intentionally not a full physics simulation yet. The first job is to create a place where movement constants can be tuned quickly.

## Getting Started

Install Rust through `rustup`, then run:

```sh
cargo run
# repo-local alias:
cargo naux
```

Useful development checks:

```sh
cargo check
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
./tools/eval.sh ground_taxi_control target/eval/ground_taxi_control
./tools/eval.sh camera_mouse_control target/eval/camera_mouse_control
./tools/eval.sh camera_yaw_stability target/eval/camera_yaw_stability
./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
./tools/eval.sh camera_strafe_stability target/eval/camera_strafe_stability
./tools/eval.sh air_control_response target/eval/air_control_response
./tools/eval.sh updraft_route target/eval/updraft_route
./tools/eval.sh branch_recovery_route target/eval/branch_recovery_route
./tools/eval.sh long_glide_visibility target/eval/long_glide_visibility
./tools/eval.sh island_launch_to_landing target/eval/island_launch_to_landing
```

`tools/eval.sh` runs metric-only evals by default and hides the native window during those runs. Use `NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh ...` when checkpoint PNG artifacts and the non-golden visual audit are needed; screenshot evals require `jq` for artifact extraction, disable debug gizmos, and use an opaque window surface so transparent clouds/updrafts cannot composite against other desktop windows. The audit checks image quality plus basic scene composition signals such as per-frame scene coverage, center detail, player visibility, HUD-text balance, severe border clipping, sequence-level route-marker readability and identity, and sequence-level sky coverage across final and checkpoint screenshots. Set `NAU_EVAL_VISUAL_AUDIT=0` to collect screenshots without the audit.

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

1. Replace placeholder visual asset slots with real glTF scenes, starting with character/glider scale and island impostor kits.
2. Add explicit streaming budget checks around asynchronous asset loading once real imported scenes exist.
3. Add richer terrain and route-marker identity signals on top of the current non-golden screenshot audit.
4. Add a simulation-only eval binary if native-window metric runs become a scaling bottleneck.

## Development Principles

- Tune movement before adding content.
- Instrument behavior before making it more complex.
- Prefer Bevy-native APIs until the project has a measured reason to go lower-level.
- Keep raw Metal out of the codebase unless it is isolated behind a clear renderer boundary and justified by profiling.
