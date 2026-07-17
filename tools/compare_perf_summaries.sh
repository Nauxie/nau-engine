#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to compare perf summaries" >&2
  exit 1
fi

if [[ "$#" -ne 2 ]]; then
  echo "Usage: $0 <baseline_perf_summary.json> <candidate_perf_summary.json>" >&2
  exit 2
fi

baseline_summary="$1"
candidate_summary="$2"
max_frame_time_ratio="${NAU_PERF_SUMMARY_MAX_FRAME_TIME_REGRESSION_RATIO:-1.10}"
camera_mouse_max_avg_frame_time_ratio="${NAU_PERF_SUMMARY_CAMERA_MOUSE_MAX_AVG_FRAME_TIME_REGRESSION_RATIO:-${max_frame_time_ratio}}"
max_frame_time_ms="${NAU_PERF_SUMMARY_MAX_FRAME_TIME_REGRESSION_MS:-2}"
max_p95_frame_time_ms="${NAU_PERF_SUMMARY_MAX_P95_FRAME_TIME_REGRESSION_MS:-4}"
max_p99_frame_time_ms="${NAU_PERF_SUMMARY_MAX_P99_FRAME_TIME_REGRESSION_MS:-3}"
max_count_ratio="${NAU_PERF_SUMMARY_MAX_COUNT_REGRESSION_RATIO:-1.10}"
max_count_abs="${NAU_PERF_SUMMARY_MAX_COUNT_REGRESSION_ABS:-5}"
camera_mouse_max_hitch_count_ratio="${NAU_PERF_SUMMARY_CAMERA_MOUSE_MAX_HITCH_COUNT_REGRESSION_RATIO:-${max_count_ratio}}"
summary_max_avg_frame_time_ms="${NAU_PERF_MAX_AVG_FRAME_TIME_MS:-24}"
summary_max_p95_frame_time_ms="${NAU_PERF_MAX_P95_FRAME_TIME_MS:-45}"
summary_max_p99_frame_time_ms="${NAU_PERF_MAX_P99_FRAME_TIME_MS:-80}"
max_host_process_cpu_percent="${NAU_PERF_MAX_HOST_PROCESS_CPU_PERCENT:-80}"
max_host_total_cpu_percent="${NAU_PERF_MAX_HOST_TOTAL_CPU_PERCENT:-160}"
allow_missing_host_snapshots="${NAU_PERF_ALLOW_MISSING_HOST_SNAPSHOTS:-0}"
require_quiet_host_after="${NAU_PERF_REQUIRE_QUIET_HOST_AFTER:-1}"
default_ignore_process_pattern="${NAU_PERF_DEFAULT_IGNORE_PROCESS_PATTERN-}"
ignore_process_pattern="${NAU_PERF_IGNORE_PROCESS_PATTERN-${default_ignore_process_pattern}}"
failed=0
required_scenarios=(
  baseline_route
  long_glide_visibility
)

case "${allow_missing_host_snapshots}" in
  0 | 1) ;;
  *)
    echo "NAU_PERF_ALLOW_MISSING_HOST_SNAPSHOTS must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${require_quiet_host_after}" in
  0 | 1) ;;
  *)
    echo "NAU_PERF_REQUIRE_QUIET_HOST_AFTER must be 0 or 1" >&2
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

for summary in "${baseline_summary}" "${candidate_summary}"; do
  if [[ ! -s "${summary}" ]]; then
    echo "missing perf summary: ${summary}" >&2
    exit 1
  fi
done

scenario_metric() {
  local summary="$1"
  local scenario="$2"
  local key="$3"
  jq -er --arg scenario "${scenario}" --arg key "${key}" \
    '.scenarios[] | select(.scenario == $scenario) | .[$key]' "${summary}"
}

optional_scenario_metric() {
  local summary="$1"
  local scenario="$2"
  local key="$3"
  jq -r --arg scenario "${scenario}" --arg key "${key}" \
    '.scenarios[] | select(.scenario == $scenario) | .[$key] // null' "${summary}"
}

scenario_exists() {
  local summary="$1"
  local scenario="$2"
  jq -e --arg scenario "${scenario}" \
    'any(.scenarios[]?; .scenario == $scenario)' "${summary}" >/dev/null
}

summary_flag() {
  local summary="$1"
  local key="$2"
  jq -r --arg key "${key}" \
    'if has($key) and (.[$key] | type == "boolean") then (.[$key] | tostring) else "missing" end' \
    "${summary}"
}

