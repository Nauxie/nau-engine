# Island Surface Content

This document defines the visual, gameplay, streaming, and verification contract for generated
island surface objects. The objective is not maximum object count. Each island should read as a
coherent place with a distinct silhouette, ecology, history, and water story while preserving the
flight sandbox's response, route clearance, camera continuity, and frame pacing.

## Authoring Model

`IslandComposition` is the authored source of truth for traversal purpose, altitude band,
neighboring islands, route relationships, and player-facing navigation. `IslandArtDirection` is
the source of truth for visual identity: epithet, environmental story, palette family, surface
pattern, hero landmark, flora, formations, ruins, and water story. Biome, terrain archetype, scale
class, and landmark role refine those profiles rather than replacing them with untracked name-only
special cases.

Surface objects are generated as bounded feature clusters:

- Flora clusters combine many plants into one mesh and select coherent species families per island.
- Ruin complexes combine architectural elements into readable precincts rather than scattering
  unrelated single props.
- Rock formations combine several geological masses into one silhouette group.
- Water details attach shore, bank, lip, stepping-stone, and plunge-pool ecology to the existing
  pond, lake, channel, and waterfall network.
- Hero landmarks give every island one profile-specific visual anchor and may remain visible
  through mid LOD when they define the island's silhouette.

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
motif and remain visually distinguishable from adjacent islands. All 41 accepted terrain,
foliage, and stone palette tuples are unique.

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
- Hero landmarks may block the camera where their visible mass requires it, but open or decorative
  silhouettes remain nonblocking for the player.

## Density and Residency Budgets

- Flora: exactly the profile inventory, currently 2–3 clustered entries per island.
- Ruin complexes: exactly the profile inventory, currently 0–2 per island.
- Rock formations: exactly the profile inventory, currently 1–2 per island.
- Hero landmarks: exactly one matching profile landmark per island.
- Water details: bounded by the source water network and present exactly for non-dry profiles.
- Ordinary surface clusters use near-detail residency.
- Hero structures large enough to define an island silhouette may use vista residency through mid
  LOD.
- Islands with at least 5,000 m² of ground-cover footprint must retain at least 20% authored-feature
  footprint coverage.
- New clusters must remain within the resident-visual and entity ceilings, pass the structural mesh
  complexity gates, and preserve debug/release mesh and triangle parity.

The accepted July 16, 2026 export contains 1,347 meshes, 991,095 vertices, and 947,092 triangles:
41 ground-cover meshes, 171 trees, 282 rocks, 102 flora clusters, 46 ruin complexes, 69 formations,
56 water-detail clusters, and 578 landmark entries. The dedicated plateau review remains bounded at
223 resident island visuals, 4,654 entities, 2,066 loaded meshes, and 1,151,952 loaded triangles.
Candidate evidence must pass the enforced ceilings and should be compared against these accepted
mesh and triangle values when density changes.

## Regression Coverage

Structural coverage:

```sh
./tools/island_art_direction_gate.sh target/eval/island_surface/art_direction
./tools/visual_content_export.sh target/eval/island_surface/visual_content
./tools/terrain_export.sh target/eval/island_surface/terrain
```

The art-direction audit requires 41 ordered profiles, accepted full-profile signatures, unique
art/palette/aggregate signatures, exact surface-feature inventories, one ground-cover entry plus
the accepted per-island tree/rock budget, one matching hero landmark, exact water-story presence,
and the large-island authored-feature coverage floor. The visual-content audit additionally
verifies route-wide count and kind floors, per-family mesh complexity and spans, aggregate
surface-feature vertices, and OBJ artifact parity.

Visual coverage:

```sh
./tools/island_hero_gallery_gate.sh target/eval/island_surface/hero_gallery
NAU_EVAL_SCREENSHOT=1 NAU_EVAL_ASSET_AUDIT=0 \
  ./tools/eval.sh island_surface_review target/eval/island_surface/surface_review
```

The hero gallery requires near, mid, and traversal captures for every island: 123 PNGs, 123 marker
sidecars, an exact review manifest, and passing visual and semantic audits. Near views must prove
target-scoped authored features; mid and traversal views must preserve readable island identity and
silhouette. The fixed plateau ruin/geology, dense-flora, and lake/river/waterfall checkpoints add
focused pixel-backed evidence. Flora semantic probes target deterministic points on generated mesh
geometry rather than arbitrary cluster centers. Hero-landmark probes likewise come from multiple
real mesh triangles instead of one coarse camera bound, and semantic island occlusion follows each
authored visual footprint rather than a generic ellipse. Route-edge waterfall rendering and review
framing share one placement contract; waterfall-garden near views keep the actual ribbon and mist
inside a stricter 70% normalized safe frame.

A passing non-golden image audit is necessary but not sufficient: review the checkpoint images
directly for foreground density, ruin readability, geological silhouette, shoreline integration,
river continuity, waterfall source/lip/plunge composition, route readability, repetition, lighting,
and player visibility. The app-only surface-review camera may use a 360 m framing radius to show the
plateau's 450+ m span, but its per-frame translation and rotation remain bounded, and it does not
change gameplay camera tuning.

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
and debug-versus-release performance evidence all pass together. The accepted isolated performance
run records 14.65/17.52 ms debug average/p95 and 16.18/17.37 ms release average/p95, with zero frames
over 33.34 ms in either build.
