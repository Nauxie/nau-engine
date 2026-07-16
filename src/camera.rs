mod follow;
mod input;
mod math;
mod metrics;
mod obstruction;
mod types;

#[cfg(test)]
mod tests;

pub use follow::{
    clamp_camera_player_distance, horizontal_follow_direction,
    movement_facing_from_follow_direction, movement_input_stable_follow_direction,
    movement_stable_follow_direction, step_camera, step_camera_with_direction,
    step_camera_with_direction_and_input, step_camera_with_orbit, update_follow_direction_state,
};
pub use input::{apply_camera_input, step_camera_control};
pub use metrics::{
    camera_distance, camera_orbit_alignment_degrees, camera_pitch_degrees,
    camera_surface_clearance, camera_target_angle_degrees, camera_view_yaw_degrees,
};
pub use obstruction::{
    CAMERA_MAX_OBSTRUCTION_HANDOFF_OFFSET_SPEED_MPS, CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS,
    CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND, CAMERA_MAX_PLAYER_DISTANCE_M,
    CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS, CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND,
    CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M, CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M,
    CAMERA_OBSTRUCTION_RELEASE_HANDOFF_SECS, CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M,
    CameraObstructionHandoffState, CameraObstructionSmoothingState, CameraObstructionStep,
    avoid_camera_obstructions, avoid_camera_obstructions_with_preferred_offset,
    camera_obstruction_is_active, clamp_camera_offset_step, clamp_camera_rotation_step,
    clamp_camera_step, enforce_camera_continuity, lift_camera_above_floor,
    resolve_camera_obstruction_handoff, revalidate_camera_obstruction, smooth_camera_obstruction,
};
pub use types::{
    CameraControlState, CameraControlTuning, CameraFrame, CameraInput, CameraObstruction,
    CameraObstructionResolution, CameraOrbit, FollowCamera, FollowCameraState,
};
