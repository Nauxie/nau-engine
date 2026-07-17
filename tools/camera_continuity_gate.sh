#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

output_root="${1:-target/eval/camera_continuity}"
run_sim="${NAU_CAMERA_CONTINUITY_RUN_SIM:-1}"
run_app="${NAU_CAMERA_CONTINUITY_RUN_APP:-1}"
run_fault_proof="${NAU_CAMERA_CONTINUITY_RUN_FAULT_PROOF:-1}"

requested_refresh_rates=(30 60 120 144)
requested_hitches_ms=(50 100)

contract_tests=(
  large_mouse_burst_applies_full_orbit_in_same_frame
  mouse_orbit_applies_same_frame_across_frame_rates
  clear_camera_static_yaw_step_meets_response_contract_across_frame_rates
  clear_camera_static_pitch_step_meets_response_contract_across_frame_rates
  clear_camera_yaw_reversal_responds_without_wrong_way_motion_across_frame_rates
  environmental_continuity_contract_is_frame_rate_independent
  continuity_contract_matches_substeps_across_hitches
  obstruction_release
  approaching_launch_spire_does_not_oscillate_camera_yaw
  first_mouse_delta_after_look_capture_quarantines_only_implausible_spikes
  inactive_look_resets_capture_history_before_resume
  camera_floor_does_not_capture_an_island_from_below
  upward_underside_crossing_does_not_capture_island_top
  deep_collision_correction_is_bounded_per_step
  hitch_sized_high_speed_step_substeps_collision_without_pop
  reset_with_held_movement_uses_reset_camera_basis_same_frame
  reset_destination_gameplay_proxies_are_complete_before_reset
)

simulation_scenarios=(
  camera_yaw_stability
  camera_turn_stability
  camera_strafe_stability
  air_control_response
  great_sky_plateau_route
  plateau_arrival_camera
  underbridge_under_route
)

app_scenarios=(
  camera_mouse_control
  playtest_reset
  world_collision_contact
  terrain_rim_collision_contact
  terrain_body_collision_contact
  terrain_edge_walkoff
)

world_collision_max_push_m="${NAU_CAMERA_CONTINUITY_WORLD_COLLISION_MAX_PUSH_M:-0.10}"
terrain_rim_max_push_m="${NAU_CAMERA_CONTINUITY_TERRAIN_RIM_MAX_PUSH_M:-0.15}"
terrain_body_max_push_m="${NAU_CAMERA_CONTINUITY_TERRAIN_BODY_MAX_PUSH_M:-0.15}"
max_relative_velocity_mps="${NAU_CAMERA_CONTINUITY_MAX_RELATIVE_VELOCITY_MPS:-24.1}"
max_relative_acceleration_mps2="${NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ACCELERATION_MPS2:-1300}"
max_relative_angular_velocity_degrees_per_sec="${NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_VELOCITY_DEGREES_PER_SEC:-180}"
max_relative_angular_acceleration_degrees_per_sec2="${NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_ACCELERATION_DEGREES_PER_SEC2:-6000}"
max_player_residual_m="${NAU_CAMERA_CONTINUITY_MAX_PLAYER_RESIDUAL_M:-0.02}"
max_player_world_correction_m="${NAU_CAMERA_CONTINUITY_MAX_PLAYER_WORLD_CORRECTION_M:-0.50}"
max_player_collision_correction_m="${NAU_CAMERA_CONTINUITY_MAX_PLAYER_COLLISION_CORRECTION_M:-1.0}"
world_collision_expected_contact_samples="${NAU_CAMERA_CONTINUITY_WORLD_CONTACT_SAMPLES:-90}"
terrain_rim_expected_contact_samples="${NAU_CAMERA_CONTINUITY_TERRAIN_RIM_CONTACT_SAMPLES:-55}"
terrain_body_expected_contact_samples="${NAU_CAMERA_CONTINUITY_TERRAIN_BODY_CONTACT_SAMPLES:-120}"

validate_toggle() {
  local name="$1"
  local value="$2"
  case "${value}" in
    0 | 1) ;;
    *)
      echo "${name} must be 0 or 1" >&2
      exit 2
      ;;
  esac
}

