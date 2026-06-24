# Reference Audit

Last updated: 2026-06-24

This document records local code audits for external Bevy references used to steer NAU work. These repositories are references only; do not copy assets or large code blocks into NAU without a separate license/provenance check.

## Audited Repositories

- `janhohenheim/foxtrot` at `db0746f`
- `olekspickle/bevy_new_3d_rpg` at `54d1dbb`
- `manankarnik/bevy_generative` at `74a17cc`

The current local audit used refreshed clones under `/tmp/nau-engine-reference` so source, manifests, documentation, shader/material examples, and license text were available for direct inspection. Earlier sparse-clone notes remain useful, but this pass treated the repositories as codebases to audit rather than search results.

## Foxtrot

Foxtrot is the strongest reference for a mature Bevy `0.18` application structure. It is not a template for NAU's traversal mechanics, but it shows how a real Bevy project wires third-party plugins, loading states, render settings, glTF scenes, assets, and debug systems.

Patterns worth adopting:

- Keep one wrapper module per third-party plugin. Foxtrot's `src/third_party` isolates Avian, `bevy_ahoy`, Landmass, TrenchBroom, Yarnspinner, Hanabi, and frame pacing setup so app wiring stays understandable.
- Add explicit asset preload resources. Foxtrot's `asset_tracking` pattern waits for all handles in a resource before gameplay screens spawn, which is the right shape for NAU's future glTF player/world/detail asset pipeline.
- Use Bevy glTF settings deliberately. Foxtrot configures `GltfPlugin` coordinate conversion and uses `GltfLoaderSettings`/`RenderAssetUsages` through load-with-settings paths for model assets.
- Discover scene-owned animation players after spawn instead of assuming handles are immediately ready. Foxtrot's runtime asset wiring is a useful reminder that NAU's future animation setup needs explicit readiness/discovery states.
- Treat render quality as app-level infrastructure. Foxtrot enables HDR, tonemapping, bloom, TAA/FXAA choices, deferred prepass, environment maps, skybox, shadow filtering, and texture sampler defaults rather than scattering one-off render tweaks.
- Precompile shaders through a loading phase. Foxtrot spawns a shader compilation map before gameplay; NAU should adapt the idea once custom shaders/materials become user-visible.
- Validate asset readiness in dev. Foxtrot has dev tooling that warns when meshes, materials, scenes, and audio appear before preload has completed.

Patterns to reject or defer:

- Do not adopt Foxtrot's first-person/view-model camera architecture for Nau. NAU needs third-person gliding, banking, and player-body visibility.
- Do not import Foxtrot's level-editing stack yet. TrenchBroom plus rererecast/navmesh is strong for authored indoor/ground levels, but it is not the first step for procedural sky islands.
- Do not import Foxtrot's assets. The audit found Dark Mod asset credits under `CC BY-NC-SA 3.0`; those are reference-only for NAU.

## bevy_new_3d_rpg

`bevy_new_3d_rpg` is the strongest direct reference for NAU's third-person movement, input, player asset loading, and animation stack. It is also on Bevy `0.18`.

Patterns worth adopting:

- Compute movement direction from the camera transform on the horizontal plane. Its `Transform::movement_direction(Vec2)` projects camera forward to X/Z, derives a flat right vector, combines stick/WASD input, and normalizes the result.
- Rotate the visible character toward desired planar movement, not toward the camera. Its player control code slerps the model toward `atan2(input_dir.x, input_dir.z)`, which directly addresses Nau's lateral/diagonal air-control problem.
- Keep camera rotation input separate from movement input. Movement reads an enhanced-input `Movement` action while camera yaw/pitch reads `RotateCamera`; this matches NAU's invariant that `A`/`D` must not orbit the camera.
- Use named glTF animations and `AnimationGraph::from_clips`. The template loads named clips such as idle, jog, sprint, jump, land, crouch, and roll, builds an animation graph on `SceneInstanceReady`, and transitions with `AnimationTransitions`.
- Keep player/world assets as preloaded resources and spawn them through Bevy scene roots. Its model resources load `models/player.glb` and `models/scene.gltf` before gameplay spawn, which is a useful shape for NAU's character/glider/island/detail assets.
- Keep particle and shader examples asset-driven. Its particle RON assets and `ExtendedMaterial<StandardMaterial, _>` examples are a better next step than hand-building every visual effect as primitive mesh animation.

