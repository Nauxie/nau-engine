use bevy::prelude::*;

use super::super::{CameraControlTuning, CameraInput, CameraOrbit, apply_camera_input};

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
