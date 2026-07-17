#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

output_dir="${1:-target/eval/island_hero_gallery}"
summary_path="${output_dir}/summary.json"
manifest_path="${output_dir}/island_review_manifest.json"
semantic_audit_path="${output_dir}/semantic_scene_audit.json"
visual_audit_path="${output_dir}/visual_audit.json"
checkpoint_dir="${output_dir}/checkpoints"
expected_islands=41
expected_captures=123
expected_views=3
settle_frames=32
hold_frames=4
frames_per_view=$((settle_frames + hold_frames))
capture_frame_offset="${settle_frames}"
eval_check_names='["sample_count","horizontal_distance","max_altitude","max_speed","gliding_samples","grounded_samples","grounded_visual_foot_gap","lifted_samples","sky_island_count","active_island_count","active_chunk_count","near_lod_island_count","mid_lod_island_count","far_lod_island_count","visible_island_terrain_count","hidden_island_terrain_count","visible_island_impostor_count","visible_island_detail_count","hidden_island_detail_count","visible_route_beacon_count","weather_cloud_count","environment_motion_visual_count","environment_motion_offset","updraft_guide_visual_count","updraft_ribbon_visual_count","crosswind_guide_visual_count","crosswind_ribbon_visual_count","updraft_field_count","updraft_fields_with_guides","updraft_fields_with_ribbons","updraft_fields_with_guides_and_ribbons","updraft_flow_coherent_field_count","crosswind_field_count","crosswind_fields_with_guides","crosswind_fields_with_ribbons","crosswind_fields_with_guides_and_ribbons","crosswind_flow_coherent_field_count","updraft_visual_motion","updraft_visual_rise","updraft_visual_swirl_displacement","updraft_visual_depth_span","updraft_visual_scale_pulse","crosswind_visual_motion","crosswind_guide_flow_displacement","crosswind_ribbon_flow_displacement","crosswind_visual_lane_depth_span","crosswind_visual_scale_pulse","updraft_flow_coherent_visual_count","crosswind_flow_coherent_visual_count","crosswind_ribbon_flow_coherent_sample_count","updraft_visual_flow_alignment","crosswind_visual_flow_alignment","crosswind_ribbon_visual_flow_alignment","observed_updraft_flow_coherent_visual_count","observed_crosswind_flow_coherent_visual_count","observed_crosswind_ribbon_flow_coherent_sample_count","observed_updraft_visual_frame_motion","observed_updraft_visual_frame_rise","observed_updraft_visual_frame_swirl_displacement","observed_crosswind_visual_frame_motion","observed_crosswind_guide_frame_flow_displacement","observed_crosswind_ribbon_frame_flow_displacement","observed_updraft_visual_speed","observed_crosswind_visual_speed","observed_wind_visual_acceleration","observed_wind_visual_jump_count","observed_updraft_visual_flow_alignment","observed_crosswind_visual_flow_alignment","observed_crosswind_ribbon_visual_flow_alignment","sustained_wind_visual_flow_samples","sustained_updraft_visual_flow_samples","sustained_crosswind_visual_flow_samples","sustained_crosswind_ribbon_advected_flow_samples","island_terrain_surface_count","island_terrain_mesh_vertices","island_terrain_color_bands","island_terrain_material_weight_bands","island_terrain_material_channels","island_terrain_material_regions","island_terrain_texture_detail_bands","island_terrain_relief_range","island_terrain_archetype_count","island_cliff_color_bands","island_impostor_mesh_vertices","island_impostor_color_bands","procedural_island_body_count","primitive_island_body_count","island_body_silhouette_segments","island_body_mesh_vertices","generated_ground_cover_patch_count","ground_cover_blade_count","ground_cover_mesh_vertices","generated_tree_trunk_count","generated_tree_canopy_count","tree_trunk_mesh_vertices","tree_canopy_mesh_vertices","detail_biome_palette_count","generated_rock_count","rock_mesh_vertices","generated_landmark_count","generated_ruin_cluster_count","generated_route_cairn_count","generated_launch_beacon_count","generated_landing_garden_marker_count","generated_pond_surface_count","landmark_mesh_vertices","generated_weather_cloud_count","generated_weather_cloud_bank_count","weather_cloud_bank_depth","weather_cloud_lobe_count","weather_cloud_bank_lobe_count","weather_cloud_mesh_vertices","weather_cloud_filament_ribbon_detail_count","resident_island_visual_count","stream_visibility_changes_per_frame","hidden_island_visual_count","resident_island_visual_fraction","stream_spawned_visuals_per_frame","stream_despawned_visuals_per_frame","entity_count","objective_total_count","completed_objective_count","visual_asset_slot_count","gltf_scene_asset_slot_count","ready_visual_asset_slot_count","missing_visual_asset_slot_count","deferred_visual_asset_scene_count","streaming_visual_asset_slot_count","loaded_visual_asset_scene_count","dependency_loaded_visual_asset_scene_count","preload_ready_visual_asset_scene_count","always_preload_ready_visual_asset_slot_count","streaming_preload_ready_visual_asset_slot_count","spawned_visual_asset_scene_count","ready_visual_asset_scene_count","visible_authored_world_fixture_count","declared_animation_clip_count","ready_animation_clip_count","animation_player_count","animation_graph_count","failed_visual_asset_scene_count","power_up_count","collected_power_up_count","power_up_effect_samples","max_camera_distance","min_camera_surface_clearance","max_camera_player_angle","max_camera_step_distance","max_camera_rotation_delta","max_camera_orbit_alignment","max_abs_camera_view_yaw","max_camera_obstruction_adjustment","max_abs_camera_yaw_offset","min_camera_pitch_offset","max_camera_pitch_offset"]'
semantic_check_names='[
  "checkpoint_scene_pixel_hits",
  "checkpoint_scene_material_family_hits",
  "min_visible_scene_material_count",
  "checkpoint_scene_sample_kind_hits",
  "min_visible_scene_sample_kind_count",
  "island_hero_gallery_checkpoint_count",
  "island_hero_gallery_passing_checkpoint_count",
  "island_hero_gallery_checkpoint_metadata_count",
  "island_hero_gallery_unique_target_count",
  "island_hero_gallery_authored_target_count",
  "island_hero_gallery_unique_target_view_count",
  "island_hero_gallery_targets_with_all_views",
  "island_hero_gallery_target_terrain_coverage",
  "island_hero_gallery_authored_hero_coverage",
  "island_hero_gallery_authored_flora_coverage",
  "island_hero_gallery_authored_formation_coverage",
  "island_hero_gallery_authored_ruin_coverage",
  "island_hero_gallery_authored_water_coverage"
]'
visual_report_check_names='[
  "max_top_sky_fraction",
  "max_distant_scene_fraction",
  "max_distant_scene_component_count",
  "max_distant_scene_color_bucket_count",
  "max_distant_scene_horizontal_span_fraction",
  "max_distant_scene_vertical_span_fraction",
  "max_scene_material_family_count",
  "max_terrain_scene_fraction",
  "max_terrain_scene_tile_count",
  "max_terrain_scene_color_bucket_count",
  "max_foliage_scene_fraction",
  "max_foliage_scene_tile_count",
  "max_cloud_layer_fraction",
  "max_cloud_layer_component_count",
  "max_cloud_layer_horizontal_span_fraction",
  "max_cloud_layer_vertical_span_fraction"
]'
visual_image_check_names='[
  "width",
  "height",
  "mean_luma",
  "mean_luma",
  "luma_stddev",
  "colorfulness",
  "quantized_colors",
  "edge_density",
  "lower_scene_fraction",
  "center_scene_fraction",
  "center_edge_density",
  "scene_detail_tile_fraction",
  "scene_candidate_tile_count",
  "flat_scene_tile_fraction",
  "dominant_low_detail_scene_component_fraction",
  "severe_clipping_fraction",
  "transparent_pixel_fraction",
  "foreign_canvas_fraction",
  "hud_text_fraction"
]'

