use super::{
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES,
    AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES, LANDING_MIN_POSE_CROUCH_M,
    metrics::{SimMetrics, SimResult},
    sample::{CameraDiagnosticsSample, SimSample},
    simulation::run_simulation,
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    animation::{
        GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
        GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
        PlayerPoseIntent,
    },
    environment::{LiftApplication, WindForceApplication},
    eval::{
        AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, AIR_CONTROL_RESPONSE,
        BRANCH_RECOVERY_ROUTE, CAMERA_MOUSE_CONTROL, EvalScenario, ISLAND_LAUNCH_TO_LANDING,
        LANDING_MIN_POSE_FLARE_DEGREES, LANDING_MIN_POSE_FOOT_FORWARD_M,
        LANDING_MIN_POSE_FOOT_SPLIT_M, LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES,
        LONG_GLIDE_VISIBILITY, MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS,
        MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE, MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
        MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES, MIN_WIND_LOAD_LATERAL_LOAD,
        MIN_WIND_LOAD_POSE_LEAN_DEGREES, MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT, POSE_STATE_COVERAGE,
        POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES,
        POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES, UPDRAFT_ROUTE, scenario_named,
    },
    movement::{Facing, FlightController, FlightInput, FlightMode, FlightState},
    world::{START_POSITION, SkyRoute},
};

#[test]
fn baseline_simulation_writes_windowless_artifacts() {
    let scenario = scenario_named("baseline_route").expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.sample_count >= scenario.thresholds.min_samples);
    assert!(!result.samples.is_empty());
    assert_eq!(result.samples.last().unwrap().frame, scenario.frame_count);
    let summary = result.to_summary_json();
    assert!(summary.contains("\"mode\": \"simulation_only\""));
    assert!(summary.contains("\"native_window_created\": false"));
    assert!(summary.contains("\"screenshot_png\": null"));
    assert!(summary.contains("\"pose_gliding_samples\""));
    assert!(summary.contains("\"pose_grounded_walk_samples\""));
    assert!(summary.contains("\"pose_grounded_run_samples\""));
    assert!(summary.contains("\"pose_launching_samples\""));
    assert!(summary.contains("\"pose_falling_samples\""));
    assert!(summary.contains("\"gliding_dive_samples\""));
    assert!(summary.contains("\"pose_air_turn_samples\""));
    assert!(summary.contains("\"pose_grounded_idle_samples\""));
    assert!(summary.contains("\"pose_landing_recovery_samples\""));
    assert!(summary.contains("\"max_pose_landing_foot_forward_m\""));
    assert!(summary.contains("\"max_pose_landing_foot_split_m\""));
    assert!(summary.contains("\"max_pose_landing_flare_degrees\""));
    assert!(summary.contains("\"max_pose_landing_recovery_flip_degrees\""));
    assert!(summary.contains("\"key_pose_transition_grace_samples\""));
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("pose_intent")
            .is_some()
    );
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("key_pose_readability_score")
            .is_some()
    );
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("pose_landing_foot_split_m")
            .is_some()
    );
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("key_pose_transition_grace")
            .is_some()
    );
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("max_wind_flow_variation")
            .is_some()
    );
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("max_wind_flow_direction_change_degrees")
            .is_some()
    );
    let last_sample_json = result.samples.last().unwrap().to_json();
    for key in [
        "active_wind_force_fields",
        "crosswind_force_fields",
        "updraft_swirl_force_fields",
        "max_wind_force_delta_mps",
        "max_crosswind_force_delta_mps",
        "max_updraft_swirl_force_delta_mps",
        "max_wind_force_flow_speed_mps",
        "max_wind_force_variation",
        "max_wind_force_flow_alignment",
        "max_crosswind_force_flow_alignment",
        "max_updraft_swirl_force_flow_alignment",
        "max_wind_force_aligned_delta_mps",
        "max_crosswind_force_aligned_delta_mps",
        "max_updraft_swirl_force_aligned_delta_mps",
        "wind_lateral_load",
        "wind_load_glider_response_degrees",
    ] {
        assert!(
            last_sample_json.get(key).is_some(),
            "{key} should be serialized"
        );
    }
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
        let check = result
            .checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("wind-force check");
        assert!(check.passed, "{check_name} should pass");
    }
}

#[test]
fn pose_state_coverage_simulation_gates_full_traversal_pose_chain() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.pose_grounded_idle_samples >= 3);
    assert!(result.metrics.pose_grounded_walk_samples >= 8);
    assert!(result.metrics.pose_grounded_run_samples >= 8);
    assert!(
        result.metrics.max_grounded_walk_stride_foot_travel_m
            >= GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M
    );
    assert!(
        result.metrics.max_grounded_run_stride_foot_travel_m
            >= GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M
    );
    assert!(
        result
            .metrics
            .max_grounded_walk_stride_leg_opposition_degrees
            >= GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES
    );
    assert!(
        result
            .metrics
            .max_grounded_run_stride_leg_opposition_degrees
            >= GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES
    );
    assert!(result.metrics.pose_launching_samples >= 3);
    assert!(result.metrics.pose_falling_samples >= 8);
    assert!(result.metrics.pose_gliding_samples >= 18);
    assert!(result.metrics.pose_air_turn_samples >= 6);
    assert!(
        result.metrics.right_pose_air_turn_samples
            >= POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES as u32
    );
    assert!(
        result.metrics.left_pose_air_turn_samples
            >= POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES as u32
    );
    assert!(
        result.metrics.right_pure_air_turn_sideways_samples
            >= AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES
    );
    assert!(
        result.metrics.left_pure_air_turn_sideways_samples
            >= AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES
    );
    assert!(result.metrics.pose_air_brake_samples >= 4);
    assert!(
        result
            .metrics
            .backward_right_diagonal_body_travel_heading_samples
            >= AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES
    );
    assert!(
        result
            .metrics
            .backward_left_diagonal_body_travel_heading_samples
            >= AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES
    );
    assert!(result.metrics.pose_diving_samples >= 1);
    assert!(result.metrics.gliding_dive_samples >= 1);
    assert!(result.metrics.pose_landing_anticipation_samples >= 1);
    assert!(result.metrics.pose_landing_recovery_samples >= 1);
    assert!(result.metrics.max_pose_landing_crouch_m >= LANDING_MIN_POSE_CROUCH_M);
    assert!(result.metrics.max_pose_landing_foot_forward_m >= LANDING_MIN_POSE_FOOT_FORWARD_M);
    assert!(result.metrics.max_pose_landing_foot_split_m >= LANDING_MIN_POSE_FOOT_SPLIT_M);
    assert!(result.metrics.max_pose_landing_flare_degrees >= LANDING_MIN_POSE_FLARE_DEGREES);
    assert!(
        result.metrics.max_pose_landing_recovery_flip_degrees
            >= LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES
    );
    assert_eq!(result.metrics.unreadable_key_pose_samples, 0);

    for name in [
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_grounded_idle_samples",
        "pose_state_walk_stride_foot_travel",
        "pose_state_run_stride_foot_travel",
        "pose_state_walk_stride_leg_opposition",
        "pose_state_run_stride_leg_opposition",
        "pose_state_launching_samples",
        "pose_state_falling_samples",
        "pose_state_gliding_samples",
        "pose_state_air_turn_samples",
        "pose_state_right_air_turn_samples",
        "pose_state_left_air_turn_samples",
        "pose_state_pure_air_turn_sideways_samples",
        "pose_state_right_pure_air_turn_sideways_samples",
        "pose_state_left_pure_air_turn_sideways_samples",
        "pose_state_air_brake_samples",
        "pose_state_backward_diagonal_body_travel_heading_samples",
        "pose_state_backward_right_diagonal_body_travel_heading_samples",
        "pose_state_backward_left_diagonal_body_travel_heading_samples",
        "pose_state_diving_samples",
        "pose_state_gliding_dive_samples",
        "pose_state_dive_pose_torso_pitch",
        "pose_state_dive_pose_arm_spread",
        "pose_state_dive_pose_leg_tuck",
        "pose_state_landing_anticipation_samples",
        "pose_state_landing_recovery_samples",
        "pose_state_landing_crouch",
        "pose_state_landing_foot_forward",
        "pose_state_landing_foot_split",
        "pose_state_landing_flare",
        "pose_state_landing_recovery_flip",
        "pose_state_unreadable_key_pose_samples",
        "pose_state_key_pose_transition_grace_samples",
    ] {
        let check = result
            .checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("missing sim check {name}"));
        assert!(check.passed, "expected {name} to pass: {check:?}");
    }
}

