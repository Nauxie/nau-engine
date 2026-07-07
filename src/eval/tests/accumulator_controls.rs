use super::*;
use crate::animation::{
    GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
    GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
    LANDING_MAX_FOOT_SPLIT_READABILITY_M,
};
use crate::camera::CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M;
use crate::movement::{LAUNCH_MAX_HORIZONTAL_SPEED_MPS, LAUNCH_MAX_UPWARD_SPEED_MPS};

#[test]
fn accumulator_summarizes_frame_time_percentiles() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame_time_ms in [8.0, 16.0, 33.0, 50.0] {
        accumulator.observe_frame_time_ms(frame_time_ms);
    }
    accumulator.observe_eval_artifact_frame_time_ms(250.0);

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

    assert_eq!(summary.metrics.avg_frame_time_ms, 71.4);
    assert_eq!(summary.metrics.p95_frame_time_ms, 250.0);
    assert_eq!(summary.metrics.p99_frame_time_ms, 250.0);
    assert_eq!(summary.metrics.max_frame_time_ms, 250.0);
    assert_eq!(summary.metrics.runtime_frame_time_sample_count, 4);
    assert_eq!(summary.metrics.avg_runtime_frame_time_ms, 26.75);
    assert_eq!(summary.metrics.p95_runtime_frame_time_ms, 50.0);
    assert_eq!(summary.metrics.p99_runtime_frame_time_ms, 50.0);
    assert_eq!(summary.metrics.max_runtime_frame_time_ms, 50.0);
    assert_eq!(summary.metrics.eval_artifact_frame_time_sample_count, 1);
    assert_eq!(summary.metrics.eval_artifact_frame_time_spike_count, 1);
    assert_eq!(summary.metrics.max_eval_artifact_frame_time_ms, 250.0);
}

#[test]
fn accumulator_reports_and_gates_launch_speed_caps() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(
            LAUNCH_MAX_HORIZONTAL_SPEED_MPS + 0.5,
            LAUNCH_MAX_UPWARD_SPEED_MPS + 0.5,
            0.0,
        ),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    );
    sample.mode = FlightMode::Launching.label();
    sample.pose_intent_label = "launching";
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

    assert_eq!(
        summary.metrics.max_launch_upward_speed_mps,
        LAUNCH_MAX_UPWARD_SPEED_MPS + 0.5
    );
    assert_eq!(
        summary.metrics.max_launch_horizontal_speed_mps,
        LAUNCH_MAX_HORIZONTAL_SPEED_MPS + 0.5
    );
    assert!(
        summary
            .to_json()
            .contains("\"max_launch_upward_speed_mps\"")
    );
    assert!(!named_check(&summary, "launch_upward_speed").passed);
    assert!(!named_check(&summary, "launch_horizontal_speed").passed);
}

#[test]
fn accumulator_gates_entity_count_as_performance_ceiling() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut sample = content_metric_sample(scenario, 0, MIN_SKY_ISLAND_COUNT, 0, 96);
    sample.entity_count = scenario.thresholds.max_entity_count + 1;

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
    let check = named_check(&summary, "entity_count");

    assert_eq!(
        summary.metrics.max_entity_count,
        scenario.thresholds.max_entity_count + 1
    );
    assert_eq!(check.comparator, "<=");
    assert_eq!(check.threshold, scenario.thresholds.max_entity_count as f32);
    assert!(!check.passed);
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
        Vec3::new(28.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        28.0,
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
    assert_eq!(summary.metrics.max_right_lateral_response_mps, 28.0);
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

    for frame in 0..40 {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(0.0, 1.0),
            0.0,
            18.0,
            2.0,
        ));
    }
    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(0.0, 1.0),
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
fn accumulator_ignores_body_yaw_intent_changes_for_oscillation_metrics() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, movement_axis, yaw_error_degrees) in [
        (10, Vec2::new(1.0, 0.0), 18.0),
        (20, Vec2::new(-1.0, 0.0), -18.0),
        (30, Vec2::new(-1.0, 0.0), -4.0),
        (40, Vec2::new(1.0, 0.0), 18.0),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(movement_axis.x * 18.0, -2.0, -18.0),
            movement_axis,
            18.0,
            18.0,
            yaw_error_degrees,
        ));
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

    assert_eq!(summary.metrics.max_body_yaw_error_step_degrees, 14.0);
    assert_eq!(summary.metrics.body_yaw_oscillation_count, 0);
    assert!(named_check(&summary, "air_control_max_body_yaw_error_step").passed);
    assert!(named_check(&summary, "air_control_body_yaw_oscillation_count").passed);
}

#[test]
fn accumulator_gates_air_control_body_travel_heading_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [90, 100, 110, 120] {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(20.0, -2.0, -18.0),
                Vec2::new(1.0, 0.0),
                20.0,
                18.0,
                3.0,
            )
            .with_body_travel_heading_error_degrees(90.0),
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
    let p95_check = named_check(
        &summary,
        "air_control_p95_lateral_body_travel_heading_error",
    );
    let max_check = named_check(
        &summary,
        "air_control_max_lateral_body_travel_heading_error",
    );
    let sample_count_check =
        named_check(&summary, "air_control_lateral_body_travel_heading_samples");
    let right_sample_count_check =
        named_check(&summary, "air_control_right_body_travel_heading_samples");

    assert_eq!(summary.metrics.lateral_body_travel_heading_sample_count, 4);
    assert_eq!(
        summary
            .metrics
            .right_lateral_body_travel_heading_sample_count,
        4
    );
    assert_eq!(
        summary
            .metrics
            .left_lateral_body_travel_heading_sample_count,
        0
    );
    assert_eq!(
        summary
            .metrics
            .max_lateral_body_travel_heading_error_degrees,
        90.0
    );
    assert_eq!(sample_count_check.value, 4.0);
    assert!(sample_count_check.passed);
    assert_eq!(right_sample_count_check.value, 4.0);
    assert!(right_sample_count_check.passed);
    assert_eq!(p95_check.value, 90.0);
    assert_eq!(max_check.value, 90.0);
    assert!(!p95_check.passed);
    assert!(!max_check.passed);

    let summary_json = summary.to_json();
    assert!(summary_json.contains("\"lateral_body_travel_heading_sample_count\": 4"));
    assert!(summary_json.contains("\"right_lateral_body_travel_heading_sample_count\": 4"));
    assert!(summary_json.contains("\"left_lateral_body_travel_heading_sample_count\": 0"));
    assert!(summary_json.contains("\"p95_lateral_body_travel_heading_error_degrees\""));
    assert!(summary_json.contains("\"max_lateral_body_travel_heading_error_degrees\""));
    assert!(summary_json.contains("\"backward_diagonal_body_travel_heading_sample_count\""));
    assert!(summary_json.contains("\"backward_right_diagonal_body_travel_heading_sample_count\""));
    assert!(summary_json.contains("\"backward_left_diagonal_body_travel_heading_sample_count\""));
    assert!(summary_json.contains("\"p95_backward_diagonal_body_travel_heading_error_degrees\""));
    assert!(summary_json.contains("\"max_backward_diagonal_body_travel_heading_error_degrees\""));
}

#[test]
fn accumulator_gates_air_control_desired_travel_heading_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..8 {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                90 + frame,
                Vec3::new(20.0, -2.0, -18.0),
                Vec2::new(1.0, 0.0),
                20.0,
                18.0,
                3.0,
            )
            .with_desired_travel_heading_error_degrees(80.0),
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
    let sample_count_check = named_check(&summary, "air_control_desired_travel_heading_samples");
    let p95_check = named_check(&summary, "air_control_p95_desired_travel_heading_error");
    let max_check = named_check(&summary, "air_control_max_desired_travel_heading_error");

    assert_eq!(summary.metrics.desired_travel_heading_sample_count, 8);
    assert_eq!(
        summary.metrics.p95_desired_travel_heading_error_degrees,
        80.0
    );
    assert_eq!(
        summary.metrics.max_desired_travel_heading_error_degrees,
        80.0
    );
    assert!(sample_count_check.passed);
    assert_eq!(p95_check.value, 80.0);
    assert_eq!(max_check.value, 80.0);
    assert!(!p95_check.passed);
    assert!(!max_check.passed);

    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");
    assert_eq!(
        summary_json["metrics"]["desired_travel_heading_sample_count"],
        8
    );
    assert_eq!(
        summary_json["metrics"]["p95_desired_travel_heading_error_degrees"],
        80.0
    );
    assert_eq!(
        summary_json["metrics"]["max_desired_travel_heading_error_degrees"],
        80.0
    );
    assert_eq!(
        summary_json["final_sample"]["desired_travel_heading_error_degrees"],
        80.0
    );
}

#[test]
fn accumulator_gates_missing_air_control_desired_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..4 {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                90 + frame,
                Vec3::new(20.0, -2.0, -18.0),
                Vec2::new(1.0, 0.0),
                20.0,
                18.0,
                3.0,
            )
            .with_desired_travel_heading_error_degrees(f32::NAN),
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
    let sample_count_check = named_check(&summary, "air_control_desired_travel_heading_samples");

    assert_eq!(summary.metrics.desired_travel_heading_sample_count, 0);
    assert_eq!(sample_count_check.value, 0.0);
    assert!(!sample_count_check.passed);
}

#[test]
fn accumulator_gates_missing_directional_air_control_desired_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [90, 120, 150, 180] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            18.0,
            3.0,
        ));
    }
    for frame in [240, 270, 300, 330] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(12.0, -2.0, 14.0),
            Vec2::new(1.0, -1.0),
            12.0,
            18.0,
            3.0,
        ));
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

    assert_eq!(summary.metrics.desired_travel_heading_sample_count, 8);
    assert_eq!(summary.metrics.right_desired_travel_heading_sample_count, 8);
    assert_eq!(summary.metrics.left_desired_travel_heading_sample_count, 0);
    assert_eq!(
        summary
            .metrics
            .backward_right_desired_travel_heading_sample_count,
        4
    );
    assert_eq!(
        summary
            .metrics
            .backward_left_desired_travel_heading_sample_count,
        0
    );
    assert!(named_check(&summary, "air_control_desired_travel_heading_samples").passed);
    assert!(named_check(&summary, "air_control_right_desired_travel_heading_samples").passed);
    assert!(!named_check(&summary, "air_control_left_desired_travel_heading_samples").passed);
    assert!(
        named_check(
            &summary,
            "air_control_backward_right_desired_travel_heading_samples"
        )
        .passed
    );
    assert!(
        !named_check(
            &summary,
            "air_control_backward_left_desired_travel_heading_samples"
        )
        .passed
    );

    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");
    assert_eq!(
        summary_json["metrics"]["right_desired_travel_heading_sample_count"],
        8
    );
    assert_eq!(
        summary_json["metrics"]["left_desired_travel_heading_sample_count"],
        0
    );
    assert_eq!(
        summary_json["metrics"]["backward_right_desired_travel_heading_sample_count"],
        4
    );
    assert_eq!(
        summary_json["metrics"]["backward_left_desired_travel_heading_sample_count"],
        0
    );
}

