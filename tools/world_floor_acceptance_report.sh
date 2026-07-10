#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"
source "${repo_root}/tools/source_identity.sh"

output_path="${1:-docs/world-floor-acceptance-report.md}"
baseline_perf="${NAU_WORLD_FLOOR_BASELINE_PERF:-target/eval/perf_baseline/main_visible_no_screenshot/perf_summary.json}"
candidate_perf="${NAU_WORLD_FLOOR_CANDIDATE_PERF:-target/eval/perf_baseline/candidate_visible_no_screenshot/perf_summary.json}"
baseline_freeflight_profile="${NAU_WORLD_FLOOR_BASELINE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_PROFILE:-target/eval/play_profile/main_scripted_freeflight.json}}"
candidate_freeflight_profile="${NAU_WORLD_FLOOR_CANDIDATE_FREEFLIGHT_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_PROFILE:-target/eval/play_profile/candidate_scripted_freeflight.json}}"
baseline_ground_profile="${NAU_WORLD_FLOOR_BASELINE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_BASELINE_GROUND_PROFILE:-target/eval/play_profile/main_scripted_ground_traversal.json}}"
candidate_ground_profile="${NAU_WORLD_FLOOR_CANDIDATE_GROUND_TRAVERSAL_PROFILE:-${NAU_WORLD_FLOOR_CANDIDATE_GROUND_PROFILE:-target/eval/play_profile/candidate_scripted_ground_traversal.json}}"
visual_evidence_dir="${NAU_WORLD_FLOOR_VISUAL_EVIDENCE_DIR:-target/eval/world_floor_visual_evidence}"
require_visual_evidence="${NAU_WORLD_FLOOR_REQUIRE_VISUAL_EVIDENCE:-1}"
min_preview_count="${NAU_WORLD_FLOOR_MIN_PREVIEW_COUNT:-4}"
min_preview_bytes="${NAU_WORLD_FLOOR_MIN_PREVIEW_BYTES:-8192}"
report_date="${NAU_WORLD_FLOOR_ACCEPTANCE_DATE:-$(date -u +%F)}"
evidence_command="${NAU_WORLD_FLOOR_EVIDENCE_COMMAND:-./tools/world_floor_full_gate.sh}"
evidence_status="${NAU_WORLD_FLOOR_EVIDENCE_STATUS:-Artifact bundle generated from the command above or compatible refreshed artifact paths; use the exact command/status from the invoking environment for final evidence.}"
manual_accepted="${NAU_WORLD_FLOOR_MANUAL_ACCEPTED:-0}"
manual_summary="${NAU_WORLD_FLOOR_MANUAL_SUMMARY:-No human acceptance has been recorded.}"

case "${require_visual_evidence}" in
  0 | 1) ;;
  *)
    echo "NAU_WORLD_FLOOR_REQUIRE_VISUAL_EVIDENCE must be 0 or 1" >&2
    exit 2
    ;;
esac

case "${manual_accepted}" in
  0)
    acceptance_requirement="Automated evidence cannot accept this slice. Acceptance remains pending until an explicit human manual playtest is completed and reviewed."
    acceptance_state="pending human manual playtest"
    manual_proof_limit="- It does not accept the slice without the required explicit human manual playtest."
    acceptance_call="Acceptance is pending. The automated evidence is readiness input only; an explicit human manual playtest is required before this world-floor slice can be accepted."
    ;;
  1)
    acceptance_requirement="Automated evidence and the recorded human release playtest jointly accept this slice."
    acceptance_state="accepted"
    manual_proof_limit="- The accepted playtest is one bounded human session, not proof that every future route or hardware state will feel perfect."
    acceptance_call="Accepted. ${manual_summary}"
    ;;
  *)
    echo "NAU_WORLD_FLOOR_MANUAL_ACCEPTED must be 0 or 1" >&2
    exit 2
    ;;
esac

