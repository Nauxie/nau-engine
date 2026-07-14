#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to aggregate perf baseline summaries" >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${script_dir}/source_identity.sh"

if [[ "$#" -gt 0 ]]; then
  scenarios=("$@")
else
  scenarios=(
    baseline_route
    long_glide_visibility
    air_control_response
    pose_state_coverage
  )
fi

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
generated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
output_root="${NAU_PERF_OUTPUT_DIR:-target/eval/perf_baseline/${timestamp}}"
status_tsv="${output_root}/eval_status.tsv"
summary_json="${output_root}/perf_summary.json"
summary_tsv="${output_root}/perf_summary.tsv"
interpretation_md="${output_root}/INTERPRETATION.md"
host_snapshot_before="${output_root}/host_snapshot_before.txt"
host_snapshot_after="${output_root}/host_snapshot_after.txt"
repo_commit="$(nau_source_commit)"
source_state="$(nau_source_state)"
source_fingerprint="$(nau_source_fingerprint)"
failed=0
max_avg_frame_time_ms="${NAU_PERF_MAX_AVG_FRAME_TIME_MS:-24}"
max_p95_frame_time_ms="${NAU_PERF_MAX_P95_FRAME_TIME_MS:-45}"
max_p99_frame_time_ms="${NAU_PERF_MAX_P99_FRAME_TIME_MS:-80}"
max_host_process_cpu_percent="${NAU_PERF_MAX_HOST_PROCESS_CPU_PERCENT:-80}"
max_host_total_cpu_percent="${NAU_PERF_MAX_HOST_TOTAL_CPU_PERCENT:-160}"
allow_busy_host="${NAU_PERF_ALLOW_BUSY_HOST:-0}"
require_quiet_host_after="${NAU_PERF_REQUIRE_QUIET_HOST_AFTER:-1}"
build_first="${NAU_PERF_BUILD_FIRST:-1}"
host_wait_secs="${NAU_PERF_HOST_WAIT_SECS:-0}"
host_wait_interval_secs="${NAU_PERF_HOST_WAIT_POLL_SECS:-5}"
visible_window="${NAU_PERF_VISIBLE_WINDOW:-1}"
capture_screenshot="${NAU_PERF_CAPTURE_SCREENSHOT:-0}"
default_ignore_process_pattern="${NAU_PERF_DEFAULT_IGNORE_PROCESS_PATTERN-}"
ignore_process_pattern="${NAU_PERF_IGNORE_PROCESS_PATTERN-${default_ignore_process_pattern}}"

case "${allow_busy_host}" in
  0 | 1) ;;
  *)
    echo "NAU_PERF_ALLOW_BUSY_HOST must be 0 or 1" >&2
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

case "${build_first}" in
  0 | 1) ;;
  *)
    echo "NAU_PERF_BUILD_FIRST must be 0 or 1" >&2
    exit 2
    ;;
esac

for value_name in host_wait_secs host_wait_interval_secs; do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    echo "${value_name} must be a non-negative integer, got: ${value}" >&2
    exit 2
  fi
done

for value_name in max_host_process_cpu_percent max_host_total_cpu_percent; do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "${value_name} must be numeric, got: ${value}" >&2
    exit 2
  fi
done

if [[ "${visible_window}" != "0" && "${visible_window}" != "1" ]]; then
  echo "NAU_PERF_VISIBLE_WINDOW must be 0 or 1" >&2
  exit 2
fi
if [[ "${capture_screenshot}" != "0" && "${capture_screenshot}" != "1" ]]; then
  echo "NAU_PERF_CAPTURE_SCREENSHOT must be 0 or 1" >&2
  exit 2
fi

eval_command="cargo run --release --bin nau-engine -- --eval <scenario> --eval-output <dir>"
if [[ "${capture_screenshot}" == "0" ]]; then
  eval_command="${eval_command} --eval-no-screenshot"
fi
if [[ "${visible_window}" == "1" ]]; then
  eval_command="${eval_command} --eval-visible-window"
fi

