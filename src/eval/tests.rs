use crate::asset_pipeline::{
    ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
    FAR_LOD_VISUAL_ASSET_SLOT_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
    MAX_MISSING_VISUAL_ASSET_SLOT_COUNT, MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
    MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
    MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
    STREAMING_VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SLOT_COUNT, WEATHER_VISUAL_ASSET_SLOT_COUNT,
};
use crate::camera::CameraInput;
use crate::environment::AERIAL_POWER_UP_ROUTE;
use crate::movement::{FlightInput, FlightMode};
use bevy::prelude::{Vec2, Vec3};

use super::*;

#[test]
fn baseline_route_has_scripted_launch_and_glide() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(!scripted_input(scenario, 2).launch);
    assert!(scripted_input(scenario, 60).glide);
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
    assert!(scripted_input(scenario, 540).dive);
    assert!(scripted_input(scenario, 624).backward);
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
    assert!(scripted_input(scenario, 370).forward);
    assert_eq!(scripted_camera_input(scenario, 90), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 210), CameraInput::default());
    assert_eq!(scripted_camera_input(scenario, 310), CameraInput::default());
    assert!(scenario.thresholds.min_gliding_samples >= 45);
}

#[test]
fn long_glide_visibility_script_crosses_archipelago() {
    let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("long glide route exists");

    assert!(scripted_input(scenario, 1).launch);
    assert!(scripted_input(scenario, 120).right);
    assert!(scripted_input(scenario, 160).left);
    assert!(scripted_input(scenario, 620).glide);
    assert!(!scripted_input(scenario, 620).dive);
    assert!(scenario.thresholds.min_sky_island_count >= 12);
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

#[test]
fn accumulator_summarizes_frame_time_percentiles() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame_time_ms in [8.0, 16.0, 33.0, 50.0] {
        accumulator.observe_frame_time_ms(frame_time_ms);
    }

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert_eq!(summary.metrics.avg_frame_time_ms, 26.75);
    assert_eq!(summary.metrics.p95_frame_time_ms, 50.0);
    assert_eq!(summary.metrics.p99_frame_time_ms, 50.0);
    assert_eq!(summary.metrics.max_frame_time_ms, 50.0);
}

#[test]
fn accumulator_requires_both_air_control_lateral_phases() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, 0.0, -18.0),
        Vec2::new(0.0, 1.0),
        0.0,
        18.0,
        8.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        20.0,
        18.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        210,
        Vec3::new(14.0, -2.0, -18.0),
        Vec2::new(-1.0, 0.0),
        2.0,
        18.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        270,
        Vec3::new(12.0, -2.0, 8.0),
        Vec2::new(1.0, -1.0),
        12.0,
        18.0,
        4.0,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let right_check = summary
        .checks
        .iter()
        .find(|check| check.name == "air_control_right_lateral_response")
        .expect("right response check exists");
    let left_check = summary
        .checks
        .iter()
        .find(|check| check.name == "air_control_left_lateral_response")
        .expect("left response check exists");

    assert!(right_check.passed);
    assert!(!left_check.passed);
    assert_eq!(summary.metrics.max_right_lateral_response_mps, 20.0);
    assert_eq!(summary.metrics.max_left_lateral_response_mps, 2.0);
}

#[test]
fn accumulator_requires_backward_diagonal_air_control_response() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, 0.0, -18.0),
        Vec2::new(0.0, 1.0),
        0.0,
        18.0,
        8.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        20.0,
        18.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        210,
        Vec3::new(-20.0, -2.0, -18.0),
        Vec2::new(-1.0, 0.0),
        20.0,
        18.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        270,
        Vec3::new(2.0, -2.0, 8.0),
        Vec2::new(1.0, -1.0),
        2.0,
        18.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        320,
        Vec3::new(-2.0, -2.0, 8.0),
        Vec2::new(-1.0, -1.0),
        2.0,
        18.0,
        4.0,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let aggregate_check = named_check(&summary, "air_control_backward_lateral_response");
    let backward_right_check = named_check(&summary, "air_control_backward_right_lateral_response");
    let backward_left_check = named_check(&summary, "air_control_backward_left_lateral_response");

    assert_eq!(summary.metrics.max_backward_lateral_response_mps, 2.0);
    assert_eq!(summary.metrics.max_backward_right_lateral_response_mps, 2.0);
    assert_eq!(summary.metrics.max_backward_left_lateral_response_mps, 2.0);
    assert!(!aggregate_check.passed);
    assert!(!backward_right_check.passed);
    assert!(!backward_left_check.passed);
}