summary_string() {
  local summary="$1"
  local key="$2"
  jq -r --arg key "${key}" \
    'if has($key) and (.[$key] | type == "string") then .[$key] else "missing" end' \
    "${summary}"
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
  local summary="$2"
  local key="$3"
  local require_quiet="$4"
  local snapshot
  local max_cpu
  local total_cpu
  local passed
  local total_passed

  snapshot="$(summary_string "${summary}" "${key}")"
  if [[ "${snapshot}" == "missing" || ! -s "${snapshot}" ]]; then
    printf '%s\t%s\tpath=%s\tpassed=missing_snapshot\n' \
      "${label}" "${key}" "${snapshot}"
    if [[ "${allow_missing_host_snapshots}" != "1" ]]; then
      failed=1
    fi
    return
  fi

  max_cpu="$(host_snapshot_max_cpu "${snapshot}")"
  total_cpu="$(host_snapshot_total_cpu "${snapshot}")"
  passed="$(passes_host_cpu_threshold "${max_cpu}")"
  total_passed="$(passes_host_total_cpu_threshold "${total_cpu}")"
  printf '%s\t%s\tpath=%s\tmax_process_cpu_percent=%s\tmax_allowed=%s\tpassed=%s\ttotal_top_process_cpu_percent=%s\ttotal_allowed=%s\ttotal_passed=%s\tquiet_required=%s\n' \
    "${label}" "${key}" "${snapshot}" "${max_cpu}" "${max_host_process_cpu_percent}" "${passed}" \
    "${total_cpu}" "${max_host_total_cpu_percent}" "${total_passed}" "${require_quiet}"
  if [[ -n "${ignore_process_pattern}" ]]; then
    printf '%s\t%s\tignored_process_pattern=%s\n' \
      "${label}" "${key}" "${ignore_process_pattern}"
  fi

  if [[ "${require_quiet}" == "1" && ( "${passed}" != "true" || "${total_passed}" != "true" ) ]]; then
    failed=1
  fi
}

validate_host_snapshots() {
  local label="$1"
  local summary="$2"

  validate_host_snapshot "${label}" "${summary}" "host_snapshot_before" "1"
  validate_host_snapshot "${label}" "${summary}" "host_snapshot_after" "${require_quiet_host_after}"
}

validate_summary() {
  local label="$1"
  local summary="$2"
  local mode
  local has_scenarios

  mode="$(jq -r '.mode // ""' "${summary}")"
  has_scenarios="$(jq -r '(.scenarios | type == "array") and (.scenarios | length > 0)' "${summary}")"
  printf '%s_mode\t%s\n' "${label}" "${mode}"
  printf '%s_has_scenarios\t%s\n' "${label}" "${has_scenarios}"

  if [[ "${mode}" != "release" || "${has_scenarios}" != "true" ]]; then
    failed=1
  fi

  for scenario in "${required_scenarios[@]}"; do
    if scenario_exists "${summary}" "${scenario}"; then
      printf '%s_required_scenario\t%s\tpresent=true\n' "${label}" "${scenario}"
    else
      printf '%s_required_scenario\t%s\tpresent=false\n' "${label}" "${scenario}"
      failed=1
    fi
  done
}

