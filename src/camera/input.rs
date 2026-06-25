use super::{
    math::wrap_radians,
    types::{CameraControlTuning, CameraInput, CameraOrbit},
};

pub fn apply_camera_input(
    orbit: CameraOrbit,
    input: CameraInput,
    tuning: &CameraControlTuning,
) -> CameraOrbit {
    let yaw = wrap_radians(orbit.yaw - input.mouse_delta.x * tuning.sensitivity_x);
    let y_sign = if tuning.invert_y { 1.0 } else { -1.0 };
    let pitch = (orbit.pitch + input.mouse_delta.y * tuning.sensitivity_y * y_sign)
        .clamp(tuning.min_pitch, tuning.max_pitch);

    CameraOrbit { yaw, pitch }
}
