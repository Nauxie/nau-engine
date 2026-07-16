use bevy::prelude::*;

use super::super::{
    CameraControlState, CameraControlTuning, CameraInput, CameraOrbit, apply_camera_input,
    step_camera_control,
};

#[test]
fn mouse_x_changes_camera_yaw_without_touching_pitch() {
    let tuning = CameraControlTuning::default();
    let orbit = apply_camera_input(
        CameraOrbit::default(),
        CameraInput {
            mouse_delta: Vec2::new(20.0, 0.0),
        },
        &tuning,
    );

    assert!(orbit.yaw < -0.08);
    assert_eq!(orbit.pitch, 0.0);
}

#[test]
fn mouse_y_maps_to_pitch_and_clamps() {
    let tuning = CameraControlTuning::default();
    let up = apply_camera_input(
        CameraOrbit::default(),
        CameraInput {
            mouse_delta: Vec2::new(0.0, -20.0),
        },
        &tuning,
    );
    let clamped = apply_camera_input(
        CameraOrbit::default(),
        CameraInput {
            mouse_delta: Vec2::new(0.0, -1000.0),
        },
        &tuning,
    );

    assert!(up.pitch > 0.07);
    assert_eq!(clamped.pitch, tuning.max_pitch);
}

#[test]
fn large_mouse_burst_applies_full_orbit_in_same_frame() {
    let tuning = CameraControlTuning::default();
    let input = CameraInput {
        mouse_delta: Vec2::new(-120.0, -60.0),
    };
    let expected = apply_camera_input(CameraOrbit::default(), input, &tuning);
    let mut state = CameraControlState::default();

    step_camera_control(&mut state, input, &tuning, 1.0 / 60.0);

    assert!(
        wrap_angle(state.orbit.yaw - expected.yaw).abs() <= 0.0001,
        "the complete mouse yaw must reach the active orbit in the input frame"
    );
    assert!(
        (state.orbit.pitch - expected.pitch).abs() <= 0.0001,
        "the complete mouse pitch must reach the active orbit in the input frame"
    );
}

#[test]
fn mouse_orbit_applies_same_frame_across_frame_rates() {
    let tuning = CameraControlTuning::default();
    let input = CameraInput {
        mouse_delta: Vec2::new(-240.0, -80.0),
    };
    let expected = apply_camera_input(CameraOrbit::default(), input, &tuning);

    for frame_rate in [30.0, 60.0, 120.0, 144.0] {
        let mut state = CameraControlState::default();
        step_camera_control(&mut state, input, &tuning, 1.0 / frame_rate);

        assert!(
            wrap_angle(state.orbit.yaw - expected.yaw)
                .abs()
                .to_degrees()
                <= 0.01,
            "{frame_rate} Hz deferred part of the requested yaw"
        );
        assert!(
            (state.orbit.pitch - expected.pitch).abs().to_degrees() <= 0.01,
            "{frame_rate} Hz deferred part of the requested pitch"
        );
    }
}

fn wrap_angle(angle: f32) -> f32 {
    angle.sin().atan2(angle.cos())
}
