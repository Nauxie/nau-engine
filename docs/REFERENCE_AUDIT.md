# Reference Audit

Last updated: 2026-06-24

This document records local code audits for external Bevy references used to steer NAU work. These repositories are references only; do not copy assets or large code blocks into NAU without a separate license/provenance check.

## Audited Repositories

- `janhohenheim/foxtrot` at `db0746f`
- `olekspickle/bevy_new_3d_rpg` at `54d1dbb`
- `manankarnik/bevy_generative` at `74a17cc`

The current local audit used refreshed clones under `target/reference_repos` so source, manifests, documentation, shader/material examples, and license text were available for direct inspection. Earlier sparse-clone notes remain useful, but this pass treated the repositories as codebases to audit rather than search results.

## Foxtrot

Foxtrot is the strongest reference for a mature Bevy `0.18` application structure. It is not a template for NAU's traversal mechanics, but it shows how a real Bevy project wires third-party plugins, loading states, render settings, glTF scenes, assets, and debug systems.

Audited files included `Cargo.toml`, `readme.md`, license files, `assets/sprites/licence.md`, map/glTF asset paths, `src/main.rs`, `src/asset_tracking.rs`, `src/screens/loading/preload_assets.rs`, `src/screens/loading/spawn_level.rs`, `src/screens/loading/shader_compilation.rs`, `src/gameplay/player/input.rs`, `src/gameplay/player/camera.rs`, `src/gameplay/player/assets.rs`, `src/gameplay/player/animation.rs`, `src/gameplay/animation.rs`, `src/hdr.rs`, `src/dev_tools/validate_preloading.rs`, `src/third_party/*`, `src/menus/credits.rs`, and `src/props/specific/burning_logs.rs`.

Patterns worth adopting:

- Keep one wrapper module per third-party plugin. Foxtrot's `src/third_party` isolates Avian, `bevy_ahoy`, Landmass, TrenchBroom, Yarnspinner, Hanabi, and frame pacing setup so app wiring stays understandable. This is also a useful future direction for splitting NAU's large `src/main.rs` along app/render/assets/world/camera boundaries once a second concrete owner exists for each module.
- Use a modular plugin tree with explicit gameplay/loading system sets. NAU should move toward `assets`, `animation`, `camera`, `environment_vfx`, and `diagnostics_eval` modules as those areas get real ownership, while avoiding a broad refactor that weakens current eval confidence.
- Add explicit asset preload resources. Foxtrot's `asset_tracking` and loading-screen systems wait for dependency-marked resource handles, use `AssetServer::is_loaded_with_dependencies`, and separate resource-readiness from scene-spawn readiness, which is the right shape for NAU's future glTF player/world/detail asset pipeline.
- Use Bevy glTF settings deliberately. Foxtrot configures `GltfPlugin` coordinate conversion and uses `GltfLoaderSettings`/`RenderAssetUsages` through load-with-settings paths for model assets.
- Discover scene-owned animation players after spawn instead of assuming handles are immediately ready. Foxtrot's runtime asset wiring is a useful reminder that NAU's future animation setup needs explicit readiness/discovery states, though NAU should keep its named-clip contract rather than Foxtrot's numeric `#AnimationN` paths.
- Treat render quality as app-level infrastructure. Foxtrot enables HDR, tonemapping, bloom, TAA/FXAA choices, deferred prepass, environment maps, skybox, shadow filtering, and texture sampler defaults rather than scattering one-off render tweaks.
- Precompile shaders through a loading phase. Foxtrot spawns a shader compilation map before gameplay; NAU should adapt the idea once custom shaders/materials become user-visible, but avoid hard-coded pipeline counts unless there is a measured warmup map and CI/eval guard.
- Validate asset readiness in dev. Foxtrot has dev tooling that warns when meshes, materials, scenes, and audio appear before preload has completed.
- Treat particles as render-layer-aware visual systems. The Hanabi fire example uses texture slots, camera-facing orientation, additive alpha, lifetime color/size curves, and a particle render layer; NAU should copy the pattern shape for lift/cloud/airflow effects only after the core traversal metrics are stable.