for dependency in cargo jq; do
  if ! command -v "${dependency}" >/dev/null 2>&1; then
    echo "${dependency} is required by the island hero gallery gate" >&2
    exit 2
  fi
done

validate_manifest() {
  local path="$1"
  local island_count="$2"
  local capture_count="$3"
  jq -e \
    --argjson expected_islands "${island_count}" \
    --argjson expected_captures "${capture_count}" \
    --argjson expected_views "${expected_views}" \
    --argjson settle_frames "${settle_frames}" \
    --argjson hold_frames "${hold_frames}" \
    --argjson frames_per_view "${frames_per_view}" \
    --argjson capture_frame_offset "${capture_frame_offset}" \
    '
    def pad($width):
      tostring as $value
      | ("0" * ([($width - ($value | length)), 0] | max)) + $value;
    . as $manifest
    | ["near", "mid", "traversal"] as $views
    | .scenario == "island_hero_gallery"
    and .island_count == $expected_islands
    and .capture_count == $expected_captures
    and .settle_frames == $settle_frames
    and .hold_frames == $hold_frames
    and .frames_per_view == $frames_per_view
    and (.islands | type) == "array"
    and (.captures | type) == "array"
    and (.islands | length) == $expected_islands
    and (.captures | length) == $expected_captures
    and ([.islands[].island_name] | unique | length) == $expected_islands
    and ([.captures[].checkpoint] | unique | length) == $expected_captures
    and ([.captures[].png_path] | unique | length) == $expected_captures
    and ([.captures[].sidecar_path] | unique | length) == $expected_captures
    and all(
      .islands | to_entries[];
      .key == .value.island_index
      and .value.capture_count == $expected_views
      and (.value.island_name | type) == "string"
      and (.value.island_slug | type) == "string"
    )
    and all(
      .captures | to_entries[];
      .key as $capture_index
      | .value as $capture
      | (($capture_index / $expected_views) | floor) as $island_index
      | ($capture_index % $expected_views) as $view_index
      | $manifest.islands[$island_index] as $island
      | $views[$view_index] as $view
      | ($capture_index * $frames_per_view + $capture_frame_offset) as $frame
      | ($frame | pad(4)) as $frame_label
      | ($island_index | pad(2)) as $island_label
      | ("island_\($island_label)_\($island.island_slug)_\($view)") as $checkpoint
      | $capture.island_index == $island_index
      and $capture.island_name == $island.island_name
      and $capture.target_island == $island.island_name
      and $capture.view == $view
      and $capture.frame == $frame
      and $capture.checkpoint == $checkpoint
      and $capture.capture_requested == true
      and $capture.screenshot_requested == true
      and $capture.sidecar_written == true
      and $capture.png_exists == true
      and $capture.sidecar_exists == true
      and ($capture.png_path | endswith("/checkpoints/\($frame_label)_\($checkpoint).png"))
      and (
        $capture.sidecar_path
        | endswith("/checkpoints/\($frame_label)_\($checkpoint).markers.json")
      )
    )
    ' "${path}" >/dev/null
}

