#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to compare play profiles" >&2
  exit 1
fi

if [[ "$#" -ne 2 ]]; then
  echo "Usage: $0 <baseline_profile.json> <candidate_profile.json>" >&2
  exit 2
fi

baseline_profile="$1"
candidate_profile="$2"
max_frame_time_ratio="${NAU_MANUAL_PROFILE_MAX_FRAME_TIME_REGRESSION_RATIO:-1.10}"
max_frame_time_ms="${NAU_MANUAL_PROFILE_MAX_FRAME_TIME_REGRESSION_MS:-2}"
max_p99_frame_time_ms="${NAU_MANUAL_PROFILE_MAX_P99_FRAME_TIME_REGRESSION_MS:-5}"
max_count_ratio="${NAU_MANUAL_PROFILE_MAX_COUNT_REGRESSION_RATIO:-1.10}"
max_count_abs="${NAU_MANUAL_PROFILE_MAX_COUNT_REGRESSION_ABS:-5}"
profile_max_avg_frame_time_ms="${NAU_MANUAL_PROFILE_MAX_AVG_FRAME_TIME_MS:-24}"
profile_max_p95_frame_time_ms="${NAU_MANUAL_PROFILE_MAX_P95_FRAME_TIME_MS:-45}"
profile_max_p99_frame_time_ms="${NAU_MANUAL_PROFILE_MAX_P99_FRAME_TIME_MS:-80}"
profile_max_steady_50ms_hitches="${NAU_MANUAL_PROFILE_MAX_STEADY_50MS_HITCHES:-3}"
profile_max_steady_100ms_hitches="${NAU_MANUAL_PROFILE_MAX_STEADY_100MS_HITCHES:-1}"
max_host_process_cpu_percent="${NAU_MANUAL_PROFILE_MAX_HOST_PROCESS_CPU_PERCENT:-80}"
max_host_total_cpu_percent="${NAU_MANUAL_PROFILE_MAX_HOST_TOTAL_CPU_PERCENT:-${NAU_PERF_MAX_HOST_TOTAL_CPU_PERCENT:-160}}"
allow_missing_host_snapshots="${NAU_MANUAL_PROFILE_ALLOW_MISSING_HOST_SNAPSHOTS:-0}"
require_quiet_host_after="${NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER:-1}"
require_baseline_absolute_checks="${NAU_MANUAL_PROFILE_REQUIRE_BASELINE_ABSOLUTE_CHECKS:-1}"
require_candidate_absolute_checks="${NAU_MANUAL_PROFILE_REQUIRE_CANDIDATE_ABSOLUTE_CHECKS:-1}"
default_ignore_process_pattern="${NAU_PERF_DEFAULT_IGNORE_PROCESS_PATTERN-}"
if [[ "${NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN+x}" == "x" ]]; then
  ignore_process_pattern="${NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN}"
elif [[ "${NAU_PERF_IGNORE_PROCESS_PATTERN+x}" == "x" ]]; then
  ignore_process_pattern="${NAU_PERF_IGNORE_PROCESS_PATTERN}"
else
  ignore_process_pattern="${default_ignore_process_pattern}"
fi
failed=0
required_checks=(
  play_profile_duration
  play_profile_horizontal_travel
  play_profile_steady_avg_frame_time_budget
  play_profile_steady_p95_frame_time_budget
  play_profile_steady_p99_frame_time_budget
  play_profile_steady_50ms_hitch_count
  play_profile_steady_100ms_hitch_count
)

case "${allow_missing_host_snapshots}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_ALLOW_MISSING_HOST_SNAPSHOTS must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${require_quiet_host_after}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_REQUIRE_QUIET_HOST_AFTER must be 0 or 1" >&2
    exit 2
    ;;
esac

for value_name in max_host_process_cpu_percent max_host_total_cpu_percent; do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "${value_name} must be numeric, got: ${value}" >&2
    exit 2
  fi
done

case "${require_baseline_absolute_checks}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_REQUIRE_BASELINE_ABSOLUTE_CHECKS must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${require_candidate_absolute_checks}" in
  0 | 1) ;;
  *)
    echo "NAU_MANUAL_PROFILE_REQUIRE_CANDIDATE_ABSOLUTE_CHECKS must be 0 or 1" >&2
    exit 2
    ;;
