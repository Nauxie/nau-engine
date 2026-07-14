# Roadmap

This roadmap is horizon-based, not date-based. It describes product direction from the current playable baseline rather than tracking branches or pull requests.

## Current Position

The foundation, core traversal loop, first large route, streamed world floor, compact UI, and measurement tooling are in place. The main product risk is no longer whether NAU can render a large sky-island sandbox; it is whether the existing traversal can become a coherent, replayable expedition with stronger character and world presentation.

Do not add world scale for its own sake. Prefer work that makes the current route easier to read, more purposeful to fly, and more satisfying to complete.

## Horizon 0: Playable Baseline

Status: complete and maintained.

Current capability:

- Mac-first Rust/Bevy app through wgpu/Metal
- launch, glide, steer, dive, brake, land, recover, and relaunch loop
- camera-relative movement, mouse orbit, camera obstruction handling, debug metrics, and gizmos
- 41-island route, 20 terrain archetypes, island LOD/residency, distant impostors, and a streamed playable world floor
- visible and mechanical wind, 18 lift routes, 20 crosswind fields, and 12 one-shot aerial gates
- self-authored player/glider fixtures with traversal pose and clip coverage
- app/simulation evals, content exports, screenshot audits, play profiles, and release performance baselines

Maintenance criteria:

- Core Rust checks remain green.
- Movement and camera feel are not retuned without a reproducible regression.
- Rendering, content-density, collision, and streaming changes include the relevant app-path and human release checks.

## Horizon 1: Expedition Clarity

Status: next.

Goal: turn the existing world data and traversal mechanics into a clear player-facing journey.

Work:

- Expose an understandable route progression using the existing expedition beats, landmarks, lift nodes, landing targets, and optional detours.
- Distinguish the main route, recovery choices, and high-risk branches through world composition and restrained UI feedback.
- Define completion, retry, and replay behavior for the twelve-gate route.
- Make landing targets and recovery routes useful gameplay decisions rather than eval-only knowledge.
- Add launch sources, hazards, checkpoints, or timing only when they improve the route's choices and feedback.

Exit criteria:

- A new player can infer where to go and why without debug overlays.
- Main-route completion and optional exploration are both legible.
- Failure has a clear recovery or retry path.
- The route remains fun when judged without metric tooling visible.

## Horizon 2: Character And Glider Fidelity

Status: prototype complete; production-quality work remains.

Current capability:

- self-authored glTF player and multi-part glider
- named grounded, launch, fall, bank, glide, dive, brake, and landing clips
- procedural pose refinement, glider attachment checks, transition checks, and pose-preview audits

Next work:

- Choose and implement a real skeletal rig and skinning pipeline when the current fixture blocks visible quality.
- Replace approximate limb posing with authored animation and controlled runtime blending.
- Preserve distinct fall, glide, dive, braking, turning, landing, and recovery silhouettes through transitions.
- Improve cloth/scarf and glider response only after attachment and readability remain stable.

Exit criteria:

- The character reads as a believable human-scale traversal avatar at gameplay distance.
- Limbs, gear, and glider remain attached and readable through fast transitions.
- Animation quality no longer depends on non-skeletal procedural offsets.

## Horizon 3: World Interaction And Physics

Status: functional prototype.

Current capability:

- deterministic island and world-floor grounding
- terrain-rim and cliff-body collision proxies
- AABB collision for generated and authored solid props
- camera obstruction bounds and collision-proxy debug visualization

Next work:

- Identify concrete gameplay cases that the current terrain/AABB model cannot support.
- Evaluate Rapier or Avian against those cases before selecting a physics layer.
- Introduce a kinematic controller, shape queries, slope handling, authored colliders, or dynamic bodies only as required by accepted gameplay.
- Move gate, lift, hazard, and checkpoint interactions to explicit query/trigger semantics if route complexity demands it.

Exit criteria:

- Collision behavior matches visible geometry for the accepted route.
- Grounding, slopes, blockers, triggers, and camera queries share a coherent representation.
- Added physics cost is observable and justified by gameplay.

## Horizon 4: Scalable World Runtime

Status: partially complete.

Current capability:

- active island chunk windows
- near/mid/far island LOD with material-split distant impostors
- lazy cached island shell meshes
- player-centered `3x3` world-floor window with a 25-tile resident pool
- stream churn, residency, entity, mesh, triangle, and frame-time diagnostics

Next work:

- Move synchronous detail and authored-fixture preparation behind measured budgets.
- Add asynchronous asset loading only when current loading produces a demonstrated hitch or memory problem.
- Add floating-origin/rebase support only when an accepted route exceeds current coordinate precision.
- Activate collision and high-detail content by chunk when the gameplay route requires more scale.

Exit criteria:

- Long traversal keeps frame time, memory, and stream churn bounded.
- New content does not become permanently resident by default.
- Distant views remain coherent while nearby collision and detail stay authoritative.

## Horizon 5: Environment Fidelity

Status: broad prototype systems exist.

Current capability:

- Bevy atmosphere, fog, volumetric light, bloom, shadows, weather variation, procedural PBR materials, layered clouds, ponds, vegetation, landmarks, wind guides, ribbons, motes, and player airflow
- shared wind-flow math connecting visuals, gameplay response, exports, and eval gates

Next work:

- Improve the art direction of existing systems before adding more effect families.
- Replace generated cues with authored particles, vegetation motion, glider/cloth response, water treatment, and weather layers where they materially improve readability.
- Profile atmosphere, fog/light, shadows, clouds, wind visuals, and island detail before increasing density.

Exit criteria:

- Wind and lift opportunities are readable without debug geometry.
- Environment fidelity reinforces route decisions and depth perception.
- Visual improvements remain inside accepted release performance and power budgets.

## Guardrails

- Keep the current traversal feel unless evidence shows a regression.
- Add abstractions only after two concrete systems need the same shape.
- Keep gameplay forces, route constants, and performance budgets visible and testable.
- Use the smallest representative route and eval set for iteration, then finish with release play.
- Treat generated content and audits as development infrastructure, not a substitute for production art judgment.