#[test]
fn accumulator_requires_backward_diagonal_rear_component() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    let lateral_only_diagonal_alignment = 12.0 / std::f32::consts::SQRT_2;

    accumulator.observe(air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, 0.0, -18.0),
        Vec2::new(0.0, 1.0),
        0.0,
        20.0,
        8.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        20.0,
        20.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        210,
        Vec3::new(-20.0, -2.0, -18.0),
        Vec2::new(-1.0, 0.0),
        20.0,
        20.0,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        270,
        Vec3::new(12.0, -2.0, 0.0),
        Vec2::new(1.0, -1.0),
        12.0,
        lateral_only_diagonal_alignment,
        4.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        320,
        Vec3::new(-12.0, -2.0, 0.0),
        Vec2::new(-1.0, -1.0),
        12.0,
        lateral_only_diagonal_alignment,
        4.0,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let backward_right_lateral_check =
        named_check(&summary, "air_control_backward_right_lateral_response");
    let backward_left_lateral_check =
        named_check(&summary, "air_control_backward_left_lateral_response");
    let backward_right_rear_check =
        named_check(&summary, "air_control_backward_right_rear_response");
    let backward_left_rear_check = named_check(&summary, "air_control_backward_left_rear_response");

    assert!(backward_right_lateral_check.passed);
    assert!(backward_left_lateral_check.passed);
    assert!(!backward_right_rear_check.passed);
    assert!(!backward_left_rear_check.passed);
    assert!(summary.metrics.max_backward_right_rear_response_mps.abs() < 0.001);
    assert!(summary.metrics.max_backward_left_rear_response_mps.abs() < 0.001);
    let summary_json = summary.to_json();
    assert!(summary_json.contains("\"max_backward_right_rear_response_mps\""));
    assert!(summary_json.contains("\"max_backward_left_rear_response_mps\""));
}

#[test]
fn accumulator_gates_air_control_camera_follow_lag() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, movement_axis) in [
        (0, Vec2::new(0.0, 1.0)),
        (90, Vec2::new(1.0, 0.0)),
        (210, Vec2::new(-1.0, 0.0)),
    ] {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(20.0, -2.0, -18.0),
                movement_axis,
                20.0,
                18.0,
                4.0,
            )
            .with_camera_follow_metrics(72.0),
        );
    }

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let check = named_check(&summary, "air_control_avg_camera_follow_direction_error");

    assert_eq!(
        summary.metrics.avg_camera_follow_direction_error_degrees,
        72.0
    );
    assert_eq!(
        summary.metrics.max_camera_follow_direction_error_degrees,
        72.0
    );
    assert_eq!(check.value, 72.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_gates_air_control_follow_error_spikes() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..20 {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(16.0, -2.0, -18.0),
                Vec2::new(0.0, 1.0),
                0.0,
                18.0,
                4.0,
            )
            .with_camera_follow_metrics(0.0),
        );
    }
    for frame in [90, 210] {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(20.0, -2.0, -18.0),
                Vec2::new(1.0, 0.0),
                20.0,
                18.0,
                4.0,
            )
            .with_camera_follow_metrics(90.0),
        );
    }

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let avg_check = named_check(&summary, "air_control_avg_camera_follow_direction_error");
    let p95_check = named_check(&summary, "air_control_p95_camera_follow_direction_error");

    assert!(avg_check.passed);
    assert_eq!(
        summary.metrics.p95_camera_follow_direction_error_degrees,
        90.0
    );
    assert_eq!(p95_check.value, 90.0);
    assert!(!p95_check.passed);
}

