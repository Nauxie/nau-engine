use super::{super::SimMetrics, SimCheck};
use crate::{
    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES, AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES, AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
    AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS, AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
    AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES, AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS, AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS, AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS, AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS, AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS, MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
};

use crate::metrics::util::{
    avg_body_heading_error_degrees, p95_body_heading_error_degrees, response_latency_secs,
};

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
    ]);
}