esac

for profile in "${baseline_profile}" "${candidate_profile}"; do
  if [[ ! -s "${profile}" ]]; then
    echo "missing play profile: ${profile}" >&2
    exit 1
  fi
done

metric_value() {
  local profile="$1"
  local path="$2"
  jq -er "${path}" "${profile}"
}

profile_check_passed() {
  local profile="$1"
  local check_name="$2"
  jq -e --arg check_name "${check_name}" \
    'any(.checks[]?; .name == $check_name and .passed == true)' "${profile}" >/dev/null
}

accepted_profile_kind() {
  local profile_kind="$1"
  [[ "${profile_kind}" == "manual_play_foreground" || "${profile_kind}" == "scripted_play_foreground" ]]
}

host_snapshot_max_cpu() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    BEGIN { in_top = 0; max_cpu = 0 }
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      cpu = $2 + 0
      if (cpu > max_cpu) {
        max_cpu = cpu
      }
    }
    END { printf "%.1f", max_cpu }
  ' "${snapshot}"
}

host_snapshot_total_cpu() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    BEGIN { in_top = 0; total_cpu = 0 }
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      total_cpu += $2 + 0
    }
    END { printf "%.1f", total_cpu }
  ' "${snapshot}"
}

passes_host_cpu_threshold() {
  local cpu="$1"
  awk -v cpu="${cpu}" -v threshold="${max_host_process_cpu_percent}" \
    'BEGIN { print (cpu <= threshold ? "true" : "false") }'
}

passes_host_total_cpu_threshold() {
  local cpu="$1"
  awk -v cpu="${cpu}" -v threshold="${max_host_total_cpu_percent}" \
    'BEGIN { print (cpu <= threshold ? "true" : "false") }'
}

validate_host_snapshot() {
  local label="$1"
  local snapshot="$2"
  local require_quiet="$3"
  local max_cpu
  local total_cpu
  local passed
  local total_passed

  if [[ ! -s "${snapshot}" ]]; then
    printf '%s\tpath=%s\tpassed=missing_snapshot\n' "${label}" "${snapshot}"
    if [[ "${allow_missing_host_snapshots}" != "1" ]]; then
      failed=1
    fi
    return
  fi

  max_cpu="$(host_snapshot_max_cpu "${snapshot}")"
  total_cpu="$(host_snapshot_total_cpu "${snapshot}")"
  passed="$(passes_host_cpu_threshold "${max_cpu}")"
  total_passed="$(passes_host_total_cpu_threshold "${total_cpu}")"
  printf '%s\tpath=%s\tmax_process_cpu_percent=%s\tmax_allowed=%s\tpassed=%s\ttotal_top_process_cpu_percent=%s\ttotal_allowed=%s\ttotal_passed=%s\tquiet_required=%s\n' \
    "${label}" "${snapshot}" "${max_cpu}" "${max_host_process_cpu_percent}" "${passed}" \
    "${total_cpu}" "${max_host_total_cpu_percent}" "${total_passed}" "${require_quiet}"
  if [[ -n "${ignore_process_pattern}" ]]; then
    printf '%s\tignored_process_pattern=%s\n' "${label}" "${ignore_process_pattern}"
  fi

  if [[ "${require_quiet}" == "1" && ( "${passed}" != "true" || "${total_passed}" != "true" ) ]]; then
    failed=1
  fi
}

validate_profile_host_snapshots() {
  local label="$1"
  local profile="$2"

  validate_host_snapshot "${label}_host_snapshot_before" "${profile}.host_snapshot_before.txt" "1"
  validate_host_snapshot "${label}_host_snapshot_after" "${profile}.host_snapshot_after.txt" "${require_quiet_host_after}"
}

