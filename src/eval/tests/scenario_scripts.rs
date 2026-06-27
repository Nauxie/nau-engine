use super::*;

#[test]
fn baseline_route_has_scripted_launch_and_glide() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 2).launch);
    assert!(scripted_input(scenario, 60).glide);
    assert!(scripted_input(scenario, 390).glide);
    assert!(scripted_input(scenario, 390).dive);
}

#[test]
fn ground_taxi_script_exercises_wasd_without_launching() {
    let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");

    assert!(scripted_input(scenario, 20).forward);
    assert!(scripted_input(scenario, 60).right);
    assert!(scripted_input(scenario, 135).backward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 60).glide);
}

#[test]
fn world_collision_contact_script_taxis_into_launch_tree() {
    let scenario = scenario_named(WORLD_COLLISION_CONTACT).expect("collision route exists");

    assert!(scripted_input(scenario, 60).backward);
    assert!(scripted_input(scenario, 150).backward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 60).glide);
}

#[test]
fn terrain_rim_collision_contact_script_taxis_into_launch_rim() {
    let scenario = scenario_named(TERRAIN_RIM_COLLISION_CONTACT).expect("rim route exists");

    assert!(scripted_input(scenario, 60).forward);
    assert!(scripted_input(scenario, 150).forward);
    assert!(!scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 60).glide);
    assert!(!scripted_input(scenario, 120).backward);
}

#[test]
fn updraft_route_steers_toward_lift_without_diving() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 90).right);
    assert!(scripted_input(scenario, 180).glide);
    assert!(!scripted_input(scenario, 180).dive);
    assert_eq!(scenario.thresholds.min_completed_objective_count, 1);
}

#[test]
fn island_launch_script_releases_forward_after_touchdown() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");

    assert!(scripted_input(scenario, 360).forward);
    assert!(scripted_input(scenario, 360).glide);
    assert!(scripted_input(scenario, 360).dive);
    assert!(scripted_input(scenario, 423).forward);
    assert!(!scripted_input(scenario, 430).forward);
    assert!(scenario.thresholds.require_target_landing);
}

#[test]
fn branch_recovery_route_targets_named_recovery_island() {
    let scenario = scenario_named(BRANCH_RECOVERY_ROUTE).expect("branch route exists");

    assert_eq!(scenario.target_island_name, Some("sunlit terrace"));
    assert!(scenario.thresholds.require_target_landing);
    assert_eq!(scenario.thresholds.min_objective_total_count, 3);
    assert_eq!(scenario.thresholds.min_completed_objective_count, 3);
    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 540).glide);
    assert!(scripted_input(scenario, 540).dive);
    assert!(scripted_input(scenario, 630).forward);
    assert!(scripted_input(scenario, 650).backward);
    assert!(!scripted_input(scenario, 675).backward);
    assert!(!scripted_input(scenario, 750).forward);
}

#[test]
fn camera_mouse_script_exercises_x_and_y_axes() {
    let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("camera route exists");

    assert!(scripted_camera_input(scenario, 30).mouse_delta.x > 0.0);
    assert!(scripted_camera_input(scenario, 70).mouse_delta.y < 0.0);
    assert!(scripted_camera_input(scenario, 105).mouse_delta.y > 0.0);
    assert_eq!(
        scripted_input(scenario, 1),
        FlightInput::default(),
        "camera eval should not hide mouse regressions behind movement"
    );
}

#[test]
fn camera_yaw_stability_script_applies_small_yaw_then_settles() {
    let scenario = scenario_named(CAMERA_YAW_STABILITY).expect("camera yaw route exists");

    assert!(scripted_camera_input(scenario, 18).mouse_delta.x > 0.0);
    assert_eq!(scripted_camera_input(scenario, 80), CameraInput::default());
    assert_eq!(
        scripted_input(scenario, 18),
        FlightInput::default(),
        "yaw stability eval should isolate mouse drift from movement"
    );
}

#[test]
fn camera_turn_script_exercises_air_turns_and_air_brake() {
    let scenario = scenario_named(CAMERA_TURN_STABILITY).expect("turn route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 80).glide);
    assert!(scripted_input(scenario, 85).left);
    assert!(scripted_input(scenario, 115).right);
    assert!(scripted_input(scenario, 180).forward);
    assert!(scripted_input(scenario, 255).backward);
}

#[test]
fn camera_strafe_script_exercises_lateral_input_without_mouse() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");

    assert!(scripted_input(scenario, 30).right);
    assert!(scripted_input(scenario, 130).left);
    assert_eq!(scripted_camera_input(scenario, 30), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 130), CameraInput::default());
}

#[test]
fn air_control_response_script_exercises_lateral_brake_and_recovery_without_mouse() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 90).forward);
    assert!(scripted_input(scenario, 90).right);
    assert!(scripted_input(scenario, 140).right);
    assert!(!scripted_input(scenario, 140).forward);
    assert!(scripted_input(scenario, 210).left);
    assert!(scripted_input(scenario, 250).backward);
    assert!(scripted_input(scenario, 250).right);
    assert!(scripted_input(scenario, 310).backward);
    assert!(scripted_input(scenario, 310).left);
    assert!(scripted_input(scenario, 350).glide);
    assert!(scripted_input(scenario, 350).dive);
    assert!(scripted_input(scenario, 370).forward);
    assert_eq!(scripted_camera_input(scenario, 90), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 210), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 310), CameraInput::default());
    assert!(scenario.thresholds.min_gliding_samples >= 45);
}

#[test]
fn pose_state_coverage_script_exercises_walk_run_launch_fall_and_glide() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");

    assert!(scripted_input(scenario, 20).forward);
    assert!(!scripted_input(scenario, 65).forward);
    assert!(scripted_input(scenario, 120).right);
    assert!(scripted_input(scenario, 153).launch);
    assert!(!scripted_input(scenario, 210).glide);
    assert!(scripted_input(scenario, 240).glide);
    assert_eq!(scripted_camera_input(scenario, 153), CameraInput::default());
    assert!(scenario.frame_count >= 360);
    assert!(scenario.thresholds.min_samples >= 65);
    assert!(scenario.thresholds.min_grounded_samples >= 12);
    assert!(scenario.thresholds.min_gliding_samples >= 12);
}

#[test]
fn long_glide_visibility_script_crosses_archipelago() {
    let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("long glide route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 120).right);
    assert!(scripted_input(scenario, 160).left);
    assert!(scripted_input(scenario, 620).glide);
    assert!(!scripted_input(scenario, 620).dive);
    assert!(scenario.thresholds.min_sky_island_count >= MIN_SKY_ISLAND_COUNT);
    assert_eq!(scenario.thresholds.min_power_up_count, 3);
    assert_eq!(scenario.thresholds.min_collected_power_up_count, 3);
    assert!(scenario.thresholds.min_power_up_effect_samples >= 3);
}

#[test]
fn scenarios_define_non_final_camera_checkpoints() {
    for name in SCENARIO_NAMES {
        let scenario = scenario_named(name).expect("scenario exists");

        assert!(!scenario.checkpoints.is_empty());
        assert!(
            scenario
                .checkpoints
                .iter()
                .all(|checkpoint| checkpoint.frame < scenario.frame_count)
        );
        assert_eq!(
            scenario.checkpoint_at(scenario.checkpoints[0].frame),
            Some(scenario.checkpoints[0])
        );
    }
}
