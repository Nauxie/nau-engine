use bevy::prelude::*;

use super::super::{camera_distance, camera_pitch_degrees, camera_view_yaw_degrees};

#[test]
fn camera_pitch_is_negative_when_looking_downward() {
    let rotation = Transform::from_xyz(0.0, 6.0, -12.0)
        .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y)
        .rotation;

    assert!(camera_pitch_degrees(rotation) < -15.0);
}

#[test]
fn camera_pitch_is_level_for_horizontal_forward() {
    assert!(camera_pitch_degrees(Quat::IDENTITY).abs() < 0.001);
}

#[test]
fn camera_view_yaw_tracks_horizontal_rotation() {
    let yaw_radians = 0.35_f32;
    let yaw_degrees = camera_view_yaw_degrees(Quat::from_rotation_y(yaw_radians), Vec3::NEG_Z);

    assert!((yaw_degrees.abs() - yaw_radians.to_degrees()).abs() < 0.001);
}

#[test]
fn camera_distance_matches_vector_length() {
    assert_eq!(camera_distance(Vec3::new(0.0, 3.0, 4.0), Vec3::ZERO), 5.0);
}
