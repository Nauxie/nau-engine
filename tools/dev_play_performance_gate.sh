#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required by the development play performance gate" >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

output_root="${1:-target/eval/dev_play_performance}"
scenario="${NAU_DEV_PLAY_PERF_SCENARIO:-camera_mouse_control}"
visible_window="${NAU_DEV_PLAY_PERF_VISIBLE_WINDOW:-0}"
run_warmup="${NAU_DEV_PLAY_PERF_RUN_WARMUP:-1}"
run_host_preflight="${NAU_DEV_PLAY_PERF_RUN_HOST_PREFLIGHT:-1}"
max_avg_frame_time_ms="${NAU_DEV_PLAY_PERF_MAX_AVG_FRAME_TIME_MS:-12}"
max_p95_frame_time_ms="${NAU_DEV_PLAY_PERF_MAX_P95_FRAME_TIME_MS:-18}"
max_frame_time_ms="${NAU_DEV_PLAY_PERF_MAX_FRAME_TIME_MS:-35}"
max_frames_over_16_67ms="${NAU_DEV_PLAY_PERF_MAX_FRAMES_OVER_16_67MS:-24}"
max_material_count="${NAU_DEV_PLAY_PERF_MAX_MATERIAL_COUNT:-128}"
max_debug_release_avg_ratio="${NAU_DEV_PLAY_PERF_MAX_DEBUG_RELEASE_AVG_RATIO:-1.25}"
debug_summary_override="${NAU_DEV_PLAY_PERF_DEBUG_SUMMARY:-}"
release_summary_override="${NAU_DEV_PLAY_PERF_RELEASE_SUMMARY:-}"

for toggle_name in visible_window run_warmup run_host_preflight; do
  toggle_value="${!toggle_name}"
  case "${toggle_value}" in
    0 | 1) ;;
    *)
      echo "${toggle_name} must be 0 or 1" >&2
      exit 2
      ;;
  esac
done

for value_name in \
  max_avg_frame_time_ms \
  max_p95_frame_time_ms \
  max_frame_time_ms \
  max_debug_release_avg_ratio
do
  value="${!value_name}"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]] \
    || ! jq -en --arg value "${value}" '$value | tonumber | . > 0' >/dev/null
  then
    echo "${value_name} must be a positive number, got: ${value}" >&2
    exit 2
  fi
done

if ! [[ "${max_frames_over_16_67ms}" =~ ^(0|[1-9][0-9]*)$ ]]; then
  echo "max_frames_over_16_67ms must be a non-negative integer, got: ${max_frames_over_16_67ms}" >&2
  exit 2
fi

if ! [[ "${max_material_count}" =~ ^[1-9][0-9]*$ ]]; then
  echo "max_material_count must be a positive integer, got: ${max_material_count}" >&2
  exit 2
fi

if [[ -n "${debug_summary_override}" || -n "${release_summary_override}" ]]; then
  if [[ -z "${debug_summary_override}" || -z "${release_summary_override}" ]]; then
    echo "set both NAU_DEV_PLAY_PERF_DEBUG_SUMMARY and NAU_DEV_PLAY_PERF_RELEASE_SUMMARY" >&2
    exit 2
  fi
fi

validate_summary() {
  local profile="$1"
  local summary="$2"

  if [[ ! -s "${summary}" ]]; then
    echo "missing ${profile} performance summary: ${summary}" >&2
    exit 1
  fi

  if ! jq -e '
    ([.checks[] | select(.name == "sample_count")][0].threshold) as $expected_samples
    | (.passed | type) == "boolean"
      and .metrics.sample_count == $expected_samples
      and .metrics.avg_frame_time_ms > 0
      and .metrics.p95_frame_time_ms > 0
      and (.metrics.max_frame_time_ms | type) == "number"
      and .metrics.max_frame_time_ms > 0
      and (.metrics.frames_over_16_67ms | type) == "number"
      and .metrics.frames_over_16_67ms >= 0
      and (.metrics.frames_over_16_67ms | floor) == .metrics.frames_over_16_67ms
      and .metrics.max_entity_count > 0
      and .metrics.max_mesh_count > 0
      and (.metrics.max_material_count | type) == "number"
      and .metrics.max_material_count > 0
      and (.metrics.max_material_count | floor) == .metrics.max_material_count
      and .metrics.max_loaded_mesh_triangles > 0
  ' "${summary}" >/dev/null; then
    echo "incomplete ${profile} performance evidence: ${summary}" >&2
    exit 1
  fi
}

