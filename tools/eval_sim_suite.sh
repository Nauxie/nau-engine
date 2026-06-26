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
          native_window_created: .metrics.native_window_created
        }
      })
    }
  ' "${summary_paths[@]}" > "${output_root}/summary.json"
  jq '{passed, scenario_count, scenarios}' "${output_root}/summary.json"
else
  printf 'wrote simulation summaries under %s\n' "${output_root}"
fi
