#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to aggregate perf baseline summaries" >&2
  exit 1
fi

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
repo_commit="$(git rev-parse --short HEAD 2>/dev/null || printf 'unknown')"
failed=0

mkdir -p "${output_root}"
printf 'scenario\teval_status\tsummary\n' > "${status_tsv}"

for scenario in "${scenarios[@]}"; do
  scenario_out="${output_root}/${scenario}"
  set +e
  cargo run --release --bin nau-engine -- \
    --eval "${scenario}" \
    --eval-output "${scenario_out}" \
    --eval-no-screenshot
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
  --arg output_root "${output_root}" \
  '{
    generated_at: $generated_at,
    repo_commit: $repo_commit,
    mode: "release",
    output_root: $output_root,
    eval_command: "cargo run --release --bin nau-engine -- --eval <scenario> --eval-output <dir> --eval-no-screenshot",
    scenarios: [
      .[] | {
        scenario,
        passed,
        frame_count,
        duration_secs,
        avg_frame_time_ms: .metrics.avg_frame_time_ms,
        p95_frame_time_ms: .metrics.p95_frame_time_ms,
        p99_frame_time_ms: .metrics.p99_frame_time_ms,
        max_frame_time_ms: .metrics.max_frame_time_ms,
        max_entity_count: .metrics.max_entity_count,
        max_visible_island_terrain_count: .metrics.max_visible_island_terrain_count,
        max_visible_island_detail_count: .metrics.max_visible_island_detail_count,
        max_resident_island_visual_count: .metrics.max_resident_island_visual_count,
        max_resident_island_visual_fraction: .metrics.max_resident_island_visual_fraction,
        max_stream_spawned_visuals_per_frame: .metrics.max_stream_spawned_visuals_per_frame,
        max_stream_despawned_visuals_per_frame: .metrics.max_stream_despawned_visuals_per_frame,
        failed_checks: [
          .checks[]
          | select(.passed == false)
          | {name, value, comparator, threshold, unit}
        ]
      }
    ]
  }' "${summary_files[@]}" > "${summary_json}"

jq -r '
  [
    "scenario",
    "passed",
    "avg_frame_time_ms",
    "p95_frame_time_ms",
    "p99_frame_time_ms",
    "max_frame_time_ms",
    "max_entity_count",
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
      .avg_frame_time_ms,
      .p95_frame_time_ms,
      .p99_frame_time_ms,
      .max_frame_time_ms,
      .max_entity_count,
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
- Treat entity, visible detail, and resident visual counts as content-budget smoke tests.
- If a scenario fails, inspect `failed_checks` in `perf_summary.json` before changing traversal feel.
- Manual feel smoke remains `cargo run --release -- --play`.
MARKDOWN

printf 'perf summary JSON: %s\n' "${summary_json}"
printf 'perf summary TSV: %s\n' "${summary_tsv}"
printf 'interpretation: %s\n' "${interpretation_md}"

if (( failed != 0 )); then
  jq '{failed_scenarios: [.scenarios[] | select(.passed == false) | {scenario, failed_checks}]}' \
    "${summary_json}" >&2 || true
  exit 1
fi
