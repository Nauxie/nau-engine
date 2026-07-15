use super::{
    math::wrap_radians,
    types::{CameraControlState, CameraControlTuning, CameraInput, CameraOrbit},
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

pub fn step_camera_control(
    state: &mut CameraControlState,
    input: CameraInput,
    tuning: &CameraControlTuning,
    dt: f32,
) {
    state.input_active = false;
    if dt <= 0.0 || !dt.is_finite() {
        return;
    }
    if !input.mouse_delta.is_finite() {
        return;
    }

    state.input_active = input.mouse_delta.length_squared() > 0.0;
    state.orbit = apply_camera_input(state.orbit, input, tuning);
}
