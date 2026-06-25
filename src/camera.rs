mod follow;
mod input;
mod math;
mod metrics;
mod obstruction;
mod types;

#[cfg(test)]
mod tests;

pub use follow::{
    horizontal_follow_direction, movement_input_stable_follow_direction,
    movement_stable_follow_direction, step_camera, step_camera_with_direction,
    step_camera_with_orbit, update_follow_direction_state,
};
pub use input::apply_camera_input;
pub use metrics::{
    camera_distance, camera_orbit_alignment_degrees, camera_pitch_degrees,
    camera_surface_clearance, camera_target_angle_degrees, camera_view_yaw_degrees,
};
pub use obstruction::{avoid_camera_obstructions, lift_camera_above_floor};
pub use types::{
    CameraControlState, CameraControlTuning, CameraFrame, CameraInput, CameraObstruction,
    CameraObstructionResolution, CameraOrbit, FollowCamera, FollowCameraState,
};