validate_summary() {
  local path="$1"
  local manifest="$2"
  local capture_count="$3"
  jq -e \
    --argjson expected_captures "${capture_count}" \
    --argjson expected_checks "${eval_check_names}" \
    --slurpfile manifest "${manifest}" \
    '
    .scenario == "island_hero_gallery"
    and .passed == true
    and (.checks | type) == "array"
    and [.checks[].name] == $expected_checks
    and all(.checks[]; (.name | type) == "string" and .passed == true)
    and (.artifacts.screenshot_png | type) == "string"
    and (.artifacts.checkpoint_screenshots | type) == "array"
    and (.artifacts.checkpoint_marker_metadata | type) == "array"
    and (.artifacts.checkpoint_screenshots | length) == $expected_captures
    and (.artifacts.checkpoint_marker_metadata | length) == $expected_captures
    and .artifacts.checkpoint_screenshots == [$manifest[0].captures[].png_path]
    and .artifacts.checkpoint_marker_metadata == [$manifest[0].captures[].sidecar_path]
    ' "${path}" >/dev/null
}

validate_sidecars() {
  local manifest="$1"
  local capture_count="$2"
  shift 2
  jq -e -s \
    --argjson expected_captures "${capture_count}" \
    --slurpfile manifest "${manifest}" \
    '
    length == $expected_captures
    and all(
      to_entries[];
      .key as $capture_index
      | .value as $sidecar
      | $manifest[0].captures[$capture_index] as $capture
      | $sidecar.scenario == "island_hero_gallery"
      and $sidecar.passed == true
      and $sidecar.frame == $capture.frame
      and $sidecar.checkpoint == $capture.checkpoint
      and $sidecar.target_island == $capture.target_island
      and $sidecar.target_view == $capture.view
      and $sidecar.screenshot == $capture.png_path
    )
    ' "$@" >/dev/null
}