#[test]
fn accumulator_gates_missing_air_control_body_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        air_control_metric_sample(
            scenario,
            90,
            Vec3::new(20.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            20.0,
            18.0,
            3.0,
        )
        .with_body_travel_heading_error_degrees(f32::NAN),
    );
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            250,
            Vec3::new(12.0, -2.0, 14.0),
            Vec2::new(1.0, -1.0),
            12.0,
            18.0,
            3.0,
        )
        .with_body_travel_heading_error_degrees(f32::NAN),
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

    for check_name in [
        "air_control_lateral_body_travel_heading_samples",
        "air_control_right_body_travel_heading_samples",
        "air_control_left_body_travel_heading_samples",
        "air_control_backward_diagonal_body_travel_heading_samples",
        "air_control_backward_right_diagonal_body_travel_heading_samples",
        "air_control_backward_left_diagonal_body_travel_heading_samples",
    ] {
        let check = named_check(&summary, check_name);
        assert!(
            !check.passed,
            "{check_name} should fail without finite body/travel samples"
        );
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn accumulator_gates_one_sided_air_control_body_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(air_control_metric_sample(
        scenario,
        90,
        Vec3::new(20.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        20.0,
        18.0,
        3.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        250,
        Vec3::new(12.0, -2.0, 14.0),
        Vec2::new(1.0, -1.0),
        12.0,
        18.0,
        3.0,
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

    for check_name in [
        "air_control_left_body_travel_heading_samples",
        "air_control_backward_left_diagonal_body_travel_heading_samples",
    ] {
        let check = named_check(&summary, check_name);
        assert!(
            !check.passed,
            "{check_name} should fail without matching-direction finite samples"
        );
        assert_eq!(check.value, 0.0);
    }
    assert_eq!(
        summary
            .metrics
            .right_lateral_body_travel_heading_sample_count,
        2
    );
    assert_eq!(
        summary
            .metrics
            .backward_right_diagonal_body_travel_heading_sample_count,
        1
    );
}

#[test]
fn accumulator_gates_air_control_backward_diagonal_body_travel_heading_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [250, 260, 270, 280] {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                frame,
                Vec3::new(12.0, -2.0, 14.0),
                Vec2::new(1.0, -1.0),
                12.0,
                18.0,
                3.0,
            )
            .with_body_travel_heading_error_degrees(70.0),
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
    let p95_check = named_check(
        &summary,
        "air_control_p95_backward_diagonal_body_travel_heading_error",
    );
    let max_check = named_check(
        &summary,
        "air_control_max_backward_diagonal_body_travel_heading_error",
    );
    let sample_count_check = named_check(
        &summary,
        "air_control_backward_diagonal_body_travel_heading_samples",
    );
    let backward_right_sample_count_check = named_check(
        &summary,
        "air_control_backward_right_diagonal_body_travel_heading_samples",
    );

    assert_eq!(
        summary
            .metrics
            .backward_diagonal_body_travel_heading_sample_count,
        4
    );
    assert_eq!(
        summary
            .metrics
            .max_backward_diagonal_body_travel_heading_error_degrees,
        70.0
    );
    assert_eq!(sample_count_check.value, 4.0);
    assert!(sample_count_check.passed);
    assert_eq!(backward_right_sample_count_check.value, 4.0);
    assert!(backward_right_sample_count_check.passed);
    assert_eq!(p95_check.value, 70.0);
    assert_eq!(max_check.value, 70.0);
    assert!(!p95_check.passed);
    assert!(!max_check.passed);
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
fn accumulator_counts_obstruction_snaps_by_camera_step() {
    let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("camera mouse route exists");
    let mut accumulator = EvalAccumulator::default();

    let mut first = content_metric_sample(scenario, 0, 12, 0, 64);
    first.camera_obstruction_hits = 1;
    first.camera_obstruction_adjustment_m = 1.0;
    first.camera_distance_m = 14.0;
    first.camera_step_distance_m = 0.2;
    accumulator.observe(first);

    let mut lateral_snap = content_metric_sample(scenario, 1, 12, 0, 64);
    lateral_snap.camera_obstruction_hits = 1;
    lateral_snap.camera_obstruction_adjustment_m = 1.0;
    lateral_snap.camera_distance_m = 14.0;
    lateral_snap.camera_step_distance_m = CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M + 0.25;
    accumulator.observe(lateral_snap);

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
    let check = named_check(&summary, "camera_obstruction_snap_count");

    assert_eq!(summary.metrics.camera_obstruction_snap_count, 1);
    assert_eq!(check.value, 1.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_gates_ground_strafe_directional_response() {
    let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        content_metric_sample(scenario, 0, 12, 0, 64).with_movement_metrics(EvalMovementMetrics {
            desired_body_yaw_error_degrees: f32::NAN,
            body_travel_heading_error_degrees: f32::NAN,
            body_roll_degrees: 0.0,
            desired_heading_alignment_mps: f32::NAN,
            desired_travel_heading_error_degrees: f32::NAN,
            lateral_response_mps: 9.0,
            lateral_input_active: false,
            movement_axis: Vec2::new(1.0, 0.0),
        }),
    );
    accumulator.observe(
        content_metric_sample(scenario, 60, 12, 0, 64).with_movement_metrics(EvalMovementMetrics {
            desired_body_yaw_error_degrees: f32::NAN,
            body_travel_heading_error_degrees: f32::NAN,
            body_roll_degrees: 0.0,
            desired_heading_alignment_mps: f32::NAN,
            desired_travel_heading_error_degrees: f32::NAN,
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
fn accumulator_summarizes_pose_intent_samples() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        air_control_metric_sample(
            scenario,
            0,
            Vec3::new(0.0, -2.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 64.0,
            arm_spread_degrees: 0.0,
            leg_tuck_degrees: 0.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        }),
    );
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            1,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 62.0,
            arm_spread_degrees: 82.0,
            leg_tuck_degrees: 58.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        }),
    );
    accumulator.observe(air_control_metric_sample(
        scenario,
        2,
        Vec3::new(0.0, -4.0, -18.0),
        Vec2::new(0.0, -1.0),
        0.0,
        18.0,
        0.0,
    ));
    accumulator.observe(air_control_metric_sample(
        scenario,
        6,
        Vec3::new(16.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        16.0,
        18.0,
        0.0,
    ));
    let mut falling_sample = air_control_metric_sample(
        scenario,
        7,
        Vec3::new(0.0, -8.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    );
    falling_sample.mode = FlightMode::Airborne.label();
    falling_sample.pose_intent_label = "falling";
    falling_sample = falling_sample.with_authored_animation_metrics("fall", "fall", 1, 140);
    accumulator.observe(falling_sample);
    let mut landing_anticipation_sample = air_control_metric_sample(
        scenario,
        3,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 37.0,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 0.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    });
    landing_anticipation_sample.mode = FlightMode::Airborne.label();
    landing_anticipation_sample.pose_intent_label = "landing_anticipation";
    landing_anticipation_sample =
        landing_anticipation_sample.with_authored_animation_metrics("land", "land", 1, 140);
    accumulator.observe(landing_anticipation_sample);
    let mut landing_recovery_sample = air_control_metric_sample(
        scenario,
        4,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    );
    landing_recovery_sample.pose_intent_label = "landing_recovery";
    landing_recovery_sample =
        landing_recovery_sample.with_authored_animation_metrics("land", "land", 1, 140);
    accumulator.observe(landing_recovery_sample);
    let mut unreadable_landing_recovery_sample = air_control_metric_sample(
        scenario,
        5,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 0.0,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 0.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 0.25,
    });
    unreadable_landing_recovery_sample.pose_intent_label = "landing_recovery";
    unreadable_landing_recovery_sample =
        unreadable_landing_recovery_sample.with_authored_animation_metrics("land", "land", 1, 140);
    accumulator.observe(unreadable_landing_recovery_sample);

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

    assert_eq!(summary.metrics.pose_gliding_samples, 1);
    assert_eq!(summary.metrics.pose_air_turn_samples, 1);
    assert_eq!(summary.metrics.right_pose_air_turn_samples, 1);
    assert_eq!(summary.metrics.left_pose_air_turn_samples, 0);
    assert_eq!(summary.metrics.pose_falling_samples, 1);
    assert_eq!(summary.metrics.pose_diving_samples, 1);
    assert_eq!(summary.metrics.gliding_dive_samples, 1);
    assert_eq!(summary.metrics.pose_air_brake_samples, 1);
    assert_eq!(summary.metrics.pose_landing_anticipation_samples, 1);
    assert_eq!(summary.metrics.gliding_landing_anticipation_samples, 0);
    assert_eq!(summary.metrics.pose_landing_recovery_samples, 1);
    assert_eq!(summary.metrics.authored_clip_match_samples, 8);
    assert_eq!(summary.metrics.authored_clip_mismatch_samples, 0);
    assert_eq!(summary.metrics.authored_bank_left_clip_samples, 0);
    assert_eq!(summary.metrics.authored_bank_right_clip_samples, 1);
    assert_eq!(summary.metrics.authored_launch_clip_samples, 0);
    assert_eq!(summary.metrics.authored_glide_clip_samples, 1);
    assert_eq!(summary.metrics.authored_fall_clip_samples, 1);
    assert_eq!(summary.metrics.authored_dive_clip_samples, 1);
    assert_eq!(summary.metrics.authored_air_brake_clip_samples, 1);
    assert_eq!(summary.metrics.authored_land_clip_samples, 3);
    assert_eq!(summary.metrics.authored_transition_active_samples, 0);
    assert_eq!(summary.metrics.max_authored_transition_duration_ms, 140);
    assert_eq!(summary.metrics.max_pose_torso_pitch_degrees, 64.0);
    assert_eq!(summary.metrics.max_dive_pose_torso_pitch_degrees, 62.0);
    assert_eq!(summary.metrics.max_dive_pose_arm_spread_degrees, 82.0);
    assert_eq!(summary.metrics.max_dive_pose_leg_tuck_degrees, 58.0);
    assert_eq!(summary.metrics.max_pose_landing_flare_degrees, 37.0);
    assert_eq!(
        summary.metrics.max_authored_glider_dive_response_degrees,
        AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES
    );
    assert_eq!(summary.metrics.max_authored_glider_dive_motion_m, 0.08);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 1);
    assert!(summary_json.contains("\"max_pose_landing_foot_forward_m\""));
    assert!(summary_json.contains("\"max_dive_pose_torso_pitch_degrees\": 62"));
    assert!(summary_json.contains("\"max_dive_pose_arm_spread_degrees\": 82"));
    assert!(summary_json.contains("\"max_dive_pose_leg_tuck_degrees\": 58"));
    assert!(summary_json.contains("\"max_pose_landing_flare_degrees\": 37"));
    assert!(summary_json.contains("\"pose_air_turn_samples\": 1"));
    assert!(summary_json.contains("\"right_pose_air_turn_samples\": 1"));
    assert!(summary_json.contains("\"left_pose_air_turn_samples\": 0"));
    assert!(summary_json.contains("\"pose_falling_samples\": 1"));
    assert!(summary_json.contains("\"gliding_dive_samples\": 1"));
    assert!(summary_json.contains("\"max_authored_glider_dive_response_degrees\""));
    assert!(summary_json.contains("\"max_authored_glider_dive_motion_m\""));
    assert!(summary_json.contains("\"pose_air_brake_samples\": 1"));
    assert!(summary_json.contains("\"pose_landing_anticipation_samples\": 1"));
    assert!(summary_json.contains("\"gliding_landing_anticipation_samples\": 0"));
    assert!(summary_json.contains("\"pose_landing_recovery_samples\": 1"));
    assert!(summary_json.contains("\"authored_clip_match_samples\": 8"));
    assert!(summary_json.contains("\"authored_clip_mismatch_samples\": 0"));
    assert!(summary_json.contains("\"authored_bank_left_clip_samples\": 0"));
    assert!(summary_json.contains("\"authored_bank_right_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_launch_clip_samples\": 0"));
    assert!(summary_json.contains("\"authored_glide_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_fall_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_dive_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_air_brake_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_land_clip_samples\": 3"));
    assert!(summary_json.contains("\"authored_transition_active_samples\": 0"));
    assert!(summary_json.contains("\"max_authored_transition_duration_ms\": 140"));
}

#[test]
fn accumulator_gates_target_landing_recovery_pose_samples_and_flare() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 0.0,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 0.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 1.0,
        landing_foot_forward_m: 0.40,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    });
    sample.mode = FlightMode::Airborne.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample = sample.with_authored_glider_metrics(0.0, 0.0);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let landing_recovery_check = named_check(&summary, "pose_landing_recovery_samples");
    let landing_foot_forward_check = named_check(&summary, "pose_landing_foot_forward");
    let landing_foot_split_check = named_check(&summary, "pose_landing_foot_split");
    let landing_foot_split_max_check = named_check(&summary, "pose_landing_foot_split_max");
    let landing_flare_check = named_check(&summary, "pose_landing_flare");
    let landing_flare_backbend_check = named_check(&summary, "pose_landing_flare_backbend");
    let landing_forward_fold_check = named_check(&summary, "pose_landing_forward_fold");
    let landing_backward_bend_check = named_check(&summary, "pose_landing_backward_bend");
    let landing_recovery_backbend_check = named_check(&summary, "pose_landing_recovery_backbend");
    let gliding_landing_check = named_check(&summary, "gliding_landing_anticipation_samples");
    let authored_land_check = named_check(&summary, "authored_landing_clip_samples");
    let target_glider_response_check =
        named_check(&summary, "target_landing_authored_glider_response");
    let target_glider_motion_check = named_check(&summary, "target_landing_authored_glider_motion");

    assert_eq!(summary.metrics.pose_landing_recovery_samples, 0);
    assert_eq!(summary.metrics.gliding_landing_anticipation_samples, 0);
    assert_eq!(summary.metrics.authored_land_clip_samples, 1);
    assert_eq!(summary.metrics.max_pose_landing_foot_forward_m, 0.40);
    assert_eq!(summary.metrics.max_pose_landing_foot_split_m, 0.0);
    assert_eq!(summary.metrics.max_pose_landing_flare_degrees, 0.0);
    assert_eq!(summary.metrics.max_pose_landing_recovery_flip_degrees, 0.0);
    assert_eq!(landing_recovery_check.value, 0.0);
    assert_eq!(landing_recovery_check.threshold, 1.0);
    assert!(!landing_recovery_check.passed);
    assert_eq!(authored_land_check.value, 1.0);
    assert_eq!(authored_land_check.threshold, 2.0);
    assert!(!authored_land_check.passed);
    assert_eq!(target_glider_response_check.value, 0.0);
    assert_eq!(
        target_glider_response_check.threshold,
        AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES
    );
    assert!(!target_glider_response_check.passed);
    assert_eq!(target_glider_motion_check.value, 0.0);
    assert_eq!(target_glider_motion_check.threshold, 0.04);
    assert!(!target_glider_motion_check.passed);
    assert_eq!(landing_foot_forward_check.value, 0.40);
    assert_eq!(landing_foot_forward_check.threshold, 0.32);
    assert!(landing_foot_forward_check.passed);
    assert_eq!(landing_foot_split_check.value, 0.0);
    assert_eq!(
        landing_foot_split_check.threshold,
        LANDING_MIN_POSE_FOOT_SPLIT_M
    );
    assert!(!landing_foot_split_check.passed);
    assert_eq!(landing_foot_split_max_check.value, 0.0);
    assert_eq!(
        landing_foot_split_max_check.threshold,
        LANDING_MAX_FOOT_SPLIT_READABILITY_M
    );
    assert!(landing_foot_split_max_check.passed);
    assert_eq!(landing_flare_check.value, 0.0);
    assert_eq!(
        landing_flare_check.threshold,
        LANDING_MIN_POSE_FLARE_DEGREES
    );
    assert!(!landing_flare_check.passed);
    assert_eq!(landing_flare_backbend_check.value, 0.0);
    assert_eq!(
        landing_flare_backbend_check.threshold,
        LANDING_MAX_POSE_ANTICIPATION_BACKBEND_DEGREES
    );
    assert!(landing_flare_backbend_check.passed);
    assert_eq!(landing_forward_fold_check.value, 0.0);
    assert_eq!(
        landing_forward_fold_check.threshold,
        LANDING_MIN_POSE_FORWARD_FOLD_DEGREES
    );
    assert!(!landing_forward_fold_check.passed);
    assert_eq!(landing_backward_bend_check.value, 0.0);
    assert_eq!(
        landing_backward_bend_check.threshold,
        LANDING_MAX_POSE_BACKWARD_BEND_DEGREES
    );
    assert!(landing_backward_bend_check.passed);
    assert_eq!(landing_recovery_backbend_check.value, 0.0);
    assert_eq!(
        landing_recovery_backbend_check.threshold,
        LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES
    );
    assert!(landing_recovery_backbend_check.passed);
    assert_eq!(gliding_landing_check.value, 0.0);
    assert_eq!(gliding_landing_check.threshold, 0.0);
    assert!(gliding_landing_check.passed);

    let mut passing_accumulator = EvalAccumulator::default();
    let mut passing_anticipation = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MIN_POSE_FLARE_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 64.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: LANDING_MIN_POSE_FOOT_FORWARD_M,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_backward_bend(-LANDING_MIN_POSE_FORWARD_FOLD_DEGREES);
    passing_anticipation.mode = FlightMode::Airborne.label();
    passing_anticipation.pose_intent_label = "landing_anticipation";
    passing_anticipation =
        passing_anticipation.with_authored_animation_metrics("land", "land", 1, 140);
    passing_anticipation = passing_anticipation
        .with_authored_glider_metrics(AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES, 0.04);
    passing_anticipation.target_distance_m = 0.0;
    passing_anticipation.on_landing_target = true;
    let mut excessive_split_anticipation = passing_anticipation.clone();
    excessive_split_anticipation.pose_landing_distal_foot_split_m =
        LANDING_MAX_FOOT_SPLIT_READABILITY_M + 0.01;
    passing_accumulator.observe(passing_anticipation);

    let mut passing_recovery = air_control_metric_sample(
        scenario,
        1,
        Vec3::new(0.0, 0.0, -5.0),
        Vec2::ZERO,
        0.0,
        12.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 42.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.09,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_backward_bend(0.0);
    passing_recovery.pose_intent_label = "landing_recovery";
    passing_recovery = passing_recovery.with_authored_animation_metrics("land", "land", 1, 140);
    passing_recovery = passing_recovery
        .with_authored_glider_metrics(AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES, 0.04);
    let mut excessive_split_recovery = passing_recovery.clone();
    excessive_split_recovery.pose_landing_distal_foot_split_m =
        LANDING_MAX_FOOT_SPLIT_READABILITY_M + 0.01;
    passing_accumulator.observe(passing_recovery);

    let passing_summary = passing_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert!(named_check(&passing_summary, "pose_landing_foot_split").passed);
    assert!(named_check(&passing_summary, "pose_landing_foot_split_max").passed);
    assert!(named_check(&passing_summary, "pose_landing_flare_backbend").passed);
    assert!(named_check(&passing_summary, "pose_landing_forward_fold").passed);
    assert!(named_check(&passing_summary, "pose_landing_backward_bend").passed);
    assert!(named_check(&passing_summary, "pose_landing_recovery_backbend").passed);
    assert!(named_check(&passing_summary, "gliding_landing_anticipation_samples").passed);
    assert!(named_check(&passing_summary, "target_landing_authored_glider_response").passed);
    assert!(named_check(&passing_summary, "target_landing_authored_glider_motion").passed);
    assert_eq!(
        passing_summary.metrics.gliding_landing_anticipation_samples,
        0
    );
    assert_eq!(
        passing_summary.metrics.max_pose_landing_foot_split_m,
        LANDING_MIN_POSE_FOOT_SPLIT_M
    );
    assert_eq!(
        passing_summary
            .metrics
            .max_pose_landing_recovery_flip_degrees,
        LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES
    );
    assert_eq!(
        passing_summary
            .metrics
            .max_pose_landing_forward_fold_degrees,
        LANDING_MIN_POSE_FORWARD_FOLD_DEGREES
    );
    assert_eq!(
        passing_summary
            .metrics
            .max_pose_landing_backward_bend_degrees,
        0.0
    );
    assert!(
        passing_summary
            .to_json()
            .contains("\"max_pose_landing_foot_split_m\"")
    );
    assert!(
        passing_summary
            .to_json()
            .contains("\"max_pose_landing_distal_foot_split_m\"")
    );
    assert!(
        passing_summary
            .to_json()
            .contains("\"max_pose_landing_recovery_flip_degrees\"")
    );
    assert!(
        passing_summary
            .to_json()
            .contains("\"max_pose_landing_forward_fold_degrees\"")
    );
    assert!(
        passing_summary
            .to_json()
            .contains("\"max_pose_landing_backward_bend_degrees\"")
    );

    let mut excessive_split_accumulator = EvalAccumulator::default();
    excessive_split_accumulator.observe(excessive_split_anticipation);
    excessive_split_accumulator.observe(excessive_split_recovery);
    let excessive_split_summary = excessive_split_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert!(named_check(&excessive_split_summary, "pose_landing_foot_split").passed);
    assert!(!named_check(&excessive_split_summary, "pose_landing_foot_split_max").passed);
    assert_eq!(
        excessive_split_summary
            .metrics
            .max_pose_landing_foot_split_m,
        LANDING_MIN_POSE_FOOT_SPLIT_M
    );
    assert_eq!(
        excessive_split_summary
            .metrics
            .max_pose_landing_distal_foot_split_m,
        LANDING_MAX_FOOT_SPLIT_READABILITY_M + 0.01
    );
}

