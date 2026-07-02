use bevy::prelude::*;

use super::super::math::yawed_horizontal_direction;
use super::super::{
    CameraControlTuning, CameraOrbit, FollowCamera, FollowCameraState,
    camera_orbit_alignment_degrees, camera_pitch_degrees, camera_target_angle_degrees,
    camera_view_yaw_degrees, horizontal_follow_direction, movement_facing_from_follow_direction,
    movement_input_stable_follow_direction, movement_stable_follow_direction, step_camera,
    step_camera_with_direction, step_camera_with_orbit, update_follow_direction_state,
};

#[test]
fn vertical_launch_velocity_does_not_pull_camera_under_player() {
    let follow = FollowCamera::default();
    let frame = step_camera(
        Vec3::new(0.0, 6.0, -12.0),
        Quat::IDENTITY,
        Vec3::new(0.0, 20.0, 0.0),
        Vec3::Z,
        Vec3::new(0.0, 40.0, 0.0),
        &follow,
        1.0,
    );

    assert!(frame.position.y > 20.0);
    assert!(frame.position.z < 0.0);
}

#[test]
fn horizontal_velocity_controls_follow_direction() {
    let direction = horizontal_follow_direction(Vec3::new(10.0, 40.0, 0.0), Vec3::Z);
    assert!(direction.x > 0.99);
    assert!(direction.y.abs() < 0.001);
}

#[test]
fn lateral_velocity_does_not_drag_stable_follow_direction() {
    let direction =
        movement_stable_follow_direction(Vec3::new(20.0, 0.0, 0.0), Vec3::X, Vec3::NEG_Z);

    assert!(direction.distance(Vec3::NEG_Z) < 0.001);
}

#[test]
fn mostly_forward_velocity_can_adjust_stable_follow_direction() {
    let velocity = Vec3::new(1.0, 0.0, -20.0);
    let direction = movement_stable_follow_direction(velocity, Vec3::NEG_Z, Vec3::NEG_Z);

    assert!(direction.distance(velocity.normalize()) < 0.001);
}

#[test]
fn lateral_or_released_input_does_not_drag_camera_follow_direction() {
    let velocity = Vec3::new(1.0, 0.0, -20.0);
    let lateral_direction =
        movement_input_stable_follow_direction(velocity, Vec3::X, Vec3::NEG_Z, Vec2::new(1.0, 0.0));
    let backward_direction = movement_input_stable_follow_direction(
        Vec3::new(0.0, 0.0, 20.0),
        Vec3::X,
        Vec3::NEG_Z,
        Vec2::NEG_Y,
    );
    let backward_diagonal_direction = movement_input_stable_follow_direction(
        Vec3::new(12.0, 0.0, 12.0),
        Vec3::X,
        Vec3::NEG_Z,
        Vec2::new(1.0, -1.0),
    );
    let released_direction =
        movement_input_stable_follow_direction(velocity, Vec3::X, Vec3::NEG_Z, Vec2::ZERO);
    let forward_direction =
        movement_input_stable_follow_direction(velocity, Vec3::NEG_Z, Vec3::NEG_Z, Vec2::Y);

    assert!(lateral_direction.distance(Vec3::NEG_Z) < 0.001);
    assert!(backward_direction.distance(Vec3::NEG_Z) < 0.001);
    assert!(backward_diagonal_direction.distance(Vec3::NEG_Z) < 0.001);
    assert!(released_direction.distance(Vec3::NEG_Z) < 0.001);
    assert!(forward_direction.distance(velocity.normalize()) < 0.001);
}

#[test]
fn movement_facing_uses_stable_follow_direction_plus_explicit_orbit() {
    let (forward, right) =
        movement_facing_from_follow_direction(Vec3::NEG_Z, CameraOrbit::default());

    assert!(forward.distance(Vec3::NEG_Z) < 0.001);
    assert!(right.distance(Vec3::X) < 0.001);

    let (yawed_forward, yawed_right) = movement_facing_from_follow_direction(
        Vec3::NEG_Z,
        CameraOrbit {
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
        },
    );

    assert!(yawed_forward.distance(Vec3::NEG_X) < 0.001);
    assert!(yawed_right.distance(Vec3::NEG_Z) < 0.001);
}