validate_positive_number() {
  local name="$1"
  local value="$2"
  if ! [[ "${value}" =~ ^[0-9]+([.][0-9]+)?$ ]] || ! jq -en --arg value "${value}" \
    '$value | tonumber | . > 0' >/dev/null
  then
    echo "${name} must be a positive number" >&2
    exit 2
  fi
}

validate_positive_integer() {
  local name="$1"
  local value="$2"
  if ! [[ "${value}" =~ ^[1-9][0-9]*$ ]]; then
    echo "${name} must be a positive integer" >&2
    exit 2
  fi
}

for dependency in cargo jq; do
  if ! command -v "${dependency}" >/dev/null 2>&1; then
    echo "${dependency} is required by the camera continuity gate" >&2
    exit 2
  fi
done

validate_toggle "NAU_CAMERA_CONTINUITY_RUN_SIM" "${run_sim}"
validate_toggle "NAU_CAMERA_CONTINUITY_RUN_APP" "${run_app}"
validate_toggle "NAU_CAMERA_CONTINUITY_RUN_FAULT_PROOF" "${run_fault_proof}"
fault_injection_proof_json="$(jq -cn \
  --argjson ran "${run_fault_proof}" \
  '{
    ran: ($ran == 1),
    tests: (if $ran == 1 then [
      "camera_continuity_fault_injection_rejects_nominally_unsampled_one_frame_snap",
      "camera_continuity_rotation_fault_injection_rejects_one_frame_snap"
    ] else [] end)
  }'
)"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_WORLD_COLLISION_MAX_PUSH_M" \
  "${world_collision_max_push_m}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_TERRAIN_RIM_MAX_PUSH_M" \
  "${terrain_rim_max_push_m}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_TERRAIN_BODY_MAX_PUSH_M" \
  "${terrain_body_max_push_m}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_RELATIVE_VELOCITY_MPS" \
  "${max_relative_velocity_mps}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ACCELERATION_MPS2" \
  "${max_relative_acceleration_mps2}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_VELOCITY_DEGREES_PER_SEC" \
  "${max_relative_angular_velocity_degrees_per_sec}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_RELATIVE_ANGULAR_ACCELERATION_DEGREES_PER_SEC2" \
  "${max_relative_angular_acceleration_degrees_per_sec2}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_PLAYER_RESIDUAL_M" \
  "${max_player_residual_m}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_PLAYER_WORLD_CORRECTION_M" \
  "${max_player_world_correction_m}"
validate_positive_number \
  "NAU_CAMERA_CONTINUITY_MAX_PLAYER_COLLISION_CORRECTION_M" \
  "${max_player_collision_correction_m}"
validate_positive_integer \
  "NAU_CAMERA_CONTINUITY_WORLD_CONTACT_SAMPLES" \
  "${world_collision_expected_contact_samples}"
validate_positive_integer \
  "NAU_CAMERA_CONTINUITY_TERRAIN_RIM_CONTACT_SAMPLES" \
  "${terrain_rim_expected_contact_samples}"
validate_positive_integer \
  "NAU_CAMERA_CONTINUITY_TERRAIN_BODY_CONTACT_SAMPLES" \
  "${terrain_body_expected_contact_samples}"

if [[ "${run_app}" == "1" && "$(uname -s)" != "Darwin" ]]; then
  echo "native camera continuity scenarios require macOS; set NAU_CAMERA_CONTINUITY_RUN_APP=0 for simulation-only runs" >&2
  exit 2
fi

mkdir -p "${output_root}"
summary_paths=()

validate_exact_sample_coverage() {
  local summary="$1"
  if ! jq -e '
    (
      .thresholds.min_samples
      // ([.checks[] | select(.name == "sample_count")][0].threshold)
    ) as $expected_samples
    | .metrics.sample_count == $expected_samples
    and .metrics.sample_count > 0
  ' "${summary}" >/dev/null; then
    echo "eval did not record its exact expected sample count: ${summary}" >&2
    jq '{
      scenario,
      frame_count,
      expected_samples: (
        .thresholds.min_samples
        // ([.checks[] | select(.name == "sample_count")][0].threshold)
      ),
      actual_samples: .metrics.sample_count
    }' \
      "${summary}" >&2
    exit 1
  fi
}

