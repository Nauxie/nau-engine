use super::*;

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
        Vec3::new(24.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        24.0,
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
    assert_eq!(summary.metrics.max_right_lateral_response_mps, 24.0);
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
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        }),
    );
    accumulator.observe(air_control_metric_sample(
        scenario,
        1,
        Vec3::new(0.0, -18.0, -26.0),
        Vec2::ZERO,
        0.0,
        18.0,
        0.0,
    ));
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
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    });
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
        landing_crouch_m: 0.0,
        landing_foot_forward_m: 0.0,
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
    assert_eq!(summary.metrics.pose_diving_samples, 1);
    assert_eq!(summary.metrics.pose_air_brake_samples, 1);
    assert_eq!(summary.metrics.pose_landing_anticipation_samples, 1);
    assert_eq!(summary.metrics.pose_landing_recovery_samples, 1);
    assert_eq!(summary.metrics.authored_clip_match_samples, 7);
    assert_eq!(summary.metrics.authored_clip_mismatch_samples, 0);
    assert_eq!(summary.metrics.authored_dive_clip_samples, 1);
    assert_eq!(summary.metrics.authored_air_brake_clip_samples, 1);
    assert_eq!(summary.metrics.authored_land_clip_samples, 3);
    assert_eq!(summary.metrics.max_authored_transition_duration_ms, 140);
    assert_eq!(summary.metrics.max_pose_torso_pitch_degrees, 64.0);
    assert_eq!(summary.metrics.max_pose_landing_flare_degrees, 37.0);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 1);
    assert!(summary_json.contains("\"max_pose_landing_foot_forward_m\""));
    assert!(summary_json.contains("\"max_pose_landing_flare_degrees\": 37"));
    assert!(summary_json.contains("\"pose_air_turn_samples\": 1"));
    assert!(summary_json.contains("\"right_pose_air_turn_samples\": 1"));
    assert!(summary_json.contains("\"left_pose_air_turn_samples\": 0"));
    assert!(summary_json.contains("\"pose_air_brake_samples\": 1"));
    assert!(summary_json.contains("\"pose_landing_anticipation_samples\": 1"));
    assert!(summary_json.contains("\"pose_landing_recovery_samples\": 1"));
    assert!(summary_json.contains("\"authored_clip_match_samples\": 7"));
    assert!(summary_json.contains("\"authored_clip_mismatch_samples\": 0"));
    assert!(summary_json.contains("\"authored_dive_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_air_brake_clip_samples\": 1"));
    assert!(summary_json.contains("\"authored_land_clip_samples\": 3"));
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
        landing_crouch_m: 1.0,
        landing_foot_forward_m: 0.40,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    });
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
    let landing_recovery_check = named_check(&summary, "pose_landing_recovery_samples");
    let landing_foot_forward_check = named_check(&summary, "pose_landing_foot_forward");
    let landing_flare_check = named_check(&summary, "pose_landing_flare");
    let authored_land_check = named_check(&summary, "authored_landing_clip_samples");

    assert_eq!(summary.metrics.pose_landing_recovery_samples, 0);
    assert_eq!(summary.metrics.authored_land_clip_samples, 1);
    assert_eq!(summary.metrics.max_pose_landing_foot_forward_m, 0.40);
    assert_eq!(summary.metrics.max_pose_landing_flare_degrees, 0.0);
    assert_eq!(landing_recovery_check.value, 0.0);
    assert_eq!(landing_recovery_check.threshold, 1.0);
    assert!(!landing_recovery_check.passed);
    assert_eq!(authored_land_check.value, 1.0);
    assert_eq!(authored_land_check.threshold, 2.0);
    assert!(!authored_land_check.passed);
    assert_eq!(landing_foot_forward_check.value, 0.40);
    assert_eq!(landing_foot_forward_check.threshold, 0.32);
    assert!(landing_foot_forward_check.passed);
    assert_eq!(landing_flare_check.value, 0.0);
    assert_eq!(landing_flare_check.threshold, 48.0);
    assert!(!landing_flare_check.passed);
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
            landing_crouch_m: 0.12,
            landing_foot_forward_m: 0.40,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        })
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: f32::NAN,
            max_pose_part_translation_delta_m: f32::NAN,
        });
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
        landing_crouch_m: 0.12,
        landing_foot_forward_m: 0.40,
        wing_airflow_strength: 0.0,
        key_pose_readability_score: 1.0,
    })
    .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
        visible_pose_part_count: 5,
        max_pose_part_rotation_delta_degrees: 121.0,
        max_pose_part_translation_delta_m: 0.56,
    });
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
            landing_crouch_m: 0.12,
            landing_foot_forward_m: 0.40,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 1.0,
        })
        .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
            visible_pose_part_count: 5,
            max_pose_part_rotation_delta_degrees: 20.0,
            max_pose_part_translation_delta_m: 0.1,
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
    let authored_air_brake_check =
        named_check(&summary, "air_control_authored_air_brake_clip_samples");
    let authored_dive_check = named_check(&summary, "air_control_authored_dive_clip_samples");

    assert_eq!(air_turn_check.value, 4.0);
    assert!(air_turn_check.passed);
    assert_eq!(air_brake_check.value, 0.0);
    assert_eq!(dive_check.value, 0.0);
    assert_eq!(authored_air_brake_check.value, 0.0);
    assert_eq!(authored_dive_check.value, 0.0);
    assert!(!air_brake_check.passed);
    assert!(!dive_check.passed);
    assert!(!authored_air_brake_check.passed);
    assert!(!authored_dive_check.passed);
    for name in [
        "air_control_pose_torso_pitch",
        "air_control_pose_arm_spread",
        "air_control_pose_leg_tuck",
        "air_control_pose_lateral_lean",
        "air_control_right_pose_lateral_lean",
        "air_control_left_pose_lateral_lean",
        "air_control_pose_wing_airflow",
    ] {
        let check = named_check(&summary, name);
        assert_eq!(check.value, 0.0);
        assert!(!check.passed, "expected {name} to fail");
    }
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
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
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
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
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
fn accumulator_gates_pose_state_coverage_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("pose state route exists");
    let mut accumulator = EvalAccumulator::default();

    observe_pose_state_samples(
        &mut accumulator,
        scenario,
        &[
            ("grounded_walk", FlightMode::Grounded.label(), 8),
            ("grounded_run", FlightMode::Grounded.label(), 8),
            ("launching", FlightMode::Launching.label(), 3),
            ("falling", FlightMode::Airborne.label(), 8),
            ("gliding", FlightMode::Gliding.label(), 18),
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

    assert_eq!(summary.metrics.pose_grounded_walk_samples, 8);
    assert_eq!(summary.metrics.pose_grounded_run_samples, 8);
    assert_eq!(summary.metrics.pose_launching_samples, 3);
    assert_eq!(summary.metrics.pose_falling_samples, 8);
    assert_eq!(summary.metrics.pose_gliding_samples, 18);
    assert_eq!(summary.metrics.unreadable_key_pose_samples, 0);
    assert!(summary.to_json().contains("\"pose_grounded_walk_samples\""));
    for name in [
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_launching_samples",
        "pose_state_falling_samples",
        "pose_state_gliding_samples",
        "pose_state_unreadable_key_pose_samples",
    ] {
        assert!(named_check(&summary, name).passed, "{name} should pass");
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
            ("grounded_walk", FlightMode::Grounded.label(), 7),
            ("grounded_run", FlightMode::Grounded.label(), 7),
            ("launching", FlightMode::Launching.label(), 2),
            ("falling", FlightMode::Airborne.label(), 7),
            ("gliding", FlightMode::Gliding.label(), 17),
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
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_launching_samples",
        "pose_state_falling_samples",
        "pose_state_gliding_samples",
    ] {
        assert!(!named_check(&summary, name).passed, "{name} should fail");
    }
}

fn observe_pose_state_samples(
    accumulator: &mut EvalAccumulator,
    scenario: EvalScenario,
    samples: &[(&'static str, &'static str, u32)],
) {
    let mut frame = 10;
    for &(pose_intent_label, mode, count) in samples {
        for _ in 0..count {
            let mut sample = content_metric_sample(scenario, frame, 20, 0, 96);
            sample.mode = mode;
            sample.pose_intent_label = pose_intent_label;
            sample.key_pose_readability_score = 1.0;
            accumulator.observe(sample);
            frame += 5;
        }
    }
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
    let summary_json = summary.to_json();

    assert_eq!(summary.metrics.max_visible_pose_part_count, 5);
    assert_eq!(summary.metrics.pose_temporal_stability_samples, 1);
    assert_eq!(summary.metrics.max_pose_part_rotation_delta_degrees, 150.0);
    assert_eq!(summary.metrics.max_pose_part_translation_delta_m, 0.8);
    assert!(!rotation_check.passed);
    assert!(!translation_check.passed);
    assert!(summary_json.contains("\"max_pose_part_rotation_delta_degrees\": 150"));
    assert!(summary_json.contains("\"max_pose_part_translation_delta_m\": 0.8000"));
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
        sample.dynamic_wind_flow_fields = 1;
        sample.max_wind_flow_speed_mps = 10.0;
        sample.max_wind_flow_variation = 0.16 + frame as f32 * 0.02;
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
    assert!(named_check(&summary, "max_wind_flow_variation_range").passed);
    assert!(named_check(&summary, "updraft_swirl_force_samples").passed);
    assert!(named_check(&summary, "aligned_updraft_swirl_force_samples").passed);
    assert!(named_check(&summary, "updraft_swirl_force_fields").passed);
    assert!(named_check(&summary, "updraft_swirl_force_delta").passed);
    assert!(named_check(&summary, "updraft_swirl_force_flow_alignment").passed);
    assert!(named_check(&summary, "updraft_swirl_force_aligned_delta").passed);
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
        .with_wind_guide_depth_metrics(0.0, 0.0, 0.0, 0.0);
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
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without animated wind guide visuals"
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
        "updraft_visual_flow_alignment",
        "crosswind_visual_flow_alignment",
    ] {
        assert!(
            !named_check(&summary, check_name).passed,
            "{check_name} should fail without flow-coherent wind guide visuals"
        );
    }
}

#[test]
fn accumulator_gates_crosswind_ribbon_flow_separately_from_guides() {
    let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
    let sample = content_metric_sample(scenario, 0, 12, 0, 96).with_wind_guide_visual_metrics(
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
