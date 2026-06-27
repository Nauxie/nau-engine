use super::{super::SimMetrics, SimCheck};
use crate::{
    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES, AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
    AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES, AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
    AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES, AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
    AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
    AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
    AIR_CONTROL_MAX_P95_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS, AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS, AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS, AIR_CONTROL_MIN_DESIRED_TRAVEL_HEADING_SAMPLES,
    AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES, AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
    AIR_CONTROL_MIN_POSE_AIR_TURN_SAMPLES, AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES,
    AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES, AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES,
    AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES, AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH,
    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS, AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
    MIN_POSE_SCARF_LATERAL_SWAY_M, MIN_POSE_SCARF_STREAM_M, MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
};

use crate::metrics::util::{
    avg_body_heading_error_degrees, p95_body_heading_error_degrees, response_latency_secs,
};

const AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES: f32 = 4.0;
const AIR_CONTROL_MIN_GLIDING_DIVE_SAMPLES: f32 = 1.0;

pub(super) fn append_checks(checks: &mut Vec<SimCheck>, metrics: &SimMetrics) {
    let lateral_response_latency_secs = response_latency_secs(
        metrics.first_lateral_input_time_secs,
        metrics.first_lateral_response_time_secs,
    );
    let right_lateral_response_latency_secs = response_latency_secs(
        metrics.first_right_lateral_input_time_secs,
        metrics.first_right_lateral_response_time_secs,
    );
    let left_lateral_response_latency_secs = response_latency_secs(
        metrics.first_left_lateral_input_time_secs,
        metrics.first_left_lateral_response_time_secs,
    );
    let backward_lateral_response_latency_secs = response_latency_secs(
        metrics.first_backward_lateral_input_time_secs,
        metrics.first_backward_lateral_response_time_secs,
    );
    let backward_right_lateral_response_latency_secs = response_latency_secs(
        metrics.first_backward_right_lateral_input_time_secs,
        metrics.first_backward_right_lateral_response_time_secs,
    );
    let backward_left_lateral_response_latency_secs = response_latency_secs(
        metrics.first_backward_left_lateral_input_time_secs,
        metrics.first_backward_left_lateral_response_time_secs,
    );

    checks.extend([
        SimCheck::at_most(
            "air_control_lateral_response_latency",
            lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_lateral_response",
            metrics.max_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_right_lateral_response_latency",
            right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_right_lateral_response",
            metrics.max_right_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_left_lateral_response_latency",
            left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_left_lateral_response",
            metrics.max_left_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_backward_lateral_response_latency",
            backward_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_backward_lateral_response",
            metrics.max_backward_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_backward_right_lateral_response_latency",
            backward_right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_backward_right_lateral_response",
            metrics.max_backward_right_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_backward_right_rear_response",
            metrics.max_backward_right_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_backward_left_lateral_response_latency",
            backward_left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        SimCheck::at_least(
            "air_control_backward_left_lateral_response",
            metrics.max_backward_left_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_backward_left_rear_response",
            metrics.max_backward_left_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_desired_heading_alignment",
            metrics.max_desired_heading_alignment_mps,
            AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
            "mps",
        ),
        SimCheck::at_most(
            "air_control_avg_body_heading_error",
            avg_body_heading_error_degrees(metrics),
            AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_p95_body_heading_error",
            p95_body_heading_error_degrees(metrics),
            AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_body_heading_error",
            metrics.max_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_body_yaw_error_step",
            metrics.max_body_yaw_error_step_degrees,
            AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_body_yaw_oscillation_count",
            metrics.body_yaw_oscillation_count as f32,
            AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
            "oscillations",
        ),
        SimCheck::at_least(
            "air_control_right_body_bank_response",
            metrics.max_right_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_left_body_bank_response",
            metrics.max_left_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_body_roll_step",
            metrics.max_body_roll_step_degrees,
            AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_lateral_body_travel_heading_samples",
            metrics
                .lateral_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_right_body_travel_heading_samples",
            metrics.right_lateral_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_left_body_travel_heading_samples",
            metrics.left_lateral_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_most(
            "air_control_p95_lateral_body_travel_heading_error",
            metrics.p95_lateral_body_travel_heading_error_degrees(),
            AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_lateral_body_travel_heading_error",
            metrics.max_lateral_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_backward_diagonal_body_travel_heading_samples",
            metrics
                .backward_diagonal_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_right_diagonal_body_travel_heading_samples",
            metrics.backward_right_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_left_diagonal_body_travel_heading_samples",
            metrics.backward_left_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_most(
            "air_control_p95_backward_diagonal_body_travel_heading_error",
            metrics.p95_backward_diagonal_body_travel_heading_error_degrees(),
            AIR_CONTROL_MAX_P95_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_backward_diagonal_body_travel_heading_error",
            metrics.max_backward_diagonal_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_desired_travel_heading_samples",
            metrics.desired_travel_heading_error_values_degrees.len() as f32,
            AIR_CONTROL_MIN_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_right_desired_travel_heading_samples",
            metrics.right_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_left_desired_travel_heading_samples",
            metrics.left_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_right_desired_travel_heading_samples",
            metrics.backward_right_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_left_desired_travel_heading_samples",
            metrics.backward_left_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_most(
            "air_control_p95_desired_travel_heading_error",
            metrics.p95_desired_travel_heading_error_degrees(),
            AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_max_desired_travel_heading_error",
            metrics.max_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_camera_orbit_yaw_offset",
            metrics.max_abs_camera_yaw_offset_degrees,
            AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_camera_rotation_delta",
            metrics.max_camera_rotation_delta_degrees,
            AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_camera_view_yaw_drift",
            metrics.max_camera_view_yaw_drift_degrees,
            AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_camera_world_yaw_drift",
            metrics.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_air_brake_speed_drop",
            metrics.max_air_brake_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_air_brake_planar_speed_drop",
            metrics.max_air_brake_planar_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_post_brake_forward_alignment",
            metrics.max_post_brake_forward_alignment_mps,
            AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "air_control_pose_torso_pitch",
            metrics.max_pose_torso_pitch_degrees,
            AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_pose_arm_spread",
            metrics.max_pose_arm_spread_degrees,
            AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_pose_leg_tuck",
            metrics.max_pose_leg_tuck_degrees,
            AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_pose_lateral_lean",
            metrics.max_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_right_pose_lateral_lean",
            metrics.max_right_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_left_pose_lateral_lean",
            metrics.max_left_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "air_control_pose_wing_airflow",
            metrics.max_pose_wing_airflow_strength,
            AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH,
            "ratio",
        ),
        SimCheck::at_least(
            "air_control_pose_scarf_stream",
            metrics.max_pose_scarf_stream_m,
            MIN_POSE_SCARF_STREAM_M,
            "m",
        ),
        SimCheck::at_least(
            "air_control_pose_scarf_lateral_sway",
            metrics.max_pose_scarf_lateral_sway_m,
            MIN_POSE_SCARF_LATERAL_SWAY_M,
            "m",
        ),
        SimCheck::at_least(
            "air_control_pose_scarf_tail_flex",
            metrics.max_pose_scarf_tail_flex_degrees,
            MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "air_control_unreadable_key_pose_samples",
            metrics.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_pose_air_turn_samples",
            metrics.pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_right_pose_air_turn_samples",
            metrics.right_pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_left_pose_air_turn_samples",
            metrics.left_pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_pose_air_brake_samples",
            metrics.pose_air_brake_samples as f32,
            4.0,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_right_pose_air_brake_samples",
            metrics.right_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_left_pose_air_brake_samples",
            metrics.left_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_right_pose_air_brake_samples",
            metrics.backward_right_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_backward_left_pose_air_brake_samples",
            metrics.backward_left_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_COVERAGE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_pose_diving_samples",
            metrics.pose_diving_samples as f32,
            1.0,
            "samples",
        ),
        SimCheck::at_least(
            "air_control_gliding_dive_samples",
            metrics.gliding_dive_samples as f32,
            AIR_CONTROL_MIN_GLIDING_DIVE_SAMPLES,
            "samples",
        ),
    ]);
}