validate_semantic_audit() {
  local path="$1"
  local summary="$2"
  local manifest="$3"
  local capture_count="$4"
  jq -e \
    --argjson expected_captures "${capture_count}" \
    --argjson expected_checks "${semantic_check_names}" \
    --slurpfile summary "${summary}" \
    --slurpfile manifest "${manifest}" \
    '
    .passed == true
    and .checkpoint_count == $expected_captures
    and .profile.name == "island_hero_gallery"
    and .profile.expected_materials == []
    and .profile.conditional_expected_materials == []
    and .profile.expected_scene_sample_kinds == []
    and (.checks | type) == "array"
    and [.checks[].name] == $expected_checks
    and all(.checks[]; .passed == true)
    and (.checkpoints | type) == "array"
    and (.checkpoints | length) == $expected_captures
    and all(
      .checkpoints | to_entries[];
      .key as $capture_index
      | .value as $checkpoint
      | $manifest[0].captures[$capture_index] as $capture
      | $checkpoint.passed == true
      and $checkpoint.metadata_path
        == $summary[0].artifacts.checkpoint_marker_metadata[$capture_index]
      and $checkpoint.metadata_path == $capture.sidecar_path
      and $checkpoint.screenshot_path == $capture.png_path
      and $checkpoint.checkpoint == $capture.checkpoint
      and $checkpoint.target_island == $capture.target_island
      and $checkpoint.review_view == $capture.view
    )
    ' "${path}" >/dev/null
}

validate_visual_audit() {
  local path="$1"
  local summary="$2"
  local expected_images="$3"
  jq -e \
    --argjson expected_images "${expected_images}" \
    --argjson expected_report_checks "${visual_report_check_names}" \
    --argjson expected_image_checks "${visual_image_check_names}" \
    --slurpfile summary "${summary}" \
    '
    ([$summary[0].artifacts.screenshot_png]
      + $summary[0].artifacts.checkpoint_screenshots) as $expected_paths
    | .passed == true
    and .profile.name == "island_gallery"
    and .image_count == $expected_images
    and (.checks | type) == "array"
    and [.checks[].name] == $expected_report_checks
    and all(.checks[]; .passed == true)
    and (.images | type) == "array"
    and (.images | length) == $expected_images
    and all(
      .images | to_entries[];
      .value.passed == true
      and .value.path == $expected_paths[.key]
      and (.value.checks | type) == "array"
      and [.value.checks[].name] == $expected_image_checks
      and all(.value.checks[]; .passed == true)
    )
    ' "${path}" >/dev/null
}

expect_validation_failure() {
  local label="$1"
  shift
  if "$@"; then
    echo "gallery gate self-test unexpectedly accepted ${label}" >&2
    return 1
  fi
}

