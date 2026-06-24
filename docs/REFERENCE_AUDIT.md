# Reference Audit

Last updated: 2026-06-24

This document records local code audits for external Bevy references used to steer NAU work. These repositories are references only; do not copy assets or large code blocks into NAU without a separate license/provenance check.

## Audited Repositories

- `janhohenheim/foxtrot` at `db0746f`
- `olekspickle/bevy_new_3d_rpg` at `54d1dbb`
- `manankarnik/bevy_generative` at `74a17cc`

The current local audit used refreshed clones under `/tmp/nau-reference` so source, manifests, documentation, shader/material examples, and license text were available for direct inspection. Earlier sparse-clone notes remain useful, but this pass treated the repositories as codebases to audit rather than search results.

## Foxtrot

Foxtrot is the strongest reference for a mature Bevy `0.18` application structure. It is not a template for NAU's traversal mechanics, but it shows how a real Bevy project wires third-party plugins, loading states, render settings, glTF scenes, assets, and debug systems.

Audited files included `Cargo.toml`, `readme.md`, `src/main.rs`, `src/asset_tracking.rs`, `src/gameplay/player/input.rs`, `src/gameplay/player/camera.rs`, `src/gameplay/player/assets.rs`, `src/gameplay/player/animation.rs`, `src/dev_tools/validate_preloading.rs`, and `src/props/specific/burning_logs.rs`.

Patterns worth adopting:

- Keep one wrapper module per third-party plugin. Foxtrot's `src/third_party` isolates Avian, `bevy_ahoy`, Landmass, TrenchBroom, Yarnspinner, Hanabi, and frame pacing setup so app wiring stays understandable.
- Add explicit asset preload resources. Foxtrot's `asset_tracking` pattern waits for all handles in a resource before gameplay screens spawn, which is the right shape for NAU's future glTF player/world/detail asset pipeline.
- Use Bevy glTF settings deliberately. Foxtrot configures `GltfPlugin` coordinate conversion and uses `GltfLoaderSettings`/`RenderAssetUsages` through load-with-settings paths for model assets.
- Discover scene-owned animation players after spawn instead of assuming handles are immediately ready. Foxtrot's runtime asset wiring is a useful reminder that NAU's future animation setup needs explicit readiness/discovery states.
- Treat render quality as app-level infrastructure. Foxtrot enables HDR, tonemapping, bloom, TAA/FXAA choices, deferred prepass, environment maps, skybox, shadow filtering, and texture sampler defaults rather than scattering one-off render tweaks.
- Precompile shaders through a loading phase. Foxtrot spawns a shader compilation map before gameplay; NAU should adapt the idea once custom shaders/materials become user-visible.
- Validate asset readiness in dev. Foxtrot has dev tooling that warns when meshes, materials, scenes, and audio appear before preload has completed.
- Treat particles as render-layer-aware visual systems. The Hanabi fire example uses texture slots, camera-facing orientation, additive alpha, lifetime color/size curves, and a particle render layer; NAU should copy the pattern shape for lift/cloud/airflow effects only after the core traversal metrics are stable.

Patterns to reject or defer:

- Do not adopt Foxtrot's first-person/view-model camera architecture for Nau. NAU needs third-person gliding, banking, and player-body visibility.
- Do not import Foxtrot's level-editing stack yet. TrenchBroom plus rererecast/navmesh is strong for authored indoor/ground levels, but it is not the first step for procedural sky islands.
- Do not import Foxtrot's assets. The audit found Dark Mod asset credits under `CC BY-NC-SA 3.0`; those are reference-only for NAU.

## bevy_new_3d_rpg

`bevy_new_3d_rpg` is the strongest direct reference for NAU's third-person movement, input, player asset loading, and animation stack. It is also on Bevy `0.18`.