validate_gate_checks() {
  local summary="$1"
  if ! jq -e '
    [
      .checks[]
      | select(
          .passed == false
          and .name != "resident_island_visual_count"
        )
    ]
    | length == 0
  ' "${summary}" >/dev/null; then
    echo "camera continuity eval has non-content failures: ${summary}" >&2
    jq '{scenario, failed_checks: [.checks[] | select(.passed == false)]}' "${summary}" >&2
    exit 1
  fi
}

validate_continuity_metrics() {
  local mode="$1"
  local summary="$2"
  if [[ "${mode}" != "app" ]]; then
    return
  fi
  if ! jq -e \
    --argjson max_relative_velocity "${max_relative_velocity_mps}" \
    --argjson max_relative_acceleration "${max_relative_acceleration_mps2}" \
    --argjson max_relative_angular_velocity \
      "${max_relative_angular_velocity_degrees_per_sec}" \
    --argjson max_relative_angular_acceleration \
      "${max_relative_angular_acceleration_degrees_per_sec2}" \
    --argjson max_player_residual "${max_player_residual_m}" \
    --argjson max_player_world_correction "${max_player_world_correction_m}" \
    --argjson max_player_collision_correction "${max_player_collision_correction_m}" \
    '
    .metrics.camera_unclassified_correction_frames == 0
    and .metrics.max_camera_player_relative_step_m
      <= (.thresholds.max_camera_step_distance_m // 1.15)
    and .metrics.max_camera_player_relative_linear_velocity_mps <= $max_relative_velocity
    and .metrics.max_camera_player_relative_linear_acceleration_mps2 <= $max_relative_acceleration
    and .metrics.max_camera_player_relative_angular_velocity_degrees_per_sec
      <= $max_relative_angular_velocity
    and .metrics.max_camera_player_relative_angular_acceleration_degrees_per_sec2
      <= $max_relative_angular_acceleration
    and .metrics.max_player_integration_residual_without_world_collision_m <= $max_player_residual
    and .metrics.max_player_world_correction_m <= $max_player_world_correction
    and .metrics.max_player_collision_correction_m <= $max_player_collision_correction
  ' "${summary}" >/dev/null; then
    echo "camera continuity attribution or relative-motion ceiling failed: ${summary}" >&2
    jq \
      --argjson max_relative_angular_velocity \
        "${max_relative_angular_velocity_degrees_per_sec}" \
      --argjson max_relative_angular_acceleration \
        "${max_relative_angular_acceleration_degrees_per_sec2}" \
      '{
      scenario,
      max_camera_player_relative_step_m: .metrics.max_camera_player_relative_step_m,
      max_allowed_step_m: .thresholds.max_camera_step_distance_m,
      max_camera_player_relative_linear_velocity_mps: .metrics.max_camera_player_relative_linear_velocity_mps,
      max_camera_player_relative_linear_acceleration_mps2: .metrics.max_camera_player_relative_linear_acceleration_mps2,
      max_camera_player_relative_angular_velocity_degrees_per_sec: .metrics.max_camera_player_relative_angular_velocity_degrees_per_sec,
      max_allowed_angular_velocity_degrees_per_sec: $max_relative_angular_velocity,
      max_camera_player_relative_angular_acceleration_degrees_per_sec2: .metrics.max_camera_player_relative_angular_acceleration_degrees_per_sec2,
      max_allowed_angular_acceleration_degrees_per_sec2: $max_relative_angular_acceleration,
      max_player_integration_residual_without_world_collision_m: .metrics.max_player_integration_residual_without_world_collision_m,
      max_player_world_correction_m: .metrics.max_player_world_correction_m,
      max_player_collision_correction_m: .metrics.max_player_collision_correction_m,
      camera_unclassified_correction_frames: .metrics.camera_unclassified_correction_frames
    }' "${summary}" >&2
    exit 1
  fi
}