run_validator_self_tests() {
  local test_dir
  test_dir="$(mktemp -d "${TMPDIR:-/tmp}/nau-gallery-gate.XXXXXX")"
  local test_manifest="${test_dir}/manifest.json"
  local test_summary="${test_dir}/summary.json"
  local test_semantic="${test_dir}/semantic.json"
  local test_visual="${test_dir}/visual.json"
  local test_islands=2
  local test_captures=$((test_islands * expected_views))
  mkdir -p "${test_dir}/checkpoints"

  jq -n \
    --arg root "${test_dir}" \
    --argjson island_count "${test_islands}" \
    --argjson view_count "${expected_views}" \
    --argjson settle_frames "${settle_frames}" \
    --argjson hold_frames "${hold_frames}" \
    --argjson frames_per_view "${frames_per_view}" \
    '
    def pad($width):
      tostring as $value
      | ("0" * ([($width - ($value | length)), 0] | max)) + $value;
    ["near", "mid", "traversal"] as $views
    | [
        range(0; $island_count)
        | {
            island_index: .,
            island_name: "island \(.)",
            island_slug: "island_\(.)",
            capture_count: $view_count
          }
      ] as $islands
    | [
        range(0; ($island_count * $view_count)) as $capture_index
        | (($capture_index / $view_count) | floor) as $island_index
        | ($capture_index % $view_count) as $view_index
        | ($capture_index * $frames_per_view + $settle_frames) as $frame
        | ($frame | pad(4)) as $frame_label
        | ($island_index | pad(2)) as $island_label
        | $views[$view_index] as $view
        | ("island_\($island_label)_island_\($island_index)_\($view)") as $checkpoint
        | {
            island_index: $island_index,
            island_name: "island \($island_index)",
            frame: $frame,
            checkpoint: $checkpoint,
            target_island: "island \($island_index)",
            view: $view,
            png_path: "\($root)/checkpoints/\($frame_label)_\($checkpoint).png",
            sidecar_path: "\($root)/checkpoints/\($frame_label)_\($checkpoint).markers.json",
            capture_requested: true,
            screenshot_requested: true,
            sidecar_written: true,
            png_exists: true,
            sidecar_exists: true
          }
      ] as $captures
    | {
        scenario: "island_hero_gallery",
        island_count: $island_count,
        capture_count: ($island_count * $view_count),
        settle_frames: $settle_frames,
        hold_frames: $hold_frames,
        frames_per_view: $frames_per_view,
        islands: $islands,
        captures: $captures
      }
    ' > "${test_manifest}"

  jq \
    --arg final "${test_dir}/final.png" \
    --argjson expected_checks "${eval_check_names}" \
    '{
      scenario,
      passed: true,
      checks: [$expected_checks[] | {name: ., passed: true}],
      artifacts: {
        screenshot_png: $final,
        checkpoint_screenshots: [.captures[].png_path],
        checkpoint_marker_metadata: [.captures[].sidecar_path]
      }
    }' "${test_manifest}" > "${test_summary}"

  local sidecars=()
  local capture_index
  for ((capture_index = 0; capture_index < test_captures; capture_index += 1)); do
    local sidecar_path
    sidecar_path="$(
      jq -r --argjson capture_index "${capture_index}" \
        '.captures[$capture_index].sidecar_path' "${test_manifest}"
    )"
    jq --argjson capture_index "${capture_index}" \
      '.captures[$capture_index] | {
        scenario: "island_hero_gallery",
        passed: true,
        frame,
        checkpoint,
        target_island,
        target_view: .view,
        screenshot: .png_path
      }' "${test_manifest}" > "${sidecar_path}"
    sidecars+=("${sidecar_path}")
  done

  jq -n \
    --argjson expected_checks "${semantic_check_names}" \
    --slurpfile summary "${test_summary}" \
    --slurpfile manifest "${test_manifest}" \
    '{
      passed: true,
      checkpoint_count: ($manifest[0].captures | length),
      profile: {
        name: "island_hero_gallery",
        expected_materials: [],
        conditional_expected_materials: [],
        expected_scene_sample_kinds: []
      },
      checks: [$expected_checks[] | {name: ., passed: true}],
      checkpoints: [
        $manifest[0].captures[]
        | {
            metadata_path: .sidecar_path,
            screenshot_path: .png_path,
            checkpoint,
            target_island,
            review_view: .view,
            passed: true
          }
      ]
    }' > "${test_semantic}"

  jq -n \
    --argjson report_checks "${visual_report_check_names}" \
    --argjson image_checks "${visual_image_check_names}" \
    --slurpfile summary "${test_summary}" \
    '([$summary[0].artifacts.screenshot_png]
      + $summary[0].artifacts.checkpoint_screenshots) as $paths
    | {
        passed: true,
        image_count: ($paths | length),
        profile: {name: "island_gallery"},
        checks: [$report_checks[] | {name: ., passed: true}],
        images: [
          $paths[]
          | {
              path: .,
              passed: true,
              checks: [$image_checks[] | {name: ., passed: true}]
            }
        ]
      }
    ' > "${test_visual}"

  validate_manifest "${test_manifest}" "${test_islands}" "${test_captures}"
  validate_summary "${test_summary}" "${test_manifest}" "${test_captures}"
  validate_sidecars "${test_manifest}" "${test_captures}" "${sidecars[@]}"
  validate_semantic_audit \
    "${test_semantic}" "${test_summary}" "${test_manifest}" "${test_captures}"
  validate_visual_audit "${test_visual}" "${test_summary}" "$((test_captures + 1))"

  jq '.captures[0:2] |= reverse' "${test_manifest}" > "${test_dir}/reordered-manifest.json"
  expect_validation_failure "reordered captures" \
    validate_manifest "${test_dir}/reordered-manifest.json" "${test_islands}" "${test_captures}"
  jq '.captures[0].frame += 1' "${test_manifest}" > "${test_dir}/wrong-frame.json"
  expect_validation_failure "wrong capture frame" \
    validate_manifest "${test_dir}/wrong-frame.json" "${test_islands}" "${test_captures}"

  jq '.checks = []' "${test_summary}" > "${test_dir}/empty-summary-checks.json"
  expect_validation_failure "empty eval checks" \
    validate_summary "${test_dir}/empty-summary-checks.json" "${test_manifest}" "${test_captures}"
  jq 'del(.checks[0])' "${test_summary}" > "${test_dir}/missing-summary-check.json"
  expect_validation_failure "missing eval check" \
    validate_summary "${test_dir}/missing-summary-check.json" "${test_manifest}" "${test_captures}"
  jq '.artifacts.checkpoint_screenshots |= reverse' "${test_summary}" \
    > "${test_dir}/reordered-summary.json"
  expect_validation_failure "reordered summary screenshots" \
    validate_summary "${test_dir}/reordered-summary.json" "${test_manifest}" "${test_captures}"

  local reordered_sidecars=("${sidecars[1]}" "${sidecars[0]}" "${sidecars[@]:2}")
  expect_validation_failure "reordered sidecars" \
    validate_sidecars "${test_manifest}" "${test_captures}" "${reordered_sidecars[@]}"

  jq '.checks = []' "${test_semantic}" > "${test_dir}/empty-semantic-checks.json"
  expect_validation_failure "empty semantic checks" \
    validate_semantic_audit \
      "${test_dir}/empty-semantic-checks.json" \
      "${test_summary}" \
      "${test_manifest}" \
      "${test_captures}"
  jq '.checks[0].name = "unexpected_check"' "${test_semantic}" \
    > "${test_dir}/wrong-semantic-check.json"
  expect_validation_failure "wrong semantic check names" \
    validate_semantic_audit \
      "${test_dir}/wrong-semantic-check.json" \
      "${test_summary}" \
      "${test_manifest}" \
      "${test_captures}"

  jq '.checks = []' "${test_visual}" > "${test_dir}/empty-visual-checks.json"
  expect_validation_failure "empty visual report checks" \
    validate_visual_audit \
      "${test_dir}/empty-visual-checks.json" "${test_summary}" "$((test_captures + 1))"
  jq '.images[0].checks = []' "${test_visual}" > "${test_dir}/empty-image-checks.json"
  expect_validation_failure "empty visual image checks" \
    validate_visual_audit \
      "${test_dir}/empty-image-checks.json" "${test_summary}" "$((test_captures + 1))"
  jq '.images |= reverse' "${test_visual}" > "${test_dir}/reordered-images.json"
  expect_validation_failure "reordered visual images" \
    validate_visual_audit \
      "${test_dir}/reordered-images.json" "${test_summary}" "$((test_captures + 1))"

  rm -rf "${test_dir}"
  echo "island hero gallery gate self-test: passed"
}