#[test]
fn pose_state_coverage_sim_checks_reject_thin_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    metrics.pose_grounded_idle_samples = 2;
    metrics.pose_grounded_walk_samples = 7;
    metrics.pose_grounded_run_samples = 7;
    metrics.pose_launching_samples = 2;
    metrics.pose_falling_samples = 7;
    metrics.pose_gliding_samples = 17;
    metrics.pose_air_turn_samples = 3;
    metrics.right_pose_air_turn_samples = 2;
    metrics.left_pose_air_turn_samples = 2;
    metrics.pose_air_brake_samples = 3;

    let checks = metrics.checks(scenario);
    for name in [
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_grounded_idle_samples",
        "pose_state_launching_samples",
        "pose_state_falling_samples",
        "pose_state_gliding_samples",
        "pose_state_air_turn_samples",
        "pose_state_right_air_turn_samples",
        "pose_state_left_air_turn_samples",
        "pose_state_pure_air_turn_sideways_samples",
        "pose_state_right_pure_air_turn_sideways_samples",
        "pose_state_left_pure_air_turn_sideways_samples",
        "pose_state_air_brake_samples",
        "pose_state_backward_diagonal_body_travel_heading_samples",
        "pose_state_backward_right_diagonal_body_travel_heading_samples",
        "pose_state_backward_left_diagonal_body_travel_heading_samples",
        "pose_state_diving_samples",
        "pose_state_gliding_dive_samples",
        "pose_state_dive_pose_torso_pitch",
        "pose_state_dive_pose_arm_spread",
        "pose_state_dive_pose_leg_tuck",
        "pose_state_landing_anticipation_samples",
        "pose_state_landing_recovery_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("missing sim check {name}"));
        assert!(!check.passed, "expected {name} to fail: {check:?}");
    }
}

#[test]
fn pose_state_coverage_sim_checks_reject_static_grounded_stride() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    metrics.pose_grounded_walk_samples = 8;
    metrics.pose_grounded_run_samples = 8;
    metrics.pose_launching_samples = 3;
    metrics.pose_falling_samples = 8;
    metrics.pose_gliding_samples = 18;
    metrics.pose_air_turn_samples = 4;
    metrics.pose_air_brake_samples = 4;
    metrics.pose_diving_samples = 1;
    metrics.gliding_dive_samples = 1;
    metrics.pose_landing_anticipation_samples = 1;
    metrics.pose_landing_recovery_samples = 1;
    metrics.max_pose_landing_crouch_m = LANDING_MIN_POSE_CROUCH_M;
    metrics.max_pose_landing_foot_forward_m = LANDING_MIN_POSE_FOOT_FORWARD_M;
    metrics.max_pose_landing_foot_split_m = LANDING_MIN_POSE_FOOT_SPLIT_M;
    metrics.max_pose_landing_flare_degrees = LANDING_MIN_POSE_FLARE_DEGREES;
    metrics.max_pose_landing_recovery_flip_degrees = LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES;

    let checks = metrics.checks(scenario);
    for name in [
        "pose_state_grounded_walk_samples",
        "pose_state_grounded_run_samples",
        "pose_state_launching_samples",
        "pose_state_falling_samples",
        "pose_state_gliding_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("missing sim check {name}"));
        assert!(check.passed, "expected {name} to pass: {check:?}");
    }
    for name in [
        "pose_state_walk_stride_foot_travel",
        "pose_state_run_stride_foot_travel",
        "pose_state_walk_stride_leg_opposition",
        "pose_state_run_stride_leg_opposition",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("missing sim check {name}"));
        assert!(!check.passed, "expected {name} to fail: {check:?}");
    }
}

#[test]
fn island_landing_simulation_reaches_target_surface() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(
        result.metrics.final_target_distance_m <= scenario.thresholds.max_final_target_distance_m
    );
    assert!(
        result.metrics.target_landing_samples >= scenario.thresholds.min_target_landing_samples
    );
    assert!(result.metrics.pose_landing_anticipation_samples > 0);
    let check = result
        .checks
        .iter()
        .find(|check| check.name == "pose_landing_anticipation_samples")
        .expect("landing pose intent check");
    assert!(check.passed, "expected landing pose intent check to pass");
    assert!(result.metrics.grounded_samples >= scenario.thresholds.min_grounded_samples);
}

#[test]
fn sim_metrics_count_readable_landing_recovery_key_pose_samples() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut readable_sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    readable_sample.pose_intent_label = "landing_recovery";
    readable_sample.key_pose_readability_score = 1.0;
    metrics.observe(&readable_sample, scenario);

    let mut unreadable_sample = readable_sample.clone();
    unreadable_sample.key_pose_readability_score = 0.0;
    metrics.observe(&unreadable_sample, scenario);

    assert_eq!(metrics.pose_landing_recovery_samples, 1);
    assert_eq!(metrics.unreadable_key_pose_samples, 1);
}

#[test]
fn sim_metrics_count_key_pose_transition_grace_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    sample.pose_intent_label = "gliding";
    sample.key_pose_readability_score = 1.0;
    sample.key_pose_transition_grace = true;

    metrics.observe(&sample, scenario);

    assert_eq!(metrics.key_pose_transition_grace_samples, 1);
}

#[test]
fn sim_metrics_track_landing_flare_and_foot_split_from_landing_pose() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut non_landing_sample =
        sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    non_landing_sample.pose_intent_label = "gliding";
    non_landing_sample.pose_torso_pitch_degrees = 72.0;
    metrics.observe(&non_landing_sample, scenario);

    let mut landing_sample = non_landing_sample.clone();
    landing_sample.pose_intent_label = "landing_anticipation";
    landing_sample.pose_torso_pitch_degrees = 34.0;
    landing_sample.pose_landing_foot_forward_m = 0.41;
    landing_sample.pose_landing_foot_split_m = 0.25;
    metrics.observe(&landing_sample, scenario);

    assert_eq!(metrics.max_pose_torso_pitch_degrees, 72.0);
    assert_eq!(metrics.max_dive_pose_torso_pitch_degrees, 0.0);
    assert_eq!(metrics.max_pose_landing_flare_degrees, 34.0);
    assert_eq!(metrics.max_pose_landing_foot_forward_m, 0.41);
    assert_eq!(metrics.max_pose_landing_foot_split_m, 0.25);
    assert_eq!(
        landing_sample.to_json()["pose_landing_foot_split_m"].as_f64(),
        Some(0.25)
    );
}

#[test]
fn sim_metrics_serialize_and_observe_landing_recovery_flip() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut non_recovery_sample =
        sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    non_recovery_sample.pose_intent_label = "gliding";
    non_recovery_sample.pose_landing_recovery_flip_degrees = 64.0;
    metrics.observe(&non_recovery_sample, scenario);
    assert_eq!(metrics.max_pose_landing_recovery_flip_degrees, 0.0);

    let mut recovery_sample = non_recovery_sample.clone();
    recovery_sample.pose_intent_label = "landing_recovery";
    recovery_sample.pose_landing_recovery_flip_degrees = 36.5;
    metrics.observe(&recovery_sample, scenario);
    assert_eq!(metrics.max_pose_landing_recovery_flip_degrees, 36.5);
    assert_eq!(
        recovery_sample.to_json()["pose_landing_recovery_flip_degrees"].as_f64(),
        Some(36.5)
    );

    let checks = metrics.checks(scenario);
    let result = SimResult {
        scenario,
        passed: checks.iter().all(|check| check.passed),
        metrics,
        checks,
        samples: vec![recovery_sample],
        elapsed_ms: 0.0,
        summary_path: String::new(),
        samples_path: String::new(),
    };
    let summary_json: serde_json::Value =
        serde_json::from_str(&result.to_summary_json()).expect("sim summary json parses");
    assert_eq!(
        summary_json["metrics"]["max_pose_landing_recovery_flip_degrees"].as_f64(),
        Some(36.5)
    );
    assert_eq!(
        summary_json["final_sample"]["pose_landing_recovery_flip_degrees"].as_f64(),
        Some(36.5)
    );
}