#[test]
fn accumulator_gates_air_control_body_heading_spikes() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..20 {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(0.0, 1.0),
            0.0,
            18.0,
            3.0,
        ));
    }
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        20.0,
        18.0,
        90.0,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let avg_check = named_check(&summary, "air_control_avg_body_heading_error");
    let p95_check = named_check(&summary, "air_control_p95_body_heading_error");
    let max_check = named_check(&summary, "air_control_max_body_heading_error");
    let step_check = named_check(&summary, "air_control_max_body_yaw_error_step");

    assert!(avg_check.passed);
    assert!(p95_check.passed);
    assert_eq!(summary.metrics.max_desired_body_heading_error_degrees, 90.0);
    assert_eq!(max_check.value, 90.0);
    assert!(!max_check.passed);
    assert!(
        summary.metrics.max_body_yaw_error_step_degrees
            > AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES
    );
    assert_eq!(
        step_check.value,
        summary.metrics.max_body_yaw_error_step_degrees
    );
    assert!(!step_check.passed);
}

#[test]
fn accumulator_gates_movement_only_camera_world_yaw_drift() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator
        .observe(content_metric_sample(scenario, 0, 12, 0, 64).with_camera_world_yaw_metrics(0.0));
    accumulator.observe(
        content_metric_sample(scenario, 60, 12, 0, 64).with_camera_world_yaw_metrics(20.0),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let check = named_check(&summary, "camera_strafe_world_yaw_drift");

    assert_eq!(summary.metrics.max_camera_world_yaw_drift_degrees, 20.0);
    assert_eq!(check.value, 20.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_resets_body_roll_step_across_grounded_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(air_control_metric_sample(
        scenario,
        30,
        Vec3::new(14.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        16.0,
        18.0,
        4.0,
    ));
    let mut grounded = air_control_metric_sample(
        scenario,
        60,
        Vec3::new(0.0, 0.0, 0.0),
        Vec2::ZERO,
        0.0,
        f32::NAN,
        f32::NAN,
    );
    grounded.mode = FlightMode::Grounded.label();
    grounded.body_roll_degrees = 0.0;
    accumulator.observe(grounded);
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(-14.0, -2.0, -18.0),
        Vec2::new(-1.0, 0.0),
        16.0,
        18.0,
        -4.0,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert_eq!(summary.metrics.max_body_roll_step_degrees, 0.0);
    assert!(named_check(&summary, "air_control_max_body_roll_step").passed);
}

#[test]
fn accumulator_gates_movement_only_camera_view_yaw_drift() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator
        .observe(content_metric_sample(scenario, 0, 12, 0, 64).with_camera_view_yaw_metrics(0.0));
    accumulator
        .observe(content_metric_sample(scenario, 60, 12, 0, 64).with_camera_view_yaw_metrics(12.0));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let check = named_check(&summary, "camera_strafe_view_yaw_drift");

    assert_eq!(summary.metrics.max_camera_view_yaw_drift_degrees, 12.0);
    assert_eq!(check.value, 12.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_gates_ground_strafe_directional_response() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 64).with_movement_metrics(EvalMovementMetrics {
            desired_body_yaw_error_degrees: f32::NAN,
            body_roll_degrees: 0.0,
            desired_heading_alignment_mps: f32::NAN,
            lateral_response_mps: 9.0,
            lateral_input_active: false,
            movement_axis: Vec2::new(1.0, 0.0),
        }),
    );
    accumulator.observe(
        content_metric_sample(scenario, 60, 12, 0, 64).with_movement_metrics(EvalMovementMetrics {
            desired_body_yaw_error_degrees: f32::NAN,
            body_roll_degrees: 0.0,
            desired_heading_alignment_mps: f32::NAN,
            lateral_response_mps: 3.0,
            lateral_input_active: false,
            movement_axis: Vec2::new(-1.0, 0.0),
        }),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let right_check = named_check(&summary, "camera_strafe_right_lateral_response");
    let left_check = named_check(&summary, "camera_strafe_left_lateral_response");

    assert!(right_check.passed);
    assert_eq!(summary.metrics.max_right_lateral_response_mps, 9.0);
    assert_eq!(summary.metrics.max_left_lateral_response_mps, 3.0);
    assert!(!left_check.passed);
}

#[test]
fn accumulator_gates_planar_air_brake_drop() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(air_control_metric_sample(
        scenario,
        240,
        Vec3::new(10.0, -52.0, 0.0),
        Vec2::new(0.0, -1.0),
        0.0,
        0.0,
        f32::NAN,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        245,
        Vec3::new(10.0, -8.0, 0.0),
        Vec2::new(0.0, -1.0),
        0.0,
        0.0,
        f32::NAN,
    ));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let total_speed_check = named_check(&summary, "air_control_air_brake_speed_drop");
    let planar_speed_check = named_check(&summary, "air_control_air_brake_planar_speed_drop");

    assert!(summary.metrics.max_air_brake_speed_drop_mps > 40.0);
    assert!(total_speed_check.passed);
    assert_eq!(summary.metrics.max_air_brake_planar_speed_drop_mps, 0.0);
    assert_eq!(planar_speed_check.value, 0.0);
    assert!(!planar_speed_check.passed);
    assert!(
        summary
            .to_json()
            .contains("\"max_air_brake_planar_speed_drop_mps\"")
    );
}

