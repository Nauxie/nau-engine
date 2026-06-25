use bevy::prelude::*;

use super::super::{
    CameraFrame, CameraObstruction, avoid_camera_obstructions, camera_surface_clearance,
    camera_target_angle_degrees, lift_camera_above_floor,
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
    assert!(resolved.adjusted_distance_m > 5.0);
    assert!(resolved.frame.position.z < 4.0);
    assert!(
        camera_target_angle_degrees(
            resolved.frame.position,
            resolved.frame.rotation,
            resolved.frame.look_target,
        ) < 0.001
    );
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
    assert!(resolved.frame.position.z < 3.0);
    assert!(resolved.frame.position.z > 2.3);
}