#[test]
fn accumulator_rejects_landing_anticipation_while_gliding() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MIN_POSE_FLARE_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 64.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: LANDING_MIN_POSE_FOOT_FORWARD_M,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_backward_bend(-LANDING_MIN_POSE_FORWARD_FOLD_DEGREES);
    sample.mode = FlightMode::Gliding.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let check = named_check(&summary, "gliding_landing_anticipation_samples");

    assert_eq!(summary.metrics.gliding_landing_anticipation_samples, 1);
    assert_eq!(check.value, 1.0);
    assert_eq!(check.threshold, 0.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_gates_landing_backward_bend_even_with_readable_pitch() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MIN_POSE_FLARE_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 64.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: LANDING_MIN_POSE_FOOT_FORWARD_M,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_backward_bend(LANDING_MAX_POSE_BACKWARD_BEND_DEGREES + 6.0);
    sample.mode = FlightMode::Airborne.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let backward_bend_check = named_check(&summary, "pose_landing_backward_bend");

    assert_eq!(
        summary.metrics.max_pose_landing_backward_bend_degrees,
        LANDING_MAX_POSE_BACKWARD_BEND_DEGREES + 6.0
    );
    assert_eq!(
        backward_bend_check.threshold,
        LANDING_MAX_POSE_BACKWARD_BEND_DEGREES
    );
    assert!(!backward_bend_check.passed);
}

#[test]
fn accumulator_gates_landing_transition_backbend_even_during_grace() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MIN_POSE_FLARE_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 64.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: LANDING_MIN_POSE_FOOT_FORWARD_M,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_local_bend(LANDING_MAX_POSE_TRANSITION_BACKBEND_DEGREES + 8.0)
    .with_key_pose_transition_grace(true);
    sample.mode = FlightMode::Airborne.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let transition_check = named_check(&summary, "pose_landing_transition_backbend");

    assert_eq!(
        summary.metrics.max_pose_landing_transition_backbend_degrees,
        LANDING_MAX_POSE_TRANSITION_BACKBEND_DEGREES + 8.0
    );
    assert_eq!(
        transition_check.threshold,
        LANDING_MAX_POSE_TRANSITION_BACKBEND_DEGREES
    );
    assert!(!transition_check.passed);
}

#[test]
fn accumulator_gates_landing_torso_offset() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: LANDING_MIN_POSE_FLARE_DEGREES,
        arm_spread_degrees: 0.0,
        leg_tuck_degrees: 64.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: LANDING_MIN_POSE_FOOT_FORWARD_M,
        landing_foot_split_m: LANDING_MIN_POSE_FOOT_SPLIT_M,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_torso_offset(LANDING_MAX_POSE_TORSO_OFFSET_M + 0.04);
    sample.mode = FlightMode::Airborne.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let torso_offset_check = named_check(&summary, "pose_landing_torso_offset");

    assert_eq!(
        summary.metrics.max_pose_landing_torso_offset_m,
        LANDING_MAX_POSE_TORSO_OFFSET_M + 0.04
    );
    assert_eq!(
        torso_offset_check.threshold,
        LANDING_MAX_POSE_TORSO_OFFSET_M
    );
    assert!(!torso_offset_check.passed);
}

#[test]
fn accumulator_gates_target_landing_pose_temporal_samples() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut non_landing_temporal_sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -4.0, -22.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
        visible_pose_part_count: 5,
        max_pose_part_rotation_delta_degrees: 24.0,
        max_pose_part_translation_delta_m: 0.12,
        min_pose_limb_clearance_m: 0.12,
        max_pose_limb_penetration_m: 0.0,
        max_pose_joint_gap_m: 0.0,
        pose_joint_gap_samples: 1,
    });
    non_landing_temporal_sample.pose_intent_label = "gliding";
    accumulator.observe(non_landing_temporal_sample);

    for (frame, pose_intent_label) in [(0, "landing_anticipation"), (1, "landing_recovery")] {
        let mut sample = air_control_metric_sample(
            scenario,
            frame + 1,
            Vec3::new(0.0, -2.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 42.0,
            arm_spread_degrees: 140.0,
            leg_tuck_degrees: 52.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.12,
            landing_foot_forward_m: 0.40,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        })
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: f32::NAN,
            max_pose_part_translation_delta_m: f32::NAN,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: 0.0,
            pose_joint_gap_samples: 1,
        });
        sample.mode = if pose_intent_label == "landing_anticipation" {
            FlightMode::Airborne.label()
        } else {
            FlightMode::Grounded.label()
        };
        sample.pose_intent_label = pose_intent_label;
        sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
        sample.target_distance_m = 0.0;
        sample.on_landing_target = true;
        accumulator.observe(sample);
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
    let temporal_sample_check = named_check(&summary, "landing_pose_temporal_stability_samples");

    assert_eq!(summary.metrics.pose_temporal_stability_samples, 1);
    assert_eq!(summary.metrics.landing_pose_temporal_stability_samples, 0);
    assert_eq!(temporal_sample_check.value, 0.0);
    assert_eq!(temporal_sample_check.threshold, 1.0);
    assert!(!temporal_sample_check.passed);
}

#[test]
fn accumulator_gates_target_landing_pose_temporal_jank() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 42.0,
        arm_spread_degrees: 140.0,
        leg_tuck_degrees: 52.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.12,
        landing_foot_forward_m: 0.40,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
        visible_pose_part_count: 5,
        max_pose_part_rotation_delta_degrees: 121.0,
        max_pose_part_translation_delta_m: 0.56,
        min_pose_limb_clearance_m: 0.12,
        max_pose_limb_penetration_m: 0.0,
        max_pose_joint_gap_m: 0.0,
        pose_joint_gap_samples: 1,
    });
    sample.mode = FlightMode::Airborne.label();
    sample.pose_intent_label = "landing_anticipation";
    sample = sample.with_authored_animation_metrics("land", "land", 1, 140);
    sample.target_distance_m = 0.0;
    sample.on_landing_target = true;
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
    let rotation_check = named_check(&summary, "landing_pose_part_rotation_delta");
    let translation_check = named_check(&summary, "landing_pose_part_translation_delta");

    assert_eq!(summary.metrics.pose_temporal_stability_samples, 1);
    assert_eq!(summary.metrics.landing_pose_temporal_stability_samples, 1);
    assert_eq!(
        summary.metrics.max_landing_pose_part_rotation_delta_degrees,
        121.0
    );
    assert_eq!(
        summary.metrics.max_landing_pose_part_translation_delta_m,
        0.56
    );
    assert_eq!(rotation_check.value, 121.0);
    assert_eq!(translation_check.value, 0.56);
    assert!(!rotation_check.passed);
    assert!(!translation_check.passed);
}

#[test]
fn accumulator_ignores_non_landing_pose_jank_for_landing_temporal_gates() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut gliding_jank_sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -4.0, -22.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
        visible_pose_part_count: 5,
        max_pose_part_rotation_delta_degrees: 150.0,
        max_pose_part_translation_delta_m: 0.8,
        min_pose_limb_clearance_m: 0.12,
        max_pose_limb_penetration_m: 0.0,
        max_pose_joint_gap_m: 0.0,
        pose_joint_gap_samples: 1,
    });
    gliding_jank_sample.pose_intent_label = "gliding";
    accumulator.observe(gliding_jank_sample);

    for (frame, pose_intent_label) in [(1, "landing_anticipation"), (2, "landing_recovery")] {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, -2.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 42.0,
            arm_spread_degrees: 140.0,
            leg_tuck_degrees: 52.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.12,
            landing_foot_forward_m: 0.40,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        })
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 20.0,
            max_pose_part_translation_delta_m: 0.1,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: 0.0,
            pose_joint_gap_samples: 1,
        });
        sample.pose_intent_label = pose_intent_label;
        sample.target_distance_m = 0.0;
        sample.on_landing_target = true;
        accumulator.observe(sample);
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

    assert_eq!(summary.metrics.pose_temporal_stability_samples, 3);
    assert_eq!(summary.metrics.max_pose_part_rotation_delta_degrees, 150.0);
    assert_eq!(summary.metrics.landing_pose_temporal_stability_samples, 2);
    assert_eq!(
        summary.metrics.max_landing_pose_part_rotation_delta_degrees,
        20.0
    );
    assert_eq!(
        summary.metrics.max_landing_pose_part_translation_delta_m,
        0.1
    );
    assert!(named_check(&summary, "landing_pose_part_rotation_delta").passed);
    assert!(named_check(&summary, "landing_pose_part_translation_delta").passed);
}

#[test]
fn accumulator_gates_air_control_pose_readability() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [0, 30, 60, 90] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            16.0,
            18.0,
            4.0,
        ));
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
    let air_brake_check = named_check(&summary, "air_control_pose_air_brake_samples");
    let air_turn_check = named_check(&summary, "air_control_pose_air_turn_samples");
    let dive_check = named_check(&summary, "air_control_pose_diving_samples");
    let gliding_dive_check = named_check(&summary, "air_control_gliding_dive_samples");
    let authored_air_brake_check =
        named_check(&summary, "air_control_authored_air_brake_clip_samples");
    let authored_dive_check = named_check(&summary, "air_control_authored_dive_clip_samples");
    let authored_glider_dive_response_check =
        named_check(&summary, "air_control_authored_glider_dive_response");
    let authored_glider_dive_motion_check =
        named_check(&summary, "air_control_authored_glider_dive_motion");

    assert_eq!(air_turn_check.value, 4.0);
    assert!(air_turn_check.passed);
    assert_eq!(air_brake_check.value, 0.0);
    assert_eq!(dive_check.value, 0.0);
    assert_eq!(gliding_dive_check.value, 0.0);
    assert_eq!(authored_air_brake_check.value, 0.0);
    assert_eq!(authored_dive_check.value, 0.0);
    assert_eq!(authored_glider_dive_response_check.value, 0.0);
    assert_eq!(authored_glider_dive_motion_check.value, 0.0);
    assert!(!air_brake_check.passed);
    assert!(!dive_check.passed);
    assert!(!gliding_dive_check.passed);
    assert!(!authored_air_brake_check.passed);
    assert!(!authored_dive_check.passed);
    assert!(!authored_glider_dive_response_check.passed);
    assert!(!authored_glider_dive_motion_check.passed);
    for name in [
        "air_control_pose_torso_pitch",
        "air_control_pose_arm_spread",
        "air_control_pose_leg_tuck",
        "air_control_dive_pose_torso_pitch",
        "air_control_dive_pose_arm_spread",
        "air_control_dive_pose_leg_tuck",
        "air_control_pose_lateral_lean",
        "air_control_right_pose_lateral_lean",
        "air_control_left_pose_lateral_lean",
        "air_control_pose_wing_airflow",
    ] {
        let check = named_check(&summary, name);
        if name == "air_control_dive_pose_arm_spread" {
            assert!(check.value.is_infinite());
        } else {
            assert_eq!(check.value, 0.0);
        }
        assert!(!check.passed, "expected {name} to fail");
    }
}

#[test]
fn accumulator_counts_gliding_air_control_dive_pose_readability() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    let mut airborne_dive = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -18.0, -26.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 80.0,
        arm_spread_degrees: 180.0,
        leg_tuck_degrees: 72.0,
        lateral_lean_degrees: 0.0,
        signed_lateral_lean_degrees: 0.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    });
    airborne_dive.mode = FlightMode::Airborne.label();
    accumulator.observe(airborne_dive);

    accumulator.observe(
        air_control_metric_sample(
            scenario,
            30,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: AIR_CONTROL_MIN_DIVE_POSE_TORSO_PITCH_DEGREES,
            arm_spread_degrees: AIR_CONTROL_MAX_DIVE_POSE_ARM_SPREAD_DEGREES,
            leg_tuck_degrees: AIR_CONTROL_MIN_DIVE_POSE_LEG_TUCK_DEGREES,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
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

    assert_eq!(
        summary.metrics.max_dive_pose_torso_pitch_degrees,
        AIR_CONTROL_MIN_DIVE_POSE_TORSO_PITCH_DEGREES
    );
    assert_eq!(
        summary.metrics.max_dive_pose_arm_spread_degrees,
        AIR_CONTROL_MAX_DIVE_POSE_ARM_SPREAD_DEGREES
    );
    assert_eq!(
        summary.metrics.max_dive_pose_leg_tuck_degrees,
        AIR_CONTROL_MIN_DIVE_POSE_LEG_TUCK_DEGREES
    );
    assert!(named_check(&summary, "air_control_dive_pose_torso_pitch").passed);
    assert!(named_check(&summary, "air_control_dive_pose_arm_spread").passed);
    assert!(named_check(&summary, "air_control_dive_pose_leg_tuck").passed);
}

#[test]
fn accumulator_gates_authored_clip_mismatch_for_air_control() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -18.0, -26.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    );
    sample = sample.with_authored_animation_metrics("glide", "dive", 1, 140);

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
    let mismatch_check = named_check(&summary, "air_control_authored_clip_mismatch_samples");

    assert_eq!(summary.metrics.authored_clip_match_samples, 0);
    assert_eq!(summary.metrics.authored_clip_mismatch_samples, 1);
    assert_eq!(summary.metrics.authored_dive_clip_samples, 0);
    assert_eq!(mismatch_check.value, 1.0);
    assert!(!mismatch_check.passed);
}

#[test]
fn accumulator_does_not_count_active_authored_transition_as_settled_clip_coverage() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = content_metric_sample(scenario, 5, 20, 0, 96);
    sample.mode = FlightMode::Launching.label();
    sample.pose_intent_label = "launching";
    sample.key_pose_readability_score = 1.0;
    sample = sample
        .with_authored_animation_metrics("launch", "launch", 1, 40)
        .with_authored_animation_transition_metrics("idle", "launch", true, 20, 0.5, "urgent_pose");
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

    assert_eq!(summary.metrics.authored_transition_active_samples, 1);
    assert_eq!(summary.metrics.max_authored_transition_elapsed_ms, 20);
    assert_eq!(summary.metrics.max_authored_transition_duration_ms, 40);
    assert_eq!(summary.metrics.max_authored_transition_progress, 0.5);
    assert_eq!(summary.metrics.authored_clip_match_samples, 0);
    assert_eq!(summary.metrics.authored_launch_clip_samples, 0);
    assert!(!named_check(&summary, "pose_state_authored_launch_clip_samples").passed);
}

#[test]
fn accumulator_gates_one_sided_authored_bank_clip_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [0, 30, 60, 90] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            16.0,
            18.0,
            4.0,
        ));
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
    let left_bank_check = named_check(&summary, "air_control_authored_bank_left_clip_samples");
    let right_bank_check = named_check(&summary, "air_control_authored_bank_right_clip_samples");

    assert_eq!(summary.metrics.authored_bank_left_clip_samples, 0);
    assert_eq!(summary.metrics.authored_bank_right_clip_samples, 4);
    assert_eq!(left_bank_check.value, 0.0);
    assert_eq!(right_bank_check.value, 4.0);
    assert!(!left_bank_check.passed);
    assert!(right_bank_check.passed);
}

#[test]
fn accumulator_gates_missing_authored_bank_clip_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, 0.0), Vec3::new(16.0, -2.0, -18.0)),
        (30, Vec2::new(-1.0, 0.0), Vec3::new(-16.0, -2.0, -18.0)),
    ] {
        accumulator.observe(
            air_control_metric_sample(scenario, frame, velocity, input, 16.0, 18.0, 4.0)
                .with_authored_animation_metrics("glide", "glide", 1, 140),
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
    let left_bank_check = named_check(&summary, "air_control_authored_bank_left_clip_samples");
    let right_bank_check = named_check(&summary, "air_control_authored_bank_right_clip_samples");

    assert_eq!(summary.metrics.authored_clip_match_samples, 2);
    assert_eq!(summary.metrics.authored_clip_mismatch_samples, 0);
    assert_eq!(summary.metrics.authored_bank_left_clip_samples, 0);
    assert_eq!(summary.metrics.authored_bank_right_clip_samples, 0);
    assert_eq!(left_bank_check.value, 0.0);
    assert_eq!(right_bank_check.value, 0.0);
    assert!(!left_bank_check.passed);
    assert!(!right_bank_check.passed);
}

#[test]
fn accumulator_gates_authored_glider_response() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(24.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        24.0,
        18.0,
        4.0,
    );
    sample.authored_glider_response_degrees = 0.0;
    sample.authored_glider_motion_m = 0.0;

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
    let check = named_check(&summary, "air_control_authored_glider_response");

    assert_eq!(summary.metrics.max_authored_glider_response_degrees, 0.0);
    assert_eq!(summary.metrics.max_authored_glider_motion_m, 0.0);
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");
    assert_eq!(
        summary_json["metrics"]["max_authored_glider_response_degrees"],
        0.0
    );
    assert_eq!(summary_json["metrics"]["max_authored_glider_motion_m"], 0.0);
    assert_eq!(check.value, 0.0);
    assert!(!check.passed);
}

