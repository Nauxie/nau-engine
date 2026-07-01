# Flight Mechanics

This file defines the intended traversal feel. Code can change, but behavior changes should update this spec.

## Design Goal

The player should feel like a human-scale character using launch sources, glider control, dive speed, and wind, not like a superhero with unrestricted free flight.

The fantasy is:

- launch upward from a grounded source
- deploy a glider
- trade altitude for distance
- steer and bank through air
- use dive speed and wind intelligently
- collect simple aerial boost gates
- land, recover, or chain into another launch/updraft

## Current Inputs

- `W`: accelerate forward
- `S`: brake/backward input
- `A`: steer/strafe left
- `D`: steer/strafe right
- `Space`: deploy glider while airborne
- `E`: launch upward from the ground
- `Shift`: dive

Input mapping is still prototype-level. In the long run, glider controls should move toward turn/bank/pitch semantics instead of raw air strafing.

## Current States

- `Grounded`: player is at or near floor height.
- `Launching`: short lockout after a valid launch.
- `Airborne`: falling or moving through air without glider.
- `Gliding`: glider input is held while airborne and not launching. Holding dive while gliding keeps the glider deployed and enters a high-sink glider-dive pose.

## Current Rules

- `E` launch is ground-gated.
- Launch is one use per airtime.
- Launch gives vertical velocity and a small forward bonus.
- Gliding reduces gravity and clamps ordinary fall speed.
- A glider dive keeps the glider deployed but bypasses the ordinary glide fall-speed clamp so dive can trade altitude for speed without hiding the wing state.
- Gliding does not create altitude on its own.
- Airborne and gliding `W`/`A`/`S`/`D` input uses the stable camera follow direction plus explicit mouse orbit as the camera-relative movement basis. `W`/`A`/`D` and rear diagonals steer planar velocity with input-aligned acceleration and faster heading rotation so Nau turns into lateral glide travel, while pure `S` stays an air-brake/reverse-speed-limited control.
- Airborne `S` input brakes forward motion first, then allows limited backward drift instead of unrestricted reverse flight.
- Visual `WindField` volumes are finite axis-aligned boxes for readable wind/updraft streams, gust/swirl diagnostics, and bounded horizontal airborne wind response.
- Crosswind `WindField`s push airborne horizontal velocity toward their dynamic flow; updraft `WindField`s add only lateral swirl current.
- Every gameplay updraft route node intentionally overlaps an angled crosswind layer so wind-current routes can prove simultaneous crosswind-plus-updraft swirl response rather than one isolated vertical field at a time.
- Gameplay `LiftField` updraft volumes are separate finite boxes that add vertical velocity while the player is airborne inside them.
- Authored gameplay updraft route nodes must pair the visual `WindField` and gameplay `LiftField` at the same center and extents.
- Lift fields clamp against their configured maximum upward speed instead of granting unbounded climb.
- Aerial power-up gates are one-time route pickups that apply a small capped forward/upward boost while airborne, then disappear.
- Diving adds downward acceleration.
- The floor clamp prevents the player from ending below the floor or retaining downward velocity after collision.
- Island containment uses deterministic per-archetype footprint profiles. Terrain mesh generation can use the visual contour, while ground containment and terrain-rim collision use the playable contour.
- Near-LOD islands expose 16 sampled terrain-rim AABB contour segments so grounded movement is blocked by the island footprint instead of a coarse edge approximation.
- Generated tree trunks, rocks, route cairns, launch beacons, recovery masts, target markers, and route-integrated obstruction spires expose simple world-collision proxies; player movement resolves horizontally out of those proxies and clears velocity into the collision normal.
- Route-integrated obstruction spires also expose camera-obstruction bounds so camera avoidance is tested against generated world objects rather than standalone cuboid blockers.
- Player facing follows desired airborne steering direction with exponential smoothing and bank response, falling back to horizontal velocity when no steering input is active.

## Forbidden Behaviors

- No midair relaunch spam unless a future mechanic explicitly grants it.
- No altitude gain from ordinary gliding or visual wind current without `LiftField`/launch support.
- No repeatable power-up farming from the same gate in one flight.
- No camera anchor based on full 3D velocity.
- No direct elapsed-time multiplied by speed animation phase. Animation phase should accumulate from delta time.
- No unbounded `rate * dt` interpolation factors that can exceed `1.0` on frame hitches.

## Desired Feel

Launch:

- clear upward impulse
- short visual launch pose
- camera should remain behind horizontal heading
- no repeated midair burst by default

Glide:

- stable descent
- readable wing deployment
- broad, smooth turn behavior
- altitude is a resource
- speed and dive can be used to extend route choices, but not create free climbing

Fall:

- should rotate the whole body into a belly-down skydiver posture, not only lift the arms
- should keep limbs spread enough to read cleanly at gameplay camera distance without obvious clipping
- should transition into glide, air brake, or dive without a one-frame silhouette pop

