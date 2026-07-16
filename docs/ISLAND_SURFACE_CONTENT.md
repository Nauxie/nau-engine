# Island Surface Content

This document defines the visual, gameplay, streaming, and verification contract for generated
island surface objects. The objective is not maximum object count. Each island should read as a
coherent place with a distinct silhouette, ecology, history, and water story while preserving the
flight sandbox's response, route clearance, camera continuity, and frame pacing.

## Authoring Model

`IslandComposition` is the authored source of truth for an island's family, visual motif,
traversal purpose, altitude band, neighboring islands, and player-facing identity. Surface
generation may also use biome, terrain archetype, water feature, scale class, and landmark role,
but those tags refine the composition rather than replacing it with name-only special cases.

Surface objects are generated as bounded feature clusters:

- Flora clusters combine many plants into one mesh and select coherent species families per island.
- Ruin complexes combine architectural elements into readable precincts rather than scattering
  unrelated single props.
- Rock formations combine several geological masses into one silhouette group.
- Water details attach shore, bank, lip, stepping-stone, and plunge-pool ecology to the existing
  pond, lake, channel, and waterfall network.

Cluster meshes keep draw submission and streaming entries bounded while allowing substantially
more geometric detail inside each entry.

## Visual Grammar

The route should expose all of these families in normal play and eval viewpoints:

- Broad-canopy, wind-bent, orchard, cypress, willow, and alpine-pine trees.
- Fern groves, flower thickets, reed beds, wind shrubs, broadleaf patches, and mushroom rings.
- Colonnades, sunken sanctums, watchtowers, broken aqueducts, and processional stairs.
- Basalt crowns, weathered arches, boulder spines, stacked monoliths, and crystal outcrops.
- Lily-pad colonies, shore reed arcs, riverbank cobbles, waterfall lip rocks, plunge-pool ripples,
  and mossy stepping stones.

Individual islands do not need every family. Their selected objects must reinforce the authored
motif and remain visually distinguishable from adjacent islands.

## Placement and Gameplay Invariants

- All generated surface placements must remain inside the playable island silhouette.
- Launch lanes, arrival lanes, reset areas, route gates, and intended ruin or bridge openings must
  remain unobstructed.
- Decorative foliage, water surfaces, stairs, and bridge openings are nonblocking.
- Collision is reserved for visually solid masses that can be represented without an enclosing
  box creating invisible walls.
- Camera obstacles use soft local-prop behavior unless the object is an authored solid wall or
  geological mass.
- Existing island terrain, route objectives, player movement, camera input, and wind behavior are
  not modified by visual-detail generation.

## Density and Residency Budgets

- Flora: normally 1–3 clustered entries per island; the great plateau may use 4.
- Ruin complexes: 0–2 per island.
- Rock formations: 0–2 per island.
- Water details: bounded by the source water network and capped on the great plateau.
- Ordinary surface clusters use near-detail residency.
- Hero structures large enough to define an island silhouette may use vista residency through mid
  LOD.
- New clusters must remain below the existing 340 resident-island-visual and 5,000 entity gates.

The clean-main baseline captured on July 15, 2026 exported 1,018 generated visual meshes with
651,272 vertices and 686,458 triangles. Its plateau vista reached 219 resident island visuals,
4,668 entities, 1,767 loaded meshes, 830,917 loaded vertices, and 968,198 loaded triangles.

The accepted surface pass exports 1,223 meshes with 834,357 vertices and 832,202 triangles. It
adds 84 flora clusters across six families, 25 ruin complexes across five families, 50 geological
formations across five families, and 46 water-detail clusters across six families. The dedicated
plateau review records 413 runtime generated landmarks and remains bounded at 220 resident island
visuals, 4,651 entities, 1,942 loaded meshes, 977,730 loaded vertices, and 1,040,102 loaded
triangles. Candidate evidence must remain within the existing absolute gates and should be
compared against these values when density changes.

## Regression Coverage

Structural coverage:

```sh
./tools/visual_content_export.sh target/eval/island_surface/visual_content
./tools/terrain_export.sh target/eval/island_surface/terrain
```

The visual-content audit must verify route-wide count and kind floors for flora clusters, ruin
complexes, rock formations, and water details; per-family mesh complexity and spans; aggregate
surface-feature vertices; and OBJ artifact parity.

Visual coverage:

```sh
NAU_EVAL_SCREENSHOT=1 NAU_EVAL_ASSET_AUDIT=0 \
  ./tools/eval.sh island_surface_review target/eval/island_surface/surface_review
```

The fixed ruin/geology, dense-flora, and lake/river/waterfall checkpoints carry bounded semantic
samples for the expected nearby feature families. Marker projection and semantic-scene audits
require pixel-backed evidence instead of accepting route-wide object counts alone. A passing
non-golden image audit is necessary but not sufficient: review the checkpoint images directly for
foreground density, ruin readability, geological silhouette, shoreline integration, river
continuity, waterfall source/lip/plunge composition, route readability, and player visibility.
Surface-family semantic probes are limited to the target plateau; route-wide coverage remains the
responsibility of the deterministic export audit. The app-only camera may use a 360 m framing
radius to show the plateau's 450+ m span, but its per-frame translation and rotation remain bounded,
and it does not change gameplay camera tuning.

Gameplay and performance coverage:

```sh
./tools/world_content_gate.sh target/eval/island_surface/world_content
./tools/camera_continuity_gate.sh target/eval/island_surface/camera
./tools/dev_play_performance_gate.sh target/eval/island_surface/performance
cargo test
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

The overhaul is complete only when structural audits, screenshot review, camera/mechanics gates,
and debug-versus-release performance evidence all pass together.