#[test]
fn accumulator_gates_authored_glider_dive_response() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut sample = air_control_metric_sample(
        scenario,
        0,
        Vec3::new(0.0, -18.0, -26.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    );
    sample.authored_glider_response_degrees = 0.0;
    sample.authored_glider_motion_m = 0.0;

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
    let response_check = named_check(&summary, "air_control_authored_glider_dive_response");
    let motion_check = named_check(&summary, "air_control_authored_glider_dive_motion");

    assert_eq!(summary.metrics.gliding_dive_samples, 1);
    assert_eq!(
        summary.metrics.max_authored_glider_dive_response_degrees,
        0.0
    );
    assert_eq!(summary.metrics.max_authored_glider_dive_motion_m, 0.0);
    assert_eq!(response_check.value, 0.0);
    assert_eq!(motion_check.value, 0.0);
    assert!(!response_check.passed);
    assert!(!motion_check.passed);
}

#[test]
fn accumulator_gates_missing_air_control_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [0, 30, 60, 90] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, -2.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        ));
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
    let air_turn_check = named_check(&summary, "air_control_pose_air_turn_samples");

    assert_eq!(summary.metrics.pose_air_turn_samples, 0);
    assert_eq!(air_turn_check.value, 0.0);
    assert!(!air_turn_check.passed);
}

#[test]
fn accumulator_gates_missing_directional_air_control_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in [0, 30, 60, 90] {
        accumulator.observe(air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            16.0,
            18.0,
            4.0,
        ));
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
    let air_turn_check = named_check(&summary, "air_control_pose_air_turn_samples");
    let right_air_turn_check = named_check(&summary, "air_control_right_pose_air_turn_samples");
    let left_air_turn_check = named_check(&summary, "air_control_left_pose_air_turn_samples");

    assert_eq!(summary.metrics.pose_air_turn_samples, 4);
    assert_eq!(summary.metrics.right_pose_air_turn_samples, 4);
    assert_eq!(summary.metrics.left_pose_air_turn_samples, 0);
    assert_eq!(air_turn_check.value, 4.0);
    assert_eq!(right_air_turn_check.value, 4.0);
    assert_eq!(left_air_turn_check.value, 0.0);
    assert!(air_turn_check.passed);
    assert!(right_air_turn_check.passed);
    assert!(!left_air_turn_check.passed);
}

#[test]
fn accumulator_counts_bidirectional_air_control_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, 0.0), Vec3::new(16.0, -2.0, -18.0)),
        (30, Vec2::new(1.0, 0.0), Vec3::new(16.0, -2.0, -18.0)),
        (60, Vec2::new(1.0, 0.0), Vec3::new(16.0, -2.0, -18.0)),
        (90, Vec2::new(1.0, 0.0), Vec3::new(16.0, -2.0, -18.0)),
        (120, Vec2::new(-1.0, 0.0), Vec3::new(-16.0, -2.0, -18.0)),
        (150, Vec2::new(-1.0, 0.0), Vec3::new(-16.0, -2.0, -18.0)),
        (180, Vec2::new(-1.0, 0.0), Vec3::new(-16.0, -2.0, -18.0)),
        (210, Vec2::new(-1.0, 0.0), Vec3::new(-16.0, -2.0, -18.0)),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 16.0, 18.0, 4.0,
        ));
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
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");

    assert_eq!(summary.metrics.pose_air_turn_samples, 8);
    assert_eq!(summary.metrics.right_pose_air_turn_samples, 4);
    assert_eq!(summary.metrics.left_pose_air_turn_samples, 4);
    assert!(named_check(&summary, "air_control_pose_air_turn_samples").passed);
    assert!(named_check(&summary, "air_control_right_pose_air_turn_samples").passed);
    assert!(named_check(&summary, "air_control_left_pose_air_turn_samples").passed);
    assert_eq!(summary_json["metrics"]["pose_air_turn_samples"], 8);
    assert_eq!(summary_json["metrics"]["right_pose_air_turn_samples"], 4);
    assert_eq!(summary_json["metrics"]["left_pose_air_turn_samples"], 4);
}

#[test]
fn accumulator_counts_pure_air_turn_sideways_alignment_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity, roll) in [
        (0, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0), 4.0),
        (30, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0), 4.0),
        (60, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0), 4.0),
        (90, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0), 4.0),
        (120, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0), 4.0),
        (
            150,
            Vec2::new(-1.0, 0.0),
            Vec3::new(-18.0, -2.0, -18.0),
            -4.0,
        ),
        (
            180,
            Vec2::new(-1.0, 0.0),
            Vec3::new(-18.0, -2.0, -18.0),
            -4.0,
        ),
        (
            210,
            Vec2::new(-1.0, 0.0),
            Vec3::new(-18.0, -2.0, -18.0),
            -4.0,
        ),
        (
            240,
            Vec2::new(-1.0, 0.0),
            Vec3::new(-18.0, -2.0, -18.0),
            -4.0,
        ),
        (
            270,
            Vec2::new(-1.0, 0.0),
            Vec3::new(-18.0, -2.0, -18.0),
            -4.0,
        ),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 18.0, 18.0, roll,
        ));
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
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");

    assert_eq!(summary.metrics.pure_air_turn_sideways_sample_count, 8);
    assert_eq!(summary.metrics.right_pure_air_turn_sideways_sample_count, 4);
    assert_eq!(summary.metrics.left_pure_air_turn_sideways_sample_count, 4);
    assert!(named_check(&summary, "air_control_pure_air_turn_sideways_samples").passed);
    assert!(named_check(&summary, "air_control_right_pure_air_turn_sideways_samples").passed);
    assert!(named_check(&summary, "air_control_left_pure_air_turn_sideways_samples").passed);
    assert!(
        named_check(
            &summary,
            "air_control_p95_pure_air_turn_sideways_body_travel_heading_error"
        )
        .passed
    );
    assert!(
        named_check(
            &summary,
            "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error"
        )
        .passed
    );
    assert_eq!(
        summary_json["metrics"]["pure_air_turn_sideways_sample_count"],
        8
    );
    assert_eq!(
        summary_json["metrics"]["right_pure_air_turn_sideways_sample_count"],
        4
    );
    assert_eq!(
        summary_json["metrics"]["left_pure_air_turn_sideways_sample_count"],
        4
    );
}

#[test]
fn accumulator_requires_air_turn_sideways_alignment_in_same_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, 1.0), Vec3::new(18.0, -2.0, -18.0)),
        (30, Vec2::new(1.0, 1.0), Vec3::new(18.0, -2.0, -18.0)),
        (60, Vec2::new(1.0, 1.0), Vec3::new(18.0, -2.0, -18.0)),
        (90, Vec2::new(1.0, 1.0), Vec3::new(18.0, -2.0, -18.0)),
        (120, Vec2::new(-1.0, 1.0), Vec3::new(-18.0, -2.0, -18.0)),
        (150, Vec2::new(-1.0, 1.0), Vec3::new(-18.0, -2.0, -18.0)),
        (180, Vec2::new(-1.0, 1.0), Vec3::new(-18.0, -2.0, -18.0)),
        (210, Vec2::new(-1.0, 1.0), Vec3::new(-18.0, -2.0, -18.0)),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 18.0, 18.0, 4.0,
        ));
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

    assert!(named_check(&summary, "air_control_pose_air_turn_samples").passed);
    assert!(named_check(&summary, "air_control_lateral_body_travel_heading_samples").passed);
    assert!(named_check(&summary, "air_control_desired_travel_heading_samples").passed);
    assert_eq!(summary.metrics.pure_air_turn_sideways_sample_count, 0);
    assert!(!named_check(&summary, "air_control_pure_air_turn_sideways_samples").passed);
    assert!(!named_check(&summary, "air_control_right_pure_air_turn_sideways_samples").passed);
    assert!(!named_check(&summary, "air_control_left_pure_air_turn_sideways_samples").passed);
}

#[test]
fn accumulator_gates_pure_air_turn_sideways_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (30, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (60, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (90, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (120, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (150, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (180, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (210, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (240, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (270, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 18.0, 18.0, 48.0,
        ));
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

    assert_eq!(summary.metrics.pure_air_turn_sideways_sample_count, 8);
    assert!(
        !named_check(
            &summary,
            "air_control_p95_pure_air_turn_sideways_body_travel_heading_error"
        )
        .passed
    );
    assert!(
        !named_check(
            &summary,
            "air_control_max_pure_air_turn_sideways_body_travel_heading_error"
        )
        .passed
    );
    assert!(
        !named_check(
            &summary,
            "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error"
        )
        .passed
    );
    assert!(
        !named_check(
            &summary,
            "air_control_max_pure_air_turn_sideways_desired_travel_heading_error"
        )
        .passed
    );
}

#[test]
fn accumulator_counts_bidirectional_air_control_air_brake_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, -1.0), Vec3::new(16.0, -2.0, -18.0)),
        (30, Vec2::new(1.0, -1.0), Vec3::new(16.0, -2.0, -18.0)),
        (60, Vec2::new(1.0, -1.0), Vec3::new(16.0, -2.0, -18.0)),
        (90, Vec2::new(1.0, -1.0), Vec3::new(16.0, -2.0, -18.0)),
        (120, Vec2::new(-1.0, -1.0), Vec3::new(-16.0, -2.0, -18.0)),
        (150, Vec2::new(-1.0, -1.0), Vec3::new(-16.0, -2.0, -18.0)),
        (180, Vec2::new(-1.0, -1.0), Vec3::new(-16.0, -2.0, -18.0)),
        (210, Vec2::new(-1.0, -1.0), Vec3::new(-16.0, -2.0, -18.0)),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 16.0, 18.0, 4.0,
        ));
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
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary.to_json()).expect("summary json parses");

    assert_eq!(summary.metrics.pose_air_brake_samples, 8);
    assert_eq!(summary.metrics.right_pose_air_brake_samples, 4);
    assert_eq!(summary.metrics.left_pose_air_brake_samples, 4);
    assert_eq!(summary.metrics.backward_right_pose_air_brake_samples, 4);
    assert_eq!(summary.metrics.backward_left_pose_air_brake_samples, 4);
    assert!(named_check(&summary, "air_control_pose_air_brake_samples").passed);
    assert!(named_check(&summary, "air_control_right_pose_air_brake_samples").passed);
    assert!(named_check(&summary, "air_control_left_pose_air_brake_samples").passed);
    assert!(
        named_check(
            &summary,
            "air_control_backward_right_pose_air_brake_samples"
        )
        .passed
    );
    assert!(named_check(&summary, "air_control_backward_left_pose_air_brake_samples").passed);
    assert_eq!(summary_json["metrics"]["pose_air_brake_samples"], 8);
    assert_eq!(summary_json["metrics"]["right_pose_air_brake_samples"], 4);
    assert_eq!(summary_json["metrics"]["left_pose_air_brake_samples"], 4);
    assert_eq!(
        summary_json["metrics"]["backward_right_pose_air_brake_samples"],
        4
    );
    assert_eq!(
        summary_json["metrics"]["backward_left_pose_air_brake_samples"],
        4
    );
}

#[test]
fn accumulator_gates_directional_air_control_air_brake_pose_lateral_lean() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut matching_accumulator = EvalAccumulator::default();
    let mut wrong_signed_accumulator = EvalAccumulator::default();

    observe_directional_air_brake_pose_lean_samples(&mut matching_accumulator, scenario, -8.0, 8.0);
    observe_directional_air_brake_pose_lean_samples(
        &mut wrong_signed_accumulator,
        scenario,
        8.0,
        -8.0,
    );

    let matching_summary = matching_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let wrong_signed_summary = wrong_signed_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let matching_json: serde_json::Value =
        serde_json::from_str(&matching_summary.to_json()).expect("summary json parses");

    assert_eq!(
        matching_summary
            .metrics
            .max_backward_right_air_brake_pose_lateral_lean_degrees,
        8.0
    );
    assert_eq!(
        matching_summary
            .metrics
            .max_backward_left_air_brake_pose_lateral_lean_degrees,
        8.0
    );
    assert!(
        named_check(
            &matching_summary,
            "air_control_backward_right_pose_air_brake_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "air_control_backward_left_pose_air_brake_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "air_control_backward_right_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "air_control_backward_left_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert_eq!(
        matching_json["metrics"]["max_backward_right_air_brake_pose_lateral_lean_degrees"].as_f64(),
        Some(8.0)
    );
    assert_eq!(
        matching_json["metrics"]["max_backward_left_air_brake_pose_lateral_lean_degrees"].as_f64(),
        Some(8.0)
    );
    assert_eq!(
        wrong_signed_summary
            .metrics
            .max_backward_right_air_brake_pose_lateral_lean_degrees,
        0.0
    );
    assert_eq!(
        wrong_signed_summary
            .metrics
            .max_backward_left_air_brake_pose_lateral_lean_degrees,
        0.0
    );
    assert!(
        named_check(
            &wrong_signed_summary,
            "air_control_backward_right_pose_air_brake_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &wrong_signed_summary,
            "air_control_backward_left_pose_air_brake_samples"
        )
        .passed
    );
    assert!(
        !named_check(
            &wrong_signed_summary,
            "air_control_backward_right_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert!(
        !named_check(
            &wrong_signed_summary,
            "air_control_backward_left_air_brake_pose_lateral_lean"
        )
        .passed
    );
}

#[test]
fn accumulator_gates_pose_state_directional_air_brake_pose_lateral_lean() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut matching_accumulator = EvalAccumulator::default();
    let mut wrong_signed_accumulator = EvalAccumulator::default();

    observe_directional_air_brake_pose_lean_samples(&mut matching_accumulator, scenario, -8.0, 8.0);
    observe_directional_air_brake_pose_lean_samples(
        &mut wrong_signed_accumulator,
        scenario,
        8.0,
        -8.0,
    );

    let matching_summary = matching_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );
    let wrong_signed_summary = wrong_signed_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert!(
        named_check(
            &matching_summary,
            "pose_state_backward_right_diagonal_body_travel_heading_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "pose_state_backward_left_diagonal_body_travel_heading_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "pose_state_backward_right_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert!(
        named_check(
            &matching_summary,
            "pose_state_backward_left_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert!(
        named_check(
            &wrong_signed_summary,
            "pose_state_backward_right_diagonal_body_travel_heading_samples"
        )
        .passed
    );
    assert!(
        named_check(
            &wrong_signed_summary,
            "pose_state_backward_left_diagonal_body_travel_heading_samples"
        )
        .passed
    );
    assert!(
        !named_check(
            &wrong_signed_summary,
            "pose_state_backward_right_air_brake_pose_lateral_lean"
        )
        .passed
    );
    assert!(
        !named_check(
            &wrong_signed_summary,
            "pose_state_backward_left_air_brake_pose_lateral_lean"
        )
        .passed
    );
}

fn observe_directional_air_brake_pose_lean_samples(
    accumulator: &mut EvalAccumulator,
    scenario: EvalScenario,
    right_signed_lean_degrees: f32,
    left_signed_lean_degrees: f32,
) {
    for sample_index in 0..4 {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                sample_index * 30,
                Vec3::new(16.0, -2.0, -18.0),
                Vec2::new(1.0, -1.0),
                16.0,
                18.0,
                4.0,
            )
            .with_pose_readability_metrics(air_brake_pose_readability_metrics(
                right_signed_lean_degrees,
            )),
        );
    }
    for sample_index in 0..4 {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                120 + sample_index * 30,
                Vec3::new(-16.0, -2.0, -18.0),
                Vec2::new(-1.0, -1.0),
                16.0,
                18.0,
                4.0,
            )
            .with_pose_readability_metrics(air_brake_pose_readability_metrics(
                left_signed_lean_degrees,
            )),
        );
    }
}

fn air_brake_pose_readability_metrics(
    signed_lateral_lean_degrees: f32,
) -> EvalPoseReadabilityMetrics {
    EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 30.0,
        arm_spread_degrees: 120.0,
        leg_tuck_degrees: 40.0,
        lateral_lean_degrees: signed_lateral_lean_degrees.abs(),
        signed_lateral_lean_degrees,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.35,
        key_pose_readability_score: 1.0,
    }
}

#[test]
fn accumulator_rejects_unreadable_air_control_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        air_control_metric_sample(
            scenario,
            90,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(1.0, 0.0),
            16.0,
            18.0,
            4.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 4.0,
            arm_spread_degrees: 12.0,
            leg_tuck_degrees: 2.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 0.25,
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
    let air_turn_check = named_check(&summary, "air_control_pose_air_turn_samples");
    let unreadable_check = named_check(&summary, "air_control_unreadable_key_pose_samples");

    assert_eq!(summary.metrics.pose_air_turn_samples, 0);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 1);
    assert_eq!(air_turn_check.value, 0.0);
    assert_eq!(unreadable_check.value, 1.0);
    assert!(!air_turn_check.passed);
    assert!(!unreadable_check.passed);
}