Dive:

- commits the player downward
- increases urgency and speed
- should read as a head-down streamlined drop with torso/head alignment, narrow limb profile, trailing legs, and deployed glider wings pitching/sweeping under dive pressure when the glider is out
- should be reversible only with enough altitude, wind, or later abilities

Landing:

- landing anticipation and post-touchdown recovery are explicit pose intents now; anticipation is only valid after glider release while airborne and descending, not while the glider pose is still active
- high-sink landings enter anticipation early enough to rotate out of dive over several frames, so the visible pose folds forward into an absorbing feet-forward crouch before contact instead of popping on the landing frame
- landing anticipation and recovery blends are bounded by landing-specific pose-temporal, signed backward-bend, forward-fold, local torso-bend, and torso-offset gates, so dive/glide-to-land transitions cannot pass eval by hiding spine separation or a backward V-shaped silhouette behind transition grace
- a full authored landing locomotion state with slope-aware collider handling is still future work
- needs collision and slope logic before polish

Wind/updraft:

- visual `WindField` volumes are the shared source for stream visuals, diagnostics, and bounded horizontal airborne wind current
- crosswinds push laterally without adding vertical lift
- updraft wind swirl can bend horizontal motion, but vertical climb still comes from paired `LiftField` volumes
- active lift should be readable through paired updraft visuals, breathing lift haze, gusting/advection-driven guide motes/ribbons, traveling clumped gust-packet fronts, secondary lane-dependent gust layers between primary fronts, crosswind stream-to-stream speed variation, flow-facing crosswind motes, curved tapered crosswind ribbons, phase-staggered thermal depth lanes, layered visual depth, flow-driven scale pulse, and debug bounds before richer particles, cloth/glider motion, vegetation, clouds, or other environment art
- wind-current evals should prove not only that current force is nonzero, but that enough applied horizontal delta points in the field-axis correction direction toward the sampled `WindField::flow_at` speed

Power-ups:

- boost gates should read as route affordances, not hidden stat changes
- boosts add momentum and a small lift bump, but stay capped below launch/updraft power
- collected gates disappear and are counted in HUD/eval metrics

## Test Coverage

Current tests cover:

