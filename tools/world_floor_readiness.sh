#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 0 && "$#" -ne 6 ]]; then
  cat >&2 <<'EOF'
Usage:
  tools/world_floor_readiness.sh
  tools/world_floor_readiness.sh <baseline_perf_summary.json> <candidate_perf_summary.json> <baseline_freeflight_profile.json> <candidate_freeflight_profile.json> <baseline_ground_traversal_profile.json> <candidate_ground_traversal_profile.json>

Default paths:
  target/eval/perf_baseline/main_visible_no_screenshot/perf_summary.json
  target/eval/perf_baseline/candidate_visible_no_screenshot/perf_summary.json
  target/eval/play_profile/main_scripted_freeflight.json
  target/eval/play_profile/candidate_scripted_freeflight.json
  target/eval/play_profile/main_scripted_ground_traversal.json
  target/eval/play_profile/candidate_scripted_ground_traversal.json
EOF
  exit 2
fi

baseline_perf="${1:-target/eval/perf_baseline/main_visible_no_screenshot/perf_summary.json}"
candidate_perf="${2:-target/eval/perf_baseline/candidate_visible_no_screenshot/perf_summary.json}"
baseline_freeflight="${3:-target/eval/play_profile/main_scripted_freeflight.json}"
candidate_freeflight="${4:-target/eval/play_profile/candidate_scripted_freeflight.json}"
baseline_ground_traversal="${5:-target/eval/play_profile/main_scripted_ground_traversal.json}"
candidate_ground_traversal="${6:-target/eval/play_profile/candidate_scripted_ground_traversal.json}"
world_floor_max_visible_tiles="${NAU_WORLD_FLOOR_MAX_VISIBLE_TILES:-9}"
world_floor_min_visible_tiles="${NAU_WORLD_FLOOR_MIN_VISIBLE_TILES:-9}"
world_floor_max_resident_tiles="${NAU_WORLD_FLOOR_MAX_RESIDENT_TILES:-25}"
world_floor_initial_spawned_tiles="${NAU_WORLD_FLOOR_INITIAL_SPAWNED_TILES:-9}"
world_floor_max_spawned_tiles_per_frame="${NAU_WORLD_FLOOR_MAX_SPAWNED_TILES_PER_FRAME:-1}"
world_floor_max_despawned_tiles_per_frame="${NAU_WORLD_FLOOR_MAX_DESPAWNED_TILES_PER_FRAME:-1}"
world_floor_max_mesh_vertices="${NAU_WORLD_FLOOR_MAX_MESH_VERTICES:-40000}"
world_floor_max_mesh_triangles="${NAU_WORLD_FLOOR_MAX_MESH_TRIANGLES:-37200}"
world_floor_max_materials="${NAU_WORLD_FLOOR_MAX_MATERIALS:-2}"
world_floor_min_biomes="${NAU_WORLD_FLOOR_MIN_BIOMES:-4}"
world_floor_min_terrain_features="${NAU_WORLD_FLOOR_MIN_TERRAIN_FEATURES:-5}"
world_floor_min_color_bands="${NAU_WORLD_FLOOR_MIN_COLOR_BANDS:-12}"
world_floor_min_river_vertices="${NAU_WORLD_FLOOR_MIN_RIVER_VERTICES:-2}"
world_floor_min_relief_range_m="${NAU_WORLD_FLOOR_MIN_RELIEF_RANGE_M:-14}"
ground_traversal_min_grounded_ratio="${NAU_WORLD_FLOOR_MIN_GROUNDED_RATIO:-0.98}"
ground_traversal_min_horizontal_travel_m="${NAU_WORLD_FLOOR_MIN_GROUND_TRAVEL_M:-300}"
failed=0

if [[ ! -s docs/DECISIONS/0002-world-floor-perf-first.md ]]; then
  echo "missing world-floor performance ADR: docs/DECISIONS/0002-world-floor-perf-first.md" >&2
  exit 1
fi

profile_max_metric() {
  local profile="$1"
  local key="$2"
  jq -er --arg key "${key}" '.max[$key] // empty' "${profile}"
}

passes_le() {
  local value="$1"
  local threshold="$2"
  awk -v value="${value}" -v threshold="${threshold}" \
    'BEGIN { print (value <= threshold ? "true" : "false") }'
}

passes_ge() {
  local value="$1"
  local threshold="$2"
  awk -v value="${value}" -v threshold="${threshold}" \
    'BEGIN { print (value >= threshold ? "true" : "false") }'
}

validate_profile_script() {
  local profile="$1"
  local expected_script="$2"
  local actual_script

  actual_script="$(jq -r '.script // ""' "${profile}")"
  printf 'scripted_profile\tpath=%s\texpected=%s\tactual=%s\tpassed=%s\n' \
    "${profile}" "${expected_script}" "${actual_script}" \
    "$([[ "${actual_script}" == "${expected_script}" ]] && printf true || printf false)"
  if [[ "${actual_script}" != "${expected_script}" ]]; then
    failed=1
  fi
}