Audited files included `Cargo.toml`, `README.md`, `src/player/control.rs`, `src/player/input.rs`, `src/player/animation.rs`, `src/player/mod.rs`, `src/player/particles.rs`, `src/camera/mod.rs`, `src/camera/third_person.rs`, `src/asset_loading/mod.rs`, `src/models/ext_traits.rs`, `src/scene/mod.rs`, and `src/scene/skybox.rs`.

Patterns worth adopting:

- Compute movement direction from the camera transform on the horizontal plane. Its `Transform::movement_direction(Vec2)` projects camera forward to X/Z, derives a flat right vector, combines stick/WASD input, and normalizes the result.
- Rotate the visible character toward desired planar movement, not toward the camera. Its player control code slerps the model toward `atan2(input_dir.x, input_dir.z)`, which directly addresses Nau's lateral/diagonal air-control problem.
- Keep camera rotation input separate from movement input. Movement reads an enhanced-input `Movement` action while camera yaw/pitch reads `RotateCamera`; this matches NAU's invariant that `A`/`D` must not orbit the camera.
- Use named glTF animations and `AnimationGraph::from_clips`. The template loads named clips such as idle, jog, sprint, jump, land, crouch, and roll, builds an animation graph on `SceneInstanceReady`, and transitions with `AnimationTransitions`.
- Keep player/world assets as preloaded resources and spawn them through Bevy scene roots. Its model resources load `models/player.glb` and `models/scene.gltf` before gameplay spawn, which is a useful shape for NAU's character/glider/island/detail assets.
- Keep particle and shader examples asset-driven. Its particle RON assets and `ExtendedMaterial<StandardMaterial, _>` examples are a better next step than hand-building every visual effect as primitive mesh animation.
- Use modern Bevy camera/render defaults intentionally. The template's camera setup combines HDR, `Tonemapping::TonyMcMapface`, `Bloom::NATURAL`, TAA, temporal shadow filtering, optional SSAO, atmosphere, distance fog, and cascaded sun/moon lights. NAU already has some of this; future changes should consolidate this as a render-quality layer rather than scattered setup code.

Patterns to reject or defer:

- Do not adopt `bevy_third_person_camera` blindly. NAU already has a heavily evaluated camera with obstruction, surface clearance, and mouse-axis regression coverage. Package evaluation can happen later, but a drop-in replacement would risk regressing known-fixed camera behavior.
- Do not move movement into `bevy_ahoy` yet. NAU's custom movement math is testable, route-aware, and directly instrumented; the immediate problem is desired-heading/body-heading control, not collision-controller replacement.
- Do not reuse template assets without separate provenance review. The template's own credits contain mixed CCBY/community entries; NAU should use explicitly compatible assets only.
- Do not import code without clarifying license terms. The manifest says `MIT OR Apache-2.0`, but the README says the code is under "CC4 licence" and the clone did not contain a top-level license file. Treat implementation details as design reference until that mismatch is resolved.

## bevy_generative

`bevy_generative` is useful terrain/noise inspiration, not a final sky-island solution. It targets Bevy `0.16.1`, so direct dependency adoption would require version/API validation or porting.

Audited files included `Cargo.toml`, `README.md`, `src/noise.rs`, `src/terrain.rs`, `src/planet.rs`, `src/tests.rs`, `src/util/gltf.rs`, and `examples/terrain.rs`.

Patterns worth adopting:

- Use explicit terrain/noise configuration: seed, scale, offset, method, fractal function, resolution, height exponent, and region/gradient mapping.
- Support multiple noise functions. The code exposes Perlin, OpenSimplex, Simplex, SuperSimplex, Value, Worley, FBM, Billow, HybridMulti, and RidgedMulti paths.
- Generate mesh data into positions, indices, normals, uvs, and colors in one deterministic pass.
- Keep export as a tool path. Its glTF export is basic, but the idea of exporting generated terrain/islands for inspection or offline iteration is useful.
- Use the planet/cube-sphere displacement as inspiration for non-heightfield rock masses and underside silhouettes.
- Keep deterministic tests around generator dimensions and noise bounds. The existing tests only assert coarse shape and value ranges, but that is still the right minimum for NAU's future island generator before adding visual gates.