- launch only fires from the ground
- relaunch is blocked during airtime
- gliding descends over time
- gliding clamps ordinary fall speed
- deployed glider dive keeps `Gliding` mode active, descends faster than ordinary glide, and is counted separately in evals
- floor collision clears downward velocity
- world collision proxies push the player out of obvious generated asset obstacles without affecting proxies above the player
- visual wind fields keep horizontal flow horizontal
- visual updraft fields include upward flow plus horizontal swirl
- wind-current evals gate sustained updraft visual rise, per-field guide/ribbon coverage, sustained updraft/crosswind visual-flow sample windows, updraft/crosswind visual depth span, baseline-relative guide/ribbon scale pulse, split guide/ribbon crosswind motion along the gameplay field direction, short-horizon guide/ribbon motion aligned with shared soft-edged gust-cell `WindField::flow_at`, observed frame-to-frame in-field guide/ribbon motion without field-center fallback, bounded observed visual speed/acceleration, zero visible wrap/jump artifacts, projected no-input horizontal crosswind drift, and wind-force deltas aligned with the field-axis correction direction toward that same shared flow; sustained visual-flow samples must clear meaningful fractions of the same visual floors instead of merely being nonzero, and focused visual tests also cover stream-specific crosswind variation, sampled-flow-facing crosswind guide orientation, phase-staggered thermal depth lanes, updraft ribbon sampling from the visible thermal lane, traveling clumped gust packets, and secondary gust energy between primary fronts so motes and ribbons do not move in lockstep
- wind response applies only while airborne and stays horizontally bounded
- lift fields only apply inside bounds while enabled
- authored gameplay lift route nodes pair visual and lift volumes
- aerial power-up route gates are collectible, directional, and capped
- visual field bounds and stream origins are deterministic
- smoothing factors do not overshoot
- camera ignores vertical-only launch velocity and sideways/backward movement for automatic follow-heading changes
- camera mouse X/Y input, pitch clamps, pitch/distance/framing helpers, surface-clearance lift, obstruction avoidance, route-spire obstruction exercise, and a bounded post-obstruction camera step so blockers cannot pull the camera into a one-frame snap
- camera follow direction smoothing limits rapid turn snaps
- lateral air input steers velocity toward the camera-relative plane
- pure backward air input brakes planar drift, while backward plus lateral input steers into rear-diagonal glide control with a directional skid/cupped-limb air-brake pose
- flight body yaw tracks lateral input direction, bounds the first-frame reversal spike, and recovers quickly after lateral input reversals
- frame-time diagnostics avoid invalid values
- animation phase advances from delta time
- idle breathing, glide airflow micro-motion, pressure-scaled dive flattening/arms-out limb trail, wind-reactive scarf streaming for generated and authored player nodes, and deployed-glider dive wing sweep are phase-driven and covered by pose unit tests
- wing visibility tracks glide mode
- `updraft_route` eval tracks `active_lift_fields`, `readable_lift_fields`, readable lift samples, unreadable lift samples, dynamic readable lift samples, wind-flow speed/variation/range, wind-guide depth/pulse/coherence, per-field visual coverage, sustained visual-flow sample windows, layered dynamic flow fields, and simultaneous crosswind-plus-updraft swirl force response so active lift must overlap a paired visible updraft with changing flow, layered aligned visual airflow, and lateral current
- `camera_mouse_control` eval tracks yaw/pitch offsets and route-spire obstruction adjustment without player movement in both app and simulation coverage
- `camera_yaw_stability` eval tracks stopped-input yaw stability
- `camera_strafe_stability` eval tracks right/left lateral movement without camera auto-orbit, including view-yaw and world-yaw drift
- `camera_turn_stability` eval tracks camera step/rotation deltas through rapid air turns and air braking while the scripted forward input stays active long enough to make the distance gate non-vacuous
- `air_control_response` eval tracks diagonal/lateral air steering, separate right/left response latency, stronger total/planar backward braking, pure-backward and diagonal body-heading intent, readable right/left air-turn plus deployed-glider dive and right/left/backward-right/backward-left air-brake key-pose coverage, authored bank-left/bank-right/dive/air-brake clip coverage, visible pose part count, bounded key-pose part rotation/translation deltas, torso pitch, arm spread, leg tuck, unsigned and signed lateral lean, wing-airflow strength, visible authored glider response/motion including dive-specific glider response/motion, zero key-pose samples below the readability floor, post-brake recovery, desired heading and aggregate plus right/left/backward-right/backward-left desired-travel alignment, average/p95/max body-heading error, tighter right/left and backward-right/backward-left body/travel heading samples and error, max body-yaw error step, body-yaw oscillation, left/right body-bank response, body-roll step smoothness, follow-direction error distribution, view-yaw/world-yaw drift, and movement-input camera non-coupling
- `pose_state_coverage` eval tracks grounded idle/walk/run samples, walk/run stride foot travel, walk/run leg opposition, readable launch/fall/glide/air-turn/air-brake/gliding-dive key-pose samples, right and left air-turn pose samples, pure-sideways right/left AirTurn samples, backward-right/backward-left diagonal air-brake samples, landing anticipation/recovery samples, zero gliding landing-anticipation samples, landing crouch, feet-forward tuck, asymmetric foot split, signed landing forward-fold/backward-bend, landing local torso bend, landing torso offset, landing-only app pose temporal deltas, zero upward-velocity fall poses, zero dive poses without dive input, and zero unreadable key poses in both the app and windowless sim harnesses; app evals also gate settled authored `idle`, `walk`, `run`, `launch`, `glide`, `fall`, and `land` clip samples separately from active transition blends
- `long_glide_visibility` eval tracks sustained archipelago traversal, aerial power-up collection/effect samples, and content-scale signals
- app evals track `world_collision_proxy_count`, `solid_world_collision_proxy_count`, `tree_world_collision_proxy_count`, `rock_world_collision_proxy_count`, `landmark_world_collision_proxy_count`, `world_collision_resolved_samples`, `world_collision_contact_samples`, `max_world_collision_push_m`, `terrain_rim_collision_proxy_count`, `terrain_rim_collision_contact_samples`, and `max_terrain_rim_collision_push_m`, with proxy-count gates so collidable props, route obstruction spires, per-kind solid asset distribution, and 16-segment terrain-rim contours cannot silently disappear; the island-visual catalog test also audits named solid visuals, per-island terrain-rim contour parity, and explicit camera-only blocker allowlists before aggregate eval counts can hide a missing proxy; `world_collision_contact` must sustain launch-mesa obstacle contact, `terrain_rim_collision_contact` must sustain launch-mesa rim contact, and `ground_taxi_control` must stay free of terrain-rim contact
- landing-required evals track landing anticipation, zero gliding landing-anticipation samples, signed forward-fold absorption, bounded backward bend, feet-forward landing tuck, asymmetric foot split, post-contact landing recovery, bounded local torso bend and torso offset, landing crouch depth, zero unreadable key-pose samples across both key landing poses, landing-only visible-pose temporal samples, and bounded landing pose-part rotation/translation deltas

Future tests should cover:

- launch source triggers
- power-up reset rules once explicit flight/session state exists
- explicit player and route-marker classification beyond the current scene-composition visual audit
- authored animation transitions for richer bank motion, brake, recovery, and landing states
- debug visualization toggles
- lift-field stacking and route-authoring rules

## Tuning Principles

- Tune with debug metrics visible.
- Change one force family at a time.
- Keep constants in testable code.
- Add tests for any fixed regression.
- Prefer a small route with repeatable measurements over subjective tuning in an empty world.
