use bevy::prelude::*;

use super::{
    math::{horizontal_or, yawed_horizontal_direction},
    types::CameraOrbit,
};

pub fn camera_distance(camera_position: Vec3, target_position: Vec3) -> f32 {
    let distance = camera_position.distance(target_position);
    if distance.is_finite() { distance } else { 0.0 }
}

pub fn camera_surface_clearance(camera_position: Vec3, floor_y: f32) -> f32 {
    (camera_position.y - floor_y).max(0.0)
}

pub fn camera_target_angle_degrees(
    camera_position: Vec3,
    camera_rotation: Quat,
    target_position: Vec3,
) -> f32 {
    let to_target = target_position - camera_position;
    if to_target.length_squared() <= 0.0001 {
        return 0.0;
    }

    let forward = camera_rotation * Vec3::NEG_Z;
    let dot = forward
        .normalize_or_zero()
        .dot(to_target.normalize())
        .clamp(-1.0, 1.0);
    if dot.is_finite() {
        dot.acos().to_degrees()
    } else {
        0.0
    }
}

pub fn camera_orbit_alignment_degrees(
    camera_position: Vec3,
    look_target: Vec3,
    follow_direction: Vec3,
    orbit: CameraOrbit,
) -> f32 {
    let expected_direction = yawed_horizontal_direction(follow_direction, orbit.yaw);
    let actual_direction = horizontal_or(look_target - camera_position, expected_direction);
    let angle = actual_direction
        .angle_between(expected_direction)
        .to_degrees();

    if angle.is_finite() { angle } else { 0.0 }
}

pub fn camera_view_yaw_degrees(camera_rotation: Quat, reference_direction: Vec3) -> f32 {
    let reference_direction = horizontal_or(reference_direction, Vec3::NEG_Z);
    let view_direction = horizontal_or(camera_rotation * Vec3::NEG_Z, reference_direction);
    let cross_y = reference_direction.cross(view_direction).y;
    let dot = reference_direction.dot(view_direction).clamp(-1.0, 1.0);
    let yaw = cross_y.atan2(dot).to_degrees();

    if yaw.is_finite() { yaw } else { 0.0 }
}

pub fn camera_pitch_degrees(rotation: Quat) -> f32 {
    let forward = rotation * Vec3::NEG_Z;
    let y = forward.y.clamp(-1.0, 1.0);

    if y.is_finite() {
        y.asin().to_degrees()
    } else {
        0.0
    }
}