#[test]
fn accumulator_gates_grounded_visual_foot_gap() {
    let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");
    let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
    sample.mode = FlightMode::Grounded.label();
    sample.visual_foot_gap_m = 0.18;

    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(sample);

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let check = named_check(&summary, "grounded_visual_foot_gap");

    assert_eq!(summary.metrics.max_grounded_visual_foot_gap_m, 0.18);
    assert_eq!(check.value, 0.18);
    assert!(!check.passed);
}

fn air_control_metric_sample(
    scenario: EvalScenario,
    frame: u32,
    velocity: Vec3,
    movement_axis: Vec2,
    lateral_response_mps: f32,
    desired_alignment_mps: f32,
    yaw_error_degrees: f32,
) -> EvalSample {
    let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 120.0, false);
    EvalSample::new(
        frame,
        scenario.fixed_dt,
        Vec3::new(frame as f32 * 0.5, 42.0, -(frame as f32) * 0.25),
        velocity,
        FlightMode::Gliding,
        14.0,
        3.0,
        4.0,
        -18.0,
        0.0,
        0.0,
        0.2,
        1.0,
        0.0,
        0.0,
        0.0,
        0,
        0,
        3,
        0,
        0,
        1,
        140.0,
        false,
        objective,
        12,
        25,
        6,
        2,
        4,
        6,
        24,
        36,
        8,
        4,
        26,
        118,
        16,
        12,
        8,
        0.08,
        160,
        0,
        12,
        12,
        335,
        175,
        0.48,
        0,
        0,
        12,
        12,
        20,
        20,
        130,
        VISUAL_ASSET_SLOT_COUNT,
        GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
        MIN_READY_VISUAL_ASSET_SLOT_COUNT,
        MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        STREAMING_VISUAL_ASSET_SLOT_COUNT,
        MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        0,
        MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
        0,
        MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
        MIN_READY_VISUAL_ASSET_SCENE_COUNT,
        ALWAYS_VISUAL_ASSET_SLOT_COUNT,
        STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
        NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        WEATHER_VISUAL_ASSET_SLOT_COUNT,
        MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_VISUAL_ANIMATION_PLAYER_COUNT,
        MIN_VISUAL_ANIMATION_GRAPH_COUNT,
        AERIAL_POWER_UP_ROUTE.len(),
        AERIAL_POWER_UP_ROUTE.len(),
        0,
        0,
        0,
    )
    .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 1633, 1633)
    .with_island_impostor_metrics(146, 24)
    .with_terrain_material_metrics(36, 3, 4, 64)
    .with_generated_visual_shape_metrics(
        528, 220, 1100, 37, 37, 62, 412, 5, 60, 74, 30, 12, 4.8, 7, 14, 574,
    )
    .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT)
    .with_movement_metrics(EvalMovementMetrics {
        desired_body_yaw_error_degrees: yaw_error_degrees,
        body_roll_degrees: -movement_axis.x.signum() * 12.0,
        desired_heading_alignment_mps: desired_alignment_mps,
        lateral_response_mps,
        lateral_input_active: movement_axis.x.abs() > f32::EPSILON,
        movement_axis,
    })
}

