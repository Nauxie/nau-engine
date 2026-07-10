# ADR 0002: World Floor Must Be Playable and Performance-Gated

Status: accepted

Date: 2026-07-10

## Context

The world floor exists to make the lower world a real traversal surface, not only distant scenery. It should retain approximately the original world-floor terrain richness while protecting the current island traversal feel and release performance.

Earlier acceptance language treated a visual-only plane near `y=-260`, a single visible tile, or automated evidence without human play as sufficient. Those outcomes do not satisfy the product goal.

## Decision

The world floor is accepted only when all of these are true:

1. The terrain is playable. The player can land, remain grounded, walk, run, launch, and return to flight.
2. Render mesh height and gameplay ground/collision height come from the same deterministic sampler. Separate visual and gameplay height implementations are rejected.
3. Islands preserve precedence. Existing island terrain, collision, routes, objectives, and traversal behavior remain authoritative wherever island and world-floor surfaces overlap.
4. Streaming keeps a player-centered `3x3` window visible. The implementation reuses a tile pool bounded to `25` tiles and must not expose holes during ordinary ground or flight traversal.
5. Terrain quality is near the original world-floor level: readable relief, silhouettes, material/biome variation, and scale. A flat plane, one-tile proof, or distant atmospheric strip is not completion.
6. Performance passes both mandatory foreground profiles, `freeflight` and `ground_traversal`, plus the required release app comparisons.
7. A final human release playtest completes the land, walk, run, launch, and fly sequence on the target Mac.

## Performance Evidence

Final automated evidence must be captured from the exact source state proposed for acceptance:

- Rust formatting, check, test, and clippy gates.
- Matching-mode main-vs-candidate release app perf for the required scenarios.
- Scripted `freeflight` and `ground_traversal` profiles on a quiet host.
- Frame-time and hitch distributions plus entity, mesh, material, vertex, triangle, visible-tile, pool-occupancy, and stream-churn metrics.
- Tests or diagnostics proving render/gameplay sampler parity.
- Tests or diagnostics proving island precedence.
- Tests or diagnostics proving continuous `3x3` visibility and a maximum pool size of `25`.
- Screenshot evidence for terrain quality, coverage, transitions, and island coexistence.

Old artifacts from a visual-only or one-tile implementation are historical diagnostics only. They cannot accept the playable streamed implementation.

## Manual Acceptance

After automated evidence passes, a human must run the release build on the target Mac and:

1. Fly to and land on the world floor.
2. Walk across uneven terrain.
3. Run across tile boundaries.
4. Approach or cross an island/world-floor overlap and confirm the island remains authoritative.
5. Launch from the world floor.
6. Fly through the streamed area and return to normal island traversal.

The tester must explicitly assess grounding/collision, movement and camera feel, tile visibility, transition quality, frame pacing, fan behavior, and heat. This is a required product gate. There is no automated or no-manual waiver.

## Consequences

- Rendering and gameplay terrain cannot drift because they share one sampler.
- Island gameplay remains stable while the lower world becomes traversable.
- Streaming has a concrete coverage target and memory bound.
- Automated profiles cover both flight and ground workloads.
- Acceptance takes longer because subjective traversal and machine behavior still require final human verification.

## Current State

Accepted. The final implementation passed the committed-source automated gates and the required human release playtest. Future behavioral changes must regenerate the source-bound evidence and repeat the manual acceptance sequence.