validate_camera_responsiveness() {
  local mode="$1"
  local scenario="$2"
  local samples="$3"
  if [[ "${mode}" != "app" || "${scenario}" != "camera_mouse_control" ]]; then
    return
  fi

  if ! jq -s -e '
    def magnitude: if . < 0 then -. else . end;
    (map(select(.frame == 11))[0]) as $yaw_before
    | (map(select(.frame == 12))[0]) as $yaw_after
    | (map(select(.frame == 53))[0]) as $pitch_before
    | (map(select(.frame == 54))[0]) as $pitch_after
    | (($yaw_after.camera_yaw_offset_degrees - $yaw_before.camera_yaw_offset_degrees) | magnitude)
      as $yaw_command
    | (($yaw_after.camera_view_yaw_degrees - $yaw_before.camera_view_yaw_degrees) | magnitude)
      as $yaw_response
    | (($pitch_after.camera_pitch_offset_degrees - $pitch_before.camera_pitch_offset_degrees)
        | magnitude) as $pitch_command
    | (($pitch_after.camera_pitch_degrees - $pitch_before.camera_pitch_degrees) | magnitude)
      as $pitch_response
    | ([.[]
        | select(.frame >= 12 and .frame <= 41)
        | ((.camera_view_yaw_degrees - .camera_yaw_offset_degrees) | magnitude)]
        | max) as $max_yaw_tracking_error
    | $yaw_command > 0.5
    and ($yaw_response / $yaw_command) >= 0.80
    and $pitch_command > 0.5
    and ($pitch_response / $pitch_command) >= 0.20
    and $max_yaw_tracking_error <= 1.0
    and $yaw_after.camera_correction_source == "input"
    and $pitch_after.camera_correction_source == "input"
  ' "${samples}" >/dev/null; then
    echo "native camera responsiveness contract failed: ${samples}" >&2
    jq -s '
      (map(select(.frame == 11))[0]) as $yaw_before
      | (map(select(.frame == 12))[0]) as $yaw_after
      | (map(select(.frame == 53))[0]) as $pitch_before
      | (map(select(.frame == 54))[0]) as $pitch_after
      | {
          yaw_before: $yaw_before,
          yaw_after: $yaw_after,
          pitch_before: $pitch_before,
          pitch_after: $pitch_after
        }
    ' "${samples}" >&2
    exit 1
  fi
}

validate_obstruction_stability() {
  local mode="$1"
  local scenario="$2"
  local samples="$3"
  if [[ "${mode}" != "app" || "${scenario}" != "world_collision_contact" ]]; then
    return
  fi

  if ! jq -s -e '
    def magnitude: if . < 0 then -. else . end;
    [range(1; length) as $i
      | select(
          .[$i].camera_obstruction_hits > 0
          and .[$i - 1].camera_obstruction_hits > 0
        )
      | (.[$i].camera_view_yaw_degrees - .[$i - 1].camera_view_yaw_degrees)
      | select(magnitude > 0.2)
      | if . > 0 then 1 else -1 end] as $directions
    | (reduce $directions[] as $direction
        ({previous: 0, reversals: 0};
         if .previous != 0 and .previous != $direction
         then {previous: $direction, reversals: (.reversals + 1)}
         else {previous: $direction, reversals: .reversals}
         end)) as $result
    | $result.reversals <= 3
  ' "${samples}" >/dev/null; then
    echo "native obstruction yaw stability contract failed: ${samples}" >&2
    jq -s '
      def magnitude: if . < 0 then -. else . end;
      [range(1; length) as $i
        | select(
            .[$i].camera_obstruction_hits > 0
            and .[$i - 1].camera_obstruction_hits > 0
          )
        | (.[$i].camera_view_yaw_degrees - .[$i - 1].camera_view_yaw_degrees)
        | select(magnitude > 0.2)
        | if . > 0 then 1 else -1 end] as $directions
      | reduce $directions[] as $direction
          ({previous: 0, reversals: 0};
           if .previous != 0 and .previous != $direction
           then {previous: $direction, reversals: (.reversals + 1)}
           else {previous: $direction, reversals: .reversals}
           end)
    ' "${samples}" >&2
    exit 1
  fi
}

