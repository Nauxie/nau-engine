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
- `Gliding`: glider input is held while airborne and not diving or launching.

## Current Rules

- `E` launch is ground-gated.
- Launch is one use per airtime.
- Launch gives vertical velocity and a small forward bonus.
- Gliding reduces gravity and clamps fall speed.
- Gliding does not create altitude on its own.
- Airborne and gliding `W`/`A`/`S`/`D` input uses the stable camera follow direction plus explicit mouse orbit as the camera-relative movement basis. `W`/`A`/`D` and rear diagonals steer planar velocity with input-aligned acceleration, while pure `S` stays an air-brake/reverse-speed-limited control.
- Airborne `S` input brakes forward motion first, then allows limited backward drift instead of unrestricted reverse flight.
- Visual `WindField` volumes are finite axis-aligned boxes for readable wind/updraft streams, gust/swirl diagnostics, and bounded horizontal airborne wind response.
- Crosswind `WindField`s push airborne horizontal velocity toward their dynamic flow; updraft `WindField`s add only lateral swirl current.
- Gameplay `LiftField` updraft volumes are separate finite boxes that add vertical velocity while the player is airborne inside them.
- Authored gameplay updraft route nodes must pair the visual `WindField` and gameplay `LiftField` at the same center and extents.
- Lift fields clamp against their configured maximum upward speed instead of granting unbounded climb.
- Aerial power-up gates are one-time route pickups that apply a small capped forward/upward boost while airborne, then disappear.
- Diving adds downward acceleration.
- The floor clamp prevents the player from ending below the floor or retaining downward velocity after collision.
- Generated tree trunks, rocks, route cairns, launch beacons, recovery masts, and target markers expose simple world-collision proxies; player movement resolves horizontally out of those proxies and clears velocity into the collision normal.
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

Dive:

- commits the player downward
- increases urgency and speed
- should be reversible only with enough altitude, wind, or later abilities

Landing:

- landing anticipation and post-touchdown recovery are explicit pose intents now
- a full authored landing locomotion state with slope-aware collider handling is still future work
- needs collision and slope logic before polish

Wind/updraft:

- visual `WindField` volumes are the shared source for stream visuals, diagnostics, and bounded horizontal airborne wind current
- crosswinds push laterally without adding vertical lift
- updraft wind swirl can bend horizontal motion, but vertical climb still comes from paired `LiftField` volumes
- active lift should be readable through paired updraft visuals, gusting ribbons/motes, and debug bounds before richer particles, cloth/glider motion, vegetation, clouds, or other environment art

Power-ups:

- boost gates should read as route affordances, not hidden stat changes
- boosts add momentum and a small lift bump, but stay capped below launch/updraft power
- collected gates disappear and are counted in HUD/eval metrics

## Test Coverage

Current tests cover:

- launch only fires from the ground
- relaunch is blocked during airtime
- gliding descends over time
- gliding clamps fall speed
- floor collision clears downward velocity
- world collision proxies push the player out of obvious generated asset obstacles without affecting proxies above the player
- visual wind fields keep horizontal flow horizontal
- visual updraft fields include upward flow plus horizontal swirl
- wind response applies only while airborne and stays horizontally bounded
- lift fields only apply inside bounds while enabled
- authored gameplay lift route nodes pair visual and lift volumes
- aerial power-up route gates are collectible, directional, and capped
- visual field bounds and stream origins are deterministic
- smoothing factors do not overshoot
- camera ignores vertical-only launch velocity and sideways/backward movement for automatic follow-heading changes
- camera mouse X/Y input, pitch clamps, pitch/distance/framing helpers, surface-clearance lift, and obstruction avoidance
- camera follow direction smoothing limits rapid turn snaps
- lateral air input steers velocity toward the camera-relative plane
- pure backward air input brakes planar drift, while backward plus lateral input steers into rear-diagonal glide control
- flight body yaw tracks lateral input direction, bounds the first-frame reversal spike, and recovers quickly after lateral input reversals
- frame-time diagnostics avoid invalid values
- animation phase advances from delta time
- wing visibility tracks glide mode
- `updraft_route` eval tracks `active_lift_fields`, `readable_lift_fields`, readable lift samples, unreadable lift samples, dynamic readable lift samples, wind-flow speed/variation/range, and wind-force response so active lift must overlap a paired visible updraft with changing flow and lateral current
- `camera_mouse_control` eval tracks yaw/pitch offsets and obstruction adjustment without player movement
- `camera_yaw_stability` eval tracks stopped-input yaw stability
- `camera_strafe_stability` eval tracks right/left lateral movement without camera auto-orbit, including view-yaw and world-yaw drift
- `camera_turn_stability` eval tracks camera step/rotation deltas through rapid air turns and air braking
- `air_control_response` eval tracks diagonal/lateral air steering, separate right/left response latency, stronger total/planar backward braking, pure-backward and diagonal body-heading intent, readable dive/air-brake key-pose coverage, torso pitch, arm spread, leg tuck, unsigned and signed lateral lean, wing-airflow strength, zero key-pose samples below the readability floor, post-brake recovery, desired heading alignment, average/p95/max body-heading error, lateral and backward-diagonal body/travel heading error, max body-yaw error step, body-yaw oscillation, left/right body-bank response, body-roll step smoothness, follow-direction error distribution, view-yaw/world-yaw drift, and movement-input camera non-coupling
- `long_glide_visibility` eval tracks sustained archipelago traversal, aerial power-up collection/effect samples, and content-scale signals
- app evals track `world_collision_proxy_count`, `world_collision_resolved_samples`, `world_collision_contact_samples`, and `max_world_collision_push_m`, with a proxy-count gate so collidable asset props cannot silently disappear and a `world_collision_contact` route that must sustain meaningful launch-mesa obstacle contact
- landing-required evals track landing anticipation, landing flare, post-contact landing recovery, landing crouch depth, and zero unreadable key-pose samples across both key landing poses

Future tests should cover:

- launch source triggers
- power-up reset rules once explicit flight/session state exists
- explicit player and route-marker classification beyond the current scene-composition visual audit
- authored animation transitions for bank, brake, recovery, and landing states
- debug visualization toggles
- lift-field stacking and route-authoring rules

## Tuning Principles

- Tune with debug metrics visible.
- Change one force family at a time.
- Keep constants in testable code.
- Add tests for any fixed regression.
- Prefer a small route with repeatable measurements over subjective tuning in an empty world.