#[test]
fn target_landing_checks_gate_landing_recovery_and_foot_split() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    assert!(scenario.thresholds.require_target_landing);
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    let checks = metrics.checks(scenario);
    for name in [
        "pose_landing_anticipation_samples",
        "pose_landing_recovery_samples",
        "pose_landing_crouch",
        "pose_landing_foot_forward",
        "pose_landing_foot_split",
        "pose_landing_flare",
        "pose_landing_recovery_flip",
        "unreadable_key_pose_samples",
    ] {
        assert!(
            checks.iter().any(|check| check.name == name),
            "missing target landing check {name}"
        );
    }
    let recovery_check = checks
        .iter()
        .find(|check| check.name == "pose_landing_recovery_samples")
        .expect("landing recovery check");
    assert!(!recovery_check.passed);
    assert_eq!(recovery_check.threshold, 1.0);
    let flare_check = checks
        .iter()
        .find(|check| check.name == "pose_landing_flare")
        .expect("landing flare check");
    assert!(!flare_check.passed);
    assert_eq!(flare_check.threshold, LANDING_MIN_POSE_FLARE_DEGREES);
    assert_eq!(flare_check.unit, "deg");
    let foot_forward_check = checks
        .iter()
        .find(|check| check.name == "pose_landing_foot_forward")
        .expect("landing foot-forward check");
    assert!(!foot_forward_check.passed);
    assert_eq!(foot_forward_check.threshold, 0.32);
    assert_eq!(foot_forward_check.unit, "m");
    let foot_split_check = checks
        .iter()
        .find(|check| check.name == "pose_landing_foot_split")
        .expect("landing foot-split check");
    assert!(!foot_split_check.passed);
    assert_eq!(foot_split_check.threshold, LANDING_MIN_POSE_FOOT_SPLIT_M);
    assert_eq!(foot_split_check.unit, "m");
    let recovery_flip_check = checks
        .iter()
        .find(|check| check.name == "pose_landing_recovery_flip")
        .expect("landing recovery flip check");
    assert!(!recovery_flip_check.passed);
    assert_eq!(
        recovery_flip_check.threshold,
        LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES
    );
    assert_eq!(recovery_flip_check.unit, "deg");

    metrics.pose_landing_recovery_samples = 1;
    metrics.max_pose_landing_foot_forward_m = 0.32;
    metrics.max_pose_landing_foot_split_m = LANDING_MIN_POSE_FOOT_SPLIT_M;
    metrics.max_pose_landing_flare_degrees = LANDING_MIN_POSE_FLARE_DEGREES;
    let failing_flip_checks = metrics.checks(scenario);
    let failing_flip_check = failing_flip_checks
        .iter()
        .find(|check| check.name == "pose_landing_recovery_flip")
        .expect("landing recovery flip check");
    assert!(!failing_flip_check.passed);
    assert_eq!(failing_flip_check.value, 0.0);

    metrics.max_pose_landing_recovery_flip_degrees = LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES;
    let passing_checks = metrics.checks(scenario);
    let passing_recovery_check = passing_checks
        .iter()
        .find(|check| check.name == "pose_landing_recovery_samples")
        .expect("landing recovery check");
    assert!(passing_recovery_check.passed);
    let passing_flare_check = passing_checks
        .iter()
        .find(|check| check.name == "pose_landing_flare")
        .expect("landing flare check");
    assert!(passing_flare_check.passed);
    let passing_foot_forward_check = passing_checks
        .iter()
        .find(|check| check.name == "pose_landing_foot_forward")
        .expect("landing foot-forward check");
    assert!(passing_foot_forward_check.passed);
    let passing_foot_split_check = passing_checks
        .iter()
        .find(|check| check.name == "pose_landing_foot_split")
        .expect("landing foot-split check");
    assert!(passing_foot_split_check.passed);
    let passing_recovery_flip_check = passing_checks
        .iter()
        .find(|check| check.name == "pose_landing_recovery_flip")
        .expect("landing recovery flip check");
    assert!(passing_recovery_flip_check.passed);
}

#[test]
fn updraft_simulation_uses_readable_lift() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.lifted_samples >= scenario.thresholds.min_lifted_samples);
    assert_eq!(result.metrics.unreadable_lift_samples, 0);
    assert!(result.metrics.readable_lift_samples >= result.metrics.lifted_samples);
    assert!(result.metrics.dynamic_readable_lift_samples >= result.metrics.lifted_samples);
    assert!(result.metrics.dynamic_lift_samples >= scenario.thresholds.min_lifted_samples);
    assert!(result.metrics.max_paired_visual_lift_fields >= 1);
    assert!(result.metrics.max_dynamic_lift_fields >= 1);
    assert!(result.metrics.max_lift_applied_delta_mps >= MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS);
    assert!(result.metrics.max_dynamic_lift_multiplier_range >= MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE);
    assert!(result.metrics.max_wind_flow_speed_mps >= 8.0);
    assert!(result.metrics.max_wind_flow_variation >= 0.12);
    assert!(
        result.metrics.max_wind_flow_direction_change_degrees
            >= MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES
    );
    assert!(result.metrics.max_wind_flow_variation_range >= 0.03);
    assert!(result.metrics.max_dynamic_wind_flow_fields >= 2);
    assert!(result.metrics.layered_wind_force_samples >= 2);
    assert!(result.metrics.aligned_layered_wind_force_samples >= 2);
    assert!(result.metrics.crosswind_updraft_overlap_samples >= 2);
    assert!(result.metrics.aligned_crosswind_updraft_overlap_samples >= 2);
    assert!(result.metrics.max_layered_wind_force_fields >= 2);
    assert!(
        result.metrics.wind_load_response_samples >= MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT,
        "updraft route should include gust-synchronized wind-current load reaction samples"
    );
    assert!(result.metrics.max_wind_load_lateral_load >= MIN_WIND_LOAD_LATERAL_LOAD);
    assert!(result.metrics.max_wind_load_pose_lean_degrees >= MIN_WIND_LOAD_POSE_LEAN_DEGREES);
    assert!(
        result.metrics.max_wind_load_glider_response_degrees
            >= MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES
    );
    for check_name in [
        "dynamic_readable_lift_samples",
        "max_wind_flow_speed",
        "max_wind_flow_variation",
        "max_wind_flow_direction_change",
        "max_wind_flow_variation_range",
        "wind_force_samples",
        "meaningful_wind_force_samples",
        "aligned_wind_force_samples",
        "active_wind_force_fields",
        "wind_force_delta",
        "wind_force_flow_speed",
        "wind_force_variation",
        "wind_force_flow_alignment",
        "wind_force_aligned_delta",
        "updraft_swirl_force_samples",
        "aligned_updraft_swirl_force_samples",
        "updraft_swirl_force_fields",
        "updraft_swirl_force_delta",
        "updraft_swirl_force_flow_alignment",
        "updraft_swirl_force_aligned_delta",
        "layered_dynamic_wind_flow_fields",
        "layered_wind_force_samples",
        "aligned_layered_wind_force_samples",
        "crosswind_updraft_overlap_samples",
        "aligned_crosswind_updraft_overlap_samples",
        "layered_wind_force_fields",
        "layered_wind_force_delta",
        "layered_wind_force_flow_alignment",
        "layered_wind_force_aligned_delta",
        "wind_load_response_samples",
        "wind_load_lateral_load",
        "wind_load_pose_lean",
        "wind_load_glider_response",
        "dynamic_lift_samples",
        "paired_visual_lift_fields",
        "dynamic_lift_fields",
        "lift_applied_delta",
        "dynamic_lift_multiplier_range",
    ] {
        let check = result
            .checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("dynamic wind check");
        assert!(check.passed, "{check_name} should pass");
    }
    let dynamic_lift_sample = result
        .samples
        .iter()
        .find(|sample| {
            sample.dynamic_lift_fields > 0
                && sample.lift_applied_delta_mps > 0.001
                && sample.max_lift_multiplier > 1.0
        })
        .expect("dynamic lift sample");
    let dynamic_lift_sample_json = dynamic_lift_sample.to_json();
    assert_eq!(
        dynamic_lift_sample_json["paired_visual_lift_fields"].as_u64(),
        Some(dynamic_lift_sample.paired_visual_lift_fields as u64)
    );
    assert_eq!(
        dynamic_lift_sample_json["dynamic_lift_fields"].as_u64(),
        Some(dynamic_lift_sample.dynamic_lift_fields as u64)
    );
    assert!(
        dynamic_lift_sample_json["lift_applied_delta_mps"]
            .as_f64()
            .expect("sample lift delta is numeric")
            > 0.001
    );
    assert!(
        dynamic_lift_sample_json["max_lift_multiplier"]
            .as_f64()
            .expect("sample lift multiplier is numeric")
            > 1.0
    );
    let summary_json: serde_json::Value =
        serde_json::from_str(&result.to_summary_json()).expect("sim summary json parses");
    for key in [
        "dynamic_lift_samples",
        "max_paired_visual_lift_fields",
        "max_dynamic_lift_fields",
        "max_lift_applied_delta_mps",
        "min_dynamic_lift_multiplier",
        "max_dynamic_lift_multiplier",
        "max_dynamic_lift_multiplier_range",
    ] {
        assert!(
            summary_json["metrics"]
                .as_object()
                .expect("metrics object")
                .contains_key(key),
            "summary should include {key}"
        );
    }
    assert!(
        summary_json["metrics"]["max_lift_applied_delta_mps"]
            .as_f64()
            .expect("summary lift delta is numeric")
            >= MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS as f64
    );
    assert!(result.metrics.max_altitude_m >= scenario.thresholds.min_max_altitude_m);
}