validate_collision_bounds() {
  local scenario="$1"
  local summary="$2"

  case "${scenario}" in
    world_collision_contact)
      jq -e \
        --argjson expected_samples "${world_collision_expected_contact_samples}" \
        --argjson max_push "${world_collision_max_push_m}" \
        '
          .metrics.world_collision_contact_samples == $expected_samples
          and .metrics.max_world_collision_push_m <= $max_push
        ' "${summary}" >/dev/null
      ;;
    terrain_rim_collision_contact)
      jq -e \
        --argjson expected_samples "${terrain_rim_expected_contact_samples}" \
        --argjson max_push "${terrain_rim_max_push_m}" \
        '
          .metrics.terrain_rim_collision_contact_samples == $expected_samples
          and .metrics.terrain_body_collision_contact_samples == 0
          and .metrics.max_terrain_rim_collision_push_m <= $max_push
        ' "${summary}" >/dev/null
      ;;
    terrain_body_collision_contact)
      jq -e \
        --argjson expected_samples "${terrain_body_expected_contact_samples}" \
        --argjson max_push "${terrain_body_max_push_m}" \
        '
          .metrics.terrain_body_collision_contact_samples == $expected_samples
          and .metrics.terrain_rim_collision_contact_samples == 0
          and .metrics.max_terrain_body_collision_push_m <= $max_push
        ' "${summary}" >/dev/null
      ;;
    terrain_edge_walkoff)
      jq -e '
        .metrics.world_collision_contact_samples == 0
        and .metrics.terrain_rim_collision_contact_samples == 0
        and .metrics.terrain_body_collision_contact_samples == 0
        and .metrics.max_world_collision_push_m == 0
        and .metrics.max_terrain_rim_collision_push_m == 0
        and .metrics.max_terrain_body_collision_push_m == 0
      ' "${summary}" >/dev/null
      ;;
    *)
      return
      ;;
  esac || {
    echo "collision contact count or maximum push ceiling failed: ${summary}" >&2
    jq '{
      scenario,
      collision_metrics: {
        world_contact_samples: .metrics.world_collision_contact_samples,
        terrain_rim_contact_samples: .metrics.terrain_rim_collision_contact_samples,
        terrain_body_contact_samples: .metrics.terrain_body_collision_contact_samples,
        max_world_push_m: .metrics.max_world_collision_push_m,
        max_terrain_rim_push_m: .metrics.max_terrain_rim_collision_push_m,
        max_terrain_body_push_m: .metrics.max_terrain_body_collision_push_m
      }
    }' "${summary}" >&2
    exit 1
  }
}

run_eval() {
  local mode="$1"
  local scenario="$2"
  local scenario_output="${output_root}/${mode}/60hz/${scenario}"
  local summary="${scenario_output}/summary.json"
  local eval_status

  set +e
  if [[ "${mode}" == "simulation" ]]; then
    NAU_EVAL_SIM_ONLY=1 \
      NAU_EVAL_ASSET_AUDIT=0 \
      ./tools/eval.sh "${scenario}" "${scenario_output}"
  else
    NAU_EVAL_NO_SCREENSHOT=1 \
      NAU_EVAL_ASSET_AUDIT=0 \
      NAU_EVAL_VISUAL_AUDIT=0 \
      NAU_EVAL_SEMANTIC_SCENE_AUDIT=0 \
      ./tools/eval.sh "${scenario}" "${scenario_output}"
  fi
  eval_status=$?
  set -e

  if [[ ! -s "${summary}" || ! -s "${scenario_output}/samples.ndjson" ]]; then
    echo "camera continuity eval failed before writing complete artifacts: ${scenario}" >&2
    if (( eval_status != 0 )); then
      exit "${eval_status}"
    fi
    exit 1
  fi

  validate_exact_sample_coverage "${summary}"
  validate_gate_checks "${summary}"
  validate_continuity_metrics "${mode}" "${summary}"
  validate_camera_responsiveness "${mode}" "${scenario}" "${scenario_output}/samples.ndjson"
  validate_obstruction_stability "${mode}" "${scenario}" "${scenario_output}/samples.ndjson"
  validate_collision_bounds "${scenario}" "${summary}"
  if (( eval_status != 0 )); then
    echo "accepted ${scenario} with only camera-gate-exempt content failures" >&2
  fi
  summary_paths+=("${summary}")
}