fn content_metric_sample(
    scenario: EvalScenario,
    frame: u32,
    procedural_body_count: usize,
    primitive_body_count: usize,
    silhouette_segments: usize,
) -> EvalSample {
    air_control_metric_sample(
        scenario,
        frame,
        Vec3::new(12.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        14.0,
        18.0,
        8.0,
    )
    .with_content_metrics(
        12,
        2305,
        61,
        0.8,
        9,
        procedural_body_count,
        primitive_body_count,
        silhouette_segments,
        silhouette_segments as f32,
        1633,
        1633,
    )
    .with_island_impostor_metrics(146, 24)
}

fn named_check<'a>(summary: &'a EvalSummary, name: &str) -> &'a EvalCheck {
    summary
        .checks
        .iter()
        .find(|check| check.name == name)
        .unwrap_or_else(|| panic!("{name} check exists"))
}

#[test]
fn accumulator_gates_authored_asset_readiness() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
    sample.ready_visual_asset_slot_count = 0;
    sample.placeholder_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
    sample.missing_visual_asset_slot_count = VISUAL_ASSET_SLOT_COUNT;
    sample.deferred_visual_asset_scene_count = 1;
    sample.queued_visual_asset_scene_count = 0;
    sample.loaded_visual_asset_scene_count = 0;
    sample.dependency_loaded_visual_asset_scene_count = 0;
    sample.preload_ready_visual_asset_scene_count = 0;
    sample.spawned_visual_asset_scene_count = 0;
    sample.ready_visual_asset_scene_count = 0;
    sample.visible_authored_world_fixture_count = 0;
    sample.always_preload_ready_visual_asset_slot_count = 0;
    sample.streaming_preload_ready_visual_asset_slot_count = 0;
    sample.ready_animation_clip_count = 0;
    sample.animation_player_count = 0;
    sample.animation_graph_count = 0;

    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(sample);
    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    for check_name in [
        "ready_visual_asset_slot_count",
        "missing_visual_asset_slot_count",
        "deferred_visual_asset_scene_count",
        "loaded_visual_asset_scene_count",
        "dependency_loaded_visual_asset_scene_count",
        "preload_ready_visual_asset_scene_count",
        "always_preload_ready_visual_asset_slot_count",
        "streaming_preload_ready_visual_asset_slot_count",
        "spawned_visual_asset_scene_count",
        "ready_visual_asset_scene_count",
        "visible_authored_world_fixture_count",
        "ready_animation_clip_count",
        "animation_player_count",
        "animation_graph_count",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without a loaded authored scene"
        );
    }
}

#[test]
fn accumulator_fails_when_procedural_body_count_disappears_after_startup() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(content_metric_sample(scenario, 10, 8, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let procedural_check = named_check(&summary, "procedural_island_body_count");

    assert!(!procedural_check.passed);
    assert_eq!(procedural_check.value, 8.0);
}

#[test]
fn accumulator_fails_registered_primitive_or_low_silhouette_body_content() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 1, 48));
    accumulator.observe(
        content_metric_sample(scenario, 5, 12, 0, 96)
            .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 900, 1633),
    );
    accumulator.observe(content_metric_sample(scenario, 10, 12, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let primitive_check = named_check(&summary, "primitive_island_body_count");
    let silhouette_check = named_check(&summary, "island_body_silhouette_segments");
    let mesh_check = named_check(&summary, "island_body_mesh_vertices");

    assert!(!primitive_check.passed);
    assert_eq!(primitive_check.value, 1.0);
    assert!(!silhouette_check.passed);
    assert_eq!(silhouette_check.value, 48.0);
    assert!(!mesh_check.passed);
    assert_eq!(mesh_check.value, 900.0);
}

#[test]
fn accumulator_fails_low_detail_island_impostors() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96).with_island_impostor_metrics(42, 4),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let mesh_check = named_check(&summary, "island_impostor_mesh_vertices");
    let color_check = named_check(&summary, "island_impostor_color_bands");

    assert!(!mesh_check.passed);
    assert_eq!(mesh_check.value, 42.0);
    assert!(!color_check.passed);
    assert_eq!(color_check.value, 4.0);
    assert!(
        summary
            .to_json()
            .contains("\"min_island_impostor_mesh_vertices\"")
    );
}

