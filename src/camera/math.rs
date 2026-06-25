use bevy::prelude::*;
use std::f32::consts::PI;

pub(super) fn yawed_horizontal_direction(direction: Vec3, yaw: f32) -> Vec3 {
    let rotated = Quat::from_rotation_y(yaw) * direction;
    horizontal_or(rotated, direction)
}

pub(super) fn horizontal_or(value: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = Vec3::new(value.x, 0.0, value.z);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        let fallback = Vec3::new(fallback.x, 0.0, fallback.z);
        if fallback.length_squared() > 0.0001 {
            fallback.normalize()
        } else {
            Vec3::NEG_Z
        }
    }
}

pub(super) fn wrap_radians(value: f32) -> f32 {
    (value + PI).rem_euclid(PI * 2.0) - PI
}
