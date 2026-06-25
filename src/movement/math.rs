use bevy::prelude::*;

pub fn smoothing_factor(rate: f32, dt: f32) -> f32 {
    (1.0 - (-rate.max(0.0) * dt.max(0.0)).exp()).clamp(0.0, 1.0)
}

pub(super) fn horizontal(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

pub(super) fn horizontal_or(v: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = horizontal(v);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        fallback.normalize()
    }
}

pub(super) fn axis_value(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}