if ! [[ "${min_preview_count}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_MIN_PREVIEW_COUNT must be a non-negative integer" >&2
  exit 2
fi

if ! [[ "${min_preview_bytes}" =~ ^[0-9]+$ ]]; then
  echo "NAU_WORLD_FLOOR_MIN_PREVIEW_BYTES must be a non-negative integer" >&2
  exit 2
fi

resolve_path() {
  local path="$1"
  case "${path}" in
    /*) printf '%s\n' "${path}" ;;
    *) printf '%s/%s\n' "${repo_root}" "${path}" ;;
  esac
}

display_path() {
  local path
  path="$(resolve_path "$1")"
  printf '%s\n' "${path#"${repo_root}/"}"
}

file_size_bytes() {
  if stat -f%z "$1" >/dev/null 2>&1; then
    stat -f%z "$1"
  else
    stat -c%s "$1"
  fi
}

require_file() {
  local path="$1"
  if [[ ! -s "${path}" ]]; then
    echo "missing required artifact: ${path}" >&2
    exit 1
  fi
}

profile_value() {
  local profile="$1"
  local filter="$2"
  jq -er "${filter}" "${profile}"
}

perf_value() {
  local summary="$1"
  local scenario="$2"
  local key="$3"
  jq -er --arg scenario "${scenario}" --arg key "${key}" \
    '.scenarios[] | select(.scenario == $scenario) | .[$key]' \
    "${summary}"
}

profile_line() {
  local label="$1"
  local profile="$2"
  local duration
  local travel
  local avg
  local p95
  local p99
  local h33
  local h50
  local h100
  local entity
  local mesh
  local material
  local triangles
  local grounded_ratio

  duration="$(profile_value "${profile}" '.duration_secs')"
  travel="$(profile_value "${profile}" '.activity.horizontal_travel_m')"
  avg="$(profile_value "${profile}" '.steady_frame_time.avg_ms')"
  p95="$(profile_value "${profile}" '.steady_frame_time.p95_ms')"
  p99="$(profile_value "${profile}" '.steady_frame_time.p99_ms')"
  h33="$(profile_value "${profile}" '.steady_frame_time.frames_over_33_34ms')"
  h50="$(profile_value "${profile}" '.steady_frame_time.frames_over_50ms')"
  h100="$(profile_value "${profile}" '.steady_frame_time.frames_over_100ms')"
  entity="$(profile_value "${profile}" '.max.entity_count')"
  mesh="$(profile_value "${profile}" '.max.mesh_count')"
  material="$(profile_value "${profile}" '.max.material_count')"
  triangles="$(profile_value "${profile}" '.max.loaded_mesh_triangles')"
  grounded_ratio="$(profile_value "${profile}" '.ground_contact.grounded_ratio')"

  printf -- '- %s: `%s s`, `%s m` horizontal travel, grounded ratio `%s`, steady avg/p95/p99 `%s/%s/%s ms`, steady hitch buckets `%s/%s/%s`, max entity/mesh/material/triangles `%s/%s/%s/%s`.\n' \
    "${label}" "${duration}" "${travel}" "${grounded_ratio}" "${avg}" "${p95}" "${p99}" \
    "${h33}" "${h50}" "${h100}" "${entity}" "${mesh}" "${material}" "${triangles}"
}

perf_line() {
  local label="$1"
  local summary="$2"
  local scenario="$3"
  local eval_status
  local avg
  local p95
  local p99
  local h33
  local h50
  local h100
  local entity
  local mesh
  local material
  local triangles

  eval_status="$(perf_value "${summary}" "${scenario}" "eval_status")"
  avg="$(perf_value "${summary}" "${scenario}" "avg_frame_time_ms")"
  p95="$(perf_value "${summary}" "${scenario}" "p95_frame_time_ms")"
  p99="$(perf_value "${summary}" "${scenario}" "p99_frame_time_ms")"
  h33="$(perf_value "${summary}" "${scenario}" "runtime_frames_over_33_34ms")"
  h50="$(perf_value "${summary}" "${scenario}" "runtime_frames_over_50ms")"
  h100="$(perf_value "${summary}" "${scenario}" "runtime_frames_over_100ms")"
  entity="$(perf_value "${summary}" "${scenario}" "max_entity_count")"
  mesh="$(perf_value "${summary}" "${scenario}" "max_mesh_count")"
  material="$(perf_value "${summary}" "${scenario}" "max_material_count")"
  triangles="$(perf_value "${summary}" "${scenario}" "max_loaded_mesh_triangles")"

  printf -- '- %s `%s`: avg/p95/p99 `%s/%s/%s ms`, runtime hitch buckets `%s/%s/%s`, max entity/mesh/material/triangles `%s/%s/%s/%s`, app eval status `%s`.\n' \
    "${label}" "${scenario}" "${avg}" "${p95}" "${p99}" "${h33}" "${h50}" \
    "${h100}" "${entity}" "${mesh}" "${material}" "${triangles}" "${eval_status}"
}

require_clean_perf() {
  local summary="$1"
  local label="$2"

  if ! jq -e '.scenarios | length > 0 and all(.eval_status == 0 and .passed == true)' \
    "${summary}" >/dev/null; then
    echo "${label} perf summary contains a failed or missing scenario" >&2
    exit 1
  fi
}

require_passing_candidate_profile() {
  local profile="$1"
  local label="$2"

  jq -er '.passed == true' "${profile}" >/dev/null || {
    echo "${label} candidate scripted profile did not pass" >&2
    exit 1
  }
}

baseline_perf="$(resolve_path "${baseline_perf}")"
candidate_perf="$(resolve_path "${candidate_perf}")"
baseline_freeflight_profile="$(resolve_path "${baseline_freeflight_profile}")"
candidate_freeflight_profile="$(resolve_path "${candidate_freeflight_profile}")"
baseline_ground_profile="$(resolve_path "${baseline_ground_profile}")"
candidate_ground_profile="$(resolve_path "${candidate_ground_profile}")"
visual_evidence_dir="$(resolve_path "${visual_evidence_dir}")"
output_path="$(resolve_path "${output_path}")"

for artifact in \
  "${baseline_perf}" \
  "${candidate_perf}" \
  "${baseline_freeflight_profile}" \
  "${candidate_freeflight_profile}" \
  "${baseline_ground_profile}" \
  "${candidate_ground_profile}"; do
  require_file "${artifact}"
done

require_clean_perf "${baseline_perf}" "main"
require_clean_perf "${candidate_perf}" "candidate"
require_passing_candidate_profile "${candidate_freeflight_profile}" "freeflight"
require_passing_candidate_profile "${candidate_ground_profile}" "ground_traversal"

if [[ "${manual_accepted}" == "1" && "$(nau_source_state)" != "clean" ]]; then
  echo "accepted reports require a clean source state" >&2
  exit 1
fi

./tools/validate_world_floor_evidence_identity.sh \
  "${baseline_perf}" \
  "${candidate_perf}" \
  "${baseline_freeflight_profile}" \
  "${candidate_freeflight_profile}" \
  "${baseline_ground_profile}" \
  "${candidate_ground_profile}" >/dev/null

./tools/world_floor_readiness.sh \
  "${baseline_perf}" \
  "${candidate_perf}" \
  "${baseline_freeflight_profile}" \
  "${candidate_freeflight_profile}" \
  "${baseline_ground_profile}" \
  "${candidate_ground_profile}" >/dev/null

preview_dir="${visual_evidence_dir}/previews"
preview_lines=""
preview_count=0
if [[ -d "${preview_dir}" ]]; then
  while IFS= read -r preview_path; do
    [[ -n "${preview_path}" ]] || continue
    preview_count=$((preview_count + 1))
    preview_size="$(file_size_bytes "${preview_path}")"
    if (( preview_size < min_preview_bytes )); then
      echo "suspiciously small world-floor preview (${preview_size} bytes): ${preview_path}" >&2
      exit 1
    fi
    preview_lines+="- \`$(display_path "${preview_path}")\`"$'\n'
  done < <(find "${preview_dir}" -maxdepth 1 -type f -name '*.png' | sort)
fi

if (( require_visual_evidence == 1 && preview_count < min_preview_count )); then
  echo "expected at least ${min_preview_count} world-floor preview PNGs, found ${preview_count}: ${preview_dir}" >&2
  exit 1
fi

if (( preview_count == 0 )); then
  preview_lines="- Screenshot evidence was not required for this report run."$'\n'
fi

branch="$(git rev-parse --abbrev-ref HEAD)"
commit="$(nau_source_commit)"
source_state_detail="clean"
if [[ "$(nau_source_state)" != "clean" ]]; then
  dirty_path_count="$(git status --porcelain | wc -l | tr -d '[:space:]')"
  source_state_detail="dirty (${dirty_path_count} uncommitted tracked or untracked paths at report generation time)"
fi
candidate_floor_line="- Candidate ground-traversal world-floor runtime budget: \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_visible_tile_count')\` visible tiles, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_resident_tile_count')\` resident tiles, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_initial_spawned_tile_count')\` startup tiles, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_spawned_tiles_per_frame')\` max runtime spawns/frame, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_despawned_tiles_per_frame')\` max runtime despawns/frame, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_mesh_vertex_count')\` visible vertices, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_mesh_triangle_count')\` visible triangles, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_material_count')\` material."
candidate_floor_grammar_line="- Candidate ground-traversal world-floor visual grammar counters: \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_biome_count')\` biomes, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_terrain_feature_count')\` terrain features, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_color_band_count')\` color bands, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_river_vertex_count')\` river vertices, \`$(profile_value "${candidate_ground_profile}" '.max.world_floor_relief_range_m') m\` relief."
baseline_freeflight_line="$(profile_line "Main scripted \`freeflight\`" "${baseline_freeflight_profile}")"
candidate_freeflight_line="$(profile_line "Candidate scripted \`freeflight\`" "${candidate_freeflight_profile}")"
baseline_ground_line="$(profile_line "Main scripted \`ground_traversal\`" "${baseline_ground_profile}")"
candidate_ground_line="$(profile_line "Candidate scripted \`ground_traversal\`" "${candidate_ground_profile}")"
main_baseline_route_line="$(perf_line "Main" "${baseline_perf}" "baseline_route")"
candidate_baseline_route_line="$(perf_line "Candidate" "${candidate_perf}" "baseline_route")"
main_long_glide_line="$(perf_line "Main" "${baseline_perf}" "long_glide_visibility")"
candidate_long_glide_line="$(perf_line "Candidate" "${candidate_perf}" "long_glide_visibility")"

mkdir -p "$(dirname "${output_path}")"
tmp_path="${output_path}.tmp"

cat >"${tmp_path}" <<EOF
# World Floor Acceptance Report

Date: ${report_date}

Branch: \`${branch}\`

Commit: \`${commit}\`

Source state: \`${source_state_detail}\`

Generated by: \`./tools/world_floor_acceptance_report.sh\`

Requirement audit: \`docs/world-floor-requirements-audit.md\`

## Acceptance Requirement

${acceptance_requirement}

## Automated Readiness Criterion

Treat the branch as ready for the required human manual playtest when all of these are true:

- \`./tools/world_floor_full_gate.sh\` passes on a quiet host.
- Measured runs must start on a quiet host; post-run host snapshots are recorded, but unrelated load after the Bevy window exits is advisory unless \`NAU_WORLD_FLOOR_REQUIRE_QUIET_HOST_AFTER=1\`.
- The acceptance report is regenerated for the exact source state being accepted; if the source state is \`dirty\`, commit or rerun before treating the commit hash as the evidence identity.
- The generated screenshot bundle exists and passes the tool sanity checks.
- Clean-main-vs-candidate scripted \`freeflight\` and \`ground_traversal\` profiles pass their comparison, hitch, and host-load checks.
- Both ground-traversal profiles remain at least 98% grounded and travel at least 300 m horizontally.
- Main-vs-candidate app perf comparisons pass for \`baseline_route\` and \`long_glide_visibility\`.
- World-floor readiness passes its cost and visual-grammar budgets.

## Latest Evidence

Evidence command/status:

- Command: \`${evidence_command}\`
- Status: ${evidence_status}
- Acceptance state: \`${acceptance_state}\`

The report was generated from these artifacts:

- Main perf summary: \`$(display_path "${baseline_perf}")\`
- Candidate perf summary: \`$(display_path "${candidate_perf}")\`
- Main scripted freeflight profile: \`$(display_path "${baseline_freeflight_profile}")\`
- Candidate scripted freeflight profile: \`$(display_path "${candidate_freeflight_profile}")\`
- Main scripted ground-traversal profile: \`$(display_path "${baseline_ground_profile}")\`
- Candidate scripted ground-traversal profile: \`$(display_path "${candidate_ground_profile}")\`
- Screenshot evidence directory: \`$(display_path "${visual_evidence_dir}")\`

Foreground profile evidence:

${baseline_freeflight_line}
${candidate_freeflight_line}
${baseline_ground_line}
${candidate_ground_line}
${candidate_floor_line}
${candidate_floor_grammar_line}

Visible-window/no-screenshot app perf evidence:

${main_baseline_route_line}
${candidate_baseline_route_line}
${main_long_glide_line}
${candidate_long_glide_line}

Screenshot evidence paths:

${preview_lines}
## What This Proves

- The current world floor is budgeted, streamed, traversable terrain with measured ground contact and travel.
- It does not introduce a meaningful measured release perf regression in the required app-path scenarios.
- The scripted freeflight and ground-traversal routes exercise real runtime movement, camera, player, terrain-contact, and streaming paths without manual input.
- The branch now has host-load guardrails, before/after comparison, live asset-cost counters, and world-floor budget gates.

## What This Does Not Prove

- It does not prove a long human play session will feel perfect.
${manual_proof_limit}
- It does not prove the M4 Max will never spin fans under all foreground play patterns.
- It does not prove the lower world has reached the same final art richness as islands.
- It does not replace future product taste review for richer ground/world work.

## Acceptance Call

${acceptance_call}
EOF

mv "${tmp_path}" "${output_path}"

cat <<EOF

World-floor acceptance report written to:
  $(display_path "${output_path}")
EOF