#[test]
fn accumulator_fails_terrain_detail_regression() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96)
            .with_content_metrics(10, 1200, 2, 0.2, 3, 12, 0, 96, 96.0, 1633, 1633)
            .with_terrain_material_metrics(4, 2, 2, 16),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let terrain_count_check = named_check(&summary, "island_terrain_surface_count");
    let terrain_vertex_check = named_check(&summary, "island_terrain_mesh_vertices");
    let terrain_color_check = named_check(&summary, "island_terrain_color_bands");
    let material_band_check = named_check(&summary, "island_terrain_material_weight_bands");
    let material_channel_check = named_check(&summary, "island_terrain_material_channels");
    let material_region_check = named_check(&summary, "island_terrain_material_regions");
    let texture_detail_check = named_check(&summary, "island_terrain_texture_detail_bands");
    let relief_check = named_check(&summary, "island_terrain_relief_range");
    let cliff_color_check = named_check(&summary, "island_cliff_color_bands");

    assert!(!terrain_count_check.passed);
    assert_eq!(terrain_count_check.value, 10.0);
    assert!(!terrain_vertex_check.passed);
    assert_eq!(terrain_vertex_check.value, 1200.0);
    assert!(!terrain_color_check.passed);
    assert_eq!(terrain_color_check.value, 2.0);
    assert!(!material_band_check.passed);
    assert_eq!(material_band_check.value, 4.0);
    assert!(!material_channel_check.passed);
    assert_eq!(material_channel_check.value, 2.0);
    assert!(!material_region_check.passed);
    assert_eq!(material_region_check.value, 2.0);
    assert!(!texture_detail_check.passed);
    assert_eq!(texture_detail_check.value, 16.0);
    assert!(!relief_check.passed);
    assert_eq!(relief_check.value, 0.2);
    assert!(!cliff_color_check.passed);
    assert_eq!(cliff_color_check.value, 3.0);
}

#[test]
fn summary_json_exposes_terrain_detail_thresholds() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let summary_json = summary.to_json();

    assert!(summary_json.contains("\"min_island_terrain_surface_count\": 12"));
    assert!(summary_json.contains("\"min_island_terrain_mesh_vertices\": 2305"));
    assert!(summary_json.contains("\"min_island_terrain_color_bands\": 61"));
    assert!(summary_json.contains("\"min_island_terrain_material_weight_bands\": 36"));
    assert!(summary_json.contains("\"min_island_terrain_material_channels\": 3"));
    assert!(summary_json.contains("\"min_island_terrain_material_regions\": 4"));
    assert!(summary_json.contains("\"min_island_terrain_texture_detail_bands\": 64"));
    assert!(summary_json.contains("\"min_island_terrain_relief_range_m\": 0.8000"));
    assert!(summary_json.contains("\"min_island_cliff_color_bands\": 9"));
    assert!(summary_json.contains("\"min_island_body_mesh_vertices\": 1633"));
    assert!(summary_json.contains("\"min_generated_ground_cover_patch_count\": 528"));
    assert!(summary_json.contains("\"min_ground_cover_blade_count\": 220"));
    assert!(summary_json.contains("\"min_ground_cover_mesh_vertices\": 1100"));
    assert!(summary_json.contains("\"min_tree_canopy_mesh_vertices\": 412"));
    assert!(summary_json.contains("\"min_detail_biome_palette_count\": 5"));
    assert!(summary_json.contains("\"min_generated_rock_count\": 60"));
    assert!(summary_json.contains("\"min_rock_mesh_vertices\": 74"));
    assert!(summary_json.contains("\"min_generated_weather_cloud_bank_count\": 12"));
    assert!(summary_json.contains("\"min_weather_cloud_bank_depth_m\": 4.8000"));
    assert!(summary_json.contains("\"min_weather_cloud_mesh_vertices\": 574"));
}

