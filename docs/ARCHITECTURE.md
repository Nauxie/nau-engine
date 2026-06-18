# Architecture

The NAU Engine is a Mac-first Bevy project. The current goal is a traversal sandbox, not a general-purpose engine.

## Stack

- Language: Rust
- App/game framework: Bevy
- Renderer path: Bevy renderer -> wgpu -> Metal on macOS
- Asset target: glTF for real 3D models and scenes
- Current physics: custom testable traversal math
- Candidate physics layer: Rapier or Avian, not chosen yet

## Current Code Shape

- `src/main.rs` owns Bevy app setup, scene spawning, input mapping, ECS queries, and visual wiring.
- `src/lib.rs` owns reusable and testable logic.
- `movement` owns flight state, input state, tuning, launch/glide/dive integration, floor clamp, velocity limits, and facing smoothing.
- `camera` owns camera follow math and horizontal follow direction.
- `animation` owns primitive character part pose math, wing visibility state, and animation phase progression.

## Frame Flow

1. Bevy input is read in `fly_player`.
2. Input is mapped into `movement::FlightInput`.
3. `movement::step_flight` produces the next position, velocity, and controller state.
4. Player orientation is smoothed toward horizontal velocity.
5. Character pose phase advances from delta time.
6. `animation::part_pose` maps flight mode and velocity into visible body/glider poses.
7. `camera::step_camera` follows horizontal travel direction and smooths position and rotation.
8. HUD text reports mode, speed, altitude, velocity, cooldown, and launch readiness.

## Core Invariants

- Movement math must stay testable outside a Bevy window.
- `E` launch is ground-gated unless a future launch-source mechanic explicitly changes that.
- Glider traversal descends without wind/updraft/launch-source help.
- Camera follow direction should use horizontal travel direction, not full 3D velocity.
- Camera, animation, and HUD should run after movement.
- `src/main.rs` should stay mostly wiring. Avoid burying gameplay rules directly in ECS systems.
- Do not add raw Metal code until profiling proves a specific renderer hotspot.
- Do not add a broad engine abstraction until at least two real systems need the same boundary.

## Scale Strategy

The route to Zelda-scale is incremental:

1. One stable island slice.
2. Multiple adjacent streamed chunks.
3. Distant impostors and terrain LOD.
4. Floating-origin or origin-rebase strategy.
5. World composition tools and asset rules.

Large-world work should not begin by making a huge map. It should begin by making a small route that exercises streaming, visibility, and traversal distance.

## Asset Strategy

Short term:

- Keep primitives for fast iteration.
- Add debug visuals before polish.
- Use glTF for real character and environment imports.

Medium term:

- Introduce a rigged humanoid character.
- Define animation states for grounded, launch, glide, dive, turn/bank, land, and recover.
- Define island environment asset conventions.

Long term:

- Add LOD variants.
- Add authoring rules for collision, nav/traversal volumes, wind zones, and visual-only geometry.

## Physics Strategy

The project should choose physics deliberately.

Questions for the physics spike:

- Does the character controller need rigid body dynamics or custom kinematic movement?
- How cleanly can we query terrain beneath/around a fast gliding player?
- How good is debug visualization?
- How hard is Bevy integration?
- How costly is simulation if the world is streamed?

Until that choice is made, keep traversal math pure and tested.