if [[ "${1:-}" == "--self-test" ]]; then
  run_validator_self_tests
  exit 0
fi

NAU_EVAL_SCREENSHOT=1 \
  NAU_EVAL_ASSET_AUDIT=0 \
  NAU_EVAL_SEMANTIC_SCENE_AUDIT=1 \
  NAU_EVAL_VISUAL_AUDIT=1 \
  ./tools/eval.sh island_hero_gallery "${output_dir}"

for artifact in \
  "${summary_path}" \
  "${manifest_path}" \
  "${semantic_audit_path}" \
  "${visual_audit_path}"
do
  if [[ ! -s "${artifact}" ]]; then
    echo "missing island hero gallery artifact: ${artifact}" >&2
    exit 1
  fi
done

png_count="$(
  find "${checkpoint_dir}" -maxdepth 1 -type f -name '*.png' -print | wc -l | tr -d '[:space:]'
)"
sidecar_count="$(
  find "${checkpoint_dir}" -maxdepth 1 -type f -name '*.markers.json' -print \
    | wc -l \
    | tr -d '[:space:]'
)"
if [[ "${png_count}" != "${expected_captures}" ]]; then
  echo "expected ${expected_captures} gallery checkpoint PNGs, found ${png_count}" >&2
  exit 1
fi
if [[ "${sidecar_count}" != "${expected_captures}" ]]; then
  echo "expected ${expected_captures} gallery sidecars, found ${sidecar_count}" >&2
  exit 1
