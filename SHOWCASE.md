# NAU Engine Showcase

Visual snapshots from the current Rust/Bevy traversal sandbox: clean eval screenshots, generated character rig review sheets, and export metrics from the content pipeline.

## Route Captures

<p align="center">
  <img src="docs/showcase/great_sky_plateau_high_crossing.png" alt="Glider crossing the high archipelago route" width="100%">
</p>

<p align="center">
  <img src="docs/showcase/updraft_high_glide.png" alt="Glider approaching updraft columns and route markers" width="49%">
  <img src="docs/showcase/long_glide_archipelago_midroute.png" alt="Long glide archipelago midroute screenshot" width="49%">
</p>

<p align="center">
  <img src="docs/showcase/great_sky_plateau_summit_climb.png" alt="Great sky plateau summit climb screenshot" width="100%">
</p>

## Player And Glider Review

<p align="center">
  <img src="docs/showcase/player_glider_attachment_sheet.png" alt="Player and glider attachment pose sheet" width="49%">
  <img src="docs/showcase/player_rig_stress_review_sheet.png" alt="Player rig stress review sheet" width="49%">
</p>

<p align="center">
  <img src="docs/showcase/player_motion_integrity_review_sheet.png" alt="Player motion integrity review sheet" width="72%">
</p>

## Generated Content

- Terrain export: 41 islands, 164 meshes, 167,444 vertices, 318,816 triangles.
- Visual content export: 543 meshes, 530,169 vertices, 542,804 triangles.
- Wind visual export: 30 fields, 2,626 visuals, 328,812 sampled tracks.

## Reproduce

```sh
./tools/player_pose_preview.sh target/player_pose_preview
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh updraft_route target/eval/updraft_route
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh long_glide_visibility target/eval/long_glide_visibility
NAU_EVAL_SCREENSHOT=1 ./tools/eval.sh great_sky_plateau_route target/eval/great_sky_plateau_route
./tools/terrain_export.sh target/terrain_export
./tools/visual_content_export.sh target/visual_content_export
./tools/wind_visual_export.sh target/wind_visual_export
```
