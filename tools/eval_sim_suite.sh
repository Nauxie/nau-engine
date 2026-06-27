#!/usr/bin/env bash
set -euo pipefail

output_root="${1:-target/eval/sim_suite}"
asset_audit_requested="${NAU_EVAL_ASSET_AUDIT:-1}"
scenarios=(
  baseline_route
  island_launch_to_landing
  ground_taxi_control
  updraft_route
  branch_recovery_route
  camera_mouse_control
  camera_yaw_stability
  camera_turn_stability
  camera_strafe_stability
  air_control_response
  pose_state_coverage
  long_glide_visibility
)

mkdir -p "${output_root}"

if [[ "${asset_audit_requested}" != "0" ]]; then
  cargo run --quiet --bin asset_fixture_audit > "${output_root}/asset_fixture_audit.json"
fi

summary_paths=()
for scenario in "${scenarios[@]}"; do
  scenario_output="${output_root}/${scenario}"
  NAU_EVAL_SIM_ONLY=1 NAU_EVAL_ASSET_AUDIT=0 ./tools/eval.sh "${scenario}" "${scenario_output}"
  summary_paths+=("${scenario_output}/summary.json")
done

if command -v jq >/dev/null 2>&1; then
  jq -s '
    {
      schema: "nau_sim_suite.v1",
      passed: all(.[]; .passed == true),
      scenario_count: length,
      native_window_created_any: any(.[]; .metrics.native_window_created == true),
      scenarios: map({
        scenario,
        passed,
        target_island,
        metrics: {
          sample_count: .metrics.sample_count,
          horizontal_distance_m: .metrics.horizontal_distance_m,
          max_altitude_m: .metrics.max_altitude_m,
          lifted_samples: .metrics.lifted_samples,
          target_landing_samples: .metrics.target_landing_samples,
          max_collected_power_up_count: .metrics.max_collected_power_up_count,
          power_up_effect_samples: .metrics.power_up_effect_samples,
          pose_grounded_walk_samples: .metrics.pose_grounded_walk_samples,
          pose_grounded_run_samples: .metrics.pose_grounded_run_samples,
          pose_launching_samples: .metrics.pose_launching_samples,
          pose_falling_samples: .metrics.pose_falling_samples,
          pose_gliding_samples: .metrics.pose_gliding_samples,
          pose_air_turn_samples: .metrics.pose_air_turn_samples,
          pose_air_brake_samples: .metrics.pose_air_brake_samples,
          pose_diving_samples: .metrics.pose_diving_samples,
          gliding_dive_samples: .metrics.gliding_dive_samples,
          pose_landing_anticipation_samples: .metrics.pose_landing_anticipation_samples,
          pose_landing_recovery_samples: .metrics.pose_landing_recovery_samples,
          max_pose_landing_crouch_m: .metrics.max_pose_landing_crouch_m,
          max_pose_landing_foot_forward_m: .metrics.max_pose_landing_foot_forward_m,
          max_pose_landing_flare_degrees: .metrics.max_pose_landing_flare_degrees,
          max_pose_landing_recovery_flip_degrees: .metrics.max_pose_landing_recovery_flip_degrees,
          unreadable_key_pose_samples: .metrics.unreadable_key_pose_samples,
          wind_force_samples: .metrics.wind_force_samples,
          meaningful_wind_force_samples: .metrics.meaningful_wind_force_samples,
          aligned_wind_force_samples: .metrics.aligned_wind_force_samples,
          max_active_wind_force_fields: .metrics.max_active_wind_force_fields,
          crosswind_force_samples: .metrics.crosswind_force_samples,
          aligned_crosswind_force_samples: .metrics.aligned_crosswind_force_samples,
          max_crosswind_force_fields: .metrics.max_crosswind_force_fields,
          updraft_swirl_force_samples: .metrics.updraft_swirl_force_samples,
          aligned_updraft_swirl_force_samples: .metrics.aligned_updraft_swirl_force_samples,
          max_updraft_swirl_force_fields: .metrics.max_updraft_swirl_force_fields,
          layered_wind_force_samples: .metrics.layered_wind_force_samples,
          aligned_layered_wind_force_samples: .metrics.aligned_layered_wind_force_samples,
          crosswind_updraft_overlap_samples: .metrics.crosswind_updraft_overlap_samples,
          aligned_crosswind_updraft_overlap_samples: .metrics.aligned_crosswind_updraft_overlap_samples,
          max_layered_wind_force_fields: .metrics.max_layered_wind_force_fields,
          max_wind_force_delta_mps: .metrics.max_wind_force_delta_mps,
          max_crosswind_force_delta_mps: .metrics.max_crosswind_force_delta_mps,
          max_updraft_swirl_force_delta_mps: .metrics.max_updraft_swirl_force_delta_mps,
          max_layered_wind_force_delta_mps: .metrics.max_layered_wind_force_delta_mps,
          max_wind_force_flow_speed_mps: .metrics.max_wind_force_flow_speed_mps,
          max_wind_force_variation: .metrics.max_wind_force_variation,
          max_wind_force_flow_alignment: .metrics.max_wind_force_flow_alignment,
          max_crosswind_force_flow_alignment: .metrics.max_crosswind_force_flow_alignment,
          max_updraft_swirl_force_flow_alignment: .metrics.max_updraft_swirl_force_flow_alignment,
          max_layered_wind_force_flow_alignment: .metrics.max_layered_wind_force_flow_alignment,
          max_wind_force_aligned_delta_mps: .metrics.max_wind_force_aligned_delta_mps,
          max_crosswind_force_aligned_delta_mps: .metrics.max_crosswind_force_aligned_delta_mps,
          max_updraft_swirl_force_aligned_delta_mps: .metrics.max_updraft_swirl_force_aligned_delta_mps,
          max_layered_wind_force_aligned_delta_mps: .metrics.max_layered_wind_force_aligned_delta_mps,
          wind_load_response_samples: .metrics.wind_load_response_samples,
          max_wind_load_lateral_load: .metrics.max_wind_load_lateral_load,
          max_wind_load_pose_lean_degrees: .metrics.max_wind_load_pose_lean_degrees,
          max_wind_load_glider_response_degrees: .metrics.max_wind_load_glider_response_degrees,
          native_window_created: .metrics.native_window_created
        }
      })
    }
  ' "${summary_paths[@]}" > "${output_root}/summary.json"
  jq '{passed, scenario_count, native_window_created_any, scenarios}' "${output_root}/summary.json"
else
  printf 'wrote simulation summaries under %s\n' "${output_root}"
fi