#[test]
fn branch_recovery_simulation_completes_branch_objectives() {
    let scenario = scenario_named(BRANCH_RECOVERY_ROUTE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert_eq!(scenario.target_island_name, Some("sunlit terrace"));
    assert!(
        result.metrics.max_completed_objective_count
            >= scenario.thresholds.min_completed_objective_count
    );
    assert!(
        result.metrics.final_objective_completed_count
            >= scenario.thresholds.min_completed_objective_count
    );
    assert!(
        result.metrics.target_landing_samples >= scenario.thresholds.min_target_landing_samples
    );
    assert!(result.metrics.dynamic_lift_samples >= scenario.thresholds.min_lifted_samples);
    assert!(result.metrics.max_paired_visual_lift_fields >= 1);
    assert!(result.metrics.max_dynamic_lift_fields >= 1);
    assert!(result.metrics.max_lift_applied_delta_mps >= MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS);
    assert!(result.metrics.max_dynamic_lift_multiplier_range >= MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE);
}

#[test]
fn dynamic_lift_delta_gate_ignores_unpaired_static_lift() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    let mut static_lift_sample =
        sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    static_lift_sample.active_lift_fields = 1;
    static_lift_sample.paired_visual_lift_fields = 0;
    static_lift_sample.dynamic_lift_fields = 0;
    static_lift_sample.lift_applied_delta_mps = MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS + 1.0;
    static_lift_sample.min_lift_multiplier = 1.0;
    static_lift_sample.max_lift_multiplier = 1.0;
    metrics.observe(&static_lift_sample, scenario);

    assert_eq!(metrics.dynamic_lift_samples, 0);
    assert_eq!(metrics.max_lift_applied_delta_mps, 0.0);

    for frame in 40..(40 + scenario.thresholds.min_lifted_samples) {
        let mut weak_dynamic_sample =
            sim_roll_sample(&route, scenario, frame, FlightMode::Gliding, 0.0, 0.0);
        weak_dynamic_sample.active_lift_fields = 1;
        weak_dynamic_sample.paired_visual_lift_fields = 1;
        weak_dynamic_sample.dynamic_lift_fields = 1;
        weak_dynamic_sample.lift_applied_delta_mps = 0.01;
        weak_dynamic_sample.min_lift_multiplier = 0.9;
        weak_dynamic_sample.max_lift_multiplier = 1.1;
        metrics.observe(&weak_dynamic_sample, scenario);
    }

    assert_eq!(
        metrics.dynamic_lift_samples,
        scenario.thresholds.min_lifted_samples
    );
    let failing_delta_check = metrics
        .checks(scenario)
        .into_iter()
        .find(|check| check.name == "lift_applied_delta")
        .expect("lift applied delta check");
    assert!(!failing_delta_check.passed);
    assert!(failing_delta_check.value < failing_delta_check.threshold);

    let mut strong_dynamic_sample =
        sim_roll_sample(&route, scenario, 90, FlightMode::Gliding, 0.0, 0.0);
    strong_dynamic_sample.active_lift_fields = 1;
    strong_dynamic_sample.paired_visual_lift_fields = 1;
    strong_dynamic_sample.dynamic_lift_fields = 1;
    strong_dynamic_sample.lift_applied_delta_mps = MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS;
    strong_dynamic_sample.min_lift_multiplier = 0.9;
    strong_dynamic_sample.max_lift_multiplier = 1.1;
    metrics.observe(&strong_dynamic_sample, scenario);

    let passing_delta_check = metrics
        .checks(scenario)
        .into_iter()
        .find(|check| check.name == "lift_applied_delta")
        .expect("lift applied delta check");
    assert!(passing_delta_check.passed);
}

#[test]
fn long_glide_simulation_collects_boosts_and_crosses_archipelago() {
    let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.horizontal_distance_m >= scenario.thresholds.min_horizontal_distance_m);
    assert!(result.metrics.max_sky_island_count >= scenario.thresholds.min_sky_island_count);
    assert!(
        result.metrics.max_collected_power_up_count
            >= scenario.thresholds.min_collected_power_up_count
    );
    assert!(
        result.metrics.power_up_effect_samples >= scenario.thresholds.min_power_up_effect_samples
    );
}

#[test]
fn camera_yaw_simulation_exercises_scripted_mouse_without_motion() {
    let scenario = scenario_named("camera_yaw_stability").expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.max_abs_camera_yaw_offset_degrees >= 8.0);
    assert_eq!(result.metrics.grounded_samples, result.metrics.sample_count);
    assert_eq!(result.metrics.horizontal_distance_m, 0.0);
}

#[test]
fn camera_mouse_simulation_exercises_yaw_and_pitch_axes() {
    let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.max_abs_camera_yaw_offset_degrees >= 25.0);
    assert!(result.metrics.min_camera_pitch_offset_degrees <= -10.0);
    assert!(result.metrics.max_camera_pitch_offset_degrees >= 10.0);
    assert!(
        result.metrics.max_camera_obstruction_adjustment_m
            >= scenario.thresholds.min_camera_obstruction_adjustment_m
    );
    assert!(result.metrics.max_camera_obstruction_hits > 0);
}

#[test]
fn air_control_simulation_measures_backward_diagonal_response() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.max_backward_right_rear_response_mps >= 10.0);
    assert!(result.metrics.max_backward_left_rear_response_mps >= 10.0);
    assert!(result.metrics.max_air_brake_planar_speed_drop_mps >= 12.0);
    assert!(result.metrics.pose_air_turn_samples > 0);
    assert!(result.metrics.pose_air_brake_samples > 0);
    assert!(result.metrics.right_pose_air_brake_samples > 0);
    assert!(result.metrics.left_pose_air_brake_samples > 0);
    assert!(result.metrics.backward_right_pose_air_brake_samples > 0);
    assert!(result.metrics.backward_left_pose_air_brake_samples > 0);
    assert!(result.metrics.pose_diving_samples > 0);
    assert!(result.metrics.gliding_dive_samples > 0);
}