#[test]
fn accumulator_rejects_unreadable_air_control_air_brake_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(
        air_control_metric_sample(
            scenario,
            90,
            Vec3::new(16.0, -2.0, -18.0),
            Vec2::new(1.0, -1.0),
            16.0,
            18.0,
            4.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 4.0,
            arm_spread_degrees: 12.0,
            leg_tuck_degrees: 2.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 0.25,
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
    let air_brake_check = named_check(&summary, "air_control_pose_air_brake_samples");
    let right_air_brake_check = named_check(&summary, "air_control_right_pose_air_brake_samples");
    let backward_right_air_brake_check = named_check(
        &summary,
        "air_control_backward_right_pose_air_brake_samples",
    );
    let unreadable_check = named_check(&summary, "air_control_unreadable_key_pose_samples");

    assert_eq!(summary.metrics.pose_air_brake_samples, 0);
    assert_eq!(summary.metrics.right_pose_air_brake_samples, 0);
    assert_eq!(summary.metrics.backward_right_pose_air_brake_samples, 0);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 1);
    assert_eq!(air_brake_check.value, 0.0);
    assert_eq!(right_air_brake_check.value, 0.0);
    assert_eq!(backward_right_air_brake_check.value, 0.0);
    assert_eq!(unreadable_check.value, 1.0);
    assert!(!air_brake_check.passed);
    assert!(!right_air_brake_check.passed);
    assert!(!backward_right_air_brake_check.passed);
    assert!(!unreadable_check.passed);
}

#[test]
fn accumulator_rejects_unreadable_key_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
            torso_pitch_degrees: 8.0,
            arm_spread_degrees: 18.0,
            leg_tuck_degrees: 4.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_foot_split_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 0.25,
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
    let dive_check = named_check(&summary, "air_control_pose_diving_samples");
    let unreadable_check = named_check(&summary, "air_control_unreadable_key_pose_samples");

    assert_eq!(summary.metrics.pose_diving_samples, 0);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 1);
    assert_eq!(dive_check.value, 0.0);
    assert_eq!(unreadable_check.value, 1.0);
    assert!(!dive_check.passed);
    assert!(!unreadable_check.passed);
}

#[test]
fn accumulator_rejects_excess_air_control_transition_grace_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..=AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES {
        accumulator.observe(
            air_control_metric_sample(
                scenario,
                120 + frame,
                Vec3::new(0.0, -18.0, -26.0),
                Vec2::ZERO,
                0.0,
                18.0,
                0.0,
            )
            .with_key_pose_transition_grace(true),
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
    let grace_check = named_check(&summary, "air_control_key_pose_transition_grace_samples");

    assert_eq!(
        summary.metrics.key_pose_transition_grace_samples,
        AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES + 1
    );
    assert_eq!(
        grace_check.threshold,
        AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32
    );
    assert!(!grace_check.passed);
}

#[test]
fn accumulator_gates_pose_state_coverage_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    observe_pose_state_samples(
        &mut accumulator,
        scenario,
        &[
            ("grounded_idle", FlightMode::Grounded.label(), 3),
            ("grounded_walk", FlightMode::Grounded.label(), 8),
            ("grounded_run", FlightMode::Grounded.label(), 8),
            ("launching", FlightMode::Launching.label(), 3),
            ("falling", FlightMode::Airborne.label(), 8),
            ("gliding", FlightMode::Gliding.label(), 18),
            ("air_turn", FlightMode::Gliding.label(), 12),
            ("air_brake", FlightMode::Gliding.label(), 8),
            ("diving", FlightMode::Gliding.label(), 1),
            ("landing_anticipation", FlightMode::Airborne.label(), 1),
            ("landing_recovery", FlightMode::Grounded.label(), 1),
        ],
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

    assert_eq!(summary.metrics.pose_grounded_idle_samples, 3);
    assert_eq!(summary.metrics.pose_grounded_walk_samples, 8);
    assert_eq!(summary.metrics.pose_grounded_run_samples, 8);
    assert!(
        summary.metrics.max_grounded_walk_stride_foot_travel_m
            >= GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M
    );
    assert!(
        summary.metrics.max_grounded_run_stride_foot_travel_m
            >= GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M
    );
    assert!(
        summary
            .metrics
            .max_grounded_walk_stride_leg_opposition_degrees
            >= GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES
    );
    assert!(
        summary
            .metrics
            .max_grounded_run_stride_leg_opposition_degrees
            >= GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES
    );
    assert_eq!(summary.metrics.pose_launching_samples, 3);
    assert_eq!(summary.metrics.pose_falling_samples, 8);
    assert_eq!(summary.metrics.authored_grounded_idle_clip_samples, 3);
    assert_eq!(summary.metrics.authored_grounded_walk_clip_samples, 8);
    assert_eq!(summary.metrics.authored_grounded_run_clip_samples, 8);
    assert_eq!(summary.metrics.authored_launch_clip_samples, 3);
    assert_eq!(summary.metrics.authored_glide_clip_samples, 18);
    assert_eq!(summary.metrics.authored_fall_clip_samples, 8);
    assert_eq!(summary.metrics.pose_gliding_samples, 18);
    assert_eq!(summary.metrics.pose_air_turn_samples, 12);
    assert_eq!(summary.metrics.pure_air_turn_sideways_sample_count, 8);
    assert_eq!(summary.metrics.authored_bank_right_clip_samples, 6);
    assert_eq!(summary.metrics.authored_bank_left_clip_samples, 6);
    assert_eq!(summary.metrics.right_pose_air_turn_samples, 6);
    assert_eq!(summary.metrics.left_pose_air_turn_samples, 6);
    assert_eq!(summary.metrics.pose_air_brake_samples, 8);
    assert_eq!(summary.metrics.authored_air_brake_clip_samples, 8);
    assert_eq!(
        summary
            .metrics
            .backward_diagonal_body_travel_heading_sample_count,
        8
    );
    assert_eq!(summary.metrics.pose_diving_samples, 1);
    assert_eq!(summary.metrics.gliding_dive_samples, 1);
    assert_eq!(summary.metrics.authored_dive_clip_samples, 1);
    assert_eq!(summary.metrics.pose_landing_anticipation_samples, 1);
    assert_eq!(summary.metrics.gliding_landing_anticipation_samples, 0);
    assert_eq!(summary.metrics.pose_landing_recovery_samples, 1);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 0);
    assert!(summary.metrics.pose_joint_gap_samples > 0);
    assert!(summary.to_json().contains("\"pose_grounded_idle_samples\""));
    assert!(summary.to_json().contains("\"pose_grounded_walk_samples\""));
    assert!(
        summary
            .to_json()
            .contains("\"authored_grounded_idle_clip_samples\": 3")
    );
    assert!(
        summary
            .to_json()
            .contains("\"authored_launch_clip_samples\": 3")
    );
    assert!(
        summary
            .to_json()
            .contains("\"authored_glide_clip_samples\": 18")
    );
    assert!(
        summary
            .to_json()
            .contains("\"authored_fall_clip_samples\": 8")
    );
    for name in [
        "pose_state_grounded_idle_samples",
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_walk_stride_foot_travel",
        "pose_state_run_stride_foot_travel",
        "pose_state_walk_stride_leg_opposition",
        "pose_state_run_stride_leg_opposition",
        "pose_state_authored_grounded_idle_clip_samples",
        "pose_state_authored_grounded_walk_clip_samples",
        "pose_state_authored_grounded_run_clip_samples",
        "pose_state_launching_samples",
        "pose_state_authored_launch_clip_samples",
        "pose_state_authored_glider_launch_samples",
        "pose_state_authored_glider_launch_response",
        "pose_state_authored_glider_launch_motion",
        "pose_state_falling_samples",
        "pose_state_authored_fall_clip_samples",
        "pose_state_gliding_samples",
        "pose_state_authored_glide_clip_samples",
        "pose_state_air_turn_samples",
        "pose_state_right_air_turn_samples",
        "pose_state_left_air_turn_samples",
        "pose_state_authored_bank_right_clip_samples",
        "pose_state_authored_bank_left_clip_samples",
        "pose_state_pure_air_turn_sideways_samples",
        "pose_state_right_pure_air_turn_sideways_samples",
        "pose_state_left_pure_air_turn_sideways_samples",
        "pose_state_p95_pure_air_turn_sideways_body_travel_heading_error",
        "pose_state_max_pure_air_turn_sideways_body_travel_heading_error",
        "pose_state_p95_pure_air_turn_sideways_desired_travel_heading_error",
        "pose_state_max_pure_air_turn_sideways_desired_travel_heading_error",
        "pose_state_air_brake_samples",
        "pose_state_authored_air_brake_clip_samples",
        "pose_state_backward_diagonal_body_travel_heading_samples",
        "pose_state_backward_right_diagonal_body_travel_heading_samples",
        "pose_state_backward_left_diagonal_body_travel_heading_samples",
        "pose_state_backward_right_air_brake_pose_lateral_lean",
        "pose_state_backward_left_air_brake_pose_lateral_lean",
        "pose_state_diving_samples",
        "pose_state_gliding_dive_samples",
        "pose_state_authored_dive_clip_samples",
        "pose_state_dive_pose_torso_pitch",
        "pose_state_dive_pose_arm_spread",
        "pose_state_dive_pose_leg_tuck",
        "pose_state_landing_anticipation_samples",
        "pose_state_gliding_landing_anticipation_samples",
        "pose_state_landing_recovery_samples",
        "pose_state_authored_land_clip_samples",
        "pose_state_landing_crouch",
        "pose_state_landing_foot_forward",
        "pose_state_landing_foot_split",
        "pose_state_landing_foot_split_max",
        "pose_state_landing_flare",
        "pose_state_landing_flare_backbend",
        "pose_state_landing_forward_fold",
        "pose_state_landing_backward_bend",
        "pose_state_landing_transition_backbend",
        "pose_state_landing_recovery_backbend",
        "pose_state_landing_torso_offset",
        "pose_state_landing_pose_temporal_samples",
        "pose_state_landing_pose_rotation_delta",
        "pose_state_landing_pose_translation_delta",
        "pose_state_unreadable_key_pose_samples",
        "pose_state_key_pose_transition_grace_samples",
        "pose_state_visible_pose_part_count",
        "pose_state_min_pose_limb_clearance",
        "pose_state_max_pose_limb_penetration",
        "pose_state_pose_joint_gap_samples",
        "pose_state_max_pose_joint_gap",
    ] {
        assert!(named_check(&summary, name).passed, "{name} should pass");
    }
    assert_eq!(
        named_check(&summary, "pose_state_right_air_turn_samples").threshold,
        POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES
    );
    assert_eq!(
        named_check(&summary, "pose_state_left_air_turn_samples").threshold,
        POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES
    );
}

#[test]
fn accumulator_gates_pose_state_launch_glider_takeout() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..3 {
        let mut sample = content_metric_sample(scenario, frame, 20, 0, 96);
        sample.mode = FlightMode::Launching.label();
        sample.pose_intent_label = "launching";
        sample.key_pose_readability_score = 1.0;
        sample = sample.with_authored_animation_metrics("launch", "launch", 1, 140);
        accumulator.observe(sample);
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

    assert!(named_check(&summary, "pose_state_launching_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_launch_clip_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_glider_launch_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_glider_launch_response").passed);
    assert!(!named_check(&summary, "pose_state_authored_glider_launch_motion").passed);
}

#[test]
fn accumulator_gates_pose_state_authored_bank_clip_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    let mut right_bank = content_metric_sample(scenario, 10, 20, 0, 96);
    right_bank.mode = FlightMode::Gliding.label();
    right_bank.pose_intent_label = "air_turn";
    right_bank.movement_input_lateral_axis = 1.0;
    right_bank.key_pose_readability_score = 1.0;
    right_bank = right_bank.with_authored_animation_metrics("bank_right", "bank_right", 1, 80);
    accumulator.observe(right_bank);

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

    let right_bank_check = named_check(&summary, "pose_state_authored_bank_right_clip_samples");
    let left_bank_check = named_check(&summary, "pose_state_authored_bank_left_clip_samples");

    assert_eq!(summary.metrics.authored_bank_right_clip_samples, 1);
    assert_eq!(summary.metrics.authored_bank_left_clip_samples, 0);
    assert!(right_bank_check.passed);
    assert!(!left_bank_check.passed);
}

#[test]
fn accumulator_gates_pose_state_authored_dive_and_air_brake_clip_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..4 {
        let mut sample = content_metric_sample(scenario, 40 + frame, 20, 0, 96);
        sample.mode = FlightMode::Gliding.label();
        sample.pose_intent_label = "air_brake";
        sample.movement_input_lateral_axis = if frame.is_multiple_of(2) { 1.0 } else { -1.0 };
        sample.movement_input_forward_axis = -1.0;
        sample.key_pose_readability_score = 1.0;
        sample = sample.with_authored_animation_metrics("glide", "glide", 1, 140);
        accumulator.observe(sample);
    }

    let mut dive = content_metric_sample(scenario, 80, 20, 0, 96);
    dive.mode = FlightMode::Gliding.label();
    dive.pose_intent_label = "diving";
    dive.key_pose_readability_score = 1.0;
    dive = dive
        .with_pose_readability_metrics(pose_state_readability_metrics_for_label("diving"))
        .with_authored_animation_metrics("glide", "glide", 1, 140);
    accumulator.observe(dive);

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

    assert_eq!(summary.metrics.pose_air_brake_samples, 4);
    assert_eq!(summary.metrics.authored_air_brake_clip_samples, 0);
    assert_eq!(summary.metrics.pose_diving_samples, 1);
    assert_eq!(summary.metrics.authored_dive_clip_samples, 0);
    assert!(named_check(&summary, "pose_state_air_brake_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_air_brake_clip_samples").passed);
    assert!(named_check(&summary, "pose_state_diving_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_dive_clip_samples").passed);
}