write_host_snapshot() {
  local output_path="$1"
  {
    printf 'generated_at\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    if [[ -n "${ignore_process_pattern}" ]]; then
      printf 'ignored_process_pattern\t%s\n' "${ignore_process_pattern}"
    fi
    printf 'thermal_status\n'
    pmset -g therm 2>/dev/null || true
    printf '\ntop_cpu_processes\n'
    ps -Ao pid,pcpu,pmem,comm | sort -k2 -nr | sed -n '1,20p'
  } > "${output_path}"
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

host_snapshot_top_process() {
  local snapshot="$1"
  awk -v ignore="${ignore_process_pattern}" '
    /^top_cpu_processes$/ { in_top = 1; next }
    in_top && $1 ~ /^[0-9]+$/ {
      if (ignore != "" && $0 ~ ignore) {
        next
      }
      print $1 " " $2 "% " $4
      exit
    }
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

host_is_quiet() {
  local snapshot="$1"
  local max_cpu
  local total_cpu
  max_cpu="$(host_snapshot_max_cpu "${snapshot}")"
  total_cpu="$(host_snapshot_total_cpu "${snapshot}")"
  awk \
    -v max_cpu="${max_cpu}" \
    -v max_threshold="${max_host_process_cpu_percent}" \
    -v total_cpu="${total_cpu}" \
    -v total_threshold="${max_host_total_cpu_percent}" \
    'BEGIN { print (max_cpu <= max_threshold && total_cpu <= total_threshold ? "true" : "false") }'
}

capture_quiet_host_snapshot() {
  local snapshot="$1"
  local phase="${2:-before release evals}"
  local waited_secs=0

  while true; do
    write_host_snapshot "${snapshot}"
    if [[ "$(host_is_quiet "${snapshot}")" == "true" ]]; then
      return 0
    fi

    if [[ "${host_wait_secs}" == "0" || "${waited_secs}" -ge "${host_wait_secs}" ]]; then
      return 1
    fi

    echo "host is busy; waiting ${host_wait_interval_secs}s ${phase}..." >&2
    echo "max process CPU: $(host_snapshot_max_cpu "${snapshot}")% (limit ${max_host_process_cpu_percent}%)" >&2
    echo "total top-process CPU: $(host_snapshot_total_cpu "${snapshot}")% (limit ${max_host_total_cpu_percent}%)" >&2
    echo "top process: $(host_snapshot_top_process "${snapshot}")" >&2
    sleep "${host_wait_interval_secs}"
    waited_secs=$((waited_secs + host_wait_interval_secs))
  done
}

if [[ "${build_first}" == "1" ]]; then
  echo "Prebuilding release eval binary before host snapshot..."
  cargo build --release --bin nau-engine
fi

mkdir -p "${output_root}"
printf 'Require quiet host after: %s\n' "${require_quiet_host_after}"
if [[ "${allow_busy_host}" == "1" ]]; then
  write_host_snapshot "${host_snapshot_before}"
elif ! capture_quiet_host_snapshot "${host_snapshot_before}" "before release evals"; then
  echo "host is too busy for a gating perf baseline; refusing to run release evals" >&2
  echo "max process CPU: $(host_snapshot_max_cpu "${host_snapshot_before}")% (limit ${max_host_process_cpu_percent}%)" >&2
  echo "total top-process CPU: $(host_snapshot_total_cpu "${host_snapshot_before}")% (limit ${max_host_total_cpu_percent}%)" >&2
  echo "top process: $(host_snapshot_top_process "${host_snapshot_before}")" >&2
  if [[ -n "${ignore_process_pattern}" ]]; then
    echo "ignored process pattern: ${ignore_process_pattern}" >&2
  fi
  echo "snapshot: ${host_snapshot_before}" >&2
  echo "Set NAU_PERF_ALLOW_BUSY_HOST=1 only for non-gating investigation." >&2
  exit 1
fi
printf 'scenario\teval_status\tsummary\n' > "${status_tsv}"

for scenario in "${scenarios[@]}"; do
  scenario_out="${output_root}/${scenario}"
  eval_args=(
    --eval "${scenario}"
    --eval-output "${scenario_out}"
  )
  if [[ "${capture_screenshot}" == "0" ]]; then
    eval_args+=(--eval-no-screenshot)
  fi
  if [[ "${visible_window}" == "1" ]]; then
    eval_args+=(--eval-visible-window)
  fi

  set +e
  cargo run --release --bin nau-engine -- "${eval_args[@]}"
  eval_status=$?
  set -e

  summary="${scenario_out}/summary.json"
  printf '%s\t%s\t%s\n' "${scenario}" "${eval_status}" "${summary}" >> "${status_tsv}"

  if [[ ! -s "${summary}" ]]; then
    echo "missing eval summary: ${summary}" >&2
    failed=1
    continue
  fi

  if (( eval_status != 0 )) || ! jq -e '.passed == true' "${summary}" >/dev/null; then
    failed=1
  fi
done

summary_files=()
for scenario in "${scenarios[@]}"; do
  summary="${output_root}/${scenario}/summary.json"
  if [[ -s "${summary}" ]]; then
    summary_files+=("${summary}")
  fi
done

if [[ "${#summary_files[@]}" -eq 0 ]]; then
  echo "no eval summaries were written under ${output_root}" >&2
  exit 1
fi

jq -s \
  --arg generated_at "${generated_at}" \
  --arg repo_commit "${repo_commit}" \
  --arg source_state "${source_state}" \
  --arg source_fingerprint "${source_fingerprint}" \
  --arg output_root "${output_root}" \
  --arg eval_command "${eval_command}" \
  --arg host_snapshot_before "${host_snapshot_before}" \
  --arg host_snapshot_after "${host_snapshot_after}" \
  --rawfile status_tsv "${status_tsv}" \
  --argjson visible_window "${visible_window}" \
  --argjson capture_screenshot "${capture_screenshot}" \
  --argjson max_avg_frame_time_ms "${max_avg_frame_time_ms}" \
  --argjson max_p95_frame_time_ms "${max_p95_frame_time_ms}" \
  --argjson max_p99_frame_time_ms "${max_p99_frame_time_ms}" \
  '($status_tsv
    | split("\n")
    | .[1:]
    | map(select(length > 0) | split("\t") | {key: .[0], value: (.[1] | tonumber)})
    | from_entries
  ) as $eval_status_by_scenario
  | {
    generated_at: $generated_at,
    repo_commit: $repo_commit,
    source_state: $source_state,
    source_fingerprint: $source_fingerprint,
    mode: "release",
    output_root: $output_root,
    eval_command: $eval_command,
    host_snapshot_before: $host_snapshot_before,
    host_snapshot_after: $host_snapshot_after,
    visible_window: ($visible_window == 1 or $capture_screenshot == 1),
    capture_screenshot: ($capture_screenshot == 1),
    scenarios: [
      .[]
      | . as $summary
      | ([
          {
            name: "avg_frame_time_budget",
            passed: (.metrics.avg_frame_time_ms <= $max_avg_frame_time_ms),
            value: .metrics.avg_frame_time_ms,
            comparator: "<=",
            threshold: $max_avg_frame_time_ms,
            unit: "ms"
          },
          {
            name: "p95_frame_time_budget",
            passed: (.metrics.p95_frame_time_ms <= $max_p95_frame_time_ms),
            value: .metrics.p95_frame_time_ms,
            comparator: "<=",
            threshold: $max_p95_frame_time_ms,
            unit: "ms"
          },
          {
            name: "p99_frame_time_budget",
            passed: (.metrics.p99_frame_time_ms <= $max_p99_frame_time_ms),
            value: .metrics.p99_frame_time_ms,
            comparator: "<=",
            threshold: $max_p99_frame_time_ms,
            unit: "ms"
          }
        ]) as $perf_budget_checks
      | ($eval_status_by_scenario[$summary.scenario] // null) as $eval_status
      | {
        scenario,
        eval_status: $eval_status,
        passed: (.passed and all($perf_budget_checks[]; .passed) and ($eval_status == 0)),
        frame_count,
        duration_secs,
        avg_frame_time_ms: .metrics.avg_frame_time_ms,
        p95_frame_time_ms: .metrics.p95_frame_time_ms,
        p99_frame_time_ms: .metrics.p99_frame_time_ms,
        max_frame_time_ms: .metrics.max_frame_time_ms,
        frame_time_count_metrics_available: (.metrics | has("runtime_frames_over_33_34ms")),
        frames_over_16_67ms: (.metrics.frames_over_16_67ms // null),
        frames_over_33_34ms: (.metrics.frames_over_33_34ms // null),
        frames_over_50ms: (.metrics.frames_over_50ms // null),
        frames_over_100ms: (.metrics.frames_over_100ms // null),
        runtime_frames_over_16_67ms: (.metrics.runtime_frames_over_16_67ms // null),
        runtime_frames_over_33_34ms: (.metrics.runtime_frames_over_33_34ms // null),
        runtime_frames_over_50ms: (.metrics.runtime_frames_over_50ms // null),
        runtime_frames_over_100ms: (.metrics.runtime_frames_over_100ms // null),
        max_entity_count: .metrics.max_entity_count,
        max_visible_island_terrain_count: .metrics.max_visible_island_terrain_count,
        max_visible_island_detail_count: .metrics.max_visible_island_detail_count,
        max_resident_island_visual_count: .metrics.max_resident_island_visual_count,
        max_resident_island_visual_fraction: .metrics.max_resident_island_visual_fraction,
        max_stream_spawned_visuals_per_frame: .metrics.max_stream_spawned_visuals_per_frame,
        max_stream_despawned_visuals_per_frame: .metrics.max_stream_despawned_visuals_per_frame,
        max_mesh_count: .metrics.max_mesh_count,
        max_material_count: .metrics.max_material_count,
        max_loaded_mesh_vertices: .metrics.max_loaded_mesh_vertices,
        max_loaded_mesh_triangles: .metrics.max_loaded_mesh_triangles,
        failed_checks: ([
          .checks[]
          | select(.passed == false)
          | {name, value, comparator, threshold, unit}
        ] + [
          $perf_budget_checks[]
          | select(.passed == false)
          | {name, value, comparator, threshold, unit}
        ] + (
          if $eval_status == 0 then
            []
          else
            [{
              name: "eval_process_exit_status",
              value: $eval_status,
              comparator: "==",
              threshold: 0,
              unit: "status"
            }]
          end
        ))
      }
    ]
  }' "${summary_files[@]}" > "${summary_json}"

if ! jq -e 'all(.scenarios[]; .passed == true)' "${summary_json}" >/dev/null; then
  failed=1
fi

jq -r '
  [
    "scenario",
    "passed",
    "eval_status",
    "avg_frame_time_ms",
    "p95_frame_time_ms",
    "p99_frame_time_ms",
    "max_frame_time_ms",
    "runtime_frames_over_33_34ms",
    "runtime_frames_over_50ms",
    "runtime_frames_over_100ms",
    "max_entity_count",
    "max_mesh_count",
    "max_material_count",
    "max_loaded_mesh_vertices",
    "max_loaded_mesh_triangles",
    "max_visible_island_terrain_count",
    "max_visible_island_detail_count",
    "max_resident_island_visual_count",
    "max_resident_island_visual_fraction"
  ],
  (
    .scenarios[]
    | [
      .scenario,
      .passed,
      .eval_status,
      .avg_frame_time_ms,
      .p95_frame_time_ms,
      .p99_frame_time_ms,
      .max_frame_time_ms,
      .runtime_frames_over_33_34ms,
      .runtime_frames_over_50ms,
      .runtime_frames_over_100ms,
      .max_entity_count,
      .max_mesh_count,
      .max_material_count,
      .max_loaded_mesh_vertices,
      .max_loaded_mesh_triangles,
      .max_visible_island_terrain_count,
      .max_visible_island_detail_count,
      .max_resident_island_visual_count,
      .max_resident_island_visual_fraction
    ]
  )
  | @tsv' "${summary_json}" > "${summary_tsv}"

cat > "${interpretation_md}" <<'MARKDOWN'
# Perf Baseline Interpretation

This is a release-only app-path baseline. Use it to compare broad runtime cost before and after a focused change, not as a production renderer benchmark.

- Use `avg_frame_time_ms` for sustained cost and `p95`/`p99`/`max` for hitch risk.
- Default frame-time budgets are avg <= 24ms, p95 <= 45ms, and p99 <= 80ms.
  Override with `NAU_PERF_MAX_AVG_FRAME_TIME_MS`, `NAU_PERF_MAX_P95_FRAME_TIME_MS`, or `NAU_PERF_MAX_P99_FRAME_TIME_MS` only when intentionally gathering non-gating data.
- `eval_status` must be 0; nonzero process exits after writing a summary still fail the gate.
- `host_snapshot_before.txt` and `host_snapshot_after.txt` are required comparison evidence; high unrelated CPU before a run makes it non-gating. Post-run host load can be recorded as advisory with `NAU_PERF_REQUIRE_QUIET_HOST_AFTER=0`.
- `./tools/compare_perf_summaries.sh` rejects missing snapshots or any process over `NAU_PERF_MAX_HOST_PROCESS_CPU_PERCENT` percent CPU, default `80`.
- `./tools/perf_baseline.sh` refuses to start when the preflight host snapshot is already over that threshold; set `NAU_PERF_ALLOW_BUSY_HOST=1` only for non-gating investigation.
- Treat entity count, live mesh/material/vertex/triangle cost, visible detail, and resident visual counts as content-budget smoke tests.
- If a scenario fails, inspect `failed_checks` in `perf_summary.json` before changing traversal feel.
- By default, perf evals keep the window visible/focused while skipping screenshot capture, so frame-time telemetry is closer to manual play than hidden metric-only eval.
- Set `NAU_PERF_VISIBLE_WINDOW=0 NAU_PERF_CAPTURE_SCREENSHOT=1` only for compatibility baselines against older commits that do not support `--eval-visible-window`.
- Compare release perf summaries:
  `./tools/compare_perf_summaries.sh target/eval/perf_baseline/main/perf_summary.json target/eval/perf_baseline/candidate/perf_summary.json`
- Compare only summaries captured with matching `visible_window` and `capture_screenshot` settings.
- Foreground scripted profile gates:
  `./tools/scripted_play_profile.sh target/eval/play_profile/main_scripted_freeflight.json`
  `./tools/scripted_play_profile.sh target/eval/play_profile/candidate_scripted_freeflight.json`
- Inspect play profile result:
  `jq '{profile_kind, control_source, script, passed, checks, activity, frame_time, steady_frame_time, max}' target/eval/play_profile/main_scripted_freeflight.json`
- Compare play profiles:
  `./tools/compare_manual_play_profiles.sh target/eval/play_profile/main_scripted_freeflight.json target/eval/play_profile/candidate_scripted_freeflight.json`
- World-floor readiness gate:
  `./tools/world_floor_readiness.sh`
- Treat a play profile as non-gating until it covers foreground play for at least 30 seconds, travels at least 50m, and reports `"passed": true`.
MARKDOWN

if [[ "${require_quiet_host_after}" == "0" ]]; then
  write_host_snapshot "${host_snapshot_after}"
  if [[ "$(host_is_quiet "${host_snapshot_after}")" != "true" ]]; then
    echo "host is busy after release evals; continuing because NAU_PERF_REQUIRE_QUIET_HOST_AFTER=0" >&2
  fi
elif [[ "${allow_busy_host}" == "1" ]]; then
  write_host_snapshot "${host_snapshot_after}"
elif ! capture_quiet_host_snapshot "${host_snapshot_after}" "after release evals"; then
  echo "host is too busy after release evals; refusing to treat this as gating evidence" >&2
  echo "max process CPU: $(host_snapshot_max_cpu "${host_snapshot_after}")% (limit ${max_host_process_cpu_percent}%)" >&2
  echo "total top-process CPU: $(host_snapshot_total_cpu "${host_snapshot_after}")% (limit ${max_host_total_cpu_percent}%)" >&2
  echo "top process: $(host_snapshot_top_process "${host_snapshot_after}")" >&2
  if [[ -n "${ignore_process_pattern}" ]]; then
    echo "ignored process pattern: ${ignore_process_pattern}" >&2
  fi
  echo "snapshot: ${host_snapshot_after}" >&2
  echo "Set NAU_PERF_REQUIRE_QUIET_HOST_AFTER=0 only when post-run host load should be advisory." >&2
  failed=1
fi

printf 'perf summary JSON: %s\n' "${summary_json}"
printf 'perf summary TSV: %s\n' "${summary_tsv}"
printf 'interpretation: %s\n' "${interpretation_md}"
printf 'host snapshot before: %s\n' "${host_snapshot_before}"
printf 'host snapshot after: %s\n' "${host_snapshot_after}"

if (( failed != 0 )); then
  jq '{failed_scenarios: [.scenarios[] | select(.passed == false) | {scenario, failed_checks}]}' \
    "${summary_json}" >&2 || true
  exit 1
fi