fi

if ! validate_manifest "${manifest_path}" "${expected_islands}" "${expected_captures}"; then
  echo "island hero gallery manifest coverage failed: ${manifest_path}" >&2
  jq '{
    scenario,
    island_count,
    capture_count,
    islands: (.islands | length),
    captures: (.captures | length),
    failed_captures: [
      .captures[]
      | select(
          .capture_requested != true
          or .screenshot_requested != true
          or .sidecar_written != true
          or .png_exists != true
          or .sidecar_exists != true
        )
    ]
  }' "${manifest_path}" >&2 || true
  exit 1
fi

if ! validate_summary "${summary_path}" "${manifest_path}" "${expected_captures}"; then
  echo "island hero gallery eval summary coverage failed: ${summary_path}" >&2
  jq '{scenario, passed, artifacts}' "${summary_path}" >&2 || true
  exit 1
fi

sidecars=()
while IFS= read -r sidecar; do
  sidecars+=("${sidecar}")
done < <(jq -r '.artifacts.checkpoint_marker_metadata[]' "${summary_path}")

if [[ "${#sidecars[@]}" -ne "${expected_captures}" ]]; then
  echo "expected ${expected_captures} sidecars in eval summary, found ${#sidecars[@]}" >&2
  exit 1
fi

if ! validate_sidecars "${manifest_path}" "${expected_captures}" "${sidecars[@]}"; then
  echo "gallery sidecar target-island/view coverage failed" >&2
  exit 1
fi

if ! validate_semantic_audit \
  "${semantic_audit_path}" \
  "${summary_path}" \
  "${manifest_path}" \
  "${expected_captures}"
then
  echo "island hero gallery semantic audit coverage failed: ${semantic_audit_path}" >&2
  jq '{
    passed,
    checkpoint_count,
    profile,
    failed_checks: [.checks[] | select(.passed != true)],
    failed_checkpoints: [
      .checkpoints[]
      | select(.passed != true)
      | {checkpoint, metadata_path}
    ]
  }' "${semantic_audit_path}" >&2 || true
  exit 1
fi

if ! validate_visual_audit \
  "${visual_audit_path}" \
  "${summary_path}" \
  "$((expected_captures + 1))"
then
  echo "island hero gallery visual audit coverage failed: ${visual_audit_path}" >&2
  jq '{
    passed,
    image_count,
    profile,
    failed_checks: [.checks[] | select(.passed != true)],
    failed_images: [.images[] | select(.passed != true) | {path}]
  }' "${visual_audit_path}" >&2 || true
  exit 1
fi

echo "island hero gallery gate: ${output_dir}"