#[test]
fn accumulator_gates_pose_state_sideways_air_turn_misalignment() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    for (frame, input, velocity) in [
        (0, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (30, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (60, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (90, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (120, Vec2::new(1.0, 0.0), Vec3::new(18.0, -2.0, -18.0)),
        (150, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (180, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (210, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (240, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
        (270, Vec2::new(-1.0, 0.0), Vec3::new(-18.0, -2.0, -18.0)),
    ] {
        accumulator.observe(air_control_metric_sample(
            scenario, frame, velocity, input, 18.0, 18.0, 48.0,
        ));
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

    assert_eq!(summary.metrics.pure_air_turn_sideways_sample_count, 8);
    for name in [
        "pose_state_p95_pure_air_turn_sideways_body_travel_heading_error",
        "pose_state_max_pure_air_turn_sideways_body_travel_heading_error",
        "pose_state_p95_pure_air_turn_sideways_desired_travel_heading_error",
        "pose_state_max_pure_air_turn_sideways_desired_travel_heading_error",
    ] {
        assert!(!named_check(&summary, name).passed, "{name} should fail");
    }
}

#[test]
fn accumulator_rejects_thin_pose_state_coverage_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    observe_pose_state_samples(
        &mut accumulator,
        scenario,
        &[
            ("grounded_idle", FlightMode::Grounded.label(), 2),
            ("grounded_walk", FlightMode::Grounded.label(), 7),
            ("grounded_run", FlightMode::Grounded.label(), 7),
            ("launching", FlightMode::Launching.label(), 2),
            ("falling", FlightMode::Airborne.label(), 7),
            ("gliding", FlightMode::Gliding.label(), 17),
            ("air_turn", FlightMode::Gliding.label(), 3),
            ("air_brake", FlightMode::Gliding.label(), 3),
        ],
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

    for name in [
        "pose_state_grounded_idle_samples",
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_authored_grounded_idle_clip_samples",
        "pose_state_authored_grounded_walk_clip_samples",
        "pose_state_authored_grounded_run_clip_samples",
        "pose_state_launching_samples",
        "pose_state_authored_launch_clip_samples",
        "pose_state_falling_samples",
        "pose_state_authored_fall_clip_samples",
        "pose_state_gliding_samples",
        "pose_state_authored_glide_clip_samples",
        "pose_state_air_turn_samples",
        "pose_state_right_air_turn_samples",
        "pose_state_left_air_turn_samples",
        "pose_state_pure_air_turn_sideways_samples",
        "pose_state_right_pure_air_turn_sideways_samples",
        "pose_state_left_pure_air_turn_sideways_samples",
        "pose_state_air_brake_samples",
        "pose_state_authored_air_brake_clip_samples",
        "pose_state_backward_diagonal_body_travel_heading_samples",
        "pose_state_backward_right_diagonal_body_travel_heading_samples",
        "pose_state_backward_left_diagonal_body_travel_heading_samples",
        "pose_state_diving_samples",
        "pose_state_gliding_dive_samples",
        "pose_state_authored_dive_clip_samples",
        "pose_state_dive_pose_torso_pitch",
        "pose_state_dive_pose_arm_spread",
        "pose_state_dive_pose_leg_tuck",
        "pose_state_landing_anticipation_samples",
        "pose_state_landing_recovery_samples",
        "pose_state_authored_land_clip_samples",
    ] {
        assert!(!named_check(&summary, name).passed, "{name} should fail");
    }
}

#[test]
fn accumulator_rejects_excess_pose_state_transition_grace_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();
    for frame in 0..=POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES {
        accumulator.observe(
            content_metric_sample(scenario, frame, 20, 0, 96).with_key_pose_transition_grace(true),
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
    let grace_check = named_check(&summary, "pose_state_key_pose_transition_grace_samples");

    assert_eq!(
        summary.metrics.key_pose_transition_grace_samples,
        POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES + 1
    );
    assert_eq!(
        grace_check.threshold,
        POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32
    );
    assert!(!grace_check.passed);
}

#[test]
fn accumulator_rejects_static_grounded_pose_state_stride() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    observe_pose_state_samples_with_grounded_stride(
        &mut accumulator,
        scenario,
        &[
            ("grounded_idle", FlightMode::Grounded.label(), 3),
            ("grounded_walk", FlightMode::Grounded.label(), 8),
            ("grounded_run", FlightMode::Grounded.label(), 8),
            ("launching", FlightMode::Launching.label(), 3),
            ("falling", FlightMode::Airborne.label(), 8),
            ("gliding", FlightMode::Gliding.label(), 18),
            ("air_turn", FlightMode::Gliding.label(), 6),
            ("air_brake", FlightMode::Gliding.label(), 4),
            ("diving", FlightMode::Gliding.label(), 1),
            ("landing_anticipation", FlightMode::Gliding.label(), 1),
            ("landing_recovery", FlightMode::Grounded.label(), 1),
        ],
        false,
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

    assert!(named_check(&summary, "pose_state_grounded_idle_samples").passed);
    assert!(named_check(&summary, "pose_state_grounded_walk_samples").passed);
    assert!(named_check(&summary, "pose_state_grounded_run_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_grounded_idle_clip_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_grounded_walk_clip_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_grounded_run_clip_samples").passed);
    for name in [
        "pose_state_walk_stride_foot_travel",
        "pose_state_run_stride_foot_travel",
        "pose_state_walk_stride_leg_opposition",
        "pose_state_run_stride_leg_opposition",
    ] {
        assert!(!named_check(&summary, name).passed, "{name} should fail");
    }
}

#[test]
fn accumulator_gates_missing_authored_fall_clip_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    observe_pose_state_samples(
        &mut accumulator,
        scenario,
        &[
            ("grounded_idle", FlightMode::Grounded.label(), 3),
            ("grounded_walk", FlightMode::Grounded.label(), 8),
            ("grounded_run", FlightMode::Grounded.label(), 8),
            ("launching", FlightMode::Launching.label(), 3),
            ("gliding", FlightMode::Gliding.label(), 18),
        ],
    );

    for frame in 0..8 {
        let mut sample = content_metric_sample(scenario, 300 + frame, 20, 0, 96);
        sample.mode = FlightMode::Airborne.label();
        sample.pose_intent_label = "falling";
        sample.key_pose_readability_score = 1.0;
        sample = sample.with_authored_animation_metrics("air_brake", "air_brake", 1, 140);
        accumulator.observe(sample);
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

    assert_eq!(summary.metrics.pose_falling_samples, 8);
    assert_eq!(summary.metrics.authored_fall_clip_samples, 0);
    assert!(named_check(&summary, "pose_state_falling_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_fall_clip_samples").passed);
}

#[test]
fn accumulator_gates_missing_authored_grounded_pose_clip_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    for (pose_intent_label, clip_label, count) in [
        ("grounded_idle", "walk", 3),
        ("grounded_walk", "walk", 8),
        ("grounded_run", "walk", 8),
    ] {
        for frame_offset in 0..count {
            let mut sample = content_metric_sample(scenario, 400 + frame_offset, 20, 0, 96);
            sample.mode = FlightMode::Grounded.label();
            sample.pose_intent_label = pose_intent_label;
            sample.key_pose_readability_score = 1.0;
            if pose_intent_label == "grounded_walk" {
                sample.pose_grounded_stride_foot_travel_m =
                    GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M + 0.04;
                sample.pose_grounded_stride_leg_opposition_degrees =
                    GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES + 6.0;
            } else if pose_intent_label == "grounded_run" {
                sample.pose_grounded_stride_foot_travel_m =
                    GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M + 0.04;
                sample.pose_grounded_stride_leg_opposition_degrees =
                    GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES + 6.0;
            }
            accumulator
                .observe(sample.with_authored_animation_metrics(clip_label, clip_label, 1, 140));
        }
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

    assert!(named_check(&summary, "pose_state_grounded_idle_samples").passed);
    assert!(named_check(&summary, "pose_state_grounded_walk_samples").passed);
    assert!(named_check(&summary, "pose_state_grounded_run_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_grounded_idle_clip_samples").passed);
    assert!(named_check(&summary, "pose_state_authored_grounded_walk_clip_samples").passed);
    assert!(!named_check(&summary, "pose_state_authored_grounded_run_clip_samples").passed);
}

fn observe_pose_state_samples(
    accumulator: &mut EvalAccumulator,
    scenario: EvalScenario,
    samples: &[(&'static str, &'static str, u32)],
) {
    observe_pose_state_samples_with_grounded_stride(accumulator, scenario, samples, true);
}

fn observe_pose_state_samples_with_grounded_stride(
    accumulator: &mut EvalAccumulator,
    scenario: EvalScenario,
    samples: &[(&'static str, &'static str, u32)],
    include_grounded_stride_metrics: bool,
) {
    let mut frame = 10;
    for &(pose_intent_label, mode, count) in samples {
        for sample_index in 0..count {
            let mut sample = content_metric_sample(scenario, frame, 20, 0, 96);
            sample.mode = mode;
            sample.pose_intent_label = pose_intent_label;
            sample.key_pose_readability_score = 1.0;
            let movement_axis =
                pose_state_movement_axis_for_label(pose_intent_label, sample_index, count);
            sample.movement_input_lateral_axis = movement_axis.x;
            sample.movement_input_forward_axis = movement_axis.y;
            sample.lateral_input_active = movement_axis.x.abs() > f32::EPSILON;
            sample.body_roll_degrees = -movement_axis.x.signum() * 12.0;
            sample.lateral_response_mps = movement_axis.x.abs() * 18.0;
            let mut readability_metrics =
                pose_state_readability_metrics_for_label(pose_intent_label);
            if movement_axis.x.abs() > f32::EPSILON {
                readability_metrics.lateral_lean_degrees = readability_metrics
                    .lateral_lean_degrees
                    .max(AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES);
                readability_metrics.signed_lateral_lean_degrees =
                    -movement_axis.x.signum() * AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES;
            }
            sample = sample.with_pose_readability_metrics(readability_metrics);
            if pose_intent_label == "landing_anticipation" {
                sample =
                    sample.with_pose_torso_backward_bend(-LANDING_MIN_POSE_FORWARD_FOLD_DEGREES);
            }
            sample = sample.with_pose_temporal_metrics(EvalPoseTemporalMetrics {
                visible_pose_part_count: MIN_VISIBLE_POSE_PART_COUNT,
                max_pose_part_rotation_delta_degrees: 8.0,
                max_pose_part_translation_delta_m: 0.02,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            });
            if include_grounded_stride_metrics
                && matches!(pose_intent_label, "grounded_walk" | "grounded_run")
            {
                let (foot_travel_m, leg_opposition_degrees) = match pose_intent_label {
                    "grounded_walk" => (
                        GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M + 0.04,
                        GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES + 6.0,
                    ),
                    _ => (
                        GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M + 0.04,
                        GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES + 6.0,
                    ),
                };
                sample.pose_grounded_stride_foot_travel_m = foot_travel_m;
                sample.pose_grounded_stride_leg_opposition_degrees = leg_opposition_degrees;
            }
            if sample.mode == FlightMode::Gliding.label() && pose_intent_label == "diving" {
                sample = sample.with_authored_glider_metrics(
                    AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES,
                    0.08,
                );
            } else if pose_intent_label == "launching" {
                sample = sample.with_authored_glider_metrics(28.0, 0.62);
            }
            let authored_clip_label =
                authored_clip_label_for_pose_intent_label(pose_intent_label, movement_axis);
            sample = sample.with_authored_animation_metrics(
                authored_clip_label,
                authored_clip_label,
                1,
                140,
            );
            accumulator.observe(sample);
            frame += 5;
        }
    }
}

fn pose_state_movement_axis_for_label(
    pose_intent_label: &str,
    sample_index: u32,
    sample_count: u32,
) -> Vec2 {
    match pose_intent_label {
        "air_turn" if sample_index < sample_count / 2 => Vec2::new(1.0, 0.0),
        "air_turn" => Vec2::new(-1.0, 0.0),
        "air_brake" if sample_index.is_multiple_of(2) => Vec2::new(1.0, -1.0),
        "air_brake" => Vec2::new(-1.0, -1.0),
        "diving" => Vec2::new(0.0, 1.0),
        _ => Vec2::ZERO,
    }
}

fn pose_state_readability_metrics_for_label(pose_intent_label: &str) -> EvalPoseReadabilityMetrics {
    let mut metrics = EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 30.0,
        arm_spread_degrees: 120.0,
        leg_tuck_degrees: 40.0,
        lateral_lean_degrees: 8.0,
        signed_lateral_lean_degrees: 8.0,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 0.35,
        key_pose_readability_score: 1.0,
    };

    match pose_intent_label {
        "landing_anticipation" => {
            metrics.torso_pitch_degrees = LANDING_MIN_POSE_FLARE_DEGREES;
            metrics.landing_crouch_m = LANDING_MIN_POSE_CROUCH_M;
            metrics.landing_foot_forward_m = LANDING_MIN_POSE_FOOT_FORWARD_M;
            metrics.landing_foot_split_m = LANDING_MIN_POSE_FOOT_SPLIT_M;
        }
        "landing_recovery" => {
            metrics.landing_crouch_m = LANDING_MIN_POSE_CROUCH_M;
            metrics.landing_foot_split_m = LANDING_MIN_POSE_FOOT_SPLIT_M;
            metrics.landing_recovery_flip_degrees = LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES;
        }
        "diving" => {
            metrics.torso_pitch_degrees = AIR_CONTROL_MIN_DIVE_POSE_TORSO_PITCH_DEGREES;
            metrics.arm_spread_degrees = AIR_CONTROL_MAX_DIVE_POSE_ARM_SPREAD_DEGREES;
            metrics.leg_tuck_degrees = AIR_CONTROL_MIN_DIVE_POSE_LEG_TUCK_DEGREES;
        }
        _ => {}
    }

    metrics
}

#[test]
fn accumulator_gates_visible_pose_temporal_jank() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 150.0,
            max_pose_part_translation_delta_m: 0.8,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: 0.0,
            pose_joint_gap_samples: 1,
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
    let rotation_check = named_check(&summary, "air_control_max_pose_part_rotation_delta");
    let translation_check = named_check(&summary, "air_control_max_pose_part_translation_delta");
    let clearance_check = named_check(&summary, "air_control_min_pose_limb_clearance");
    let part_count_check = named_check(&summary, "air_control_visible_pose_part_count");
    let summary_json = summary.to_json();

    assert_eq!(summary.metrics.max_visible_pose_part_count, 5);
    assert_eq!(summary.metrics.pose_temporal_stability_samples, 1);
    assert_eq!(summary.metrics.max_pose_part_rotation_delta_degrees, 150.0);
    assert_eq!(summary.metrics.max_pose_part_translation_delta_m, 0.8);
    assert_eq!(summary.metrics.min_pose_limb_clearance_m, 0.12);
    assert!(!rotation_check.passed);
    assert!(!translation_check.passed);
    assert!(clearance_check.passed);
    assert!(!part_count_check.passed);
    assert!(summary_json.contains("\"max_pose_part_rotation_delta_degrees\": 150"));
    assert!(summary_json.contains("\"max_pose_part_translation_delta_m\": 0.8000"));
    assert!(summary_json.contains("\"min_pose_limb_clearance_m\": 0.1200"));
}

#[test]
fn accumulator_gates_visible_pose_limb_clearance() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 20.0,
            max_pose_part_translation_delta_m: 0.08,
            min_pose_limb_clearance_m: -0.02,
            max_pose_limb_penetration_m: 0.02,
            max_pose_joint_gap_m: 0.0,
            pose_joint_gap_samples: 1,
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
    let clearance_check = named_check(&summary, "air_control_min_pose_limb_clearance");
    let penetration_check = named_check(&summary, "air_control_max_pose_limb_penetration");

    assert_eq!(summary.metrics.min_pose_limb_clearance_m, -0.02);
    assert!(!clearance_check.passed);
    assert_eq!(summary.metrics.max_pose_limb_penetration_m, 0.02);
    assert!(!penetration_check.passed);
}

#[test]
fn accumulator_gates_visible_pose_joint_gap_samples_and_distance() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut missing_sample_accumulator = EvalAccumulator::default();
    missing_sample_accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 20.0,
            max_pose_part_translation_delta_m: 0.08,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: f32::NAN,
            pose_joint_gap_samples: 0,
        }),
    );
    let missing_summary = missing_sample_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert_eq!(missing_summary.metrics.pose_joint_gap_samples, 0);
    assert!(!named_check(&missing_summary, "air_control_pose_joint_gap_samples").passed);

    let mut gapped_accumulator = EvalAccumulator::default();
    gapped_accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 20.0,
            max_pose_part_translation_delta_m: 0.08,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: 0.14,
            pose_joint_gap_samples: 1,
        }),
    );
    let gapped_summary = gapped_accumulator.summary(
        scenario,
        EvalArtifacts {
            summary_json: "summary.json".to_string(),
            samples_ndjson: "samples.ndjson".to_string(),
            screenshot_png: None,
            checkpoint_screenshots: Vec::new(),
            checkpoint_marker_metadata: Vec::new(),
        },
    );

    assert_eq!(gapped_summary.metrics.max_pose_joint_gap_m, 0.14);
    assert!(named_check(&gapped_summary, "air_control_pose_joint_gap_samples").passed);
    assert!(!named_check(&gapped_summary, "air_control_max_pose_joint_gap").passed);
}

#[test]
fn accumulator_gates_missing_visible_pose_temporal_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("air control route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(
        air_control_metric_sample(
            scenario,
            120,
            Vec3::new(0.0, -18.0, -26.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        )
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: f32::NAN,
            max_pose_part_translation_delta_m: f32::NAN,
            min_pose_limb_clearance_m: 0.12,
            max_pose_limb_penetration_m: 0.0,
            max_pose_joint_gap_m: 0.0,
            pose_joint_gap_samples: 1,
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
    let sample_check = named_check(&summary, "air_control_pose_temporal_stability_samples");
    let rotation_check = named_check(&summary, "air_control_max_pose_part_rotation_delta");
    let translation_check = named_check(&summary, "air_control_max_pose_part_translation_delta");

    assert_eq!(summary.metrics.max_visible_pose_part_count, 5);
    assert_eq!(summary.metrics.pose_temporal_stability_samples, 0);
    assert!(!sample_check.passed);
    assert!(rotation_check.passed);
    assert!(translation_check.passed);
}

#[test]
fn accumulator_gates_dynamic_wind_flow_for_lift_routes() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..scenario.thresholds.min_lifted_samples {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, 8.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        );
        sample.active_lift_fields = 1;
        sample.readable_lift_fields = 1;
        sample.dynamic_wind_flow_fields = 2;
        sample.active_wind_force_fields = 2;
        sample.crosswind_force_fields = 1;
        sample.updraft_swirl_force_fields = 1;
        sample.max_wind_flow_speed_mps = 10.0;
        sample.max_wind_flow_variation = 0.16 + frame as f32 * 0.02;
        sample.max_wind_flow_direction_change_degrees = 8.0;
        accumulator.observe(sample);
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

    assert_eq!(
        summary.metrics.dynamic_readable_lift_samples,
        scenario.thresholds.min_lifted_samples
    );
    assert!(named_check(&summary, "dynamic_readable_lift_samples").passed);
    assert!(named_check(&summary, "max_wind_flow_speed").passed);
    assert!(named_check(&summary, "max_wind_flow_variation").passed);
    assert!(named_check(&summary, "max_wind_flow_direction_change").passed);
    assert!(named_check(&summary, "max_wind_flow_variation_range").passed);
    assert!(named_check(&summary, "updraft_swirl_force_samples").passed);
    assert!(named_check(&summary, "aligned_updraft_swirl_force_samples").passed);
    assert!(named_check(&summary, "updraft_swirl_force_fields").passed);
    assert!(named_check(&summary, "updraft_swirl_force_delta").passed);
    assert!(named_check(&summary, "updraft_swirl_force_flow_alignment").passed);
    assert!(named_check(&summary, "updraft_swirl_force_aligned_delta").passed);
    assert!(named_check(&summary, "layered_dynamic_wind_flow_fields").passed);
    assert!(named_check(&summary, "layered_wind_force_samples").passed);
    assert!(named_check(&summary, "aligned_layered_wind_force_samples").passed);
    assert!(named_check(&summary, "crosswind_updraft_overlap_samples").passed);
    assert!(named_check(&summary, "aligned_crosswind_updraft_overlap_samples").passed);
    assert!(named_check(&summary, "layered_wind_force_fields").passed);
    assert!(named_check(&summary, "layered_wind_force_delta").passed);
    assert!(named_check(&summary, "layered_wind_force_flow_alignment").passed);
    assert!(named_check(&summary, "layered_wind_force_aligned_delta").passed);
}

