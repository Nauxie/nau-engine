#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-baseline_route}"
output_dir="${2:-target/eval/${scenario}}"
extra_args=()
min_png_bytes=16384
screenshot_requested="${NAU_EVAL_SCREENSHOT:-0}"
no_screenshot_requested="${NAU_EVAL_NO_SCREENSHOT:-0}"
sim_only_requested="${NAU_EVAL_SIM_ONLY:-0}"
visual_audit_requested="${NAU_EVAL_VISUAL_AUDIT:-1}"
asset_audit_requested="${NAU_EVAL_ASSET_AUDIT:-1}"
semantic_scene_audit_requested="${NAU_EVAL_SEMANTIC_SCENE_AUDIT:-1}"
visual_audit_path="${output_dir}/visual_audit.json"
marker_projection_audit_path="${output_dir}/marker_projection_audit.json"
semantic_scene_audit_path="${output_dir}/semantic_scene_audit.json"
asset_audit_path="${output_dir}/asset_fixture_audit.json"
visual_audit_status=0
marker_projection_audit_status=0
semantic_scene_audit_status=0
asset_audit_status=0
eval_status=0

file_size_bytes() {
  if stat -f%z "$1" >/dev/null 2>&1; then
    stat -f%z "$1"
  else
    stat -c%s "$1"
  fi
}

if [[ "${no_screenshot_requested}" == "1" || "${screenshot_requested}" != "1" ]]; then
  extra_args+=(--eval-no-screenshot)
fi

rm -f "${visual_audit_path}" "${marker_projection_audit_path}" "${semantic_scene_audit_path}" "${asset_audit_path}"

if [[ "${no_screenshot_requested}" != "1" && "${screenshot_requested}" == "1" ]] && ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for screenshot artifact validation; install jq or run without NAU_EVAL_SCREENSHOT=1" >&2
  exit 1
fi

if [[ "${sim_only_requested}" == "1" && "${screenshot_requested}" == "1" && "${no_screenshot_requested}" != "1" ]]; then
  echo "NAU_EVAL_SIM_ONLY=1 does not support screenshot artifacts; unset NAU_EVAL_SCREENSHOT or set NAU_EVAL_NO_SCREENSHOT=1" >&2
  exit 1
fi

set +e
if [[ "${sim_only_requested}" == "1" ]]; then
  cargo run --bin traversal_sim_eval -- "${scenario}" "${output_dir}"
elif [[ "${#extra_args[@]}" -gt 0 ]]; then
  cargo run --bin nau-engine -- --eval "${scenario}" --eval-output "${output_dir}" "${extra_args[@]}"
else
  cargo run --bin nau-engine -- --eval "${scenario}" --eval-output "${output_dir}"
fi
eval_status=$?
set -e

summary="${output_dir}/summary.json"
samples="${output_dir}/samples.ndjson"

if [[ ! -s "${summary}" ]]; then
  echo "missing eval summary: ${summary}" >&2
  if (( eval_status != 0 )); then
    exit "${eval_status}"
  fi
  exit 1
fi

if [[ ! -s "${samples}" ]]; then
  echo "missing eval samples: ${samples}" >&2
  if (( eval_status != 0 )); then
    exit "${eval_status}"
  fi
  exit 1
fi

if command -v jq >/dev/null 2>&1 && ! jq -e '.passed == true' "${summary}" >/dev/null; then
  echo "eval summary failed: ${summary}" >&2
  jq '{passed, failed_checks: [.checks[] | select(.passed == false)], final_sample}' \
    "${summary}" >&2 || true
  if (( eval_status != 0 )); then
    exit "${eval_status}"
  fi
  exit 1
elif ! command -v jq >/dev/null 2>&1 && grep -Eq '"passed"[[:space:]]*:[[:space:]]*false' "${summary}"; then
  echo "eval summary failed: ${summary}" >&2
  sed -n '1,220p' "${summary}" >&2
  if (( eval_status != 0 )); then
    exit "${eval_status}"
  fi
  exit 1
fi

if (( eval_status != 0 )); then
  echo "eval command failed with status ${eval_status}: ${summary}" >&2
  exit "${eval_status}"
fi

if [[ "${asset_audit_requested}" != "0" ]]; then
  set +e
  cargo run --quiet --bin asset_fixture_audit > "${asset_audit_path}"
  asset_audit_status=$?
  set -e
fi

