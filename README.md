# The NAU Engine

The NAU Engine is a Mac-first Rust/Bevy sandbox for flight traversal experiments. The project starts with a small, measurable playground rather than a giant world: first make glide, dive, lift, and camera feel good; then scale the world around those mechanics.

## Why This Stack

- **Rust** for performance, explicit systems programming, and a strong open-source package ecosystem.
- **Bevy** for a transparent Rust game engine layer with ECS, rendering, input, assets, cameras, and app structure.
- **wgpu** through Bevy for portable GPU access. On macOS this routes to Metal, without tying the whole project to Apple-only rendering code.
- **Mac-first, not Mac-only** as the default posture. The M-series hardware is the main development target, but the code should stay portable until a measured hotspot proves otherwise.

## Current Sandbox

The first executable is a simple 3D flight testbed:

- primitive humanoid character with separate head, torso, limbs, grounded walk/run stride, readable launch/fall, glide, deeper arms-out dive, air-brake, landing-anticipation flare/crouch, post-touchdown landing recovery, turn-readable airborne lean, speed-responsive glider wing flex, and measurable key-pose readability
- deployable glider wing panels with subtle wingtip airflow trails and visible authored traversal response on `Space`
- one-launch-per-airtime vertical burst on `E`
- dive on `Shift`
- camera-relative grounded and airborne steering on `WASD`, with planar air-control response, rear-diagonal glide steering, smoothed body yaw and bank toward intended movement, bounded lateral reversal spikes, body-local fallback pose lean, and separate ground friction so walking is playable before launch
- mouse-look third-person follow camera with player-centered orbit pitch, separate yaw/pitch tuning, click-to-lock cursor capture, obstruction avoidance, and surface-clearance clamping
- a 20-island floating archipelago with launch, midpoint, landing, high-altitude, branch, far-horizon, and satellite reference islands
- deterministic collision-aware island relief with 19 named terrain archetypes, route-aligned ravine/channel incisions, terrace/shelf/basin/ridge/needle/satellite variation, fine microrelief, smoother generated terrain normals, higher-resolution vertex-colored terrain, per-island biome palettes, world-space tiled terrain UVs, encoded terrain material-weight channels, quantized material-region identity, sharper terrain-specific procedural PBR textures with smoothed broad material noise, irregular procedural island rims shared by terrain and ground containment, generated stratified cliff/underside body meshes, stream-windowed terrain, low-poly distant impostors, and distance-managed detail props: biome-tinted generated terrain colors, ground-cover blades, collidable multi-ring branched trunks with root flares, denser multi-lobed wind-responsive canopies with organic detail cards, collidable stones, irregular pond surfaces, collidable stacked route cairns, a collidable crystalized launch beacon, and organic landing-garden markers
- declared glTF visual asset slots for player, glider, island terrain, foliage, rock, water, route-marker, weather, and impostor assets, with deeper self-authored fixture scenes carrying registry-aligned NAU metadata, proving Bevy `SceneRoot` load/spawn/readiness across every residency class, visible non-player world fixture placements, named player animation-clip discovery through `Gltf`, pose-intent-driven `AnimationGraph`/`AnimationTransitions`, named authored player node pose parity, deterministic load admission, missing/deferred/queued/loading/loaded/failed load diagnostics, spawned/ready scene-instance diagnostics, animation-readiness diagnostics, and residency-split metrics while generated gameplay visuals remain the fallback
- Bevy-native atmosphere, dynamic sun/fog/exposure weather, volumetric fog/light, bloom, filmic tonemapping, procedural PBR surface maps, reflective/transmissive water, emissive markers, denser five-layer drifting cloud banks with wisp-card edge and filament-ribbon detail, layered high-cirrus cloud clusters, wind-responsive near-LOD environment motion, and shared dynamic `WindField` flow for gusting updraft and crosswind ribbons/motes
- simple terrain-surface landing detection with one-shot landing friction
- live debug readout for frame time, speed, altitude, target distance, current route objective, camera pitch/distance/framing angle/motion/obstruction/yaw offset, velocity, aerial power-up visibility/collection/effect state, visual asset slot/load-state/scene-readiness/animation/LOD-residency metrics, visible authored world fixture count, visual wind-field count, lift-field count, world-collision and terrain-rim proxy/resolution/push metrics, sky-island count, terrain surface vertex/color/material-weight/material-region/texture-detail/relief/cliff-band metrics, procedural-vs-primitive island body counts, island body silhouette and mesh min/max complexity, generated tree/cloud mesh and cloud filament-ribbon complexity, generated landmark counts, generated detail biome-palette count, active chunk window, near/mid/far LOD island buckets, visible/hidden terrain, impostor, detail counts, environment-motion count/offset, resident/catalog/hidden island visual pressure, body-roll/bank response, and stream spawn/despawn churn
- visible debug gizmos for player velocity, facing, camera line, visual wind/updraft fields, and gameplay lift fields
- authored crosswind fields with animated guide ribbons/motes, a paired gameplay updraft route with aligned visual wind volumes, collectible aerial boost gates with glowing route-ring markers, dynamic gusting lift haze/ribbon/mote cues, and marked recovery branch islands
- background-safe terrain export and audit for offline inspection, writing per-island terrain/cliff/underside OBJ meshes, terrain material-weight CSV sidecars with derived material-region coverage, terrain-archetype diversity, per-island base/transition/highland/exposed presence floors plus stronger aggregate archipelago coverage floors, manifest and OBJ height-band/normal-slope-band floors, mesh/material/texture-detail/texture-edge floors, and an `audit.json` pass/fail report
- repo-native asset fixture audit for every declared glTF fixture, checking provenance, semantic component names, strengthened mesh/material/vertex/triangle floors, normals, UVs, blend-material expectations, and the player named animation clip inventory
- deterministic unit tests for movement, ground control, glider, world route, visual wind fields, gameplay lift, camera, diagnostics, eval metrics, and animation-state/pose-intent/pose-readability/airflow math
- scripted eval runs for ground taxi control, world-collision contact, terrain-rim contact, mouse camera control, camera yaw/strafe/turn stability, air-control response, pose-state coverage, baseline traversal, long-glide visibility, updraft lift, branch recovery landing, and island launch-to-landing with traversal, camera, movement-heading/response, body-roll/bank response, readable pose-intent coverage, pose torso/arm/leg/turn/landing/recovery/wing-airflow readability metrics, explicit walk/run/launch/fall/glide pose coverage, authored dive/air-brake/landing clip coverage, visible authored glider response/motion metrics, key-pose readability scoring, global and landing-only pose temporal gates, rear-right/rear-left lateral and rearward response, grounded visual footing, objective-progress, aerial power-up collection/effect, dynamic wind-flow speed/variation/range, updraft/crosswind guide/ribbon visual count, motion, rise, direction-flow, and shared-field flow-coherence gates, frame-time, content-scale, generated terrain mesh/color/material-weight/material-region/texture-detail/relief/archetype/cliff-band floors, procedural island body, primitive-body, silhouette-complexity, island body mesh floor, generated tree/cloud/filament shape complexity, generated landmark count/per-kind/mesh floors, world-collision proxy count plus sustained meaningful-contact and peak-push gates, terrain-rim proxy/contact gates with no-contact ground-taxi protection, asset-slot/load-state/scene-instance readiness, visible authored world fixture count, streaming/LOD, spawn/despawn churn, resident pressure, weather-cloud, environment-motion, resident visual, entity-churn, and visible-detail summary metrics plus fixed camera checkpoint screenshots whose semantic scene sidecars carry `terrain_surface` material variant identity and audit aggregate terrain material/biome variant diversity

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
./tools/eval.sh world_collision_contact target/eval/world_collision_contact
./tools/eval.sh terrain_rim_collision_contact target/eval/terrain_rim_collision_contact
./tools/eval.sh camera_mouse_control target/eval/camera_mouse_control
./tools/eval.sh camera_yaw_stability target/eval/camera_yaw_stability
./tools/eval.sh camera_turn_stability target/eval/camera_turn_stability
./tools/eval.sh camera_strafe_stability target/eval/camera_strafe_stability
./tools/eval.sh air_control_response target/eval/air_control_response
./tools/eval.sh pose_state_coverage target/eval/pose_state_coverage
./tools/eval.sh updraft_route target/eval/updraft_route
./tools/eval.sh branch_recovery_route target/eval/branch_recovery_route
./tools/eval.sh long_glide_visibility target/eval/long_glide_visibility
./tools/eval.sh island_launch_to_landing target/eval/island_launch_to_landing
./tools/eval_sim_suite.sh target/eval/sim_suite
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
```

`tools/eval.sh` runs metric-only evals by default and hides the native window during those runs. Set `NAU_EVAL_SIM_ONLY=1` to run `traversal_sim_eval`, which exercises the shared scripted input, movement, route, lift, power-up, and camera-follow math without creating a Bevy app or native window. `./tools/eval_sim_suite.sh target/eval/sim_suite` runs every simulation-supported scripted scenario through that path, writes one aggregate `summary.json`, and runs the asset fixture audit once instead of once per scenario; app-only visual/runtime routes such as `world_collision_contact` and `terrain_rim_collision_contact` should be run through the default app eval path. The default path also writes `asset_fixture_audit.json` unless `NAU_EVAL_ASSET_AUDIT=0` is set; that audit now requires each glTF fixture's `extras.nau` metadata to match the asset registry kind, label, residency class, schema, and self-authored license contract, and it gates richer world-fixture semantic fragments plus mesh/material/vertex/triangle floors for terrain erosion/path detail, foliage roots/ferns/moss, water ripples/lily/specular detail, rock seams, route glyphs, cloud haze/filaments, and distant waterfall/broken-cliff impostor detail. Use `NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh ...` when checkpoint PNG artifacts, projection-backed route-marker/scene-sample `.markers.json` sidecars, marker-projection pixel audit, semantic-scene pixel audit, and the non-golden visual audit are needed; screenshot evals require `jq` for artifact extraction, disable debug gizmos, and use an opaque window surface so transparent clouds/updrafts cannot composite against other desktop windows. The visual audit checks image quality plus basic scene composition signals such as per-frame scene coverage, center detail, scene detail tile frequency, flat low-detail scene-tile dominance, player visibility, HUD-text balance, severe border clipping, non-opaque PNG alpha, large foreign bright-canvas regions, sequence-level route-marker readability/component identity/hue diversity, sequence-level distant horizon/impostor component readability/color-bucket/span identity, sequence-level terrain/material family diversity plus terrain material coverage/color/tile spread, sequence-level foliage coverage/tile spread, cloud-layer coverage/component/span identity, and sky coverage across final and checkpoint screenshots. The marker sidecars separately classify known route, objective, or power-up markers as visible, occluded, offscreen, or behind-camera while projecting and visibility-classifying terrain/foliage/cloud/distant-island scene samples into each checkpoint camera viewport; `marker_projection_audit.json` verifies marker-colored pixels near at least one non-occluded visible marker per checkpoint, and `semantic_scene_audit.json` requires visible terrain/foliage/cloud/distant-island material families to produce material-like pixels per checkpoint, requires enough distinct visible scene sample kinds per checkpoint, requires visible projected samples plus pixel hits for each expected kind (`terrain_surface`, `tree_canopy`, `weather_cloud`, and `distant_island`) across the checkpoint sequence, and now requires aggregate material/kind pixel-coverage floors so one tiny matching speck cannot satisfy a projected world-quality gate. Set `NAU_EVAL_VISUAL_AUDIT=0` to collect screenshots without the visual audit.

`./tools/terrain_export.sh target/terrain_export` does not open the native window. It writes `manifest.json`, per-island OBJ meshes, `*_terrain_material_weights.csv` files, and `audit.json` so terrain shape, color variation, topology counts, material-weight coverage, texture-detail and local edge-frequency floors, manifest and OBJ height-band/normal-slope-band floors, derived material-region coverage, per-island material-region presence, and aggregate base/transition/highland/exposed region distribution can be checked outside the live app. The underlying export can also be run directly with `cargo run -- --export-terrain target/terrain_export`.

`./tools/visual_content_export.sh target/visual_content_export` also runs without opening the native window. It writes a visual-content `manifest.json`, OBJ artifacts, and `audit.json` for generated ground cover, trees, clouds, route/launch/landing/pond landmarks, and biome detail palettes. The audit checks artifact presence plus OBJ vertex/face counts, blade density/height variance, multi-ring trunk mesh floors, trunk taper, branch reach/count, root-flare count, canopy lobe/card structure, tree height/canopy-radius variation, cloud veil plus lobe/wisp/filament/depth-span floors, generated landmark mesh/count/span floors, and palette diversity so high-vertex blobs, stick-like trees, or primitive route props cannot silently replace the current generated visual substrate.

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

1. Replace the deepened self-authored fixture scenes with real compatible glTF scenes, starting with a rigged character, production terrain, foliage, water, route props, and richer island impostor kits.
2. Grow the deterministic visual-asset admission budget into asynchronous distance streaming once real imported scenes outnumber the current always-loaded fixture manifest.
3. Add per-biome terrain/material screenshot checks beyond the current broad terrain mask.
4. Expand async asset-loading simulation once real imported scenes outnumber the current fixture manifest.

## Development Principles

- Tune movement before adding content.
- Instrument behavior before making it more complex.
- Prefer Bevy-native APIs until the project has a measured reason to go lower-level.
- Keep raw Metal out of the codebase unless it is isolated behind a clear renderer boundary and justified by profiling.