NAU follow-through from this audit now uses the same broad shape without adding `bevy_generative` as a dependency: deterministic mesh generation writes positions, indices, normals, uvs, vertex colors, and encoded `UV_1` material weights in one pass; island surfaces have denser radial topology; cliff/underside meshes carry measurable strata/color bands; and eval gates now track terrain surface count, mesh vertex floors, vertex-color bands, material-weight bands/channels, relief range, and cliff color-band floors.

Patterns to reject or replace:

- Do not use the terrain generator directly for NAU sky islands. It emits a rectangular heightfield, regenerates in `Update`, uses flat up normals, and has no island rim, cliff skirt, underside mass, chunk LOD, collider generation, erosion, PBR material weights, or streaming integration.
- Do not take `bevy_generative` as an app dependency until Bevy `0.18` compatibility and the git `noise` dependency are evaluated.
- Do not treat vertex-color gradients as final material quality. NAU needs PBR texture/material weights, sharper detail, and screenshot gates for visual granularity.

## Immediate Dependency Decisions

- Keep NAU's current custom camera and movement systems for the next flight-feel PR. The reference audit supports camera-relative planar input and body-facing smoothing, not a camera package replacement.
- Do not add `bevy_ahoy`, Avian, `bevy_third_person_camera`, Foxtrot's level stack, or `bevy_generative` in the next PR. Each solves a larger architectural problem than the current flight-control issue and would weaken the eval signal by changing too many variables at once.
- Use Bevy-native `SceneRoot`, `Gltf`, `AnimationGraph`, `AnimationPlayer`, and `AnimationTransitions` when the asset/animation branch starts. That path matches both references and avoids a custom animation abstraction too early.
- Build the first improved island generator internally, with small pure functions and tests, before considering a procedural-generation dependency.

## NAU Implementation Order

1. Fix flight feel with NAU's existing pure movement and evaluated camera. Add a pure camera-relative planar input helper, desired planar heading, body-yaw smoothing, bank response, and eval metrics before touching the camera package stack.
2. Add or strengthen an eval scenario for diagonal/lateral air steering and braking. Gate average/p95 body-heading error, lateral response, yaw overshoot, and movement-input camera non-coupling.
3. Keep extending the `bevy_new_3d_rpg` asset/animation shape in stages: NAU now has declared glTF slots, a self-authored glider fixture, `SceneRoot` spawning, scene-instance readiness, named player clip declarations, `Gltf` lookup, `AnimationPlayer` discovery, and `AnimationGraph`/`AnimationTransitions` readiness metrics; the next asset branch should supply compatible character clips and then drive real state transitions.
4. Adapt Foxtrot's asset preload/resource tracking and render infrastructure once real assets start replacing primitives.
5. Continue the internal island mesh generator inspired by `bevy_generative`: NAU now has irregular island masks, rim/cliff/underside geometry, computed normals, deterministic vertex-color/strata signals, encoded terrain material-weight semantics, collision surface compatibility, and streaming counters; the next terrain branch should add export/offline inspection tools and richer material identity gates.
6. Add the next visual eval gates for screenshot-level terrain/material identity, vegetation/cloud depth/readability, glTF readiness, and asset residency.

## License Notes

- Foxtrot code is `MIT OR Apache-2.0 OR CC0-1.0`, but its asset credits include Dark Mod assets under `CC BY-NC-SA 3.0`. Treat Foxtrot assets as non-importable unless a specific asset has separate compatible provenance.
- `bevy_new_3d_rpg` has conflicting license signals: `Cargo.toml` says `MIT OR Apache-2.0`, while `README.md` says "CC4 licence" and the clone did not expose a top-level license file. Treat code and assets as reference-only unless this is clarified.
- `bevy_generative` is `MIT OR Apache-2.0`.
- NAU must not import proprietary Zelda/TOTK assets, names, maps, or designs. References are for traversal quality and Bevy implementation patterns only.
