# Roadmap

This roadmap is horizon-based, not date-based. The point is to preserve direction across sessions without pretending we know the exact schedule.

## Horizon 0: Project Foundation

Status: mostly complete.

- Rust/Bevy project scaffold
- public GitHub repo
- Mac-first wgpu/Metal path
- primitive flight sandbox
- testable movement/camera/animation modules
- basic project docs

Exit criteria:

- `cargo fmt --all --check`, `cargo check`, `cargo test`, and clippy pass on `main`.
- A future session can understand the project by reading `docs/STATUS.md`, `docs/ARCHITECTURE.md`, and `docs/MECHANICS/flight.md`.

## Horizon 1: Traversal Feel

Goal: make the core launch/glide/dive loop feel deliberate and measurable.

Work:

- Add runtime debug overlay for camera pitch, camera distance, frame time, velocity, altitude, glide state, and launch state.
- Add manual test routes: launch, glide, dive, low-altitude recovery, landing, obstacle pass.
- Add camera collision/obstruction handling.
- Add camera mode profiles for launch, glide, dive, and ground.
- Add bank/turn behavior that feels like glider traversal rather than free flight.
- Add wind/updraft test volumes after baseline gliding is stable.

Exit criteria:

- From a standing start, the player can launch, deploy glider, steer, dive, land, and repeat without obvious camera snaps or limb jitter.
- The glider cannot gain altitude without a defined force such as launch, wind, or updraft.
- Known feel regressions are captured by tests or debug metrics.

## Horizon 2: Character And Animation Pipeline

Goal: replace the primitive character with a believable humanoid traversal avatar.

Work:

- Import a rigged glTF humanoid.
- Define animation clips and blend states.
- Add glider mesh attachment points.
- Add launch, glide, dive, turn, land, and idle states.
- Add animation debugging for current clip/state/weight.

Exit criteria:

- The character reads as human-like at gameplay camera distance.
- Glider deployment is visually attached to the character.
- State transitions do not pop or visibly detach limbs/gear.

## Horizon 3: Physics And World Interaction

Goal: make terrain, collision, and immersive forces real enough to build worlds on.

Work:

- Spike Rapier and Avian.
- Choose kinematic character controller vs custom controller with physics queries.
- Add terrain collision and slope/ground detection.
- Add trigger volumes for launch sources, wind, updrafts, hazards, and checkpoints.
- Add basic dynamic objects that react to wind or impact.

Exit criteria:

- Player movement queries real collision geometry.
- Wind/updraft volumes affect traversal in a deterministic and debuggable way.
- Physics debug visualization can be toggled on.

## Horizon 4: Island Slice

Goal: build one high-quality island route before attempting a massive world.

Work:

- Import or generate an island terrain mesh.
- Add water plane, sky, fog, lighting, shadows, and PBR materials.
- Add launch point, glide route, landing target, and recovery path.
- Add simple vegetation/rocks/landmarks.
- Add route timing and traversal metrics.

Exit criteria:

- One island can be launched into, crossed, descended onto, and visually inspected.
- The environment supports the traversal loop rather than just looking decorative.

## Horizon 5: Scale

Goal: support Zelda-level traversal distances without pretending the whole world is active.

Work:

- Chunk streaming.
- Terrain and prop LOD.
- Distant impostors.
- Floating origin or origin rebase.
- Async asset loading.
- Visibility/culling rules.
- Per-chunk collision activation.

Exit criteria:

- The player can fly far enough that streaming and LOD are exercised.
- Memory and frame time remain observable and bounded.
- Distant views look coherent from glider altitude.

## Horizon 6: High-Fidelity Environment Systems

Goal: support wind, weather, atmosphere, water, vegetation, and effects as systems.

Work:

- Visual wind on grass, cloth/glider, particles, clouds, and water.
- Gameplay wind fields integrated with traversal.
- Atmospheric fog and distance haze.
- Volumetric or layered cloud experiments.
- Water shader and shoreline treatment.
- Lighting and time-of-day experiments.

Exit criteria:

- Wind is both visible and mechanically meaningful.
- The player can read wind/updraft opportunities from the environment.
- Environment fidelity does not hide traversal readability.