run_eval() {
  local profile="$1"
  local output_dir="${output_root}/${profile}"
  local warmup_dir="${output_root}/${profile}_warmup"
  local summary="${output_dir}/summary.json"
  local run_log="${output_dir}/run.log"
  local eval_status
  local command=(cargo run --quiet)

  if [[ "${profile}" == "release" ]]; then
    command+=(--release)
  fi

  command+=(
    --bin nau-engine
    --
    --eval "${scenario}"
    --eval-no-screenshot
  )

  if [[ "${visible_window}" == "1" ]]; then
    command+=(--eval-visible-window)
  fi

  if [[ "${run_warmup}" == "1" ]]; then
    rm -rf "${warmup_dir}"
    mkdir -p "${warmup_dir}"
    "${command[@]}" --eval-output "${warmup_dir}" \
      > "${warmup_dir}/run.log" 2>&1
    validate_summary "${profile} warmup" "${warmup_dir}/summary.json"
  fi

  rm -rf "${output_dir}"
  mkdir -p "${output_dir}"

  set +e
  "${command[@]}" --eval-output "${output_dir}" 2>&1 | tee "${run_log}"
  eval_status="${PIPESTATUS[0]}"
  set -e

  validate_summary "${profile}" "${summary}"
  if (( eval_status != 0 )); then
    echo "${profile} eval exited ${eval_status}; performance evidence is complete and will still be compared" >&2
  fi

  printf '%s' "${eval_status}" > "${output_dir}/eval_status.txt"
}

mkdir -p "${output_root}"

if [[ -n "${debug_summary_override}" ]]; then
  debug_summary="${debug_summary_override}"
  release_summary="${release_summary_override}"
  debug_eval_status="0"
  release_eval_status="0"
else
  if [[ "${run_host_preflight}" == "1" ]]; then
    NAU_PERF_HOST_PREFLIGHT_OUTPUT="${output_root}/host_preflight.txt" \
      ./tools/perf_host_preflight.sh
  fi

  cargo build --bin nau-engine
  cargo build --release --bin nau-engine

  run_eval "debug"
  run_eval "release"

  debug_summary="${output_root}/debug/summary.json"
  release_summary="${output_root}/release/summary.json"
  debug_eval_status="$(cat "${output_root}/debug/eval_status.txt")"
  release_eval_status="$(cat "${output_root}/release/eval_status.txt")"
fi

validate_summary "debug" "${debug_summary}"
validate_summary "release" "${release_summary}"