validate_ground_traversal_profile() {
  local profile="$1"
  local label="$2"
  local grounded_ratio
  local horizontal_travel_m
  local grounded_passed
  local travel_passed

  if ! grounded_ratio="$(jq -er '.ground_contact.grounded_ratio // empty' "${profile}")"; then
    printf '%s\tgrounded_ratio\tpassed=missing_metric\n' "${label}"
    failed=1
  else
    grounded_passed="$(passes_ge "${grounded_ratio}" "${ground_traversal_min_grounded_ratio}")"
    printf '%s\tgrounded_ratio\tvalue=%s\tmin_required=%s\tpassed=%s\n' \
      "${label}" "${grounded_ratio}" "${ground_traversal_min_grounded_ratio}" "${grounded_passed}"
    if [[ "${grounded_passed}" != "true" ]]; then
      failed=1
    fi
  fi

  if ! horizontal_travel_m="$(jq -er '.activity.horizontal_travel_m // empty' "${profile}")"; then
    printf '%s\thorizontal_travel_m\tpassed=missing_metric\n' "${label}"
    failed=1
  else
    travel_passed="$(passes_ge "${horizontal_travel_m}" "${ground_traversal_min_horizontal_travel_m}")"
    printf '%s\thorizontal_travel_m\tvalue=%s\tmin_required=%s\tpassed=%s\n' \
      "${label}" "${horizontal_travel_m}" "${ground_traversal_min_horizontal_travel_m}" "${travel_passed}"
    if [[ "${travel_passed}" != "true" ]]; then
      failed=1
    fi
  fi
}

validate_floor_max_le() {
  local profile="$1"
  local key="$2"
  local threshold="$3"
  local value
  local passed

  if ! value="$(profile_max_metric "${profile}" "${key}")"; then
    printf 'candidate_world_floor\t%s\tpassed=missing_metric\n' "${key}"
    failed=1
    return
  fi

  passed="$(passes_le "${value}" "${threshold}")"
  printf 'candidate_world_floor\t%s\tvalue=%s\tmax_allowed=%s\tpassed=%s\n' \
    "${key}" "${value}" "${threshold}" "${passed}"
  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

validate_floor_max_ge() {
  local profile="$1"
  local key="$2"
  local threshold="$3"
  local value
  local passed

  if ! value="$(profile_max_metric "${profile}" "${key}")"; then
    printf 'candidate_world_floor\t%s\tpassed=missing_metric\n' "${key}"
    failed=1
    return
  fi

  passed="$(passes_ge "${value}" "${threshold}")"
  printf 'candidate_world_floor\t%s\tvalue=%s\tmin_required=%s\tpassed=%s\n' \
    "${key}" "${value}" "${threshold}" "${passed}"
  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

validate_candidate_world_floor_profile() {
  local profile="$1"

  validate_floor_max_ge "${profile}" "world_floor_visible_tile_count" \
    "${world_floor_min_visible_tiles}"
  validate_floor_max_le "${profile}" "world_floor_visible_tile_count" \
    "${world_floor_max_visible_tiles}"
  validate_floor_max_le "${profile}" "world_floor_resident_tile_count" \
    "${world_floor_max_resident_tiles}"
  validate_floor_max_ge "${profile}" "world_floor_initial_spawned_tile_count" \
    "${world_floor_initial_spawned_tiles}"
  validate_floor_max_le "${profile}" "world_floor_initial_spawned_tile_count" \
    "${world_floor_initial_spawned_tiles}"
  validate_floor_max_le "${profile}" "world_floor_spawned_tiles_per_frame" \
    "${world_floor_max_spawned_tiles_per_frame}"
  validate_floor_max_le "${profile}" "world_floor_despawned_tiles_per_frame" \
    "${world_floor_max_despawned_tiles_per_frame}"
  validate_floor_max_le "${profile}" "world_floor_mesh_vertex_count" \
    "${world_floor_max_mesh_vertices}"
  validate_floor_max_le "${profile}" "world_floor_mesh_triangle_count" \
    "${world_floor_max_mesh_triangles}"
  validate_floor_max_le "${profile}" "world_floor_material_count" \
    "${world_floor_max_materials}"
  validate_floor_max_ge "${profile}" "world_floor_biome_count" \
    "${world_floor_min_biomes}"
  validate_floor_max_ge "${profile}" "world_floor_terrain_feature_count" \
    "${world_floor_min_terrain_features}"
  validate_floor_max_ge "${profile}" "world_floor_color_band_count" \
    "${world_floor_min_color_bands}"
  validate_floor_max_ge "${profile}" "world_floor_river_vertex_count" \
    "${world_floor_min_river_vertices}"
  validate_floor_max_ge "${profile}" "world_floor_relief_range_m" \
    "${world_floor_min_relief_range_m}"
}

echo "World-floor readiness gate"
echo
echo "Comparing release app-path perf summaries..."
./tools/compare_perf_summaries.sh "${baseline_perf}" "${candidate_perf}"
echo
echo "Comparing scripted freeflight profiles..."
validate_profile_script "${baseline_freeflight}" "freeflight"
validate_profile_script "${candidate_freeflight}" "freeflight"
./tools/compare_manual_play_profiles.sh "${baseline_freeflight}" "${candidate_freeflight}"
echo
echo "Comparing scripted ground-traversal profiles..."
validate_profile_script "${baseline_ground_traversal}" "ground_traversal"
validate_profile_script "${candidate_ground_traversal}" "ground_traversal"
./tools/compare_manual_play_profiles.sh "${baseline_ground_traversal}" "${candidate_ground_traversal}"
validate_ground_traversal_profile "${baseline_ground_traversal}" "baseline_ground_traversal"
validate_ground_traversal_profile "${candidate_ground_traversal}" "candidate_ground_traversal"
echo
echo "Checking candidate ground-traversal world-floor budgets..."
validate_candidate_world_floor_profile "${candidate_ground_traversal}"
if (( failed != 0 )); then
  echo "candidate world-floor profile budgets failed" >&2
  exit 1
fi
echo
echo "World-floor readiness checks passed."
