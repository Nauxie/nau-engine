use super::super::{super::EvalAccumulator, derived::SummaryDerivedMetrics};
use crate::eval::{
    scenarios::{
        AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_STRAFE_STABILITY,
        EvalScenario, LONG_GLIDE_VISIBILITY, UPDRAFT_ROUTE,
    },
    summary::EvalCheck,
    thresholds::{EvalThresholds, *},
};

pub(super) fn append_scenario_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    scenario: EvalScenario,
    derived: &SummaryDerivedMetrics,
    thresholds: &EvalThresholds,
) {
    if thresholds.min_lifted_samples > 0 {
        checks.push(EvalCheck::at_least(
            "readable_lift_samples",
            acc.readable_lift_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_most(
            "unreadable_lift_samples",
            acc.unreadable_lift_samples as f32,
            0.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "dynamic_readable_lift_samples",
            acc.dynamic_readable_lift_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_speed",
            acc.max_wind_flow_speed_mps,
            MIN_DYNAMIC_WIND_FLOW_SPEED_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_variation",
            acc.max_wind_flow_variation,
            MIN_DYNAMIC_WIND_FLOW_VARIATION,
            "ratio",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_variation_range",
            acc.max_wind_flow_variation_range,
            MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE,
            "ratio",
        ));
    }
    if thresholds.require_target_landing {
        checks.push(EvalCheck::at_most(
            "final_target_distance",
            derived.final_target_distance_m,
            thresholds.max_final_target_distance_m,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "target_landing_samples",
            acc.target_landing_samples as f32,
            thresholds.min_target_landing_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_anticipation_samples",
            acc.pose_landing_anticipation_samples as f32,
            1.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_recovery_samples",
            acc.pose_landing_recovery_samples as f32,
            1.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_crouch",
            acc.max_pose_landing_crouch_m,
            LANDING_MIN_POSE_CROUCH_M,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_flare",
            acc.max_pose_landing_flare_degrees,
            LANDING_MIN_POSE_FLARE_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "unreadable_key_pose_samples",
            acc.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ));
    }
    if wind_force_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "wind_force_samples",
            acc.wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "active_wind_force_fields",
            acc.max_active_wind_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_delta",
            acc.max_wind_force_delta_mps,
            MIN_WIND_FORCE_DELTA_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_flow_speed",
            acc.max_wind_force_flow_speed_mps,
            MIN_WIND_FORCE_FLOW_SPEED_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_variation",
            acc.max_wind_force_variation,
            MIN_WIND_FORCE_VARIATION,
            "ratio",
        ));
    }
    if crosswind_force_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "crosswind_force_samples",
            acc.crosswind_force_samples as f32,
            MIN_CROSSWIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_fields",
            acc.max_crosswind_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_delta",
            acc.max_crosswind_force_delta_mps,
            MIN_CROSSWIND_FORCE_DELTA_MPS,
            "m/s",
        ));
    }
    if thresholds.min_lifted_samples > 0 {
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_samples",
            acc.updraft_swirl_force_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_fields",
            acc.max_updraft_swirl_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_delta",
            acc.max_updraft_swirl_force_delta_mps,
            MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
            "m/s",
        ));
    }
    if scenario.name == AIR_CONTROL_RESPONSE {
        append_air_control_checks(checks, acc, derived);
    }
    if scenario.name == CAMERA_STRAFE_STABILITY {
        append_camera_strafe_checks(checks, acc);
    }
}

fn wind_force_scenario(scenario: EvalScenario) -> bool {
    matches!(
        scenario.name,
        BASELINE_ROUTE | UPDRAFT_ROUTE | BRANCH_RECOVERY_ROUTE | LONG_GLIDE_VISIBILITY
    )
}

fn crosswind_force_scenario(scenario: EvalScenario) -> bool {
    matches!(scenario.name, BASELINE_ROUTE | BRANCH_RECOVERY_ROUTE)
}

fn append_air_control_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    derived: &SummaryDerivedMetrics,
) {
    checks.extend([
        EvalCheck::at_most(
            "air_control_lateral_response_latency",
            derived.lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_lateral_response",
            acc.max_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_right_lateral_response_latency",
            derived.right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_right_lateral_response",
            acc.max_right_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_left_lateral_response_latency",
            derived.left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_left_lateral_response",
            acc.max_left_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_lateral_response_latency",
            derived.backward_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_lateral_response",
            acc.max_backward_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_right_lateral_response_latency",
            derived.backward_right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_lateral_response",
            acc.max_backward_right_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_rear_response",
            acc.max_backward_right_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_left_lateral_response_latency",
            derived.backward_left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_lateral_response",
            acc.max_backward_left_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_rear_response",
            acc.max_backward_left_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_air_brake_speed_drop",
            acc.max_air_brake_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_air_brake_planar_speed_drop",
            acc.max_air_brake_planar_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_pose_air_brake_samples",
            acc.pose_air_brake_samples as f32,
            4.0,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_diving_samples",
            acc.pose_diving_samples as f32,
            1.0,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_torso_pitch",
            acc.max_pose_torso_pitch_degrees,
            AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_arm_spread",
            acc.max_pose_arm_spread_degrees,
            AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_leg_tuck",
            acc.max_pose_leg_tuck_degrees,
            AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_lateral_lean",
            acc.max_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_right_pose_lateral_lean",
            acc.max_right_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_left_pose_lateral_lean",
            acc.max_left_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_wing_airflow",
            acc.max_pose_wing_airflow_strength,
            AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH,
            "ratio",
        ),
        EvalCheck::at_most(
            "air_control_unreadable_key_pose_samples",
            acc.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_post_brake_forward_alignment",
            acc.max_post_brake_forward_alignment_mps,
            AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_desired_heading_alignment",
            acc.max_desired_heading_alignment_mps,
            AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_avg_body_heading_error",
            derived.avg_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_p95_body_heading_error",
            derived.p95_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_heading_error",
            acc.max_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_yaw_error_step",
            acc.max_body_yaw_error_step_degrees,
            AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_body_yaw_oscillation_count",
            acc.body_yaw_oscillation_count as f32,
            AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
            "sign changes",
        ),
        EvalCheck::at_least(
            "air_control_right_body_bank_response",
            acc.max_right_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_left_body_bank_response",
            acc.max_left_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_roll_step",
            acc.max_body_roll_step_degrees,
            AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_orbit_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_avg_camera_follow_direction_error",
            derived.avg_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_p95_camera_follow_direction_error",
            derived.p95_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
    ]);
}

fn append_camera_strafe_checks(checks: &mut Vec<EvalCheck>, acc: &EvalAccumulator) {
    checks.extend([
        EvalCheck::at_least(
            "camera_strafe_right_lateral_response",
            acc.max_right_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "camera_strafe_left_lateral_response",
            acc.max_left_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "camera_strafe_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "camera_strafe_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
    ]);
}