validate_profile() {
  local label="$1"
  local profile="$2"
  local profile_kind
  local control_source
  local script
  local duration_passed
  local io_clean
  local require_absolute_checks
  local schema_version

  profile_kind="$(jq -r '.profile_kind // ""' "${profile}")"
  control_source="$(jq -r '.control_source // ""' "${profile}")"
  script="$(jq -r '.script // ""' "${profile}")"
  duration_passed="$(jq -r '.duration_secs >= 30' "${profile}")"
  io_clean="$(jq -r '.io_error == null' "${profile}")"
  require_absolute_checks="$(profile_requires_absolute_checks "${label}")"
  schema_version="$(jq -r '.schema_version // 1' "${profile}")"

  if ! [[ "${schema_version}" =~ ^[0-9]+$ ]]; then
    echo "${label} has invalid schema_version: ${schema_version}" >&2
    failed=1
    schema_version=1
  fi

  printf '%s_kind\t%s\n' "${label}" "${profile_kind}"
  printf '%s_control_source\t%s\n' "${label}" "${control_source}"
  if [[ -n "${script}" ]]; then
    printf '%s_script\t%s\n' "${label}" "${script}"
  fi
  printf '%s_duration_at_least_30s\t%s\n' "${label}" "${duration_passed}"
  printf '%s_io_clean\t%s\n' "${label}" "${io_clean}"
  printf '%s_absolute_checks_required\t%s\n' "${label}" "${require_absolute_checks}"
  printf '%s_schema_version\t%s\n' "${label}" "${schema_version}"

  if ! accepted_profile_kind "${profile_kind}"; then
    failed=1
  fi
  if [[ "${profile_kind}" == "scripted_play_foreground" && -z "${script}" ]]; then
    failed=1
  fi
  if [[ "${duration_passed}" != "true" || "${io_clean}" != "true" ]]; then
    failed=1
  fi

  for check_name in "${required_checks[@]}"; do
    if profile_check_passed "${profile}" "${check_name}"; then
      printf '%s_check\t%s\tpassed=true\trequired=%s\n' \
        "${label}" "${check_name}" "${require_absolute_checks}"
    else
      printf '%s_check\t%s\tpassed=false\trequired=%s\n' \
        "${label}" "${check_name}" "${require_absolute_checks}"
      if [[ "${require_absolute_checks}" == "1" ]]; then
        failed=1
      fi
    fi
  done

  if (( schema_version >= 2 )); then
    if profile_check_passed "${profile}" "play_profile_window_focused_ratio"; then
      printf '%s_check\tplay_profile_window_focused_ratio\tpassed=true\trequired=%s\n' \
        "${label}" "${require_absolute_checks}"
    else
      printf '%s_check\tplay_profile_window_focused_ratio\tpassed=false\trequired=%s\n' \
        "${label}" "${require_absolute_checks}"
      if [[ "${require_absolute_checks}" == "1" ]]; then
        failed=1
      fi
    fi
  else
    printf '%s_check\tplay_profile_window_focused_ratio\tpassed=not_available\trequired=false\treason=schema_v1\n' \
      "${label}"
  fi

  validate_profile_host_snapshots "${label}" "${profile}"
}

allowed_threshold() {
  local baseline_value="$1"
  local ratio="$2"
  local absolute_slack="$3"
  awk -v baseline="${baseline_value}" -v ratio="${ratio}" -v slack="${absolute_slack}" \
    'BEGIN {
      relative = baseline * ratio
      absolute = baseline + slack
      printf "%.4f", (relative > absolute ? relative : absolute)
    }'
}

passes_threshold() {
  local candidate_value="$1"
  local threshold="$2"
  awk -v candidate="${candidate_value}" -v threshold="${threshold}" \
    'BEGIN { print (candidate <= threshold ? "true" : "false") }'
}

validate_max_metric() {
  local label="$1"
  local profile="$2"
  local metric_label="$3"
  local path="$4"
  local threshold="$5"
  local required="${6:-1}"
  local value
  local passed

  value="$(metric_value "${profile}" "${path}")"
  passed="$(passes_threshold "${value}" "${threshold}")"
  printf '%s_absolute\t%s\tvalue=%s\tmax_allowed=%s\tpassed=%s\trequired=%s\n' \
    "${label}" "${metric_label}" "${value}" "${threshold}" "${passed}" "${required}"

  if [[ "${passed}" != "true" && "${required}" == "1" ]]; then
    failed=1
  fi
}