validate_capture_modes_match() {
  local baseline_visible_window
  local candidate_visible_window
  local baseline_capture_screenshot
  local candidate_capture_screenshot

  baseline_visible_window="$(summary_flag "${baseline_summary}" "visible_window")"
  candidate_visible_window="$(summary_flag "${candidate_summary}" "visible_window")"
  baseline_capture_screenshot="$(summary_flag "${baseline_summary}" "capture_screenshot")"
  candidate_capture_screenshot="$(summary_flag "${candidate_summary}" "capture_screenshot")"

  printf 'capture_mode\tbaseline_visible_window=%s\tcandidate_visible_window=%s\tbaseline_capture_screenshot=%s\tcandidate_capture_screenshot=%s\n' \
    "${baseline_visible_window}" "${candidate_visible_window}" \
    "${baseline_capture_screenshot}" "${candidate_capture_screenshot}"

  if [[ "${baseline_visible_window}" == "missing" || "${candidate_visible_window}" == "missing" ]]; then
    failed=1
  fi
  if [[ "${baseline_capture_screenshot}" == "missing" || "${candidate_capture_screenshot}" == "missing" ]]; then
    failed=1
  fi
  if [[ "${baseline_visible_window}" != "${candidate_visible_window}" ]]; then
    failed=1
  fi
  if [[ "${baseline_capture_screenshot}" != "${candidate_capture_screenshot}" ]]; then
    failed=1
  fi
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

validate_absolute_metric() {
  local label="$1"
  local summary="$2"
  local scenario="$3"
  local key="$4"
  local threshold="$5"
  local value
  local passed

  value="$(scenario_metric "${summary}" "${scenario}" "${key}")"
  passed="$(passes_threshold "${value}" "${threshold}")"
  printf '%s\t%s\tabsolute_%s\tvalue=%s\tmax_allowed=%s\tpassed=%s\n' \
    "${label}" "${scenario}" "${key}" "${value}" "${threshold}" "${passed}"

  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

compare_metric() {
  local scenario="$1"
  local key="$2"
  local ratio="$3"
  local absolute_slack="$4"
  local baseline_value
  local candidate_value
  local threshold
  local passed

  baseline_value="$(optional_scenario_metric "${baseline_summary}" "${scenario}" "${key}")"
  candidate_value="$(optional_scenario_metric "${candidate_summary}" "${scenario}" "${key}")"
  if [[ "${baseline_value}" == "null" || "${candidate_value}" == "null" ]]; then
    printf '%s\t%s\tbaseline=%s\tcandidate=%s\tpassed=missing_required_metric\n' \
      "${scenario}" "${key}" "${baseline_value}" "${candidate_value}"
    failed=1
    return
  fi

  threshold="$(allowed_threshold "${baseline_value}" "${ratio}" "${absolute_slack}")"
  passed="$(passes_threshold "${candidate_value}" "${threshold}")"

  printf '%s\t%s\tbaseline=%s\tcandidate=%s\tmax_allowed=%s\tpassed=%s\n' \
    "${scenario}" "${key}" "${baseline_value}" "${candidate_value}" "${threshold}" "${passed}"

  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

compare_advisory_metric() {
  local scenario="$1"
  local key="$2"
  local ratio="$3"
  local absolute_slack="$4"
  local baseline_value
  local candidate_value
  local threshold
  local within_threshold

  baseline_value="$(optional_scenario_metric "${baseline_summary}" "${scenario}" "${key}")"
  candidate_value="$(optional_scenario_metric "${candidate_summary}" "${scenario}" "${key}")"
  if [[ "${baseline_value}" == "null" || "${candidate_value}" == "null" ]]; then
    printf '%s\t%s_advisory\tbaseline=%s\tcandidate=%s\tgating=false\tstatus=missing_metric\n' \
      "${scenario}" "${key}" "${baseline_value}" "${candidate_value}"
    return
  fi

  threshold="$(allowed_threshold "${baseline_value}" "${ratio}" "${absolute_slack}")"
  within_threshold="$(passes_threshold "${candidate_value}" "${threshold}")"

  printf '%s\t%s_advisory\tbaseline=%s\tcandidate=%s\tmax_allowed=%s\twithin_threshold=%s\tgating=false\n' \
    "${scenario}" "${key}" "${baseline_value}" "${candidate_value}" "${threshold}" "${within_threshold}"
}

compare_optional_count_metric() {
  local scenario="$1"
  local key="$2"
  local ratio="$3"
  local absolute_slack="$4"
  local baseline_value
  local candidate_value
  local threshold
  local passed

  baseline_value="$(optional_scenario_metric "${baseline_summary}" "${scenario}" "${key}")"
  candidate_value="$(optional_scenario_metric "${candidate_summary}" "${scenario}" "${key}")"
  if [[ "${baseline_value}" == "null" || "${candidate_value}" == "null" ]]; then
    printf '%s\t%s\tbaseline=%s\tcandidate=%s\tpassed=skipped_missing_metric\n' \
      "${scenario}" "${key}" "${baseline_value}" "${candidate_value}"
    return
  fi

  threshold="$(allowed_threshold "${baseline_value}" "${ratio}" "${absolute_slack}")"
  passed="$(passes_threshold "${candidate_value}" "${threshold}")"

  printf '%s\t%s\tbaseline=%s\tcandidate=%s\tmax_allowed=%s\tpassed=%s\n' \
    "${scenario}" "${key}" "${baseline_value}" "${candidate_value}" "${threshold}" "${passed}"

  if [[ "${passed}" != "true" ]]; then
    failed=1
  fi
}

validate_summary "baseline_summary" "${baseline_summary}"
validate_summary "candidate_summary" "${candidate_summary}"
validate_capture_modes_match
validate_host_snapshots "baseline_summary" "${baseline_summary}"
validate_host_snapshots "candidate_summary" "${candidate_summary}"

while IFS= read -r scenario; do
  if ! scenario_exists "${baseline_summary}" "${scenario}"; then
    echo "candidate scenario missing from baseline summary: ${scenario}" >&2
    failed=1
    continue
  fi

  baseline_passed="$(jq -r --arg scenario "${scenario}" \
    '.scenarios[] | select(.scenario == $scenario) | .passed == true' "${baseline_summary}")"
  candidate_passed="$(jq -r --arg scenario "${scenario}" \
    '.scenarios[] | select(.scenario == $scenario) | .passed == true' "${candidate_summary}")"
  baseline_eval_status="$(optional_scenario_metric "${baseline_summary}" "${scenario}" "eval_status")"
  candidate_eval_status="$(optional_scenario_metric "${candidate_summary}" "${scenario}" "eval_status")"
  printf '%s\tbaseline_passed=%s\tcandidate_passed=%s\n' \
    "${scenario}" "${baseline_passed}" "${candidate_passed}"
  printf '%s\teval_status\tbaseline=%s\tcandidate=%s\n' \
    "${scenario}" "${baseline_eval_status}" "${candidate_eval_status}"

  if [[ "${baseline_passed}" != "true" || "${candidate_passed}" != "true" ]]; then
    failed=1
  fi
  if [[ "${baseline_eval_status}" != "null" && "${baseline_eval_status}" != "0" ]]; then
    failed=1
  fi
  if [[ "${candidate_eval_status}" != "null" && "${candidate_eval_status}" != "0" ]]; then
    failed=1
  fi

  validate_absolute_metric "baseline_summary" "${baseline_summary}" "${scenario}" \
    "avg_frame_time_ms" "${summary_max_avg_frame_time_ms}"
  validate_absolute_metric "baseline_summary" "${baseline_summary}" "${scenario}" \
    "p95_frame_time_ms" "${summary_max_p95_frame_time_ms}"
  validate_absolute_metric "baseline_summary" "${baseline_summary}" "${scenario}" \
    "p99_frame_time_ms" "${summary_max_p99_frame_time_ms}"
  validate_absolute_metric "candidate_summary" "${candidate_summary}" "${scenario}" \
    "avg_frame_time_ms" "${summary_max_avg_frame_time_ms}"
  validate_absolute_metric "candidate_summary" "${candidate_summary}" "${scenario}" \
    "p95_frame_time_ms" "${summary_max_p95_frame_time_ms}"
  validate_absolute_metric "candidate_summary" "${candidate_summary}" "${scenario}" \
    "p99_frame_time_ms" "${summary_max_p99_frame_time_ms}"

  scenario_avg_frame_time_ratio="${max_frame_time_ratio}"
  if [[ "${scenario}" == "camera_mouse_control" ]]; then
    scenario_avg_frame_time_ratio="${camera_mouse_max_avg_frame_time_ratio}"
  fi
  scenario_hitch_count_ratio="${max_count_ratio}"
  if [[ "${scenario}" == "camera_mouse_control" ]]; then
    scenario_hitch_count_ratio="${camera_mouse_max_hitch_count_ratio}"
  fi
  compare_metric "${scenario}" "avg_frame_time_ms" \
    "${scenario_avg_frame_time_ratio}" "${max_frame_time_ms}"
  compare_metric "${scenario}" "p95_frame_time_ms" \
    "${max_frame_time_ratio}" "${max_p95_frame_time_ms}"
  # Live-window app eval p99 is useful signal, but too sensitive to a few
  # scheduler/windowing frames for a hard relative gate. Tail regressions are
  # still gated by absolute p99 plus explicit >33/>50/>100 ms hitch buckets.
  compare_advisory_metric "${scenario}" "p99_frame_time_ms" \
    "${max_frame_time_ratio}" "${max_p99_frame_time_ms}"
  compare_advisory_metric "${scenario}" "max_frame_time_ms" \
    "${max_frame_time_ratio}" "${max_frame_time_ms}"
  compare_optional_count_metric "${scenario}" "runtime_frames_over_33_34ms" \
    "${scenario_hitch_count_ratio}" "${max_count_abs}"
  compare_optional_count_metric "${scenario}" "runtime_frames_over_50ms" \
    "${scenario_hitch_count_ratio}" "${max_count_abs}"
  compare_optional_count_metric "${scenario}" "runtime_frames_over_100ms" \
    "${scenario_hitch_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_entity_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_mesh_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_material_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_loaded_mesh_vertices" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_loaded_mesh_triangles" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_visible_island_terrain_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_visible_island_detail_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_resident_island_visual_count" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_stream_spawned_visuals_per_frame" \
    "${max_count_ratio}" "${max_count_abs}"
  compare_metric "${scenario}" "max_stream_despawned_visuals_per_frame" \
    "${max_count_ratio}" "${max_count_abs}"
done < <(jq -r '.scenarios[]?.scenario' "${candidate_summary}")

if (( failed != 0 )); then
  echo "perf summary comparison failed" >&2
  exit 1
fi