#[test]
fn air_control_simulation_keeps_pose_direction_through_short_input_gap() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let result = run_simulation(scenario);

    let held_turn_sample = result
        .samples
        .iter()
        .find(|sample| {
            sample.pose_intent_label == "air_turn"
                && sample.movement_input_lateral_axis.abs() <= f32::EPSILON
                && sample.pose_signed_lateral_lean_degrees.abs() >= 8.0
        })
        .expect("held air-turn pose should retain signed lean after input release");

    assert_eq!(held_turn_sample.movement_input_lateral_axis, 0.0);
    assert!(held_turn_sample.key_pose_readability_score >= 0.9);
}

#[test]
fn air_control_simulation_gates_directional_strafe_and_camera_drift() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    for name in [
        "air_control_right_lateral_response_latency",
        "air_control_right_lateral_response",
        "air_control_left_lateral_response_latency",
        "air_control_left_lateral_response",
        "air_control_backward_right_lateral_response_latency",
        "air_control_backward_right_lateral_response",
        "air_control_backward_left_lateral_response_latency",
        "air_control_backward_left_lateral_response",
        "air_control_camera_orbit_yaw_offset",
        "air_control_camera_rotation_delta",
        "air_control_camera_view_yaw_drift",
        "air_control_camera_world_yaw_drift",
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
        "air_control_unreadable_key_pose_samples",
        "air_control_key_pose_transition_grace_samples",
        "air_control_pose_air_turn_samples",
        "air_control_right_pose_air_turn_samples",
        "air_control_left_pose_air_turn_samples",
        "air_control_pose_air_brake_samples",
        "air_control_right_pose_air_brake_samples",
        "air_control_left_pose_air_brake_samples",
        "air_control_backward_right_pose_air_brake_samples",
        "air_control_backward_left_pose_air_brake_samples",
        "air_control_pose_diving_samples",
        "air_control_gliding_dive_samples",
        "air_control_lateral_body_travel_heading_samples",
        "air_control_right_body_travel_heading_samples",
        "air_control_left_body_travel_heading_samples",
        "air_control_p95_lateral_body_travel_heading_error",
        "air_control_max_lateral_body_travel_heading_error",
        "air_control_backward_diagonal_body_travel_heading_samples",
        "air_control_backward_right_diagonal_body_travel_heading_samples",
        "air_control_backward_left_diagonal_body_travel_heading_samples",
        "air_control_p95_backward_diagonal_body_travel_heading_error",
        "air_control_max_backward_diagonal_body_travel_heading_error",
        "air_control_desired_travel_heading_samples",
        "air_control_right_desired_travel_heading_samples",
        "air_control_left_desired_travel_heading_samples",
        "air_control_backward_right_desired_travel_heading_samples",
        "air_control_backward_left_desired_travel_heading_samples",
        "air_control_p95_desired_travel_heading_error",
        "air_control_max_desired_travel_heading_error",
        "air_control_pure_air_turn_sideways_samples",
        "air_control_right_pure_air_turn_sideways_samples",
        "air_control_left_pure_air_turn_sideways_samples",
        "air_control_p95_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_desired_travel_heading_error",
    ] {
        let check = result
            .checks
            .iter()
            .find(|check| check.name == name)
            .unwrap_or_else(|| panic!("missing sim check {name}"));
        assert!(check.passed, "expected {name} to pass: {check:?}");
    }

    let summary = result.to_summary_json();
    assert!(summary.contains("\"backward_right_lateral_response_latency_secs\""));
    assert!(summary.contains("\"backward_left_lateral_response_latency_secs\""));
    assert!(summary.contains("\"max_right_pose_lateral_lean_degrees\""));
    assert!(summary.contains("\"max_left_pose_lateral_lean_degrees\""));
    assert!(summary.contains("\"pose_air_turn_samples\""));
    assert!(summary.contains("\"right_pose_air_turn_samples\""));
    assert!(summary.contains("\"left_pose_air_turn_samples\""));
    assert!(summary.contains("\"right_pose_air_brake_samples\""));
    assert!(summary.contains("\"left_pose_air_brake_samples\""));
    assert!(summary.contains("\"backward_right_pose_air_brake_samples\""));
    assert!(summary.contains("\"backward_left_pose_air_brake_samples\""));
    assert!(summary.contains("\"key_pose_transition_grace_samples\""));
    assert!(summary.contains("\"lateral_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"right_lateral_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"left_lateral_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"p95_lateral_body_travel_heading_error_degrees\""));
    assert!(summary.contains("\"max_lateral_body_travel_heading_error_degrees\""));
    assert!(summary.contains("\"backward_diagonal_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"backward_right_diagonal_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"backward_left_diagonal_body_travel_heading_sample_count\""));
    assert!(summary.contains("\"p95_backward_diagonal_body_travel_heading_error_degrees\""));
    assert!(summary.contains("\"max_backward_diagonal_body_travel_heading_error_degrees\""));
    assert!(summary.contains("\"desired_travel_heading_sample_count\""));
    assert!(summary.contains("\"right_desired_travel_heading_sample_count\""));
    assert!(summary.contains("\"left_desired_travel_heading_sample_count\""));
    assert!(summary.contains("\"backward_right_desired_travel_heading_sample_count\""));
    assert!(summary.contains("\"backward_left_desired_travel_heading_sample_count\""));
    assert!(summary.contains("\"p95_desired_travel_heading_error_degrees\""));
    assert!(summary.contains("\"max_desired_travel_heading_error_degrees\""));
    assert!(summary.contains("\"pure_air_turn_sideways_sample_count\""));
    assert!(summary.contains("\"right_pure_air_turn_sideways_sample_count\""));
    assert!(summary.contains("\"left_pure_air_turn_sideways_sample_count\""));
    assert!(summary.contains("\"p95_pure_air_turn_sideways_body_travel_heading_error_degrees\""));
    assert!(summary.contains("\"max_pure_air_turn_sideways_body_travel_heading_error_degrees\""));
    assert!(
        summary.contains("\"p95_pure_air_turn_sideways_desired_travel_heading_error_degrees\"")
    );
    assert!(
        summary.contains("\"max_pure_air_turn_sideways_desired_travel_heading_error_degrees\"")
    );

    let summary_json: serde_json::Value =
        serde_json::from_str(&summary).expect("sim summary json parses");
    assert!(
        summary_json["metrics"]["desired_travel_heading_sample_count"]
            .as_u64()
            .expect("desired travel sample count is numeric")
            >= 8
    );
    for key in [
        "right_desired_travel_heading_sample_count",
        "left_desired_travel_heading_sample_count",
        "backward_right_desired_travel_heading_sample_count",
        "backward_left_desired_travel_heading_sample_count",
        "right_pure_air_turn_sideways_sample_count",
        "left_pure_air_turn_sideways_sample_count",
        "right_pose_air_turn_samples",
        "left_pose_air_turn_samples",
        "right_pose_air_brake_samples",
        "left_pose_air_brake_samples",
        "backward_right_pose_air_brake_samples",
        "backward_left_pose_air_brake_samples",
        "gliding_dive_samples",
    ] {
        assert!(
            summary_json["metrics"][key]
                .as_u64()
                .unwrap_or_else(|| panic!("{key} is numeric"))
                > 0,
            "{key} should have coverage"
        );
    }
    assert!(
        summary_json["metrics"]["p95_desired_travel_heading_error_degrees"]
            .as_f64()
            .expect("p95 desired travel heading error is numeric")
            <= 45.0
    );
    assert!(
        summary_json["metrics"]["max_desired_travel_heading_error_degrees"]
            .as_f64()
            .expect("max desired travel heading error is numeric")
            <= 65.0
    );
    assert!(
        summary_json["final_sample"]
            .as_object()
            .expect("final sample is an object")
            .contains_key("desired_travel_heading_error_degrees")
    );
}