#[test]
fn accumulator_fails_generated_visual_shape_regression() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 96).with_generated_visual_shape_metrics(
            528, 220, 1100, 12, 12, 62, 316, 5, 48, 74, 12, 12, 4.8, 6, 10, 270,
        ),
    );
    accumulator.observe(
        content_metric_sample(scenario, 10, 12, 0, 96).with_generated_visual_shape_metrics(
            10, 12, 60, 0, 0, 8, 45, 1, 1, 12, 0, 0, 0.4, 1, 1, 45,
        ),
    );

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let tree_count_check = named_check(&summary, "generated_tree_trunk_count");
    let ground_patch_check = named_check(&summary, "generated_ground_cover_patch_count");
    let ground_blade_check = named_check(&summary, "ground_cover_blade_count");
    let ground_vertex_check = named_check(&summary, "ground_cover_mesh_vertices");
    let canopy_vertex_check = named_check(&summary, "tree_canopy_mesh_vertices");
    let detail_palette_check = named_check(&summary, "detail_biome_palette_count");
    let rock_count_check = named_check(&summary, "generated_rock_count");
    let rock_vertex_check = named_check(&summary, "rock_mesh_vertices");
    let cloud_lobe_check = named_check(&summary, "weather_cloud_lobe_count");
    let cloud_bank_lobe_check = named_check(&summary, "weather_cloud_bank_lobe_count");
    let cloud_bank_count_check = named_check(&summary, "generated_weather_cloud_bank_count");
    let cloud_bank_depth_check = named_check(&summary, "weather_cloud_bank_depth");

    assert!(!ground_patch_check.passed);
    assert_eq!(ground_patch_check.value, 10.0);
    assert!(!ground_blade_check.passed);
    assert_eq!(ground_blade_check.value, 12.0);
    assert!(!ground_vertex_check.passed);
    assert_eq!(ground_vertex_check.value, 60.0);
    assert!(!tree_count_check.passed);
    assert_eq!(tree_count_check.value, 0.0);
    assert!(!canopy_vertex_check.passed);
    assert_eq!(canopy_vertex_check.value, 45.0);
    assert!(!detail_palette_check.passed);
    assert_eq!(detail_palette_check.value, 1.0);
    assert!(!rock_count_check.passed);
    assert_eq!(rock_count_check.value, 1.0);
    assert!(!rock_vertex_check.passed);
    assert_eq!(rock_vertex_check.value, 12.0);
    assert!(!cloud_lobe_check.passed);
    assert_eq!(cloud_lobe_check.value, 1.0);
    assert!(!cloud_bank_lobe_check.passed);
    assert_eq!(cloud_bank_lobe_check.value, 1.0);
    assert!(!cloud_bank_count_check.passed);
    assert_eq!(cloud_bank_count_check.value, 0.0);
    assert!(!cloud_bank_depth_check.passed);
    assert_eq!(cloud_bank_depth_check.value, 0.4);
}