Patterns to reject or defer:

- Do not adopt Foxtrot's first-person/view-model camera architecture for Nau. NAU needs third-person gliding, banking, and player-body visibility.
- Do not import Foxtrot's level-editing stack yet. TrenchBroom plus rererecast/navmesh is strong for authored indoor/ground levels, but it is not the first step for procedural sky islands.
- Do not import Foxtrot's assets. The audit found Dark Mod asset credits under `CC BY-NC-SA 3.0`; those are reference-only for NAU.

## bevy_new_3d_rpg

`bevy_new_3d_rpg` is the strongest direct reference for NAU's third-person movement, input, player asset loading, and animation stack. It is also on Bevy `0.18`.

Audited files included `Cargo.toml`, `README.md`, `assets/credits.ron`, `assets/config.ron`, `assets/settings.ron`, `assets/models/player.glb`, `assets/models/scene.gltf`, `assets/particles/*.ron`, `assets/shaders/*.wgsl`, `src/player/control.rs`, `src/player/input.rs`, `src/player/animation.rs`, `src/player/mod.rs`, `src/player/particles.rs`, `src/camera/mod.rs`, `src/camera/third_person.rs`, `src/camera/hdr.rs`, `src/asset_loading/mod.rs`, `src/asset_loading/tracking.rs`, `src/models/ext_traits.rs`, `src/scene/mod.rs`, `src/scene/cosmic_sphere.rs`, `src/scene/shader_material.rs`, `src/scene/skybox.rs`, and `src/screens/loading/*`.

Patterns worth adopting:

- Compute movement direction from the camera transform on the horizontal plane. Its `Transform::movement_direction(Vec2)` projects camera forward to X/Z, derives a flat right vector, combines stick/WASD input, and normalizes the result.
- Rotate the visible character toward desired planar movement, not toward the camera. Its player control code slerps the model toward `atan2(input_dir.x, input_dir.z)`, which directly addresses Nau's lateral/diagonal air-control problem.
- Keep camera rotation input separate from movement input. Movement reads an enhanced-input `Movement` action while camera yaw/pitch reads `RotateCamera`; this matches NAU's invariant that `A`/`D` must not orbit the camera.
- Use named glTF animations and `AnimationGraph::from_clips`. The template loads named clips such as idle, jog, sprint, jump, land, crouch, and roll, recursively discovers nested `AnimationPlayer`s after `SceneInstanceReady`, builds an animation graph, and transitions with `AnimationTransitions`; NAU should keep checked clip lookup plus diagnostics instead of copying hard-coded clip order or indexing `named_animations` with panicking lookups.
- Keep player/world assets as preloaded resources and spawn them through Bevy scene roots. Its model resources load `models/player.glb` and `models/scene.gltf` before gameplay spawn, which is a useful shape for NAU's character/glider/island/detail assets and should pair with NAU's existing asset-slot readiness metrics.
- Keep traversal and camera tuning data-driven once constants become hard to compare in review. The template's RON config-as-asset pattern is a good fit for future flight/camera/render constants, but NAU should only move constants out of Rust after the corresponding eval gates are stable enough to protect changes.
- Keep particle examples asset-driven. Its particle RON assets are a better next step than hand-building every airflow/updraft cue as primitive mesh animation.
- Use modern Bevy camera/render defaults intentionally. The template's camera setup combines HDR, `Tonemapping::TonyMcMapface`, `Bloom::NATURAL`, TAA, temporal shadow filtering, optional SSAO, atmosphere, distance fog, and cascaded sun/moon lights. NAU already has some of this; future changes should consolidate this as a render-quality layer rather than scattered setup code.

Patterns to reject or defer:

