use bevy::prelude::*;

use super::super::{
    CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M, CameraFrame, CameraObstruction,
    avoid_camera_obstructions, camera_surface_clearance, camera_target_angle_degrees,
    clamp_camera_step, lift_camera_above_floor,
};

#[test]
fn camera_surface_clearance_lifts_clipping_frame() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 3.0, 0.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 4.0, -4.0),
    };

    let lifted = lift_camera_above_floor(frame, 2.5, 2.0);

    assert_eq!(lifted.position.y, 4.5);
    assert_eq!(camera_surface_clearance(lifted.position, 2.5), 2.0);
}

#[test]
fn camera_obstruction_moves_camera_in_front_of_blocker() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 2.0, 10.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 2.0, 0.0),
    };

    let resolved = avoid_camera_obstructions(
        frame,
        [CameraObstruction::new(
            Vec3::new(0.0, 2.0, 5.0),
            Vec3::new(1.0, 1.0, 1.0),
        )],
        0.5,
    );

    assert_eq!(resolved.hit_count, 1);
    assert!(resolved.adjusted_distance_m > 3.0);
    assert!(resolved.frame.position.distance(resolved.frame.look_target) > 10.0);
    assert!(resolved.frame.position.y > frame.position.y || resolved.frame.position.x.abs() > 3.0);
    assert_eq!(resolved.frame.position.z, frame.position.z);
    assert!(
        camera_target_angle_degrees(
            resolved.frame.position,
            resolved.frame.rotation,
            resolved.frame.look_target,
        ) < 0.001
    );
}

#[test]
fn camera_obstruction_clears_broad_blocker_when_readable_fallback_is_blocked() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 2.0, 12.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 2.0, 0.0),
    };

    let resolved = avoid_camera_obstructions(
        frame,
        [CameraObstruction::new(
            Vec3::new(0.0, 2.0, 4.0),
            Vec3::new(20.0, 20.0, 1.0),
        )],
        0.25,
    );

    assert_eq!(resolved.hit_count, 1);
    assert!(
        resolved.frame.position.distance(resolved.frame.look_target)
            < CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
    );
    assert!(resolved.frame.position.z < 2.75);
    assert!(
        camera_target_angle_degrees(
            resolved.frame.position,
            resolved.frame.rotation,
            resolved.frame.look_target,
        ) < 0.001
    );
}

#[test]
fn camera_obstruction_treats_near_narrow_prop_as_transparent_when_fallback_is_blocked() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 2.0, 12.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 2.0, 0.0),
    };

    let resolved = avoid_camera_obstructions(
        frame,
        [
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.0, 4.0), Vec3::new(0.6, 2.0, 0.6)),
            CameraObstruction::soft_local_prop(Vec3::new(1.6, 2.0, 4.0), Vec3::new(0.35, 2.0, 0.6)),
            CameraObstruction::soft_local_prop(
                Vec3::new(-1.6, 2.0, 4.0),
                Vec3::new(0.35, 2.0, 0.6),
            ),
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.8, 4.0), Vec3::new(0.6, 0.25, 0.6)),
            CameraObstruction::soft_local_prop(
                Vec3::new(1.6, 2.5, 4.0),
                Vec3::new(0.35, 0.35, 0.6),
            ),
            CameraObstruction::soft_local_prop(
                Vec3::new(-1.6, 2.5, 4.0),
                Vec3::new(0.35, 0.35, 0.6),
            ),
        ],
        0.45,
    );

    assert_eq!(resolved.hit_count, 1);
    assert_eq!(resolved.adjusted_distance_m, 0.0);
    assert_eq!(resolved.frame.position, frame.position);
}

#[test]
fn camera_obstruction_keeps_clear_view_when_blocker_is_off_segment() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 2.0, 10.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 2.0, 0.0),
    };

    let resolved = avoid_camera_obstructions(
        frame,
        [CameraObstruction::new(
            Vec3::new(5.0, 2.0, 5.0),
            Vec3::new(1.0, 1.0, 1.0),
        )],
        0.5,
    );

    assert_eq!(resolved.hit_count, 0);
    assert_eq!(resolved.adjusted_distance_m, 0.0);
    assert_eq!(resolved.frame.position, frame.position);
}

#[test]
fn camera_obstruction_uses_nearest_blocker() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 2.0, 12.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 2.0, 0.0),
    };

    let resolved = avoid_camera_obstructions(
        frame,
        [
            CameraObstruction::new(Vec3::new(0.0, 2.0, 8.0), Vec3::splat(1.0)),
            CameraObstruction::new(Vec3::new(0.0, 2.0, 4.0), Vec3::splat(1.0)),
        ],
        0.25,
    );

    assert_eq!(resolved.hit_count, 2);
    assert!(
        resolved.frame.position.distance(resolved.frame.look_target)
            >= CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
    );
}

#[test]
fn camera_step_clamp_limits_large_obstruction_snaps() {
    let frame = CameraFrame {
        position: Vec3::new(0.0, 4.0, -16.0),
        rotation: Quat::IDENTITY,
        look_target: Vec3::new(0.0, 3.0, 0.0),
    };
    let previous_position = Vec3::new(0.0, 4.0, 2.0);

    let clamped = clamp_camera_step(frame, previous_position, 9.5);

    assert!(previous_position.distance(clamped.position) <= 9.5001);
    assert!(clamped.position.z > frame.position.z);
    assert!(
        camera_target_angle_degrees(clamped.position, clamped.rotation, clamped.look_target)
            < 0.001
    );
}