#[test]
fn accumulator_observe_for_scenario_uses_underbridge_updraft_swirl_delta() {
    let scenario = scenario_named(UNDERBRIDGE_UNDER_ROUTE).expect("underbridge route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96).with_wind_force_metrics(
        1,
        0,
        1,
        0.0,
        0.0,
        UNDER_ROUTE_MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
        6.0,
        0.16,
        1.0,
        0.0,
        1.0,
        0.0,
        0.0,
        UNDER_ROUTE_MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
    );

    let mut default_accumulator = EvalAccumulator::default();
    default_accumulator.observe(sample.clone());

    let mut scenario_accumulator = EvalAccumulator::default();
    scenario_accumulator.observe_for_scenario(sample, scenario);

    let artifacts = EvalArtifacts {
        summary_json: "summary.json".to_string(),
        samples_ndjson: "samples.ndjson".to_string(),
        screenshot_png: None,
        checkpoint_screenshots: Vec::new(),
        checkpoint_marker_metadata: Vec::new(),
    };
    let default_summary = default_accumulator.summary(scenario, artifacts.clone());
    let scenario_summary = scenario_accumulator.summary(scenario, artifacts);

    assert_eq!(default_summary.metrics.meaningful_wind_force_samples, 0);
    assert_eq!(
        default_summary.metrics.aligned_updraft_swirl_force_samples,
        0
    );
    assert_eq!(scenario_summary.metrics.meaningful_wind_force_samples, 1);
    assert_eq!(
        scenario_summary.metrics.aligned_updraft_swirl_force_samples,
        1
    );
}

#[test]
fn accumulator_rejects_missing_updraft_swirl_force_metrics_for_lift_routes() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..scenario.thresholds.min_lifted_samples {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, 8.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        );
        sample.active_lift_fields = 1;
        sample.readable_lift_fields = 1;
        sample.dynamic_wind_flow_fields = 1;
        sample.max_wind_flow_speed_mps = 10.0;
        sample.max_wind_flow_variation = 0.16 + frame as f32 * 0.02;
        sample.max_wind_flow_direction_change_degrees = 8.0;
        sample.updraft_swirl_force_fields = 0;
        sample.max_updraft_swirl_force_flow_alignment = 0.0;
        sample.max_updraft_swirl_force_aligned_delta_mps = 0.0;
        sample.max_updraft_swirl_force_delta_mps = 0.0;
        accumulator.observe(sample);
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

    assert!(named_check(&summary, "dynamic_readable_lift_samples").passed);
    assert!(!named_check(&summary, "updraft_swirl_force_samples").passed);
    assert!(!named_check(&summary, "aligned_updraft_swirl_force_samples").passed);
    assert!(!named_check(&summary, "updraft_swirl_force_fields").passed);
    assert!(!named_check(&summary, "updraft_swirl_force_delta").passed);
    assert!(!named_check(&summary, "updraft_swirl_force_flow_alignment").passed);
    assert!(!named_check(&summary, "updraft_swirl_force_aligned_delta").passed);
    assert!(!named_check(&summary, "crosswind_updraft_overlap_samples").passed);
    assert!(!named_check(&summary, "aligned_crosswind_updraft_overlap_samples").passed);
}

#[test]
fn accumulator_rejects_single_source_wind_force_for_layered_lift_routes() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..scenario.thresholds.min_lifted_samples {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, 8.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        );
        sample.active_lift_fields = 1;
        sample.readable_lift_fields = 1;
        sample.dynamic_wind_flow_fields = 2;
        sample.max_wind_flow_speed_mps = 10.0;
        sample.max_wind_flow_variation = 0.16 + frame as f32 * 0.02;
        sample.max_wind_flow_direction_change_degrees = 8.0;
        sample.active_wind_force_fields = 1;
        sample.crosswind_force_fields = 0;
        sample.updraft_swirl_force_fields = 1;
        accumulator.observe(sample);
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

    assert!(named_check(&summary, "dynamic_readable_lift_samples").passed);
    assert!(named_check(&summary, "updraft_swirl_force_samples").passed);
    assert!(named_check(&summary, "layered_dynamic_wind_flow_fields").passed);
    assert!(!named_check(&summary, "layered_wind_force_samples").passed);
    assert!(!named_check(&summary, "aligned_layered_wind_force_samples").passed);
    assert!(!named_check(&summary, "crosswind_updraft_overlap_samples").passed);
    assert!(!named_check(&summary, "aligned_crosswind_updraft_overlap_samples").passed);
    assert!(!named_check(&summary, "layered_wind_force_fields").passed);
    assert!(!named_check(&summary, "layered_wind_force_delta").passed);
    assert!(!named_check(&summary, "layered_wind_force_flow_alignment").passed);
    assert!(!named_check(&summary, "layered_wind_force_aligned_delta").passed);
}

#[test]
fn accumulator_gates_wind_force_response_metrics() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_FORCE_SAMPLE_COUNT {
        accumulator.observe(content_metric_sample(scenario, frame, 12, 0, 96));
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

    for check_name in [
        "wind_force_samples",
        "meaningful_wind_force_samples",
        "aligned_wind_force_samples",
        "active_wind_force_fields",
        "wind_force_delta",
        "wind_force_flow_speed",
        "wind_force_variation",
        "wind_force_flow_alignment",
        "wind_force_aligned_delta",
        "crosswind_force_samples",
        "aligned_crosswind_force_samples",
        "crosswind_force_fields",
        "crosswind_force_delta",
        "crosswind_force_flow_alignment",
        "crosswind_force_aligned_delta",
    ] {
        assert!(
            named_check(&summary, check_name).passed,
            "{check_name} should pass with current wind response metrics"
        );
    }
}

#[test]
fn accumulator_requires_sustained_meaningful_wind_force_samples() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    accumulator.observe(content_metric_sample(scenario, 0, 12, 0, 96));

    let mut weak_sample = content_metric_sample(scenario, 1, 12, 0, 96);
    weak_sample.max_wind_force_delta_mps = MIN_WIND_FORCE_DELTA_MPS * 0.5;
    weak_sample.max_crosswind_force_delta_mps = MIN_CROSSWIND_FORCE_DELTA_MPS * 0.5;
    weak_sample.max_updraft_swirl_force_delta_mps = MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS * 0.5;
    weak_sample.max_wind_force_variation = MIN_WIND_FORCE_VARIATION * 0.5;
    weak_sample.max_wind_force_flow_alignment = MIN_WIND_FORCE_FLOW_ALIGNMENT * 0.5;
    weak_sample.max_crosswind_force_flow_alignment = MIN_WIND_FORCE_FLOW_ALIGNMENT * 0.5;
    weak_sample.max_updraft_swirl_force_flow_alignment = MIN_WIND_FORCE_FLOW_ALIGNMENT * 0.5;
    weak_sample.max_wind_force_aligned_delta_mps = MIN_WIND_FORCE_ALIGNED_DELTA_MPS * 0.5;
    weak_sample.max_crosswind_force_aligned_delta_mps = MIN_WIND_FORCE_ALIGNED_DELTA_MPS * 0.5;
    weak_sample.max_updraft_swirl_force_aligned_delta_mps = MIN_WIND_FORCE_ALIGNED_DELTA_MPS * 0.5;
    accumulator.observe(weak_sample);

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

    assert!(named_check(&summary, "wind_force_samples").passed);
    assert!(named_check(&summary, "wind_force_delta").passed);
    assert!(named_check(&summary, "wind_force_variation").passed);
    assert!(!named_check(&summary, "meaningful_wind_force_samples").passed);
    assert!(!named_check(&summary, "aligned_wind_force_samples").passed);
    assert!(!named_check(&summary, "aligned_crosswind_force_samples").passed);
}

#[test]
fn accumulator_rejects_missing_wind_force_response_metrics() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let mut sample = content_metric_sample(scenario, 0, 12, 0, 96);
    sample.active_wind_force_fields = 0;
    sample.crosswind_force_fields = 0;
    sample.updraft_swirl_force_fields = 0;
    sample.max_wind_force_delta_mps = 0.0;
    sample.max_crosswind_force_delta_mps = 0.0;
    sample.max_updraft_swirl_force_delta_mps = 0.0;
    sample.max_wind_force_flow_speed_mps = 0.0;
    sample.max_wind_force_variation = 0.0;
    sample.max_wind_force_flow_alignment = 0.0;
    sample.max_crosswind_force_flow_alignment = 0.0;
    sample.max_updraft_swirl_force_flow_alignment = 0.0;
    sample.max_wind_force_aligned_delta_mps = 0.0;
    sample.max_crosswind_force_aligned_delta_mps = 0.0;
    sample.max_updraft_swirl_force_aligned_delta_mps = 0.0;
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
        "wind_force_samples",
        "meaningful_wind_force_samples",
        "aligned_wind_force_samples",
        "active_wind_force_fields",
        "wind_force_delta",
        "wind_force_flow_speed",
        "wind_force_variation",
        "wind_force_flow_alignment",
        "wind_force_aligned_delta",
        "crosswind_force_samples",
        "aligned_crosswind_force_samples",
        "crosswind_force_fields",
        "crosswind_force_delta",
        "crosswind_force_flow_alignment",
        "crosswind_force_aligned_delta",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without measured wind response"
        );
    }
}

#[test]
fn accumulator_gates_wind_load_response_metrics() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT {
        let sample = wind_load_metric_sample(
            scenario,
            frame,
            MIN_WIND_LOAD_LATERAL_LOAD,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
        );
        accumulator.observe(sample);
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

    for check_name in [
        "wind_load_response_samples",
        "wind_load_lateral_load",
        "wind_load_pose_lean",
        "wind_load_glider_response",
        "player_wind_shear_visual_count",
        "visible_player_wind_shear_visual_count",
        "player_wind_shear_length_scale",
        "player_wind_shear_lateral_offset",
        "player_wind_shear_depth_offset",
        "crosswind_neutral_drift_samples",
        "crosswind_neutral_horizontal_drift",
        "crosswind_neutral_horizontal_step",
    ] {
        assert!(
            named_check(&summary, check_name).passed,
            "{check_name} should pass with gust-synchronized wind-current load"
        );
    }
    assert_eq!(
        summary.metrics.wind_load_response_samples,
        MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT
    );
    assert_eq!(
        summary.metrics.max_wind_load_lateral_load,
        MIN_WIND_LOAD_LATERAL_LOAD
    );
    assert_eq!(
        summary.metrics.max_wind_load_pose_lean_degrees,
        MIN_WIND_LOAD_POSE_LEAN_DEGREES
    );
    assert_eq!(
        summary.metrics.max_wind_load_glider_response_degrees,
        MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES
    );
    assert_eq!(
        summary.metrics.max_player_wind_shear_visual_count,
        MIN_PLAYER_WIND_SHEAR_VISUAL_COUNT
    );
    assert_eq!(
        summary.metrics.max_visible_player_wind_shear_visual_count,
        MIN_VISIBLE_PLAYER_WIND_SHEAR_VISUAL_COUNT
    );
    assert_eq!(
        summary.metrics.max_player_wind_shear_length_scale,
        MIN_PLAYER_WIND_SHEAR_LENGTH_SCALE
    );
    assert_eq!(
        summary.metrics.max_player_wind_shear_lateral_offset_m,
        MIN_PLAYER_WIND_SHEAR_LATERAL_OFFSET_M
    );
    assert_eq!(
        summary.metrics.max_player_wind_shear_depth_offset_m,
        MIN_PLAYER_WIND_SHEAR_DEPTH_OFFSET_M
    );
    assert_eq!(
        summary.metrics.crosswind_neutral_drift_samples,
        MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT
    );
    assert!(
        summary.metrics.crosswind_neutral_horizontal_drift_m
            >= MIN_CROSSWIND_NEUTRAL_HORIZONTAL_DRIFT_M
    );
    assert!(
        summary.metrics.max_crosswind_neutral_horizontal_step_m
            <= MAX_CROSSWIND_NEUTRAL_HORIZONTAL_STEP_M
    );
}

#[test]
fn accumulator_counts_only_wind_aligned_crosswind_neutral_drift() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT {
        let mut sample = wind_load_metric_sample(
            scenario,
            frame,
            MIN_WIND_LOAD_LATERAL_LOAD,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
        );
        sample.crosswind_force_delta = [-MIN_CROSSWIND_FORCE_DELTA_MPS, 0.0, 0.0];
        accumulator.observe(sample);
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

    assert_eq!(
        summary.metrics.crosswind_neutral_drift_samples,
        MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT
    );
    assert_eq!(summary.metrics.crosswind_neutral_horizontal_drift_m, 0.0);
    assert!(
        !named_check(&summary, "crosswind_neutral_horizontal_drift").passed,
        "anti-wind travel should not satisfy the neutral crosswind drift floor"
    );
}

#[test]
fn accumulator_resets_crosswind_neutral_drift_across_nonqualifying_samples() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    accumulator.observe(wind_load_metric_sample(
        scenario,
        0,
        MIN_WIND_LOAD_LATERAL_LOAD,
        MIN_WIND_LOAD_POSE_LEAN_DEGREES,
        MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
    ));

    let mut nonqualifying = wind_load_metric_sample(
        scenario,
        200,
        MIN_WIND_LOAD_LATERAL_LOAD,
        MIN_WIND_LOAD_POSE_LEAN_DEGREES,
        MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
    );
    nonqualifying.crosswind_force_fields = 0;
    nonqualifying.max_crosswind_force_delta_mps = 0.0;
    nonqualifying.max_crosswind_force_flow_alignment = 0.0;
    nonqualifying.max_crosswind_force_aligned_delta_mps = 0.0;
    accumulator.observe(nonqualifying);

    for frame in [201, 202, 203] {
        accumulator.observe(wind_load_metric_sample(
            scenario,
            frame,
            MIN_WIND_LOAD_LATERAL_LOAD,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
        ));
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

    assert_eq!(summary.metrics.crosswind_neutral_drift_samples, 4);
    assert!(
        summary.metrics.crosswind_neutral_horizontal_drift_m <= 1.0,
        "drift should ignore the large transition into the crosswind field"
    );
    assert!(
        summary.metrics.max_crosswind_neutral_horizontal_step_m <= 0.5,
        "max step should come only from consecutive qualifying crosswind samples"
    );
}

#[test]
fn accumulator_rejects_wind_load_without_crosswind_force_evidence() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT {
        let mut sample = wind_load_metric_sample(
            scenario,
            frame,
            MIN_WIND_LOAD_LATERAL_LOAD,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
        );
        sample.crosswind_force_fields = 0;
        sample.max_crosswind_force_delta_mps = 0.0;
        sample.max_crosswind_force_flow_alignment = 0.0;
        sample.max_crosswind_force_aligned_delta_mps = 0.0;
        accumulator.observe(sample);
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

    assert_eq!(summary.metrics.wind_load_response_samples, 0);
    assert_eq!(summary.metrics.crosswind_neutral_drift_samples, 0);
    for check_name in [
        "wind_load_response_samples",
        "wind_load_lateral_load",
        "wind_load_pose_lean",
        "wind_load_glider_response",
        "player_wind_shear_visual_count",
        "visible_player_wind_shear_visual_count",
        "player_wind_shear_length_scale",
        "player_wind_shear_lateral_offset",
        "player_wind_shear_depth_offset",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without crosswind force evidence"
        );
    }
}