#[test]
fn accumulator_marks_current_baseline_shape_as_passing() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 140.0, false);

    observe_current_content(
        &mut accumulator,
        EvalSample::new(
            0,
            scenario.fixed_dt,
            Vec3::new(0.0, 1.2, 0.0),
            Vec3::ZERO,
            FlightMode::Grounded,
            12.0,
            3.0,
            4.0,
            -20.0,
            0.0,
            0.0,
            0.2,
            2.0,
            0.0,
            0.0,
            0.0,
            0,
            0,
            3,
            0,
            0,
            1,
            140.0,
            false,
            objective,
            12,
            25,
            6,
            2,
            4,
            6,
            24,
            36,
            8,
            4,
            26,
            118,
            16,
            12,
            8,
            0.08,
            160,
            0,
            12,
            12,
            335,
            175,
            0.48,
            0,
            0,
            12,
            12,
            20,
            20,
            130,
            VISUAL_ASSET_SLOT_COUNT,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            STREAMING_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT,
            ALWAYS_VISUAL_ASSET_SLOT_COUNT,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT,
            AERIAL_POWER_UP_ROUTE.len(),
            AERIAL_POWER_UP_ROUTE.len(),
            0,
            0,
            0,
        ),
    );
    observe_current_content(
        &mut accumulator,
        EvalSample::new(
            scenario.frame_count,
            scenario.fixed_dt,
            Vec3::new(0.0, 32.0, 140.0),
            Vec3::new(0.0, -4.0, 30.0),
            FlightMode::Gliding,
            14.0,
            3.0,
            4.0,
            -18.0,
            0.0,
            0.0,
            0.2,
            2.0,
            0.0,
            0.0,
            0.0,
            0,
            0,
            3,
            0,
            0,
            1,
            0.0,
            false,
            objective,
            12,
            25,
            6,
            2,
            4,
            6,
            24,
            36,
            8,
            4,
            26,
            118,
            16,
            12,
            8,
            0.08,
            160,
            0,
            12,
            12,
            335,
            175,
            0.48,
            0,
            0,
            12,
            12,
            20,
            20,
            130,
            VISUAL_ASSET_SLOT_COUNT,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            STREAMING_VISUAL_ASSET_SLOT_COUNT,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
            0,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT,
            ALWAYS_VISUAL_ASSET_SLOT_COUNT,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT,
            AERIAL_POWER_UP_ROUTE.len(),
            AERIAL_POWER_UP_ROUTE.len(),
            0,
            0,
            0,
        ),
    );
    for frame in 1..=scenario.thresholds.min_gliding_samples {
        observe_current_content(
            &mut accumulator,
            EvalSample::new(
                frame,
                scenario.fixed_dt,
                Vec3::new(0.0, 24.0, frame as f32 * 4.0),
                Vec3::new(0.0, -3.0, 25.0),
                FlightMode::Gliding,
                13.0,
                3.0,
                4.0,
                -18.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                140.0 - frame as f32 * 4.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
                0,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
                DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ),
        );
    }

    let summary = accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: vec!["checkpoints/glide_midroute.png".to_string()],
            checkpoint_marker_metadata: vec!["checkpoints/glide_midroute.markers.json".to_string()],
        },
    );

    assert!(summary.passed);
    assert_eq!(summary.metrics.objective_total_count, 2);
    assert_eq!(summary.metrics.max_completed_objective_count, 0);
    assert!(summary.to_json().contains("\"passed\": true"));
    assert!(summary.to_json().contains("\"objective\":"));
    assert!(
        summary
            .to_json()
            .contains("\"checkpoint_screenshots\": [\"checkpoints/glide_midroute.png\"]")
    );
    assert!(
        summary.to_json().contains(
            "\"checkpoint_marker_metadata\": [\"checkpoints/glide_midroute.markers.json\"]"
        )
    );
}

fn observe_current_content(accumulator: &mut EvalAccumulator, sample: EvalSample) {
    accumulator.observe(
        sample
            .with_content_metrics(12, 2305, 61, 0.8, 9, 12, 0, 96, 96.0, 1633, 1633)
            .with_island_impostor_metrics(146, 24)
            .with_terrain_material_metrics(36, 3, 4, 64)
            .with_generated_visual_shape_metrics(
                528, 220, 1100, 37, 37, 62, 412, 5, 60, 74, 30, 12, 4.8, 7, 14, 574,
            )
            .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT),
    );
}