#[test]
fn lateral_camera_tracking_translates_without_yawing_view() {
    let follow = FollowCamera::default();
    let player_position = Vec3::new(8.0, 30.0, 0.0);
    let frame = step_camera_with_direction(
        Vec3::new(0.0, 34.0, 12.0),
        Quat::IDENTITY,
        player_position,
        Vec3::NEG_Z,
        &follow,
        CameraOrbit::default(),
        1.0 / 60.0,
    );

    assert!(
        (frame.position.x - player_position.x).abs() < 0.001,
        "camera should translate laterally with the player instead of lagging into a yaw"
    );
    assert!(camera_view_yaw_degrees(frame.rotation, Vec3::NEG_Z).abs() < 0.001);
}

#[test]
fn orbit_pitch_moves_view_pitch_in_expected_direction() {
    let follow = FollowCamera::default();
    let low = step_camera_with_orbit(
        Vec3::new(0.0, 6.0, -12.0),
        Quat::IDENTITY,
        Vec3::ZERO,
        Vec3::NEG_Z,
        Vec3::NEG_Z * 10.0,
        &follow,
        CameraOrbit {
            pitch: -0.25,
            yaw: 0.0,
        },
        1.0,
    );
    let high = step_camera_with_orbit(
        Vec3::new(0.0, 6.0, -12.0),
        Quat::IDENTITY,
        Vec3::ZERO,
        Vec3::NEG_Z,
        Vec3::NEG_Z * 10.0,
        &follow,
        CameraOrbit {
            pitch: 0.25,
            yaw: 0.0,
        },
        1.0,
    );

    assert!(camera_pitch_degrees(high.rotation) > camera_pitch_degrees(low.rotation));
}

#[test]
fn orbit_pitch_keeps_player_focus_centered() {
    let follow = FollowCamera::default();
    let player_position = Vec3::ZERO;
    let frame = step_camera_with_orbit(
        Vec3::new(0.0, follow.height, follow.distance),
        Quat::IDENTITY,
        player_position,
        Vec3::NEG_Z,
        Vec3::ZERO,
        &follow,
        CameraOrbit {
            pitch: CameraControlTuning::default().max_pitch,
            yaw: 0.0,
        },
        1.0,
    );
    let player_focus = player_position + Vec3::Y * follow.look_height;

    assert!(camera_target_angle_degrees(frame.position, frame.rotation, player_focus) < 3.0);
}

#[test]
fn follow_direction_smoothing_limits_turnaround_snap() {
    let follow = FollowCamera::default();
    let mut state = FollowCameraState {
        direction: Vec3::Z,
        initialized: true,
    };
    let follow_direction =
        update_follow_direction_state(&mut state, Vec3::NEG_Z, &follow, 1.0 / 60.0);
    let frame = step_camera_with_direction(
        Vec3::new(0.0, 6.0, 12.0),
        Quat::IDENTITY,
        Vec3::ZERO,
        follow_direction,
        &follow,
        CameraOrbit::default(),
        1.0 / 60.0,
    );

    assert!(follow_direction.z > 0.9);
    assert!(
        frame.position.z > 0.0,
        "camera should not instantly orbit across the player on a velocity flip"
    );
}

#[test]
fn persistent_yaw_offset_does_not_compound_into_spin() {
    let follow = FollowCamera::default();
    let orbit = CameraOrbit {
        yaw: 0.2,
        pitch: 0.0,
    };
    let player_position = Vec3::ZERO;
    let player_forward = Vec3::NEG_Z;
    let mut camera_position = Vec3::new(0.0, follow.height, follow.distance);
    let mut camera_rotation = Transform::from_translation(camera_position)
        .looking_at(player_position + Vec3::Y * follow.look_height, Vec3::Y)
        .rotation;
    let expected_direction = yawed_horizontal_direction(
        horizontal_follow_direction(Vec3::ZERO, player_forward),
        orbit.yaw,
    );

    for _ in 0..240 {
        let frame = step_camera_with_orbit(
            camera_position,
            camera_rotation,
            player_position,
            player_forward,
            Vec3::ZERO,
            &follow,
            orbit,
            1.0 / 60.0,
        );
        camera_position = frame.position;
        camera_rotation = frame.rotation;
    }

    let drift_degrees = camera_orbit_alignment_degrees(
        camera_position,
        player_position + Vec3::Y * follow.look_height,
        expected_direction,
        CameraOrbit::default(),
    );

    assert!(
        drift_degrees < 3.0,
        "persistent yaw drifted by {drift_degrees} degrees"
    );
}