#[test]
fn sim_sample_measures_pure_backward_body_heading_intent() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let input = FlightInput {
        backward: true,
        glide: true,
        ..Default::default()
    };
    let state = FlightState::new(
        START_POSITION + Vec3::Y * 8.0,
        Vec3::new(0.0, -2.0, 28.0),
        FlightController {
            mode: FlightMode::Gliding,
            ..Default::default()
        },
    );
    let player_rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(Vec3::Z, Vec3::Y)
        .rotation;
    let camera = CameraDiagnosticsSample {
        distance_m: 14.0,
        surface_clearance_m: 5.0,
        player_angle_degrees: 0.0,
        pitch_degrees: -18.0,
        step_distance_m: 0.0,
        rotation_delta_degrees: 0.0,
        orbit_alignment_degrees: 0.0,
        follow_direction_error_degrees: 0.0,
        view_yaw_degrees: 0.0,
        world_yaw_degrees: 0.0,
        obstruction_adjustment_m: 0.0,
        obstruction_hits: 0,
    };
    let objective = ObjectiveState::for_route(&route, scenario.target_island_name);
    let sample = SimSample::new(
        scenario,
        0,
        state,
        player_rotation,
        0.0,
        PlayerPoseIntent::AirBrake,
        nau_engine::camera::CameraOrbit::default(),
        camera,
        input,
        input,
        Facing::new(Vec3::Z, Vec3::X),
        &route,
        &[],
        &[],
        LiftApplication::default(),
        WindForceApplication::default(),
        &objective,
        &SimPowerUps::default(),
    );

    assert!(sample.desired_body_heading_error_degrees > 170.0);
    assert!(sample.desired_heading_alignment_mps < -20.0);
}

#[test]
fn sim_sample_uses_resolved_pose_input_without_relabeling_movement_axis() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let state = FlightState::new(
        START_POSITION + Vec3::new(0.0, 80.0, 0.0),
        Vec3::new(0.0, -2.0, -32.0),
        FlightController {
            mode: FlightMode::Gliding,
            ..Default::default()
        },
    );
    let player_rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(Vec3::Z, Vec3::Y)
        .rotation;
    let camera = CameraDiagnosticsSample {
        distance_m: 14.0,
        surface_clearance_m: 5.0,
        player_angle_degrees: 0.0,
        pitch_degrees: -18.0,
        step_distance_m: 0.0,
        rotation_delta_degrees: 0.0,
        orbit_alignment_degrees: 0.0,
        follow_direction_error_degrees: 0.0,
        view_yaw_degrees: 0.0,
        world_yaw_degrees: 0.0,
        obstruction_adjustment_m: 0.0,
        obstruction_hits: 0,
    };
    let objective = ObjectiveState::for_route(&route, scenario.target_island_name);
    let raw_input = FlightInput {
        glide: true,
        ..Default::default()
    };
    let held_pose_input = FlightInput {
        right: true,
        glide: true,
        ..Default::default()
    };

    let sample = SimSample::new(
        scenario,
        90,
        state,
        player_rotation,
        0.0,
        PlayerPoseIntent::AirTurn,
        nau_engine::camera::CameraOrbit::default(),
        camera,
        raw_input,
        held_pose_input,
        Facing::new(Vec3::Z, Vec3::X),
        &route,
        &[],
        &[],
        LiftApplication::default(),
        WindForceApplication::default(),
        &objective,
        &SimPowerUps::default(),
    );

    assert_eq!(sample.pose_intent_label, "air_turn");
    assert_eq!(sample.movement_input_lateral_axis, 0.0);
    assert!(sample.pose_signed_lateral_lean_degrees < -8.0);
}

#[test]
fn sim_metrics_fail_lateral_body_travel_heading_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let sample = sim_roll_sample(&route, scenario, 60, FlightMode::Gliding, 0.0, 1.0);

    metrics.observe(&sample, scenario);

    assert_eq!(
        metrics
            .lateral_body_travel_heading_error_values_degrees
            .len(),
        1
    );
    let checks = metrics.checks(scenario);
    let check = checks
        .iter()
        .find(|check| check.name == "air_control_max_lateral_body_travel_heading_error")
        .expect("lateral body/travel heading check");
    assert!(!check.passed, "expected wrong body/travel heading to fail");
    assert!(check.value > check.threshold);
}

#[test]
fn sim_metrics_ignore_body_yaw_intent_changes_for_oscillation_metrics() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for (frame, lateral_axis, yaw_error_degrees) in [
        (10, 1.0, 18.0),
        (20, -1.0, -18.0),
        (30, -1.0, -4.0),
        (40, 1.0, 18.0),
    ] {
        let mut sample = sim_roll_sample(
            &route,
            scenario,
            frame,
            FlightMode::Gliding,
            0.0,
            lateral_axis,
        );
        sample.desired_body_yaw_error_degrees = yaw_error_degrees;
        sample.desired_body_heading_error_degrees = yaw_error_degrees.abs();
        metrics.observe(&sample, scenario);
    }

    let checks = metrics.checks(scenario);
    let step = checks
        .iter()
        .find(|check| check.name == "air_control_max_body_yaw_error_step")
        .expect("yaw-step check");
    let oscillations = checks
        .iter()
        .find(|check| check.name == "air_control_body_yaw_oscillation_count")
        .expect("yaw-oscillation check");

    assert_eq!(metrics.max_body_yaw_error_step_degrees, 14.0);
    assert_eq!(metrics.body_yaw_oscillation_count, 0);
    assert!(step.passed);
    assert!(oscillations.passed);
}

#[test]
fn sim_metrics_fail_missing_body_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut lateral = sim_roll_sample(&route, scenario, 60, FlightMode::Gliding, 0.0, 1.0);
    lateral.body_travel_heading_error_degrees = f32::NAN;
    let mut backward_diagonal =
        sim_roll_sample(&route, scenario, 240, FlightMode::Gliding, 0.0, 1.0);
    backward_diagonal.movement_input_forward_axis = -1.0;
    backward_diagonal.body_travel_heading_error_degrees = f32::NAN;

    metrics.observe(&lateral, scenario);
    metrics.observe(&backward_diagonal, scenario);

    let checks = metrics.checks(scenario);
    for check_name in [
        "air_control_lateral_body_travel_heading_samples",
        "air_control_right_body_travel_heading_samples",
        "air_control_left_body_travel_heading_samples",
        "air_control_backward_diagonal_body_travel_heading_samples",
        "air_control_backward_right_diagonal_body_travel_heading_samples",
        "air_control_backward_left_diagonal_body_travel_heading_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("body/travel heading sample-count check");
        assert!(
            !check.passed,
            "{check_name} should fail without finite body/travel samples"
        );
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn sim_metrics_fail_one_sided_body_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let right = sim_roll_sample(&route, scenario, 60, FlightMode::Gliding, 0.0, 1.0);
    let mut backward_right = sim_roll_sample(&route, scenario, 240, FlightMode::Gliding, 0.0, 1.0);
    backward_right.movement_input_forward_axis = -1.0;

    metrics.observe(&right, scenario);
    metrics.observe(&backward_right, scenario);

    let checks = metrics.checks(scenario);
    for check_name in [
        "air_control_left_body_travel_heading_samples",
        "air_control_backward_left_diagonal_body_travel_heading_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("directional body/travel sample-count check");
        assert!(
            !check.passed,
            "{check_name} should fail without matching-direction finite samples"
        );
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn sim_metrics_fail_one_sided_desired_travel_heading_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for frame in [60, 90, 120, 150] {
        let sample = sim_roll_sample(&route, scenario, frame, FlightMode::Gliding, 0.0, 1.0);
        metrics.observe(&sample, scenario);
    }
    for frame in [240, 270, 300, 330] {
        let mut sample = sim_roll_sample(&route, scenario, frame, FlightMode::Gliding, 0.0, 1.0);
        sample.movement_input_forward_axis = -1.0;
        metrics.observe(&sample, scenario);
    }

    assert_eq!(metrics.desired_travel_heading_error_values_degrees.len(), 8);
    assert_eq!(metrics.right_desired_travel_heading_samples, 8);
    assert_eq!(metrics.backward_right_desired_travel_heading_samples, 4);

    let checks = metrics.checks(scenario);
    for check_name in [
        "air_control_left_desired_travel_heading_samples",
        "air_control_backward_left_desired_travel_heading_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("directional desired/travel sample-count check");
        assert!(
            !check.passed,
            "{check_name} should fail without matching-direction samples"
        );
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn sim_metrics_fail_backward_diagonal_body_travel_heading_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut sample = sim_roll_sample(&route, scenario, 240, FlightMode::Gliding, 0.0, 1.0);
    sample.movement_input_forward_axis = -1.0;

    metrics.observe(&sample, scenario);

    assert_eq!(
        metrics
            .backward_diagonal_body_travel_heading_error_values_degrees
            .len(),
        1
    );
    let checks = metrics.checks(scenario);
    let check = checks
        .iter()
        .find(|check| check.name == "air_control_max_backward_diagonal_body_travel_heading_error")
        .expect("backward diagonal body/travel heading check");
    assert!(
        !check.passed,
        "expected wrong backward diagonal body/travel heading to fail"
    );
    assert!(check.value > check.threshold);
}

#[test]
fn sim_metrics_reset_body_roll_step_across_grounded_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for sample in [
        sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, -12.0, 1.0),
        sim_roll_sample(&route, scenario, 60, FlightMode::Grounded, 0.0, 0.0),
        sim_roll_sample(&route, scenario, 90, FlightMode::Gliding, 12.0, -1.0),
    ] {
        metrics.observe(&sample, scenario);
    }

    assert_eq!(metrics.max_body_roll_step_degrees, 0.0);
    assert_eq!(metrics.max_right_body_bank_degrees, 12.0);
    assert_eq!(metrics.max_left_body_bank_degrees, 12.0);
}