- Do not adopt `bevy_third_person_camera` blindly. NAU already has a heavily evaluated camera with obstruction, surface clearance, and mouse-axis regression coverage. Package evaluation can happen later, but a drop-in replacement would risk regressing known-fixed camera behavior.
- Do not move movement into `bevy_ahoy` yet. NAU's custom movement math is testable, route-aware, and directly instrumented; the immediate problem is desired-heading/body-heading control, not collision-controller replacement.
- Do not adopt `bevy_skein` until NAU actually starts authoring collision volumes, markers, or component extras in Blender.
- Do not import broad template systems such as dialogue, audio, UI screens, top-down camera, or generalized settings until NAU has a concrete player-facing need.
- Do not copy the demo shader stack as-is. Some shader material examples are unused or path-fragile, and manual shader-pipeline loading is brittle without measured warmup coverage.
- Do not reuse template assets without separate provenance review. The template's own credits contain mixed CCBY/community entries; NAU should use explicitly compatible assets only.
- Do not import code without clarifying license terms. The manifest says `MIT OR Apache-2.0`, but the README says the code is under "CC4 licence" and the clone did not contain a top-level license file. Treat implementation details as design reference until that mismatch is resolved.

## bevy_generative

`bevy_generative` is useful terrain/noise inspiration, not a final sky-island solution. It targets Bevy `0.16.1`, so direct dependency adoption would require version/API validation or porting.

