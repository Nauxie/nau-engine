use super::{super::SimMetrics, SimCheck};
use crate::{
    CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES, CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
};

pub(super) fn append_checks(checks: &mut Vec<SimCheck>, metrics: &SimMetrics) {
    checks.extend([
        SimCheck::at_most(
            "camera_strafe_view_yaw_drift",
            metrics.max_camera_view_yaw_drift_degrees,
            CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "camera_strafe_world_yaw_drift",
            metrics.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "camera_strafe_right_lateral_response",
            metrics.max_right_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "mps",
        ),
        SimCheck::at_least(
            "camera_strafe_left_lateral_response",
            metrics.max_left_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "mps",
        ),
    ]);
}