for contract_test in "${contract_tests[@]}"; do
  cargo test --quiet --all-targets "${contract_test}"
done

if [[ "${run_fault_proof}" == "1" ]]; then
  cargo test --quiet --all-targets \
    camera_continuity_fault_injection_rejects_nominally_unsampled_one_frame_snap
  cargo test --quiet --all-targets \
    camera_continuity_rotation_fault_injection_rejects_one_frame_snap
fi

if [[ "${run_sim}" == "1" ]]; then
  for scenario in "${simulation_scenarios[@]}"; do
    run_eval "simulation" "${scenario}"
  done
fi

if [[ "${run_app}" == "1" ]]; then
  for scenario in "${app_scenarios[@]}"; do
    run_eval "app" "${scenario}"
  done
fi

if [[ "${#summary_paths[@]}" -eq 0 ]]; then
  echo "camera continuity gate did not run any scenarios" >&2
  exit 2
fi

requested_refresh_json="$(jq -cn '$ARGS.positional | map(tonumber)' --args "${requested_refresh_rates[@]}")"
requested_hitches_json="$(jq -cn '$ARGS.positional | map(tonumber)' --args "${requested_hitches_ms[@]}")"

jq -s \
  --argjson requested_refresh_hz "${requested_refresh_json}" \
  --argjson requested_hitch_ms "${requested_hitches_json}" \
  --argjson fault_injection_proof "${fault_injection_proof_json}" \
  '
  {
    schema: "nau_camera_continuity_gate.v1",
    passed: true,
    scenario_count: length,
    fault_injection_proof: $fault_injection_proof,
    runtime_timing: {
      requested_refresh_hz: $requested_refresh_hz,
      deterministic_contract_exercised_refresh_hz: $requested_refresh_hz,
      native_app_exercised_refresh_hz: [60],
      requested_hitch_ms: $requested_hitch_ms,
      deterministic_contract_exercised_hitch_ms: $requested_hitch_ms
    },
    scenarios: map({
      scenario,
      eval_passed: .passed,
      ignored_failed_checks: [.checks[] | select(.passed == false) | .name],
      frame_count,
      expected_sample_count: (
        .thresholds.min_samples
        // ([.checks[] | select(.name == "sample_count")][0].threshold)
      ),
      actual_sample_count: .metrics.sample_count,
      max_camera_step_distance_m: .metrics.max_camera_step_distance_m,
      max_camera_player_relative_step_m: .metrics.max_camera_player_relative_step_m,
      max_camera_player_relative_linear_velocity_mps: .metrics.max_camera_player_relative_linear_velocity_mps,
      max_camera_player_relative_linear_acceleration_mps2: .metrics.max_camera_player_relative_linear_acceleration_mps2,
      max_camera_player_relative_angular_velocity_degrees_per_sec: .metrics.max_camera_player_relative_angular_velocity_degrees_per_sec,
      max_camera_player_relative_angular_acceleration_degrees_per_sec2: .metrics.max_camera_player_relative_angular_acceleration_degrees_per_sec2,
      max_player_integration_residual_without_world_collision_m: .metrics.max_player_integration_residual_without_world_collision_m,
      max_player_world_correction_m: .metrics.max_player_world_correction_m,
      max_player_collision_correction_m: .metrics.max_player_collision_correction_m,
      camera_unclassified_correction_frames: .metrics.camera_unclassified_correction_frames,
      max_camera_rotation_delta_degrees: .metrics.max_camera_rotation_delta_degrees,
      max_world_collision_push_m: .metrics.max_world_collision_push_m,
      max_terrain_rim_collision_push_m: .metrics.max_terrain_rim_collision_push_m,
      max_terrain_body_collision_push_m: .metrics.max_terrain_body_collision_push_m
    })
  }
' "${summary_paths[@]}" > "${output_root}/summary.json"

jq '{passed, scenario_count, fault_injection_proof, runtime_timing, scenarios}' \
  "${output_root}/summary.json"
