# World Floor Requirements Audit

Date: 2026-07-10

Branch: `abhinav/world-floor-revamp`

Overall status: **accepted**. Automated readiness and the required human release playtest pass; evidence must be regenerated after any future behavioral source change.

## Status Key

- `Pending implementation`: the final behavior is not yet established.
- `Pending evidence`: the behavior may exist in source, but final-source automated evidence has not been recorded here.
- `Pending manual`: requires the final human release playtest.
- `Complete`: supported by current-source evidence and, where required, manual observation.

## Product Requirements

- Playable near-original-level terrain: `Complete (automated)`.
  Evidence: shared terrain spans approximately `26.9 m` of relief with four biomes, six terrain feature classes, ground cover, and measured fully grounded traversal over `493.150 m`. The real player pipeline has direct land-then-relaunch coverage, while manual and scripted `R` reset behavior retains the established central-island contract.

- Shared render/gameplay terrain sampler: `Complete`.
  Evidence: mesh generation and gameplay grounding query `src/world/terrain.rs`; triangle interpolation, render/gameplay parity, representative points, and tile-boundary continuity are covered by tests.

- Island precedence: `Complete (automated)`.
  Evidence: `SkyRoute::ground_at` retains highest qualifying island-surface precedence, island-edge behavior is tested, and the existing island route/eval suites pass.

- Streamed coverage: `Complete (automated)`.
  Evidence: candidate profiles report exactly `9` visible tiles, and centered-window, seam, and sustained traversal tests pass.

- Bounded residency: `Complete (automated)`.
  Evidence: the reusable pool is capped at `25`, the measured peak is `15`, stream churn is bounded to one spawn and one despawn per frame, and pool-eviction/window tests pass.

## Performance Requirements

- Release app comparison: `Complete for the final committed candidate source`.
  Evidence: fresh quiet-host matching-mode `baseline_route` and `long_glide_visibility` comparisons pass all hard budgets.

- Scripted `freeflight` profile: `Complete for the final committed candidate source`.
  Evidence: the 35-second main and exact-source 45-second candidate quiet-host profiles pass absolute and relative frame-time, hitch, asset-cost, island, and streaming gates.

- Scripted `ground_traversal` profile: `Complete for the final committed candidate source`.
  Evidence: the exact-source 45-second candidate profile remains `100%` grounded over `493.150 m`; candidate crosses streamed terrain while staying inside all floor budgets.

- No meaningful release regression: `Complete for the final committed candidate source`.
  Evidence: readiness passes every hard comparison; candidate app-path averages improve on main, while both foreground profiles remain inside their absolute and relative frame-time and hitch budgets.

## Visual Requirements

- Terrain quality and scale: `Complete`.
  Automated screenshots show continuous biome-colored terrain at playable elevation with visible relief and ground cover. The human profile exercised the required route and reported good overall and frame-rate feel.

- Island coexistence: `Complete`.
  Screenshot, route, precedence, and human gameplay evidence show islands retaining visual and gameplay authority without observed floor intrusion.

## Verification

- `cargo fmt --check`: `Passed`.
- `cargo check --all-targets`: `Passed`.
- `cargo test --quiet`: `Passed`.
- `cargo clippy --all-targets --all-features -- -D warnings`: `Passed`.
- Final release app perf comparison: `Passed for the final committed candidate source`.
- Final `freeflight` profile: `Passed for the final committed candidate source`.
- Final `ground_traversal` profile: `Passed for the final committed candidate source`.
- Final screenshot review: `Passed as automated readiness evidence`.
- Manual land/walk/run/launch/fly playtest: `Passed`.

## Completion Call

Complete. Automated readiness passes, and the recorded human release playtest accepted terrain quality, traversal behavior, island coexistence, and frame-rate feel after restoring the central-island reset contract.