Patterns to reject or defer:

- Do not adopt `bevy_third_person_camera` blindly. NAU already has a heavily evaluated camera with obstruction, surface clearance, and mouse-axis regression coverage. Package evaluation can happen later, but a drop-in replacement would risk regressing known-fixed camera behavior.
- Do not move movement into `bevy_ahoy` yet. NAU's custom movement math is testable, route-aware, and directly instrumented; the immediate problem is desired-heading/body-heading control, not collision-controller replacement.
- Do not reuse template assets without separate provenance review. The template's own credits contain mixed CCBY/community entries; NAU should use explicitly compatible assets only.

## bevy_generative

`bevy_generative` is useful terrain/noise inspiration, not a final sky-island solution. It targets Bevy `0.16.1`, so direct dependency adoption would require version/API validation or porting.

Patterns worth adopting:

- Use explicit terrain/noise configuration: seed, scale, offset, method, fractal function, resolution, height exponent, and region/gradient mapping.
- Support multiple noise functions. The code exposes Perlin, OpenSimplex, Simplex, SuperSimplex, Value, Worley, FBM, Billow, HybridMulti, and RidgedMulti paths.
- Generate mesh data into positions, indices, normals, uvs, and colors in one deterministic pass.
- Keep export as a tool path. Its glTF export is basic, but the idea of exporting generated terrain/islands for inspection or offline iteration is useful.
- Use the planet/cube-sphere displacement as inspiration for non-heightfield rock masses and underside silhouettes.

Patterns to reject or replace:

- Do not use the terrain generator directly for NAU sky islands. It emits a rectangular heightfield, regenerates in `Update`, uses flat up normals, and has no island rim, cliff skirt, underside mass, chunk LOD, collider generation, erosion, PBR material weights, or streaming integration.
- Do not take `bevy_generative` as an app dependency until Bevy `0.18` compatibility and the git `noise` dependency are evaluated.
- Do not treat vertex-color gradients as final material quality. NAU needs PBR texture/material weights, sharper detail, and screenshot gates for visual granularity.

## NAU Implementation Order

1. Fix flight feel with NAU's existing pure movement and evaluated camera. Add desired planar heading, body-yaw smoothing, bank response, and eval metrics before touching the camera package stack.
2. Add an eval scenario for diagonal/lateral air steering and braking. Gate average/p95 body-heading error, lateral response, yaw overshoot, and movement-input camera non-coupling.
3. Adapt the `bevy_new_3d_rpg` asset/animation shape for NAU in stages: first keep declared glTF slots, `SceneRoot` spawning, scene-instance readiness, and placeholder fallback measurable; next add named clips, `AnimationGraph`, and `AnimationPlayer` discovery once compatible character assets exist.
4. Adapt Foxtrot's asset preload/resource tracking and render infrastructure once real assets start replacing primitives.
5. Build an internal island mesh generator inspired by `bevy_generative`, but with NAU-specific requirements: irregular island masks, rim/cliff/underside geometry, computed normals, deterministic LOD outputs, collision surface compatibility, material weights, and streaming counters.
6. Add visual eval gates for primitive-shape dominance, terrain silhouette complexity, texture/detail frequency, cloud depth/readability, glTF readiness, and asset residency.

## License Notes

- Foxtrot code is `MIT OR Apache-2.0 OR CC0-1.0`, but its asset credits include Dark Mod assets under `CC BY-NC-SA 3.0`. Treat Foxtrot assets as non-importable unless a specific asset has separate compatible provenance.
- `bevy_new_3d_rpg` code is `MIT OR Apache-2.0`; its asset credits include CCBY/community entries that need case-by-case review before import.
- `bevy_generative` is `MIT OR Apache-2.0`.
- NAU must not import proprietary Zelda/TOTK assets, names, maps, or designs. References are for traversal quality and Bevy implementation patterns only.