#[test]
fn sim_metrics_track_signed_pose_lateral_lean_by_lateral_input_direction() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    let mut right_wrong_sign = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 1.0);
    right_wrong_sign.pose_signed_lateral_lean_degrees = 30.0;
    metrics.observe(&right_wrong_sign, scenario);

    let mut left_wrong_sign = sim_roll_sample(&route, scenario, 60, FlightMode::Gliding, 0.0, -1.0);
    left_wrong_sign.pose_signed_lateral_lean_degrees = -30.0;
    metrics.observe(&left_wrong_sign, scenario);

    let mut right_sample = right_wrong_sign.clone();
    right_sample.pose_signed_lateral_lean_degrees = -9.0;
    metrics.observe(&right_sample, scenario);

    let mut left_sample = left_wrong_sign.clone();
    left_sample.pose_signed_lateral_lean_degrees = 11.0;
    metrics.observe(&left_sample, scenario);

    assert_eq!(metrics.max_right_pose_lateral_lean_degrees, 9.0);
    assert_eq!(metrics.max_left_pose_lateral_lean_degrees, 11.0);
    assert_eq!(metrics.pose_air_turn_samples, 4);
    assert_eq!(metrics.right_pose_air_turn_samples, 2);
    assert_eq!(metrics.left_pose_air_turn_samples, 2);
}

#[test]
fn sim_metrics_count_readable_directional_air_brake_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    let mut right_sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 1.0);
    right_sample.pose_intent_label = "air_brake";
    right_sample.movement_input_forward_axis = -1.0;
    metrics.observe(&right_sample, scenario);

    let mut left_sample = sim_roll_sample(&route, scenario, 60, FlightMode::Gliding, 0.0, -1.0);
    left_sample.pose_intent_label = "air_brake";
    left_sample.movement_input_forward_axis = -1.0;
    metrics.observe(&left_sample, scenario);

    let mut forward_right_sample = right_sample.clone();
    forward_right_sample.movement_input_forward_axis = 1.0;
    metrics.observe(&forward_right_sample, scenario);

    assert_eq!(metrics.pose_air_brake_samples, 3);
    assert_eq!(metrics.right_pose_air_brake_samples, 2);
    assert_eq!(metrics.left_pose_air_brake_samples, 1);
    assert_eq!(metrics.backward_right_pose_air_brake_samples, 1);
    assert_eq!(metrics.backward_left_pose_air_brake_samples, 1);
}

#[test]
fn sim_metrics_count_deployed_glider_dive_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 0.0);
    sample.pose_intent_label = "diving";

    assert_eq!(sample.mode, "gliding");

    metrics.observe(&sample, scenario);

    assert_eq!(metrics.pose_diving_samples, 1);
    assert_eq!(metrics.gliding_dive_samples, 1);
    let check = metrics
        .checks(scenario)
        .into_iter()
        .find(|check| check.name == "air_control_gliding_dive_samples")
        .expect("gliding dive sample check");
    assert!(check.passed);
}

#[test]
fn sim_metrics_fail_one_sided_air_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for frame in [30, 60, 90, 120] {
        let sample = sim_roll_sample(&route, scenario, frame, FlightMode::Gliding, 0.0, 1.0);
        metrics.observe(&sample, scenario);
    }

    assert_eq!(metrics.pose_air_turn_samples, 4);
    assert_eq!(metrics.right_pose_air_turn_samples, 4);
    assert_eq!(metrics.left_pose_air_turn_samples, 0);

    let checks = metrics.checks(scenario);
    let aggregate = checks
        .iter()
        .find(|check| check.name == "air_control_pose_air_turn_samples")
        .expect("aggregate air-turn pose check");
    assert!(aggregate.passed);
    let left = checks
        .iter()
        .find(|check| check.name == "air_control_left_pose_air_turn_samples")
        .expect("left air-turn pose check");
    assert!(!left.passed, "left air-turn coverage should fail");
    assert_eq!(left.value, 0.0);
}

#[test]
fn sim_metrics_count_pure_air_turn_sideways_alignment_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for (frame, lateral_axis) in [
        (30, 1.0),
        (60, 1.0),
        (90, 1.0),
        (120, 1.0),
        (150, -1.0),
        (180, -1.0),
        (210, -1.0),
        (240, -1.0),
    ] {
        let mut sample = sim_roll_sample(
            &route,
            scenario,
            frame,
            FlightMode::Gliding,
            0.0,
            lateral_axis,
        );
        sample.lateral_response_mps = 18.0;
        sample.body_travel_heading_error_degrees = 4.0;
        sample.desired_travel_heading_error_degrees = 3.0;
        metrics.observe(&sample, scenario);
    }

    assert_eq!(
        metrics
            .pure_air_turn_sideways_body_travel_heading_error_values_degrees
            .len(),
        8
    );
    assert_eq!(metrics.right_pure_air_turn_sideways_samples, 4);
    assert_eq!(metrics.left_pure_air_turn_sideways_samples, 4);
    assert_eq!(
        metrics.p95_pure_air_turn_sideways_body_travel_heading_error_degrees(),
        4.0
    );
    assert_eq!(
        metrics.p95_pure_air_turn_sideways_desired_travel_heading_error_degrees(),
        3.0
    );

    for check_name in [
        "air_control_pure_air_turn_sideways_samples",
        "air_control_right_pure_air_turn_sideways_samples",
        "air_control_left_pure_air_turn_sideways_samples",
        "air_control_p95_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_desired_travel_heading_error",
    ] {
        let check = metrics
            .checks(scenario)
            .into_iter()
            .find(|check| check.name == check_name)
            .unwrap_or_else(|| panic!("missing sim check {check_name}"));
        assert!(check.passed, "{check_name} should pass: {check:?}");
    }
}

