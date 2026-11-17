#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -lt 2 ]]; then
  echo "Usage: $0 <perf_summary.json> <expected_scenario>..." >&2
  exit 2
fi
summary="$1"
shift
expected_scenarios="$(printf '%s\n' "$@" | jq -Rsc 'split("\n") | map(select(length > 0)) | sort')"

# Capture validity is separate from acceptance: failed timing budgets may be
# remeasured, but failed gameplay, incomplete data and unknown clocks may not.
jq -e --argjson expected "${expected_scenarios}" '
  .mode == "release"
  and .frame_time_clock == "real"
  and .frame_time_scope == "runtime_after_startup_warmup"
  and .startup_warmup_frames == 30
  and (.source_fingerprint | type == "string" and length > 0)
  and ([.scenarios[].scenario] | sort) == $expected
  and all(.scenarios[];
    .eval_status == 0 and .eval_passed == true
    and (.frame_count | type == "number" and . > 0)
    and all(
      .avg_frame_time_ms, .p95_frame_time_ms, .p99_frame_time_ms,
      .max_frame_time_ms, .runtime_frame_time_sample_count,
      .avg_runtime_frame_time_ms, .p95_runtime_frame_time_ms,
      .p99_runtime_frame_time_ms, .max_runtime_frame_time_ms,
      .eval_artifact_frame_time_sample_count,
      .runtime_frames_over_33_34ms, .runtime_frames_over_50ms,
      .runtime_frames_over_100ms, .max_entity_count, .max_mesh_count,
      .max_material_count, .max_loaded_mesh_vertices, .max_loaded_mesh_triangles,
      .max_visible_island_terrain_count, .max_visible_island_detail_count,
      .max_resident_island_visual_count, .max_stream_spawned_visuals_per_frame,
      .max_stream_despawned_visuals_per_frame;
      type == "number" and . >= 0
    )
    and .runtime_frame_time_sample_count > 0
    and (.runtime_frame_time_sample_count + .eval_artifact_frame_time_sample_count)
      == (.frame_count + 1 - 30)
  )
' "${summary}" >/dev/null

for key in host_snapshot_before host_snapshot_after; do
  snapshot="$(jq -er --arg key "${key}" '.[$key] | select(type == "string")' "${summary}")"
  [[ -s "${snapshot}" ]] || { echo "missing capture snapshot: ${snapshot}" >&2; exit 1; }
done
