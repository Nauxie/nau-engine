use super::{
    metrics::SimMetrics,
    sample::{CameraDiagnosticsSample, SimSample},
    simulation::run_simulation,
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
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
    assert!(
        result
            .samples
            .last()
            .unwrap()
            .to_json()
            .get("pose_intent")
            .is_some()
    );
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
fn updraft_simulation_uses_readable_lift() {
    let scenario = scenario_named(UPDRAFT_ROUTE).expect("scenario");
    let result = run_simulation(scenario);

    assert!(result.passed);
    assert!(result.metrics.lifted_samples >= scenario.thresholds.min_lifted_samples);
    assert_eq!(result.metrics.unreadable_lift_samples, 0);
    assert!(result.metrics.readable_lift_samples >= result.metrics.lifted_samples);
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
        "air_control_pose_air_brake_samples",
        "air_control_pose_diving_samples",
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
        nau_engine::camera::CameraOrbit::default(),
        camera,
        input,
        Facing::new(Vec3::Z, Vec3::X),
        route,
        &[],
        &[],
        &objective,
        &power_ups,
    )
}
