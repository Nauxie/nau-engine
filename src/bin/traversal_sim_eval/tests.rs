use super::{
    metrics::SimMetrics,
    sample::{CameraDiagnosticsSample, SimSample},
    simulation::run_simulation,
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    environment::WindForceApplication,
    eval::{
        AIR_CONTROL_RESPONSE, BRANCH_RECOVERY_ROUTE, CAMERA_MOUSE_CONTROL, EvalScenario,
        ISLAND_LAUNCH_TO_LANDING, LONG_GLIDE_VISIBILITY, UPDRAFT_ROUTE, scenario_named,
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
    assert!(summary.contains("\"pose_air_turn_samples\""));
    assert!(summary.contains("\"pose_landing_recovery_samples\""));
    assert!(summary.contains("\"max_pose_landing_flare_degrees\""));
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
            .get("max_wind_flow_variation")
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
    ] {
        assert!(
            last_sample_json.get(key).is_some(),
            "{key} should be serialized"
        );
    }
    for check_name in [
        "wind_force_samples",
        "meaningful_wind_force_samples",
        "active_wind_force_fields",
        "wind_force_delta",
        "wind_force_flow_speed",
        "wind_force_variation",
        "crosswind_force_samples",
        "crosswind_force_fields",
        "crosswind_force_delta",
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
fn sim_metrics_track_landing_flare_from_landing_anticipation_pose_only() {
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
    metrics.observe(&landing_sample, scenario);

    assert_eq!(metrics.max_pose_torso_pitch_degrees, 72.0);
    assert_eq!(metrics.max_pose_landing_flare_degrees, 34.0);
}

#[test]
fn target_landing_checks_gate_landing_recovery_samples_and_flare() {
    let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("scenario");
    assert!(scenario.thresholds.require_target_landing);
    let route = SkyRoute::default();
    let mut metrics = SimMetrics::new(&route);

    let checks = metrics.checks(scenario);
    for name in [
        "pose_landing_anticipation_samples",
        "pose_landing_recovery_samples",
        "pose_landing_crouch",
        "pose_landing_flare",
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
    assert_eq!(flare_check.threshold, 48.0);
    assert_eq!(flare_check.unit, "deg");

    metrics.pose_landing_recovery_samples = 1;
    metrics.max_pose_landing_flare_degrees = 48.0;
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
    assert!(result.metrics.max_wind_flow_speed_mps >= 8.0);
    assert!(result.metrics.max_wind_flow_variation >= 0.12);
    assert!(result.metrics.max_wind_flow_variation_range >= 0.03);
    for check_name in [
        "dynamic_readable_lift_samples",
        "max_wind_flow_speed",
        "max_wind_flow_variation",
        "max_wind_flow_variation_range",
        "wind_force_samples",
        "meaningful_wind_force_samples",
        "active_wind_force_fields",
        "wind_force_delta",
        "wind_force_flow_speed",
        "wind_force_variation",
        "updraft_swirl_force_samples",
        "updraft_swirl_force_fields",
        "updraft_swirl_force_delta",
    ] {
        let check = result
            .checks
            .iter()
            .find(|check| check.name == check_name)
            .expect("dynamic wind check");
        assert!(check.passed, "{check_name} should pass");
    }
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
    assert!(result.metrics.pose_diving_samples > 0);
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
        "air_control_pose_lateral_lean",
        "air_control_right_pose_lateral_lean",
        "air_control_left_pose_lateral_lean",
        "air_control_pose_wing_airflow",
        "air_control_unreadable_key_pose_samples",
        "air_control_pose_air_turn_samples",
        "air_control_right_pose_air_turn_samples",
        "air_control_left_pose_air_turn_samples",
        "air_control_pose_air_brake_samples",
        "air_control_pose_diving_samples",
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
        "right_pose_air_turn_samples",
        "left_pose_air_turn_samples",
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
        nau_engine::camera::CameraOrbit::default(),
        camera,
        input,
        Facing::new(Vec3::Z, Vec3::X),
        &route,
        &[],
        &[],
        WindForceApplication::default(),
        &objective,
        &SimPowerUps::default(),
    );

    assert!(sample.desired_body_heading_error_degrees > 170.0);
    assert!(sample.desired_heading_alignment_mps < -20.0);
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

    SimSample::new(
        scenario,
        frame,
        state,
        player_rotation,
        0.0,
        nau_engine::camera::CameraOrbit::default(),
        camera,
        input,
        Facing::new(Vec3::Z, Vec3::X),
        route,
        &[],
        &[],
        WindForceApplication::default(),
        &objective,
        &power_ups,
    )
}