#[test]
fn accumulator_rejects_low_variation_wind_load_response() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT {
        let mut sample = wind_load_metric_sample(
            scenario,
            frame,
            MIN_WIND_LOAD_LATERAL_LOAD,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
        );
        sample.max_wind_force_variation = MIN_WIND_FORCE_VARIATION * 0.5;
        accumulator.observe(sample);
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

    assert_eq!(summary.metrics.wind_load_response_samples, 0);
    for check_name in [
        "wind_load_response_samples",
        "wind_load_lateral_load",
        "wind_load_pose_lean",
        "wind_load_glider_response",
        "player_wind_shear_visual_count",
        "visible_player_wind_shear_visual_count",
        "player_wind_shear_length_scale",
        "player_wind_shear_lateral_offset",
        "player_wind_shear_depth_offset",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail when wind-current load lacks gust variation"
        );
    }
}

#[test]
fn accumulator_rejects_missing_wind_load_response() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT {
        accumulator.observe(wind_load_metric_sample(scenario, frame, 0.0, 0.0, 0.0));
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

    for check_name in [
        "wind_load_response_samples",
        "wind_load_lateral_load",
        "wind_load_pose_lean",
        "wind_load_glider_response",
        "player_wind_shear_visual_count",
        "visible_player_wind_shear_visual_count",
        "player_wind_shear_length_scale",
        "player_wind_shear_lateral_offset",
        "player_wind_shear_depth_offset",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without wind-driven pose/glider response"
        );
    }
}

fn wind_load_metric_sample(
    scenario: EvalScenario,
    frame: u32,
    lateral_load: f32,
    pose_lean_degrees: f32,
    glider_response_degrees: f32,
) -> EvalSample {
    air_control_metric_sample(
        scenario,
        frame,
        Vec3::new(0.0, -2.0, -18.0),
        Vec2::new(0.0, 1.0),
        0.0,
        18.0,
        0.0,
    )
    .with_wind_lateral_load(lateral_load)
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: 10.0,
        arm_spread_degrees: 30.0,
        leg_tuck_degrees: 0.0,
        lateral_lean_degrees: pose_lean_degrees,
        signed_lateral_lean_degrees: -pose_lean_degrees,
        grounded_stride_foot_travel_m: 0.0,
        grounded_stride_leg_opposition_degrees: 0.0,
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        landing_foot_split_m: 0.0,
        landing_recovery_flip_degrees: 0.0,
        wing_airflow_strength: 1.0,
        key_pose_readability_score: 1.0,
    })
    .with_authored_glider_metrics(glider_response_degrees, 0.08)
    .with_player_wind_shear_visual_metrics(
        MIN_PLAYER_WIND_SHEAR_VISUAL_COUNT,
        MIN_VISIBLE_PLAYER_WIND_SHEAR_VISUAL_COUNT,
        MIN_PLAYER_WIND_SHEAR_LENGTH_SCALE,
        MIN_PLAYER_WIND_SHEAR_LATERAL_OFFSET_M,
        MIN_PLAYER_WIND_SHEAR_DEPTH_OFFSET_M,
    )
}

#[test]
fn accumulator_gates_wind_guide_visual_presence_and_motion() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96)
        .with_wind_guide_visual_metrics(
            MIN_UPDRAFT_GUIDE_VISUAL_COUNT - 1,
            MIN_UPDRAFT_RIBBON_VISUAL_COUNT - 1,
            MIN_CROSSWIND_GUIDE_VISUAL_COUNT - 1,
            MIN_CROSSWIND_RIBBON_VISUAL_COUNT - 1,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        )
        .with_wind_guide_depth_metrics(0.0, 0.0, 0.0, 0.0)
        .with_observed_wind_visual_motion_metrics(
            0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
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
        "updraft_guide_visual_count",
        "updraft_ribbon_visual_count",
        "crosswind_guide_visual_count",
        "crosswind_ribbon_visual_count",
        "updraft_visual_motion",
        "updraft_visual_rise",
        "updraft_visual_swirl_displacement",
        "updraft_visual_depth_span",
        "updraft_visual_scale_pulse",
        "crosswind_visual_motion",
        "crosswind_guide_flow_displacement",
        "crosswind_ribbon_flow_displacement",
        "crosswind_visual_lane_depth_span",
        "crosswind_visual_scale_pulse",
        "observed_updraft_flow_coherent_visual_count",
        "observed_crosswind_flow_coherent_visual_count",
        "observed_crosswind_ribbon_flow_coherent_sample_count",
        "observed_updraft_visual_frame_motion",
        "observed_updraft_visual_frame_rise",
        "observed_updraft_visual_frame_swirl_displacement",
        "observed_crosswind_visual_frame_motion",
        "observed_crosswind_guide_frame_flow_displacement",
        "observed_crosswind_ribbon_frame_flow_displacement",
        "observed_updraft_visual_flow_alignment",
        "observed_crosswind_visual_flow_alignment",
        "observed_crosswind_ribbon_visual_flow_alignment",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without animated wind guide visuals"
        );
    }
}

#[test]
fn accumulator_gates_observed_wind_visual_motion_quality() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96)
        .with_observed_wind_visual_quality_metrics(
            MAX_OBSERVED_UPDRAFT_VISUAL_SPEED_MPS + 1.0,
            MAX_OBSERVED_CROSSWIND_VISUAL_SPEED_MPS + 1.0,
            MAX_OBSERVED_WIND_VISUAL_ACCELERATION_MPS2 + 1.0,
            MAX_OBSERVED_WIND_VISUAL_JUMP_COUNT + 1,
        );
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
        "observed_updraft_visual_speed",
        "observed_crosswind_visual_speed",
        "observed_wind_visual_acceleration",
        "observed_wind_visual_jump_count",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail for fast or discontinuous wind visuals"
        );
    }
}

#[test]
fn accumulator_gates_wind_guide_visual_flow_direction() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96).with_wind_guide_visual_metrics(
        MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
        MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
        MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
        MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
        MIN_UPDRAFT_VISUAL_MOTION_M,
        0.0,
        0.0,
        MIN_CROSSWIND_VISUAL_MOTION_M,
        0.0,
        0.0,
    );
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

    assert!(named_check(&summary, "updraft_visual_motion").passed);
    assert!(named_check(&summary, "crosswind_visual_motion").passed);
    assert!(!named_check(&summary, "updraft_visual_rise").passed);
    assert!(!named_check(&summary, "updraft_visual_swirl_displacement").passed);
    assert!(!named_check(&summary, "crosswind_guide_flow_displacement").passed);
    assert!(!named_check(&summary, "crosswind_ribbon_flow_displacement").passed);
}

#[test]
fn accumulator_gates_wind_guide_visual_flow_coherence() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96)
        .with_wind_guide_flow_coherence_metrics(
            MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT - 1,
            MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT - 1,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
        )
        .with_crosswind_ribbon_flow_coherence_metrics(
            MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT - 1,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
        );
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
        "updraft_flow_coherent_visual_count",
        "crosswind_flow_coherent_visual_count",
        "crosswind_ribbon_flow_coherent_sample_count",
        "updraft_visual_flow_alignment",
        "crosswind_visual_flow_alignment",
        "crosswind_ribbon_visual_flow_alignment",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without flow-coherent wind guide visuals"
        );
    }
}

#[test]
fn accumulator_gates_wind_field_visual_coverage_per_field() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96)
        .with_wind_field_visual_coverage_metrics(
            GAMEPLAY_LIFT_ROUTE.len() - 1,
            GAMEPLAY_LIFT_ROUTE.len() - 1,
            GAMEPLAY_LIFT_ROUTE.len() - 1,
            GAMEPLAY_LIFT_ROUTE.len() - 1,
            GAMEPLAY_LIFT_ROUTE.len() - 1,
            VISUAL_CROSSWIND_FIELD_COUNT - 1,
            VISUAL_CROSSWIND_FIELD_COUNT - 1,
            VISUAL_CROSSWIND_FIELD_COUNT - 1,
            VISUAL_CROSSWIND_FIELD_COUNT - 1,
            VISUAL_CROSSWIND_FIELD_COUNT - 1,
        );
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
        "updraft_field_count",
        "updraft_fields_with_guides",
        "updraft_fields_with_ribbons",
        "updraft_fields_with_guides_and_ribbons",
        "updraft_flow_coherent_field_count",
        "crosswind_field_count",
        "crosswind_fields_with_guides",
        "crosswind_fields_with_ribbons",
        "crosswind_fields_with_guides_and_ribbons",
        "crosswind_flow_coherent_field_count",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail when a wind field is missing visual coverage"
        );
    }
}

#[test]
fn accumulator_requires_sustained_wind_visual_flow_samples() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let undersampled_flow_count = MIN_SUSTAINED_WIND_VISUAL_FLOW_SAMPLES
        .min(MIN_SUSTAINED_UPDRAFT_VISUAL_FLOW_SAMPLES)
        .min(MIN_SUSTAINED_CROSSWIND_VISUAL_FLOW_SAMPLES)
        .min(MIN_SUSTAINED_CROSSWIND_RIBBON_ADVECTED_FLOW_SAMPLES)
        - 1;
    for frame in 0..undersampled_flow_count {
        accumulator.observe(content_metric_sample(scenario, frame, 12, 0, 96));
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

    assert_eq!(
        summary.metrics.sustained_wind_visual_flow_samples,
        undersampled_flow_count
    );
    for check_name in [
        "sustained_wind_visual_flow_samples",
        "sustained_updraft_visual_flow_samples",
        "sustained_crosswind_visual_flow_samples",
        "sustained_crosswind_ribbon_advected_flow_samples",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without a sustained visual-flow window"
        );
    }
}

#[test]
fn accumulator_rejects_weak_sustained_wind_visual_flow_samples() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let weak_visual_ratio = SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO - 0.05;
    for frame in 0..MIN_SUSTAINED_WIND_VISUAL_FLOW_SAMPLES {
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96)
                .with_wind_guide_visual_metrics(
                    MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
                    MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
                    MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
                    MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
                    MIN_UPDRAFT_VISUAL_MOTION_M * weak_visual_ratio,
                    MIN_UPDRAFT_VISUAL_RISE_M * weak_visual_ratio,
                    MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M * weak_visual_ratio,
                    MIN_CROSSWIND_VISUAL_MOTION_M * weak_visual_ratio,
                    MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M * weak_visual_ratio,
                    MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M * weak_visual_ratio,
                )
                .with_wind_guide_depth_metrics(
                    MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M * weak_visual_ratio,
                    MIN_UPDRAFT_VISUAL_SCALE_PULSE * weak_visual_ratio,
                    MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M * weak_visual_ratio,
                    MIN_CROSSWIND_VISUAL_SCALE_PULSE * weak_visual_ratio,
                )
                .with_wind_guide_flow_coherence_metrics(
                    MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
                    MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
                    MIN_WIND_VISUAL_FLOW_ALIGNMENT,
                    MIN_WIND_VISUAL_FLOW_ALIGNMENT,
                ),
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

    assert_eq!(summary.metrics.sustained_wind_visual_flow_samples, 0);
    assert_eq!(summary.metrics.sustained_updraft_visual_flow_samples, 0);
    assert_eq!(summary.metrics.sustained_crosswind_visual_flow_samples, 0);
    assert!(!named_check(&summary, "sustained_wind_visual_flow_samples").passed);
    assert!(!named_check(&summary, "sustained_updraft_visual_flow_samples").passed);
    assert!(!named_check(&summary, "sustained_crosswind_visual_flow_samples").passed);
}

#[test]
fn accumulator_rejects_weak_sustained_wind_visual_count_and_alignment_samples() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let mut accumulator = EvalAccumulator::default();
    let weak_updraft_count = ((MIN_UPDRAFT_GUIDE_VISUAL_COUNT as f32)
        * SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO)
        .ceil() as usize
        - 1;
    let weak_crosswind_count = ((MIN_CROSSWIND_GUIDE_VISUAL_COUNT as f32)
        * SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO)
        .ceil() as usize
        - 1;
    for frame in 0..MIN_SUSTAINED_WIND_VISUAL_FLOW_SAMPLES {
        accumulator.observe(
            content_metric_sample(scenario, frame, 12, 0, 96)
                .with_wind_guide_visual_metrics(
                    weak_updraft_count,
                    MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
                    weak_crosswind_count,
                    MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
                    MIN_UPDRAFT_VISUAL_MOTION_M,
                    MIN_UPDRAFT_VISUAL_RISE_M,
                    MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
                    MIN_CROSSWIND_VISUAL_MOTION_M,
                    MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
                    MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
                )
                .with_wind_guide_depth_metrics(
                    MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M,
                    MIN_UPDRAFT_VISUAL_SCALE_PULSE,
                    MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M,
                    MIN_CROSSWIND_VISUAL_SCALE_PULSE,
                )
                .with_wind_guide_flow_coherence_metrics(
                    MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
                    MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
                    MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
                    MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
                ),
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

    assert_eq!(summary.metrics.sustained_wind_visual_flow_samples, 0);
    assert_eq!(summary.metrics.sustained_updraft_visual_flow_samples, 0);
    assert_eq!(summary.metrics.sustained_crosswind_visual_flow_samples, 0);
    assert!(!named_check(&summary, "sustained_wind_visual_flow_samples").passed);
    assert!(!named_check(&summary, "sustained_updraft_visual_flow_samples").passed);
    assert!(!named_check(&summary, "sustained_crosswind_visual_flow_samples").passed);
}

#[test]
fn accumulator_gates_crosswind_ribbon_flow_separately_from_guides() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96)
        .with_wind_guide_visual_metrics(
            MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
            MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
            MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
            MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
            MIN_UPDRAFT_VISUAL_MOTION_M,
            MIN_UPDRAFT_VISUAL_RISE_M,
            MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
            MIN_CROSSWIND_VISUAL_MOTION_M,
            MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
            0.0,
        )
        .with_wind_guide_flow_coherence_metrics(
            MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
            MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
        )
        .with_crosswind_ribbon_flow_coherence_metrics(
            MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT - 1,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT - 0.01,
        );
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

    assert!(named_check(&summary, "crosswind_guide_flow_displacement").passed);
    assert!(!named_check(&summary, "crosswind_ribbon_flow_displacement").passed);
    assert!(
        !named_check(&summary, "crosswind_ribbon_flow_coherent_sample_count").passed,
        "crosswind ribbons should need their own advected scene-sample flow alignment"
    );
    assert!(
        !named_check(&summary, "crosswind_ribbon_visual_flow_alignment").passed,
        "crosswind ribbon flow alignment should not be covered by guide motes"
    );
}

#[test]
fn accumulator_rejects_nonvarying_lift_visual_flow() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..scenario.thresholds.min_lifted_samples {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, 8.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        );
        sample.active_lift_fields = 1;
        sample.readable_lift_fields = 1;
        sample.dynamic_wind_flow_fields = 1;
        sample.max_wind_flow_speed_mps = 10.0;
        sample.max_wind_flow_variation = 0.2;
        accumulator.observe(sample);
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

    assert!(named_check(&summary, "dynamic_readable_lift_samples").passed);
    assert!(named_check(&summary, "max_wind_flow_speed").passed);
    assert!(named_check(&summary, "max_wind_flow_variation").passed);
    assert!(!named_check(&summary, "max_wind_flow_direction_change").passed);
    assert!(!named_check(&summary, "max_wind_flow_variation_range").passed);
}

#[test]
fn accumulator_rejects_static_lift_visual_flow() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");
    let mut accumulator = EvalAccumulator::default();

    for frame in 0..scenario.thresholds.min_lifted_samples {
        let mut sample = air_control_metric_sample(
            scenario,
            frame,
            Vec3::new(0.0, 8.0, -18.0),
            Vec2::ZERO,
            0.0,
            18.0,
            0.0,
        );
        sample.active_lift_fields = 1;
        sample.readable_lift_fields = 1;
        accumulator.observe(sample);
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

    assert_eq!(summary.metrics.dynamic_readable_lift_samples, 0);
    assert!(!named_check(&summary, "dynamic_readable_lift_samples").passed);
    assert!(!named_check(&summary, "max_wind_flow_speed").passed);
    assert!(!named_check(&summary, "max_wind_flow_variation").passed);
    assert!(!named_check(&summary, "max_wind_flow_direction_change").passed);
    assert!(!named_check(&summary, "max_wind_flow_variation_range").passed);
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