profile_requires_absolute_checks() {
  local label="$1"
  case "${label}" in
    baseline_profile) printf '%s' "${require_baseline_absolute_checks}" ;;
    candidate_profile) printf '%s' "${require_candidate_absolute_checks}" ;;
    *) printf '1' ;;
  esac
}

compare_metric() {
  local label="$1"
  local path="$2"
  local ratio="$3"
  local absolute_slack="$4"
  local baseline_value
  local candidate_value
  local threshold
  local passed

  baseline_value="$(metric_value "${baseline_profile}" "${path}")"
  candidate_value="$(metric_value "${candidate_profile}" "${path}")"
  threshold="$(allowed_threshold "${baseline_value}" "${ratio}" "${absolute_slack}")"
  passed="$(passes_threshold "${candidate_value}" "${threshold}")"

  printf '%s\tbaseline=%s\tcandidate=%s\tmax_allowed=%s\tpassed=%s\n' \
    "${label}" "${baseline_value}" "${candidate_value}" "${threshold}" "${passed}"

  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

compare_advisory_metric() {
  local label="$1"
  local path="$2"
  local ratio="$3"
  local absolute_slack="$4"
  local baseline_value
  local candidate_value
  local threshold
  local within_threshold

  baseline_value="$(metric_value "${baseline_profile}" "${path}")"
  candidate_value="$(metric_value "${candidate_profile}" "${path}")"
  threshold="$(allowed_threshold "${baseline_value}" "${ratio}" "${absolute_slack}")"
  within_threshold="$(passes_threshold "${candidate_value}" "${threshold}")"

  printf '%s_advisory\tbaseline=%s\tcandidate=%s\tmax_allowed=%s\twithin_threshold=%s\tgating=false\n' \
    "${label}" "${baseline_value}" "${candidate_value}" "${threshold}" "${within_threshold}"
}

baseline_passed="$(jq -r '.passed == true' "${baseline_profile}")"
candidate_passed="$(jq -r '.passed == true' "${candidate_profile}")"
baseline_kind="$(jq -r '.profile_kind // ""' "${baseline_profile}")"
candidate_kind="$(jq -r '.profile_kind // ""' "${candidate_profile}")"
baseline_script="$(jq -r '.script // ""' "${baseline_profile}")"
candidate_script="$(jq -r '.script // ""' "${candidate_profile}")"

printf 'baseline_profile\t%s\tpassed=%s\n' "${baseline_profile}" "${baseline_passed}"
printf 'candidate_profile\t%s\tpassed=%s\n' "${candidate_profile}" "${candidate_passed}"
validate_profile "baseline_profile" "${baseline_profile}"
validate_profile "candidate_profile" "${candidate_profile}"

if [[ "${baseline_kind}" != "${candidate_kind}" ]]; then
  printf 'profile_kind_match\tbaseline=%s\tcandidate=%s\tpassed=false\n' \
    "${baseline_kind}" "${candidate_kind}"
  failed=1
else
  printf 'profile_kind_match\tbaseline=%s\tcandidate=%s\tpassed=true\n' \
    "${baseline_kind}" "${candidate_kind}"
fi

if [[ "${baseline_kind}" == "scripted_play_foreground" || "${candidate_kind}" == "scripted_play_foreground" ]]; then
  if [[ -n "${baseline_script}" && "${baseline_script}" == "${candidate_script}" ]]; then
    printf 'profile_script_match\tbaseline=%s\tcandidate=%s\tpassed=true\n' \
      "${baseline_script}" "${candidate_script}"
  else
    printf 'profile_script_match\tbaseline=%s\tcandidate=%s\tpassed=false\n' \
      "${baseline_script}" "${candidate_script}"
    failed=1
  fi
fi

validate_max_metric "baseline_profile" "${baseline_profile}" "steady_avg_frame_time_ms" \
  ".steady_frame_time.avg_ms" "${profile_max_avg_frame_time_ms}" \
  "${require_baseline_absolute_checks}"
validate_max_metric "baseline_profile" "${baseline_profile}" "steady_p95_frame_time_ms" \
  ".steady_frame_time.p95_ms" "${profile_max_p95_frame_time_ms}" \
  "${require_baseline_absolute_checks}"
validate_max_metric "baseline_profile" "${baseline_profile}" "steady_p99_frame_time_ms" \
  ".steady_frame_time.p99_ms" "${profile_max_p99_frame_time_ms}" \
  "${require_baseline_absolute_checks}"
validate_max_metric "baseline_profile" "${baseline_profile}" "steady_50ms_hitch_count" \
  ".steady_frame_time.frames_over_50ms" "${profile_max_steady_50ms_hitches}" \
  "${require_baseline_absolute_checks}"
validate_max_metric "baseline_profile" "${baseline_profile}" "steady_100ms_hitch_count" \
  ".steady_frame_time.frames_over_100ms" "${profile_max_steady_100ms_hitches}" \
  "${require_baseline_absolute_checks}"
validate_max_metric "candidate_profile" "${candidate_profile}" "steady_avg_frame_time_ms" \
  ".steady_frame_time.avg_ms" "${profile_max_avg_frame_time_ms}" \
  "${require_candidate_absolute_checks}"
validate_max_metric "candidate_profile" "${candidate_profile}" "steady_p95_frame_time_ms" \
  ".steady_frame_time.p95_ms" "${profile_max_p95_frame_time_ms}" \
  "${require_candidate_absolute_checks}"
validate_max_metric "candidate_profile" "${candidate_profile}" "steady_p99_frame_time_ms" \
  ".steady_frame_time.p99_ms" "${profile_max_p99_frame_time_ms}" \
  "${require_candidate_absolute_checks}"
validate_max_metric "candidate_profile" "${candidate_profile}" "steady_50ms_hitch_count" \
  ".steady_frame_time.frames_over_50ms" "${profile_max_steady_50ms_hitches}" \
  "${require_candidate_absolute_checks}"
validate_max_metric "candidate_profile" "${candidate_profile}" "steady_100ms_hitch_count" \
  ".steady_frame_time.frames_over_100ms" "${profile_max_steady_100ms_hitches}" \
  "${require_candidate_absolute_checks}"

if [[ "${baseline_passed}" != "true" && "${require_baseline_absolute_checks}" == "1" ]]; then
  failed=1
fi
if [[ "${candidate_passed}" != "true" && "${require_candidate_absolute_checks}" == "1" ]]; then
  failed=1
fi

compare_metric "steady_avg_frame_time_ms" ".steady_frame_time.avg_ms" \
  "${max_frame_time_ratio}" "${max_frame_time_ms}"
compare_metric "steady_p95_frame_time_ms" ".steady_frame_time.p95_ms" \
  "${max_frame_time_ratio}" "${max_frame_time_ms}"
compare_metric "steady_p99_frame_time_ms" ".steady_frame_time.p99_ms" \
  "${max_frame_time_ratio}" "${max_p99_frame_time_ms}"
compare_advisory_metric "steady_max_frame_time_ms" ".steady_frame_time.max_ms" \
  "${max_frame_time_ratio}" "${max_frame_time_ms}"
compare_advisory_metric "steady_frames_over_33_34ms" ".steady_frame_time.frames_over_33_34ms" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "steady_frames_over_50ms" ".steady_frame_time.frames_over_50ms" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "steady_frames_over_100ms" ".steady_frame_time.frames_over_100ms" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_entity_count" ".max.entity_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_active_chunk_count" ".max.active_chunk_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_active_island_count" ".max.active_island_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_visible_island_terrain_count" ".max.visible_island_terrain_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_visible_island_detail_count" ".max.visible_island_detail_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_advisory_metric "max_visible_island_impostor_count" ".max.visible_island_impostor_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_stream_visibility_changes_per_frame" ".max.stream_visibility_changes_per_frame" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_stream_spawned_visuals_per_frame" ".max.stream_spawned_visuals_per_frame" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_stream_despawned_visuals_per_frame" ".max.stream_despawned_visuals_per_frame" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_mesh_count" ".max.mesh_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_loaded_mesh_triangles" ".max.loaded_mesh_triangles" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_material_count" ".max.material_count" \
  "${max_count_ratio}" "${max_count_abs}"
compare_metric "max_resident_island_visual_count" ".max.resident_island_visual_count" \
  "${max_count_ratio}" "${max_count_abs}"

if (( failed != 0 )); then
  echo "play profile comparison failed" >&2
  exit 1
fi