#[test]
fn sim_metrics_require_air_turn_sideways_alignment_in_same_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for (frame, lateral_axis) in [
        (30, 1.0),
        (60, 1.0),
        (90, 1.0),
        (120, 1.0),
        (150, -1.0),
        (180, -1.0),
        (210, -1.0),
        (240, -1.0),
    ] {
        let mut sample = sim_roll_sample(
            &route,
            scenario,
            frame,
            FlightMode::Gliding,
            0.0,
            lateral_axis,
        );
        sample.movement_input_forward_axis = 1.0;
        sample.lateral_response_mps = 18.0;
        sample.body_travel_heading_error_degrees = 4.0;
        sample.desired_travel_heading_error_degrees = 3.0;
        metrics.observe(&sample, scenario);
    }

    assert_eq!(metrics.pose_air_turn_samples, 8);
    assert_eq!(
        metrics
            .lateral_body_travel_heading_error_values_degrees
            .len(),
        8
    );
    assert_eq!(metrics.desired_travel_heading_error_values_degrees.len(), 8);
    assert_eq!(
        metrics
            .pure_air_turn_sideways_body_travel_heading_error_values_degrees
            .len(),
        0
    );

    for check_name in [
        "air_control_pure_air_turn_sideways_samples",
        "air_control_right_pure_air_turn_sideways_samples",
        "air_control_left_pure_air_turn_sideways_samples",
    ] {
        let check = metrics
            .checks(scenario)
            .into_iter()
            .find(|check| check.name == check_name)
            .unwrap_or_else(|| panic!("missing sim check {check_name}"));
        assert!(
            !check.passed,
            "{check_name} should fail without pure sideways samples"
        );
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn sim_metrics_fail_pure_air_turn_sideways_misalignment() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for (frame, lateral_axis) in [
        (30, 1.0),
        (60, 1.0),
        (90, 1.0),
        (120, 1.0),
        (150, -1.0),
        (180, -1.0),
        (210, -1.0),
        (240, -1.0),
    ] {
        let mut sample = sim_roll_sample(
            &route,
            scenario,
            frame,
            FlightMode::Gliding,
            0.0,
            lateral_axis,
        );
        sample.lateral_response_mps = 18.0;
        sample.body_travel_heading_error_degrees = 48.0;
        sample.desired_travel_heading_error_degrees = 48.0;
        metrics.observe(&sample, scenario);
    }

    assert_eq!(
        metrics
            .pure_air_turn_sideways_body_travel_heading_error_values_degrees
            .len(),
        8
    );

    for check_name in [
        "air_control_p95_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_body_travel_heading_error",
        "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error",
        "air_control_max_pure_air_turn_sideways_desired_travel_heading_error",
    ] {
        let check = metrics
            .checks(scenario)
            .into_iter()
            .find(|check| check.name == check_name)
            .unwrap_or_else(|| panic!("missing sim check {check_name}"));
        assert!(
            !check.passed,
            "{check_name} should fail for high heading error"
        );
        assert!(check.value > check.threshold);
    }
}

#[test]
fn sim_metrics_fail_one_sided_air_brake_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    for frame in [240, 270, 300, 330] {
        let mut sample = sim_roll_sample(&route, scenario, frame, FlightMode::Gliding, 0.0, 1.0);
        sample.pose_intent_label = "air_brake";
        sample.movement_input_forward_axis = -1.0;
        metrics.observe(&sample, scenario);
    }

    assert_eq!(metrics.pose_air_brake_samples, 4);
    assert_eq!(metrics.right_pose_air_brake_samples, 4);
    assert_eq!(metrics.left_pose_air_brake_samples, 0);
    assert_eq!(metrics.backward_right_pose_air_brake_samples, 4);
    assert_eq!(metrics.backward_left_pose_air_brake_samples, 0);

    let checks = metrics.checks(scenario);
    let aggregate = checks
        .iter()
        .find(|check| check.name == "air_control_pose_air_brake_samples")
        .expect("aggregate air-brake pose check");
    assert!(aggregate.passed);
    for check_name in [
        "air_control_left_pose_air_brake_samples",
        "air_control_backward_left_pose_air_brake_samples",
    ] {
        let check = checks
            .iter()
            .find(|check| check.name == check_name)
            .unwrap_or_else(|| panic!("missing sim check {check_name}"));
        assert!(!check.passed, "{check_name} coverage should fail");
        assert_eq!(check.value, 0.0);
    }
}

#[test]
fn sim_metrics_reject_unreadable_air_turn_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 1.0);
    sample.key_pose_readability_score = 0.25;

    metrics.observe(&sample, scenario);

    assert_eq!(metrics.pose_air_turn_samples, 0);
    assert_eq!(metrics.unreadable_key_pose_samples, 1);
}

#[test]
fn sim_metrics_reject_unreadable_air_brake_pose_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    let mut sample = sim_roll_sample(&route, scenario, 30, FlightMode::Gliding, 0.0, 1.0);
    sample.pose_intent_label = "air_brake";
    sample.movement_input_forward_axis = -1.0;
    sample.key_pose_readability_score = 0.25;

    metrics.observe(&sample, scenario);

    assert_eq!(metrics.pose_air_brake_samples, 0);
    assert_eq!(metrics.right_pose_air_brake_samples, 0);
    assert_eq!(metrics.backward_right_pose_air_brake_samples, 0);
    assert_eq!(metrics.unreadable_key_pose_samples, 1);
}

#[test]
fn air_control_sim_checks_reject_excess_transition_grace_samples() {
    let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    metrics.key_pose_transition_grace_samples =
        AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES + 1;

    let checks = metrics.checks(scenario);
    let check = checks
        .iter()
        .find(|check| check.name == "air_control_key_pose_transition_grace_samples")
        .expect("transition-grace check");

    assert_eq!(
        check.threshold,
        AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32
    );
    assert!(!check.passed);
}

#[test]
fn pose_state_sim_checks_reject_excess_transition_grace_samples() {
    let scenario = scenario_named(POSE_STATE_COVERAGE).expect("scenario");
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);
    metrics.key_pose_transition_grace_samples =
        POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES + 1;

    let checks = metrics.checks(scenario);
    let check = checks
        .iter()
        .find(|check| check.name == "pose_state_key_pose_transition_grace_samples")
        .expect("transition-grace check");

    assert_eq!(
        check.threshold,
        POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32
    );
    assert!(!check.passed);
}

fn sim_roll_sample(
    route: &SkyRoute,
    scenario: EvalScenario,
    frame: u32,
    mode: FlightMode,
    roll_degrees: f32,
    lateral_axis: f32,
) -> SimSample {
    let input = FlightInput {
        left: lateral_axis < 0.0,
        right: lateral_axis > 0.0,
        glide: mode == FlightMode::Gliding,
        ..Default::default()
    };
    let controller = FlightController {
        mode,
        ..Default::default()
    };
    let state = FlightState::new(
        START_POSITION + Vec3::Y * 8.0,
        Vec3::new(lateral_axis * 14.0, -2.0, -18.0),
        controller,
    );
    let player_rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(Vec3::Z, Vec3::Y)
        .rotation
        * Quat::from_rotation_z(roll_degrees.to_radians());
    let camera = CameraDiagnosticsSample {
        distance_m: 14.0,
        surface_clearance_m: 5.0,
        player_angle_degrees: 0.0,
        pitch_degrees: -18.0,
        step_distance_m: 0.0,
        rotation_delta_degrees: 0.0,
        orbit_alignment_degrees: 0.0,
        follow_direction_error_degrees: 0.0,
        view_yaw_degrees: 0.0,
        world_yaw_degrees: 0.0,
        obstruction_adjustment_m: 0.0,
        obstruction_hits: 0,
    };
    let objective = ObjectiveState::for_route(route, scenario.target_island_name);
    let power_ups = SimPowerUps::default();
    let pose_intent = if mode == FlightMode::Gliding && lateral_axis.abs() > 0.0 {
        PlayerPoseIntent::AirTurn
    } else {
        PlayerPoseIntent::Gliding
    };

    SimSample::new(
        scenario,
        frame,
        state,
        player_rotation,
        0.0,
        pose_intent,
        nau_engine::camera::CameraOrbit::default(),
        camera,
        input,
        input,
        Facing::new(Vec3::Z, Vec3::X),
        route,
        &[],
        &[],
        LiftApplication::default(),
        WindForceApplication::default(),
        &objective,
        &power_ups,
    )
}