report="${output_root}/report.json"
jq -n \
  --slurpfile debug "${debug_summary}" \
  --slurpfile release "${release_summary}" \
  --arg scenario "${scenario}" \
  --arg debug_summary "${debug_summary}" \
  --arg release_summary "${release_summary}" \
  --argjson debug_eval_status "${debug_eval_status}" \
  --argjson release_eval_status "${release_eval_status}" \
  --argjson max_avg_frame_time_ms "${max_avg_frame_time_ms}" \
  --argjson max_p95_frame_time_ms "${max_p95_frame_time_ms}" \
  --argjson max_frame_time_ms "${max_frame_time_ms}" \
  --argjson max_frames_over_16_67ms "${max_frames_over_16_67ms}" \
  --argjson max_material_count "${max_material_count}" \
  --argjson max_debug_release_avg_ratio "${max_debug_release_avg_ratio}" \
  --argjson run_warmup "${run_warmup}" \
  --argjson run_host_preflight "${run_host_preflight}" \
  '
  ($debug[0].metrics.avg_frame_time_ms
    / $release[0].metrics.avg_frame_time_ms) as $avg_ratio
  | {
      schema: "nau_dev_play_performance_gate.v1",
      scenario: $scenario,
      measurement: {
        warmup_run: ($run_warmup == 1),
        host_preflight: ($run_host_preflight == 1)
      },
      passed: (
        $debug_eval_status == 0
        and $release_eval_status == 0
        and $debug[0].passed == true
        and $release[0].passed == true
        and $debug[0].metrics.avg_frame_time_ms <= $max_avg_frame_time_ms
        and $release[0].metrics.avg_frame_time_ms <= $max_avg_frame_time_ms
        and $debug[0].metrics.p95_frame_time_ms <= $max_p95_frame_time_ms
        and $release[0].metrics.p95_frame_time_ms <= $max_p95_frame_time_ms
        and $debug[0].metrics.max_frame_time_ms <= $max_frame_time_ms
        and $release[0].metrics.max_frame_time_ms <= $max_frame_time_ms
        and $debug[0].metrics.frames_over_16_67ms <= $max_frames_over_16_67ms
        and $release[0].metrics.frames_over_16_67ms <= $max_frames_over_16_67ms
        and $debug[0].metrics.max_material_count <= $max_material_count
        and $release[0].metrics.max_material_count <= $max_material_count
        and $avg_ratio <= $max_debug_release_avg_ratio
        and $debug[0].metrics.max_entity_count == $release[0].metrics.max_entity_count
        and $debug[0].metrics.max_mesh_count == $release[0].metrics.max_mesh_count
        and $debug[0].metrics.max_loaded_mesh_triangles
          == $release[0].metrics.max_loaded_mesh_triangles
      ),
      thresholds: {
        max_avg_frame_time_ms: $max_avg_frame_time_ms,
        max_p95_frame_time_ms: $max_p95_frame_time_ms,
        max_frame_time_ms: $max_frame_time_ms,
        max_frames_over_16_67ms: $max_frames_over_16_67ms,
        max_material_count: $max_material_count,
        max_debug_release_avg_ratio: $max_debug_release_avg_ratio
      },
      debug: {
        summary: $debug_summary,
        eval_status: $debug_eval_status,
        avg_frame_time_ms: $debug[0].metrics.avg_frame_time_ms,
        p95_frame_time_ms: $debug[0].metrics.p95_frame_time_ms,
        max_frame_time_ms: $debug[0].metrics.max_frame_time_ms,
        frames_over_16_67ms: $debug[0].metrics.frames_over_16_67ms,
        frames_over_33_34ms: $debug[0].metrics.frames_over_33_34ms,
        frames_over_50ms: $debug[0].metrics.frames_over_50ms,
        entity_count: $debug[0].metrics.max_entity_count,
        mesh_count: $debug[0].metrics.max_mesh_count,
        max_material_count: $debug[0].metrics.max_material_count,
        loaded_mesh_triangles: $debug[0].metrics.max_loaded_mesh_triangles
      },
      release: {
        summary: $release_summary,
        eval_status: $release_eval_status,
        avg_frame_time_ms: $release[0].metrics.avg_frame_time_ms,
        p95_frame_time_ms: $release[0].metrics.p95_frame_time_ms,
        max_frame_time_ms: $release[0].metrics.max_frame_time_ms,
        frames_over_16_67ms: $release[0].metrics.frames_over_16_67ms,
        frames_over_33_34ms: $release[0].metrics.frames_over_33_34ms,
        frames_over_50ms: $release[0].metrics.frames_over_50ms,
        entity_count: $release[0].metrics.max_entity_count,
        mesh_count: $release[0].metrics.max_mesh_count,
        max_material_count: $release[0].metrics.max_material_count,
        loaded_mesh_triangles: $release[0].metrics.max_loaded_mesh_triangles
      },
      debug_release_avg_ratio: $avg_ratio
    }
  ' > "${report}"

jq . "${report}"

if ! jq -e '.passed == true' "${report}" >/dev/null; then
  echo "development play performance gate failed: ${report}" >&2
  exit 1
fi