if [[ "${no_screenshot_requested}" != "1" && "${screenshot_requested}" == "1" ]]; then
  screenshot_artifacts=()
  marker_metadata_artifacts=()
  while IFS= read -r artifact; do
    if [[ -z "${artifact}" || "${artifact}" == "null" ]]; then
      continue
    fi
    if [[ ! -s "${artifact}" ]]; then
      echo "missing screenshot artifact: ${artifact}" >&2
      exit 1
    fi
    artifact_size="$(file_size_bytes "${artifact}")"
    if (( artifact_size < min_png_bytes )); then
      echo "suspiciously small screenshot artifact (${artifact_size} bytes): ${artifact}" >&2
      exit 1
    fi
    screenshot_artifacts+=("${artifact}")
  done < <(jq -r '.artifacts.screenshot_png, (.artifacts.checkpoint_screenshots[]?)' "${summary}")

  while IFS= read -r artifact; do
    if [[ -z "${artifact}" || "${artifact}" == "null" ]]; then
      continue
    fi
    if [[ ! -s "${artifact}" ]]; then
      echo "missing checkpoint marker metadata: ${artifact}" >&2
      exit 1
    fi
    if ! jq -e '.passed == true' "${artifact}" >/dev/null; then
      echo "checkpoint marker semantic audit failed: ${artifact}" >&2
      jq '{passed, frame, checkpoint, semantic_marker_count, expected_objective_marker_count, in_viewport_semantic_marker_count, occluded_semantic_marker_count, visible_semantic_marker_count, current_objective_visible, semantic_scene_sample_count, in_viewport_semantic_scene_sample_count, occluded_semantic_scene_sample_count, visible_semantic_scene_sample_count, visible_semantic_scene_material_count, markers: [.markers[] | {kind, label, current_objective, in_viewport, visibility, occluder, screen}], scene_samples: [.scene_samples[]? | {kind, label, expected_material, in_viewport, visibility, occluder, screen}]}' \
        "${artifact}" >&2 || true
      exit 1
    fi
    marker_metadata_artifacts+=("${artifact}")
  done < <(jq -r '.artifacts.checkpoint_marker_metadata[]?' "${summary}")

  if [[ "${#marker_metadata_artifacts[@]}" -gt 0 ]]; then
    set +e
    cargo run --quiet --bin marker_projection_audit -- "${marker_metadata_artifacts[@]}" \
      > "${marker_projection_audit_path}"
    marker_projection_audit_status=$?
    set -e

    if [[ "${semantic_scene_audit_requested}" != "0" ]]; then
      set +e
      cargo run --quiet --bin semantic_scene_audit -- "${marker_metadata_artifacts[@]}" \
        > "${semantic_scene_audit_path}"
      semantic_scene_audit_status=$?
      set -e
    fi
  fi

  if [[ "${visual_audit_requested}" != "0" && "${#screenshot_artifacts[@]}" -gt 0 ]]; then
    visual_audit_args=()
    if [[ "${scenario}" == "world_collision_contact" ]]; then
      visual_audit_args+=(--profile close_obstruction)
    fi
    set +e
    cargo run --quiet --bin visual_audit -- "${visual_audit_args[@]}" "${screenshot_artifacts[@]}" \
      > "${visual_audit_path}"
    visual_audit_status=$?
    set -e
  fi
fi

if command -v jq >/dev/null 2>&1; then
  jq '{scenario, passed, metrics, checks, artifacts}' "${summary}"
else
  sed -n '1,220p' "${summary}"
fi

if (( visual_audit_status != 0 )); then
  echo "visual audit failed: ${visual_audit_path}" >&2
  if command -v jq >/dev/null 2>&1 && [[ -s "${visual_audit_path}" ]]; then
    jq '{passed, checks, failed_images: [.images[] | select(.passed == false) | {path, checks: [.checks[] | select(.passed == false)]}]}' \
      "${visual_audit_path}" >&2 || true
  fi
  exit "${visual_audit_status}"
fi

if (( marker_projection_audit_status != 0 )); then
  echo "marker projection audit failed: ${marker_projection_audit_path}" >&2
  if command -v jq >/dev/null 2>&1 && [[ -s "${marker_projection_audit_path}" ]]; then
    jq '{passed, checks, failed_checkpoints: [.checkpoints[] | select(.passed == false) | {checkpoint, metadata_path, screenshot_path, in_viewport_marker_count, occluded_marker_count, visible_marker_count, marker_pixel_hit_count, markers: [.markers[] | select(.in_viewport == true) | {kind, label, visibility, screen, marker_pixel_hits, passed}]}]}' \
      "${marker_projection_audit_path}" >&2 || true
  fi
  exit "${marker_projection_audit_status}"
fi

if (( semantic_scene_audit_status != 0 )); then
  echo "semantic scene audit failed: ${semantic_scene_audit_path}" >&2
  if command -v jq >/dev/null 2>&1 && [[ -s "${semantic_scene_audit_path}" ]]; then
    jq '{passed, checks, failed_checkpoints: [.checkpoints[] | select(.passed == false) | {checkpoint, metadata_path, screenshot_path, in_viewport_scene_sample_count, occluded_scene_sample_count, visible_scene_sample_count, scene_sample_pixel_hit_count, visible_scene_material_count, scene_material_pixel_hit_count, materials, samples: [.samples[] | select(.in_viewport == true and .passed == false) | {kind, label, expected_material, visibility, screen, semantic_pixel_hits, passed}]}]}' \
      "${semantic_scene_audit_path}" >&2 || true
  fi
  exit "${semantic_scene_audit_status}"
fi

if (( asset_audit_status != 0 )); then
  echo "asset fixture audit failed: ${asset_audit_path}" >&2
  if command -v jq >/dev/null 2>&1 && [[ -s "${asset_audit_path}" ]]; then
    jq '{passed, checks, failed_fixtures: [.fixtures[] | select(.passed == false) | {kind, path, checks: [.checks[] | select(.passed == false)]}]}' \
      "${asset_audit_path}" >&2 || true
  fi
  exit "${asset_audit_status}"
fi
