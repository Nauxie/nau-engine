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
- camera-relative grounded and airborne steering on `WASD`, with planar air-control response, rear-diagonal glide steering, smoothed body yaw toward intended movement, bounded lateral reversal spikes, airborne banking, and separate ground friction so walking is playable before launch
- mouse-look third-person follow camera with player-centered orbit pitch, separate yaw/pitch tuning, click-to-lock cursor capture, obstruction avoidance, and surface-clearance clamping
- a 12-island floating archipelago with launch, midpoint, landing, high-altitude, and distant reference islands
- deterministic collision-aware island relief with smoother generated terrain normals, higher-resolution vertex-colored terrain, per-island biome palettes, world-space tiled terrain UVs, encoded terrain material-weight channels, quantized material-region identity, sharper terrain-specific procedural PBR textures with smoothed broad material noise, irregular procedural island rims, generated stratified cliff/underside body meshes, stream-windowed terrain, low-poly distant impostors, and distance-managed detail props: biome-tinted generated terrain colors, ground-cover blades, branched tapered trunks, denser multi-lobed wind-responsive canopies with organic detail cards, stones, ponds, route cairns, launch beacon, and landing-garden markers
- declared glTF visual asset slots for player, glider, island terrain, foliage, rock, water, route-marker, weather, and impostor assets, with self-authored fixture scenes proving Bevy `SceneRoot` load/spawn/readiness across every residency class, visible non-player world fixture placements, named player animation-clip discovery through `Gltf`, `AnimationGraph`, and `AnimationTransitions`, missing/queued/loading/loaded/failed load diagnostics, spawned/ready scene-instance diagnostics, animation-readiness diagnostics, and residency-split metrics while generated gameplay visuals remain the fallback
- Bevy-native atmosphere, dynamic sun/fog/exposure weather, volumetric fog/light, bloom, filmic tonemapping, procedural PBR surface maps, reflective/transmissive water, emissive markers, multi-lobed drifting cloud banks with wisp-card edge detail, layered high-cirrus cloud clusters, and wind-responsive near-LOD environment motion
- simple terrain-surface landing detection with one-shot landing friction
- live debug readout for frame time, speed, altitude, target distance, current route objective, camera pitch/distance/framing angle/motion/obstruction/yaw offset, velocity, aerial power-up visibility/collection/effect state, visual asset slot/load-state/scene-readiness/animation/LOD-residency metrics, visible authored world fixture count, visual wind-field count, lift-field count, sky-island count, terrain surface vertex/color/material-weight/material-region/texture-detail/relief/cliff-band metrics, procedural-vs-primitive island body counts, island body silhouette and mesh min/max complexity, generated tree/cloud mesh complexity, generated detail biome-palette count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, detail counts, environment-motion count/offset, resident/catalog/hidden island visual pressure, and stream spawn/despawn churn
- visible debug gizmos for player velocity, facing, camera line, visual wind/updraft fields, and gameplay lift fields
- authored crosswind fields, a paired gameplay updraft route with aligned visual wind volumes, collectible aerial boost gates with glowing route-ring markers, cinematic lift haze/ribbon/mote cues, and marked recovery branch islands
- background-safe terrain export and audit for offline inspection, writing per-island terrain/cliff/underside OBJ meshes, terrain material-weight CSV sidecars with derived material-region coverage, per-island base/transition/highland/exposed coverage floors, a manifest of mesh/material/texture-detail/texture-edge floors, and an `audit.json` pass/fail report
- repo-native asset fixture audit for every declared glTF fixture, checking provenance, semantic component names, mesh/material/vertex/triangle floors, normals, UVs, blend-material expectations, and the player named animation clip inventory
- deterministic unit tests for movement, ground control, glider, world route, visual wind fields, gameplay lift, camera, diagnostics, eval metrics, and animation-state/pose/airflow math
- scripted eval runs for ground taxi control, mouse camera control, camera yaw/strafe/turn stability, air-control response, baseline traversal, long-glide visibility, updraft lift, branch recovery landing, and island launch-to-landing with traversal, camera, movement-heading/response, rear-right/rear-left lateral and rearward response, grounded visual footing, objective-progress, aerial power-up collection/effect, frame-time, content-scale, generated terrain mesh/color/material-weight/material-region/texture-detail/relief/cliff-band floors, procedural island body, primitive-body, silhouette-complexity, island body mesh floor, generated tree/cloud shape complexity, asset-slot/load-state/scene-instance readiness, visible authored world fixture count, streaming/LOD, spawn/despawn churn, resident pressure, weather-cloud, environment-motion, resident visual, entity-churn, and visible-detail summary metrics plus fixed camera checkpoint screenshots

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
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
```

`tools/eval.sh` runs metric-only evals by default and hides the native window during those runs. It also writes `asset_fixture_audit.json` unless `NAU_EVAL_ASSET_AUDIT=0` is set. Use `NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh ...` when checkpoint PNG artifacts, projection-backed route-marker/scene-sample `.markers.json` sidecars, marker-projection pixel audit, semantic-scene pixel audit, and the non-golden visual audit are needed; screenshot evals require `jq` for artifact extraction, disable debug gizmos, and use an opaque window surface so transparent clouds/updrafts cannot composite against other desktop windows. The visual audit checks image quality plus basic scene composition signals such as per-frame scene coverage, center detail, scene detail tile frequency, flat low-detail scene-tile dominance, player visibility, HUD-text balance, severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, sequence-level route-marker readability/component identity/hue diversity, sequence-level distant horizon/impostor component readability and color-bucket identity, sequence-level terrain/material family diversity, sequence-level foliage coverage, cloud-layer coverage/component identity, and sky coverage across final and checkpoint screenshots. The marker sidecars separately classify known route, objective, or power-up markers as visible, occluded, offscreen, or behind-camera while projecting terrain/foliage/cloud/distant-island scene samples into each checkpoint camera viewport; `marker_projection_audit.json` verifies marker-colored pixels near at least one non-occluded visible marker per checkpoint, and `semantic_scene_audit.json` now requires visible terrain/foliage/cloud/distant-island material families to produce material-like pixels per checkpoint instead of only somewhere in the sequence. Set `NAU_EVAL_VISUAL_AUDIT=0` to collect screenshots without the visual audit.

`./tools/terrain_export.sh target/terrain_export` does not open the native window. It writes `manifest.json`, per-island OBJ meshes, `*_terrain_material_weights.csv` files, and `audit.json` so terrain shape, color variation, topology counts, material-weight coverage, texture-detail and local edge-frequency floors, derived material-region coverage, and minimum base/transition/highland/exposed region distribution can be checked outside the live app. The underlying export can also be run directly with `cargo run -- --export-terrain target/terrain_export`.

`./tools/visual_content_export.sh target/visual_content_export` also runs without opening the native window. It writes a visual-content `manifest.json`, OBJ artifacts, and `audit.json` for generated ground cover, trees, clouds, and biome detail palettes. The audit checks artifact presence plus OBJ vertex/face counts, blade density/height variance, trunk taper, branch reach, canopy lobe/card structure, cloud lobe/wisp/depth floors, and palette diversity so high-vertex blobs cannot silently replace the current generated visual substrate.

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

1. Replace or deepen the self-authored visible fixture scenes with real compatible glTF scenes, starting with a rigged character, production terrain, foliage, water, route props, and richer island impostor kits.
2. Add explicit streaming budget checks around asynchronous asset loading once real imported scenes exist.
3. Refine projection-backed world-semantic screenshot checks beyond route markers into terrain, vegetation, cloud, and distant-impostor classifications.
4. Add a simulation-only eval binary if native-window metric runs become a scaling bottleneck.

## Development Principles

- Tune movement before adding content.
- Instrument behavior before making it more complex.
- Prefer Bevy-native APIs until the project has a measured reason to go lower-level.
- Keep raw Metal out of the codebase unless it is isolated behind a clear renderer boundary and justified by profiling.