Audited files included `Cargo.toml`, `README.md`, `CHANGELOG.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `src/lib.rs`, `src/noise.rs`, `src/map.rs`, `src/terrain.rs`, `src/planet.rs`, `src/tests.rs`, `src/util/mod.rs`, `src/util/gltf.rs`, `src/util/save.js`, `examples/map.rs`, `examples/terrain.rs`, `examples/planet.rs`, and `examples/export.rs`.

Patterns worth adopting:

- Use explicit terrain/noise configuration: seed, scale, offset, method, fractal function and parameters, resolution, height exponent, and region/gradient mapping.
- Support multiple noise functions. The code exposes Perlin, OpenSimplex, Simplex, SuperSimplex, Value, Worley, FBM, Billow, HybridMulti, and RidgedMulti paths.
- Generate mesh data into positions, indices, normals, uvs, and colors in one deterministic pass.
- Keep export as a tool path. Its glTF export is basic, but the idea of exporting generated terrain/islands for inspection or offline iteration is useful; if NAU adds GLB export, it should emit indexed glTF from NAU mesh data with normals, UVs, material weights, and material metadata instead of copying the reference's position/color-only triangle duplication.
- Use the planet/cube-sphere displacement as inspiration for non-heightfield rock masses and underside silhouettes.
- Prototype a small internal seeded noise sampler before adding a dependency. It should perturb NAU's island silhouette scale, terrain relief, cliff strata, and material weights in deterministic tests before affecting the default route.
- Keep deterministic tests around generator dimensions and noise bounds. The existing tests only assert coarse shape and value ranges, but that is still the right minimum for NAU's future island generator before adding visual gates.

NAU follow-through from this audit now uses the same broad shape without adding `bevy_generative` as a dependency: deterministic mesh generation writes positions, indices, normals, tiled terrain UVs, vertex colors, and encoded `UV_1` material weights in one pass; island surfaces have denser radial topology; cliff/underside meshes carry measurable strata/color bands; distant island impostors use layered generated geometry and vertex-color bands instead of single low-detail blobs; terrain materials use sharper generated PBR maps with smoothed broad value-noise variation instead of block-stepped color patches; canopy and cloud meshes add organic detail-card silhouettes on top of lobed primitives; eval gates track terrain surface count, mesh vertex floors, vertex-color bands, material-weight bands/channels/regions, texture-detail bands, terrain texture-edge frequency, relief range, cliff color-band floors, terrain/body/impostor shape floors, and island-body mesh density; and `--export-terrain` plus `terrain_export_audit` emit and validate terrain/cliff/underside/impostor OBJ, terrain-material CSV, and manifest artifacts for offline terrain inspection.

Patterns to reject or replace:

- Do not use the terrain generator directly for NAU sky islands. It emits a rectangular heightfield, regenerates in `Update`, uses flat up normals and raw grid UVs, and has no island rim, cliff skirt, underside mass, chunk LOD, collider generation, erosion, PBR material weights, or streaming integration.
- Do not take `bevy_generative` as an app dependency. It targets Bevy `0.16.1`, and its git `noise` dependency should not be inherited. If NAU adds procedural noise, prefer a small local wrapper around a pinned crates.io `noise` release with tests that prove deterministic output.
- Do not treat vertex-color gradients as final material quality. NAU needs PBR texture/material weights, sharper detail, and screenshot gates for visual granularity.

## Immediate Dependency Decisions

- Keep NAU's current custom camera and movement systems. The reference audit supports NAU's camera-relative planar input and body-facing smoothing, not a camera package replacement.
- Do not add `bevy_ahoy`, Avian, `bevy_third_person_camera`, Foxtrot's level stack, `bevy_skein`, Hanabi, or `bevy_generative` in the next PR. Each solves a larger architectural problem than the current asset/content bottleneck and would weaken the eval signal by changing too many variables at once.
- Continue Bevy-native `SceneRoot`, `Gltf`, `SceneInstanceReady`, `AnimationGraph`, `AnimationPlayer`, and `AnimationTransitions` work in NAU's own asset pipeline. That path matches both references and avoids a custom animation abstraction too early.
- Keep terrain generation internal, deterministic, and audited with pure helpers/tests before considering a procedural-generation dependency.

## NAU Implementation Order

1. Keep flight/camera tuning under the current evaluated movement stack. NAU already has camera-relative planar air control, lateral/braking evals, body-heading metrics, and movement-input camera non-coupling gates; future flight work should tighten those metrics rather than replace the controller wholesale.
2. Keep the small Bevy-native asset preload/resource layer narrow. NAU now tracks recursive dependency readiness for declared scene handles by residency class; the next asset work should build on those metrics rather than introducing a broad loading-screen rewrite.
3. Keep glTF animation-player linking retryable and scene-ready driven. NAU now separates `SceneInstanceReady` lifecycle marking from nested `AnimationPlayer` discovery, named-clip validation, and `AnimationGraph`/`AnimationTransitions` attachment; future asset work should preserve the named-clip contract and avoid hard-coded animation indices.
4. Extend the `bevy_new_3d_rpg` asset/animation shape in stages: NAU now has declared glTF slots, self-authored player/glider/world fixture scenes, `SceneRoot` spawning, scene-instance readiness, named player clip declarations, `Gltf` lookup, retryable `AnimationPlayer` discovery, and `AnimationGraph`/`AnimationTransitions` readiness metrics; the next asset branch should move from fixtures toward compatible production-quality character/world assets before driving real state transitions.
5. Adapt Foxtrot's render infrastructure only after real assets expose a measurable need for better texture samplers, HDR/TAA/bloom defaults, or shader warmup.
6. Continue the internal island mesh generator inspired by `bevy_generative`: NAU now has irregular island masks, rim/cliff/underside geometry, layered far-LOD island impostors, computed normals, tiled terrain UVs, deterministic vertex-color/strata signals, encoded terrain material-weight semantics, derived material-region gates, terrain texture-detail and texture-edge gates, terrain/body/impostor shape gates, collision surface compatibility, streaming counters, and audited offline OBJ/CSV/manifest export including impostor artifacts; the next terrain branch should move from material readability toward richer biome identity, vegetation density, and screenshot-level terrain/material/impostor semantic checks.
7. Add the next visual eval gates for screenshot-level terrain/material identity, vegetation/cloud depth/readability, distant-impostor readability, glTF readiness, and asset residency.

## License Notes

- Foxtrot code is `MIT OR Apache-2.0 OR CC0-1.0`, but its asset credits include Dark Mod assets under `CC BY-NC-SA 3.0`. Treat Foxtrot assets as non-importable unless a specific asset has separate compatible provenance.
- `bevy_new_3d_rpg` has conflicting license signals: `Cargo.toml` says `MIT OR Apache-2.0`, while `README.md` says "CC4 licence" and the clone did not expose a top-level license file. Treat code and assets as reference-only unless this is clarified.
- `bevy_generative` is `MIT OR Apache-2.0`.
- NAU must not import proprietary Zelda/TOTK assets, names, maps, or designs. References are for traversal quality and Bevy implementation patterns only.
