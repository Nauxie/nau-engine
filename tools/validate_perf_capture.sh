#!/usr/bin/env bash
set -euo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$#" -lt 2 ]]; then
  echo "Usage: $0 <perf_summary.json> <expected_scenario>..." >&2
  exit 2
fi
summary="$1"
shift
expected_scenarios="$(printf '%s\n' "$@" | jq -Rsc 'split("\n") | map(select(length > 0)) | sort')"

# Capture validity is separate from acceptance: failed timing budgets may be
# remeasured, but failed gameplay, incomplete data and unknown clocks may not.
jq -L "${script_dir}" -e --argjson expected "${expected_scenarios}" '
  include "perf_timing";
  perf_timing_valid
  and (.source_fingerprint | type == "string" and length > 0)
  and ([.scenarios[].scenario] | sort) == $expected
  and all(.scenarios[];
    .eval_status == 0 and .eval_passed == true
    and all(
      .max_entity_count, .max_mesh_count,
      .max_material_count, .max_loaded_mesh_vertices, .max_loaded_mesh_triangles,
      .max_visible_island_terrain_count, .max_visible_island_detail_count,
      .max_resident_island_visual_count, .max_stream_spawned_visuals_per_frame,
      .max_stream_despawned_visuals_per_frame;
      type == "number" and . >= 0
    )
  )
' "${summary}" >/dev/null

for key in host_snapshot_before host_snapshot_after; do
  snapshot="$(jq -er --arg key "${key}" '.[$key] | select(type == "string")' "${summary}")"
  [[ -s "${snapshot}" ]] || { echo "missing capture snapshot: ${snapshot}" >&2; exit 1; }
done
