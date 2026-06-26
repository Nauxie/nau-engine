use bevy::prelude::*;

use crate::movement::smoothing_factor;

use super::{
    math::{horizontal_or, yawed_horizontal_direction},
    types::{CameraFrame, CameraOrbit, FollowCamera, FollowCameraState},
};

pub fn step_camera(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    player_forward: Vec3,
    player_velocity: Vec3,
    follow: &FollowCamera,
    dt: f32,
) -> CameraFrame {
    step_camera_with_orbit(
        current_position,
        current_rotation,
        player_position,
        player_forward,
        player_velocity,
        follow,
        CameraOrbit::default(),
        dt,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn step_camera_with_orbit(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    player_forward: Vec3,
    player_velocity: Vec3,
    follow: &FollowCamera,
    orbit: CameraOrbit,
    dt: f32,
) -> CameraFrame {
    let direction = horizontal_follow_direction(player_velocity, player_forward);
    step_camera_with_direction(
        current_position,
        current_rotation,
        player_position,
        direction,
        follow,
        orbit,
        dt,
    )
}

pub fn update_follow_direction_state(
    state: &mut FollowCameraState,
    desired_direction: Vec3,
    follow: &FollowCamera,
    dt: f32,
) -> Vec3 {
    let fallback = if state.initialized {
        state.direction
    } else {
        Vec3::NEG_Z
    };
    let desired_direction = horizontal_or(desired_direction, fallback);
    if !state.initialized {
        state.direction = desired_direction;
        state.initialized = true;
        return state.direction;
    }

    state.direction = horizontal_or(
        state.direction.lerp(
            desired_direction,
            smoothing_factor(follow.direction_smoothing, dt),
        ),
        desired_direction,
    );
    state.direction
}

pub fn movement_stable_follow_direction(
    velocity: Vec3,
    player_forward: Vec3,
    current_follow_direction: Vec3,
) -> Vec3 {
    const MIN_FOLLOW_SPEED_SQUARED: f32 = 1.0;
    const MIN_FORWARD_FOLLOW_DOT: f32 = 0.99;

    let current_direction = horizontal_or(current_follow_direction, Vec3::NEG_Z);
    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length_squared() > MIN_FOLLOW_SPEED_SQUARED {
        let velocity_direction = horizontal_velocity.normalize();
        if velocity_direction.dot(current_direction) >= MIN_FORWARD_FOLLOW_DOT {
            return velocity_direction;
        }
        return current_direction;
    }

    let forward_direction = horizontal_or(player_forward, current_direction);
    if forward_direction.dot(current_direction) >= MIN_FORWARD_FOLLOW_DOT {
        forward_direction
    } else {
        current_direction
    }
}

pub fn movement_input_stable_follow_direction(
    velocity: Vec3,
    player_forward: Vec3,
    current_follow_direction: Vec3,
    movement_axis: Vec2,
) -> Vec3 {
    let current_direction = horizontal_or(current_follow_direction, Vec3::NEG_Z);
    let forward_only = movement_axis.y > 0.0 && movement_axis.x.abs() <= f32::EPSILON;
    if forward_only {
        movement_stable_follow_direction(velocity, player_forward, current_direction)
    } else {
        current_direction
    }
}

pub fn movement_facing_from_follow_direction(
    follow_direction: Vec3,
    orbit: CameraOrbit,
) -> (Vec3, Vec3) {
    let forward =
        yawed_horizontal_direction(horizontal_or(follow_direction, Vec3::NEG_Z), orbit.yaw);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    (forward, horizontal_or(right, Vec3::X))
}

#[allow(clippy::too_many_arguments)]
pub fn step_camera_with_direction(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    follow_direction: Vec3,
    follow: &FollowCamera,
    orbit: CameraOrbit,
    dt: f32,
) -> CameraFrame {
    let direction = horizontal_or(follow_direction, Vec3::NEG_Z);
    let direction = yawed_horizontal_direction(direction, orbit.yaw);
    let look_target =
        player_position + Vec3::Y * follow.look_height + direction * follow.look_ahead;
    let base_horizontal_distance = follow.distance + follow.look_ahead;
    let base_vertical_offset = follow.height - follow.look_height;
    let boom_distance = Vec2::new(base_horizontal_distance, base_vertical_offset)
        .length()
        .max(0.001);
    let base_elevation = base_vertical_offset.atan2(base_horizontal_distance);
    let elevation = base_elevation - orbit.pitch;
    let horizontal_distance = elevation.cos().max(0.0) * boom_distance;
    let vertical_offset = elevation.sin() * boom_distance;
    let mut desired_position =
        look_target - direction * horizontal_distance + Vec3::Y * vertical_offset;
    desired_position.y = desired_position.y.max(follow.min_height);

    let mut position = current_position.lerp(
        desired_position,
        smoothing_factor(follow.position_smoothing, dt),
    );
    let lateral_axis = direction.cross(Vec3::Y).normalize_or_zero();
    if lateral_axis.length_squared() > 0.0001 {
        position += lateral_axis * (desired_position - position).dot(lateral_axis);
    }
    let target_rotation = Transform::from_translation(position)
        .looking_at(look_target, Vec3::Y)
        .rotation;
    let rotation = current_rotation.slerp(
        target_rotation,
        smoothing_factor(follow.rotation_smoothing, dt),
    );

    CameraFrame {
        position,
        rotation,
        look_target,
    }
}

pub fn horizontal_follow_direction(velocity: Vec3, player_forward: Vec3) -> Vec3 {
    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length_squared() > 1.0 {
        horizontal_velocity.normalize()
    } else {
        let horizontal_forward = Vec3::new(player_forward.x, 0.0, player_forward.z);
        if horizontal_forward.length_squared() > 0.0001 {
            horizontal_forward.normalize()
        } else {
            Vec3::Z
        }
    }
}
