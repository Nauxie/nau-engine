# NAU Engine Showcase

Current gameplay captures and generated review artifacts from the Rust/Bevy traversal sandbox.

## Current Gameplay

<p align="center">
  <img src="docs/showcase/air_gate_route_midflight.png" alt="Nau gliding through the twelve-gate floating-island route" width="100%">
</p>

<p align="center">
  <img src="docs/showcase/updraft_high_glide.png" alt="Glider approaching visible updraft columns and route markers" width="49%">
  <img src="docs/showcase/great_sky_plateau_high_crossing.png" alt="Glider crossing the high archipelago route" width="49%">
</p>

<p align="center">
  <img src="docs/showcase/long_glide_archipelago_midroute.png" alt="Long-glide archipelago route in flight" width="49%">
  <img src="docs/showcase/great_sky_plateau_summit_climb.png" alt="Glider climbing toward the Great Sky Plateau summit" width="49%">
</p>

## World-Floor Scale

<p align="center">
  <img src="docs/showcase/world_floor_far_route.png" alt="The floating-island route above the streamed playable world floor" width="49%">
  <img src="docs/showcase/world_floor_distant_islands.png" alt="Distant floating islands above the playable world floor" width="49%">
</p>

The route spans 41 floating islands above a streamed, landable world floor that supports grounded traversal and relaunch.

## UI And Objectives

<p align="center">
  <img src="docs/showcase/pause_menu.png" alt="Current pause menu showing the shared twelve-gate objective total" width="72%">
</p>

Twelve one-shot air gates cover the main corridor and optional low and high-altitude branches. The HUD and pause menu share the authoritative collection total.

## Player And Glider Review

<p align="center">
  <img src="docs/showcase/player_glider_attachment_sheet.png" alt="Player and glider attachment pose sheet" width="49%">
  <img src="docs/showcase/player_rig_stress_review_sheet.png" alt="Player rig stress review sheet" width="49%">
</p>

<p align="center">
  <img src="docs/showcase/player_motion_integrity_review_sheet.png" alt="Player motion integrity review sheet" width="72%">
</p>

These generated sheets review authored attachment, silhouette, transition, and motion integrity across grounded movement, launch, glide, dive, air-brake, and landing states.

## Measured Content

- Terrain export: 41 islands, 164 meshes, 194,996 vertices, 366,048 triangles.
- Visual content export: 570 meshes, 538,211 vertices, 554,894 triangles.
- Wind visual export: 38 fields, 3,338 visuals, 417,852 sampled tracks.

These totals were regenerated from the current source with the commands below. The checked-in images are selected outputs from the same eval, world-floor evidence, UI, and pose-preview pipelines.

## Reproduce

```sh
./tools/player_pose_preview.sh target/player_pose_preview
cargo run -- --eval updraft_route --eval-output target/eval/updraft_route
cargo run -- --eval long_glide_visibility --eval-output target/eval/long_glide_visibility
cargo run -- --eval great_sky_plateau_route --eval-output target/eval/great_sky_plateau_route
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
./tools/wind_visual_export.sh target/wind_visual_export
```
