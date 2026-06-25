#![recursion_limit = "512"]

use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    camera::{
        CameraControlState, CameraControlTuning, CameraObstruction, FollowCamera,
        FollowCameraState, apply_camera_input, avoid_camera_obstructions, camera_distance,
        camera_orbit_alignment_degrees, camera_pitch_degrees, camera_surface_clearance,
        camera_target_angle_degrees, camera_view_yaw_degrees, lift_camera_above_floor,
        movement_input_stable_follow_direction, step_camera_with_direction,
        update_follow_direction_state,
    },
    environment::{
        AERIAL_POWER_UP_ROUTE, GAMEPLAY_LIFT_ROUTE, LiftField, WindField, active_lift_fields_at,
        apply_aerial_power_up, apply_lift_fields, readable_lift_fields_at, visible_fields_at,
    },
    eval::{EvalScenario, SCENARIO_NAMES, scenario_named, scripted_camera_input, scripted_input},
    movement::{
        Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning,
        body_roll_degrees, body_yaw_error_degrees, desired_heading_alignment_speed,
        desired_planar_movement_direction, face_flight_direction, lateral_response_speed,
    },
    world::{START_POSITION, SkyRoute},
};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

#[path = "traversal_sim_eval/metrics.rs"]
mod metrics;

use metrics::{SimMetrics, SimResult};

const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;
const AIR_CONTROL_RESPONSE_THRESHOLD_MPS: f32 = 4.0;
const AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS: f32 = 18.0;
const AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS: f32 = 10.0;
const AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS: f32 = 10.0;
const AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS: f32 = 20.0;
const AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES: f32 = 8.0;
const AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES: f32 = 22.0;
const AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES: f32 = 36.0;
const AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES: f32 = 36.0;
const AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS: f32 = 4.0;
const AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES: f32 = 8.0;
const AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES: f32 = 12.0;
const AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES: f32 = 0.01;
const AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES: f32 = 2.0;
const AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
const AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS: f32 = 0.20;
const AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS: f32 = 12.0;
const AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS: f32 = 12.0;
const AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS: f32 = 14.0;
const AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES: f32 = 8.0;
const CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS: f32 = 8.0;
const CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
const MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES: f32 = 2.0;
const GROUND_VISUAL_FOOT_GAP_M: f32 = 0.0;

fn main() {
    let options = match SimOptions::from_env() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    if let Err(error) = run_and_write(options) {
        eprintln!("traversal simulation eval failed: {error}");
        std::process::exit(1);
    }
}

#[derive(Clone, Debug)]
struct SimOptions {
    scenario: EvalScenario,
    output_dir: PathBuf,
}

impl SimOptions {
    fn from_env() -> Result<Self, String> {
        parse_args(env::args().skip(1))
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<SimOptions, String> {
    let mut scenario_name = None;
    let mut output_dir = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err("help requested".to_string()),
            "--scenario" => {
                scenario_name = Some(
                    args.next()
                        .ok_or_else(|| "--scenario requires a scenario name".to_string())?,
                );
            }
            "--output" => {
                output_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "--output requires a directory".to_string()
                    })?));
            }
            _ if arg.starts_with("--scenario=") => {
                scenario_name = Some(arg.trim_start_matches("--scenario=").to_string());
            }
            _ if arg.starts_with("--output=") => {
                output_dir = Some(PathBuf::from(arg.trim_start_matches("--output=")));
            }
            _ if scenario_name.is_none() => scenario_name = Some(arg),
            _ if output_dir.is_none() => output_dir = Some(PathBuf::from(arg)),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let scenario_name = scenario_name.unwrap_or_else(|| "baseline_route".to_string());
    let scenario = scenario_named(&scenario_name).ok_or_else(|| {
        format!(
            "unknown eval scenario: {scenario_name}. available scenarios: {}",
            SCENARIO_NAMES.join(", ")
        )
    })?;
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("target/eval").join(scenario.name));

    Ok(SimOptions {
        scenario,
        output_dir,
    })
}

fn usage() -> String {
    format!(
        "Usage:\n  cargo run --bin traversal_sim_eval -- [scenario] [output_dir]\n  cargo run --bin traversal_sim_eval -- --scenario <scenario> --output <dir>\n\nScenarios: {}",
        SCENARIO_NAMES.join(", ")
    )
}

fn run_and_write(options: SimOptions) -> Result<(), String> {
    fs::create_dir_all(&options.output_dir)
        .map_err(|error| format!("failed to create output directory: {error}"))?;
    let summary_path = options.output_dir.join("summary.json");
    let samples_path = options.output_dir.join("samples.ndjson");
    remove_existing_file(&summary_path)?;
    remove_existing_file(&samples_path)?;
    File::create(&samples_path)
        .map_err(|error| format!("failed to create samples file: {error}"))?;

    let started = Instant::now();
    let mut result = run_simulation(options.scenario);
    result.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    result.summary_path = path_string(&summary_path);
    result.samples_path = path_string(&samples_path);

    let mut samples_file = OpenOptions::new()
        .append(true)
        .open(&samples_path)
        .map_err(|error| format!("failed to open samples file: {error}"))?;
    for sample in &result.samples {
        writeln!(samples_file, "{}", sample.to_json())
            .map_err(|error| format!("failed to write sample: {error}"))?;
    }
    fs::write(&summary_path, result.to_summary_json())
        .map_err(|error| format!("failed to write summary: {error}"))?;

    eprintln!("traversal sim summary: {}", path_string(&summary_path));
    if result.passed {
        Ok(())
    } else {
        Err(format!(
            "simulation checks failed: {}",
            path_string(&summary_path)
        ))
    }
}

fn remove_existing_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to remove {}: {error}", path_string(path))),
    }
}

fn run_simulation(scenario: EvalScenario) -> SimResult {
    let route = SkyRoute::default();
    let tuning = FlightTuning::default();
    let follow = FollowCamera::default();
    let camera_tuning = CameraControlTuning::default();
    let lift_fields = GAMEPLAY_LIFT_ROUTE
        .iter()
        .map(|node| node.lift_field())
        .collect::<Vec<_>>();
    let visual_fields = visual_wind_fields();
    let obstructions = camera_obstructions();
    let mut power_ups = SimPowerUps::default();
    let mut objective = ObjectiveState::for_route(&route, scenario.target_island_name);
    let mut state = FlightState::new(START_POSITION, Vec3::ZERO, FlightController::default());
    let mut player_rotation = Quat::IDENTITY;
    let initial_camera_direction = Vec3::NEG_Z;
    let mut camera_transform = Transform::from_translation(
        START_POSITION - initial_camera_direction * follow.distance + Vec3::Y * follow.height,
    )
    .looking_at(
        START_POSITION
            + Vec3::Y * follow.look_height
            + initial_camera_direction * follow.look_ahead,
        Vec3::Y,
    );
    let mut camera_control = CameraControlState::default();
    let mut follow_state = FollowCameraState::default();
    let mut samples = Vec::new();
    let mut metrics = SimMetrics::new(&route);

    for frame in 0..=scenario.frame_count {
        let input = scripted_input(scenario, frame);
        power_ups.begin_frame(scenario.fixed_dt);
        let facing = Facing::new(*camera_transform.forward(), *camera_transform.right());
        let movement_facing = facing;
        let was_grounded = route.is_grounded_at(state.position);
        let mut frame_tuning = tuning;
        frame_tuning.floor_y = route.ground_at(state.position).floor_y;
        state = step_flight_with_world(
            state,
            input,
            facing,
            &frame_tuning,
            &route,
            &lift_fields,
            &mut power_ups,
            scenario.fixed_dt,
            was_grounded,
        );
        player_rotation = face_flight_direction(
            player_rotation,
            state.velocity,
            input,
            facing,
            state.controller,
            &frame_tuning,
            scenario.fixed_dt,
        );

        let camera_input = scripted_camera_input(scenario, frame);
        if camera_input.mouse_delta.length_squared() > 0.0 {
            camera_control.orbit =
                apply_camera_input(camera_control.orbit, camera_input, &camera_tuning);
        }
        let previous_camera_position = camera_transform.translation;
        let previous_camera_rotation = camera_transform.rotation;
        let player_forward = player_rotation * Vec3::NEG_Z;
        let desired_follow_direction = movement_input_stable_follow_direction(
            state.velocity,
            player_forward,
            follow_state.direction,
            input.planar_axis(),
        );
        let follow_direction = update_follow_direction_state(
            &mut follow_state,
            desired_follow_direction,
            &follow,
            scenario.fixed_dt,
        );
        let camera_step = step_camera_frame(
            camera_transform,
            state.position,
            follow_direction,
            &follow,
            camera_control.orbit,
            &route,
            &obstructions,
            scenario.fixed_dt,
        );
        camera_transform.translation = camera_step.position;
        camera_transform.rotation = camera_step.rotation;
        objective.update(
            &route,
            scenario.target_island_name,
            state.position,
            state.controller.mode,
        );

        if scenario.should_sample(frame) {
            let camera_diagnostics = CameraDiagnosticsSample::new(
                previous_camera_position,
                previous_camera_rotation,
                camera_transform,
                state.position,
                follow_direction,
                desired_follow_direction,
                camera_step,
                &route,
            );
            let sample = SimSample::new(
                scenario,
                frame,
                state,
                player_rotation,
                camera_control.orbit,
                camera_diagnostics,
                input,
                movement_facing,
                &route,
                &lift_fields,
                &visual_fields,
                &objective,
                &power_ups,
            );
            metrics.observe(&sample, scenario);
            samples.push(sample);
        }
    }

    let checks = metrics.checks(scenario);
    let passed = checks.iter().all(|check| check.passed);
    SimResult {
        scenario,
        passed,
        metrics,
        checks,
        samples,
        elapsed_ms: 0.0,
        summary_path: String::new(),
        samples_path: String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn step_flight_with_world(
    state: FlightState,
    input: FlightInput,
    facing: Facing,
    tuning: &FlightTuning,
    route: &SkyRoute,
    lift_fields: &[LiftField],
    power_ups: &mut SimPowerUps,
    dt: f32,
    was_grounded: bool,
) -> FlightState {
    let mut next = nau_engine::movement::step_flight(state, input, facing, tuning, dt);
    let lift = apply_lift_fields(
        next.position,
        next.velocity,
        lift_fields.iter().copied(),
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = lift.velocity;
    collect_aerial_power_ups(&mut next, power_ups);
    route.resolve_ground_contact_after_step(next, was_grounded)
}

fn collect_aerial_power_ups(state: &mut FlightState, power_ups: &mut SimPowerUps) {
    if state.controller.mode == FlightMode::Grounded {
        return;
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if !power_ups.is_collected(power_up.name) && power_up.contains(state.position) {
            state.velocity = apply_aerial_power_up(state.velocity, power_up);
            power_ups.collect(power_up.name, power_up.effect_duration_secs);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn step_camera_frame(
    current: Transform,
    player_position: Vec3,
    follow_direction: Vec3,
    follow: &FollowCamera,
    orbit: nau_engine::camera::CameraOrbit,
    route: &SkyRoute,
    obstructions: &[CameraObstruction],
    dt: f32,
) -> CameraStepSample {
    let frame = step_camera_with_direction(
        current.translation,
        current.rotation,
        player_position,
        follow_direction,
        follow,
        orbit,
        dt,
    );
    let orbit_alignment_degrees =
        camera_orbit_alignment_degrees(frame.position, frame.look_target, follow_direction, orbit);
    let camera_floor_y = route.ground_at(frame.position).floor_y;
    let frame = lift_camera_above_floor(frame, camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE);
    let obstruction = avoid_camera_obstructions(
        frame,
        obstructions.iter().copied(),
        CAMERA_OBSTRUCTION_CLEARANCE,
    );
    let camera_floor_y = route.ground_at(obstruction.frame.position).floor_y;
    let frame = lift_camera_above_floor(
        obstruction.frame,
        camera_floor_y,
        CAMERA_MIN_SURFACE_CLEARANCE,
    );

    CameraStepSample {
        position: frame.position,
        rotation: frame.rotation,
        orbit_alignment_degrees,
        obstruction_adjustment_m: obstruction.adjusted_distance_m,
        obstruction_hits: obstruction.hit_count,
    }
}

fn visual_wind_fields() -> Vec<WindField> {
    let mut fields = vec![
        WindField::crosswind(
            Vec3::new(0.0, 5.0, 20.0),
            Vec3::new(20.0, 4.0, 8.0),
            Vec3::X,
            10.0,
        ),
        WindField::crosswind(
            Vec3::new(34.0, 10.0, -8.0),
            Vec3::new(18.0, 8.0, 10.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        ),
    ];
    fields.extend(GAMEPLAY_LIFT_ROUTE.iter().map(|node| node.visual_field()));
    fields
}

fn camera_obstructions() -> Vec<CameraObstruction> {
    (-5..=5)
        .enumerate()
        .map(|(index, x)| {
            let height = 5.0 + (index as f32 % 4.0) * 4.0;
            let z = if index % 2 == 0 { -28.0 } else { 34.0 };
            let center = Vec3::new(x as f32 * 20.0, height * 0.5, z);
            CameraObstruction::new(center, Vec3::new(2.5, height * 0.5, 2.5))
        })
        .collect()
}

#[derive(Clone, Debug)]
struct CameraStepSample {
    position: Vec3,
    rotation: Quat,
    orbit_alignment_degrees: f32,
    obstruction_adjustment_m: f32,
    obstruction_hits: usize,
}

#[derive(Clone, Debug)]
struct CameraDiagnosticsSample {
    distance_m: f32,
    surface_clearance_m: f32,
    player_angle_degrees: f32,
    pitch_degrees: f32,
    step_distance_m: f32,
    rotation_delta_degrees: f32,
    orbit_alignment_degrees: f32,
    follow_direction_error_degrees: f32,
    view_yaw_degrees: f32,
    world_yaw_degrees: f32,
    obstruction_adjustment_m: f32,
    obstruction_hits: usize,
}

impl CameraDiagnosticsSample {
    #[allow(clippy::too_many_arguments)]
    fn new(
        previous_position: Vec3,
        previous_rotation: Quat,
        camera: Transform,
        player_position: Vec3,
        follow_direction: Vec3,
        desired_follow_direction: Vec3,
        camera_step: CameraStepSample,
        route: &SkyRoute,
    ) -> Self {
        let camera_floor_y = route.ground_at(camera.translation).floor_y;
        let player_focus = player_position + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
        Self {
            distance_m: camera_distance(camera.translation, player_position),
            surface_clearance_m: camera_surface_clearance(camera.translation, camera_floor_y),
            player_angle_degrees: camera_target_angle_degrees(
                camera.translation,
                camera.rotation,
                player_focus,
            ),
            pitch_degrees: camera_pitch_degrees(camera.rotation),
            step_distance_m: previous_position.distance(camera.translation),
            rotation_delta_degrees: previous_rotation
                .angle_between(camera.rotation)
                .to_degrees(),
            orbit_alignment_degrees: camera_step.orbit_alignment_degrees,
            follow_direction_error_degrees: follow_direction
                .angle_between(desired_follow_direction)
                .to_degrees(),
            view_yaw_degrees: camera_view_yaw_degrees(camera.rotation, follow_direction),
            world_yaw_degrees: camera_view_yaw_degrees(camera.rotation, Vec3::NEG_Z),
            obstruction_adjustment_m: camera_step.obstruction_adjustment_m,
            obstruction_hits: camera_step.obstruction_hits,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SimPowerUps {
    collected: HashSet<&'static str>,
    activations_this_frame: usize,
    total_activations: usize,
    effect_timer_secs: f32,
}

impl SimPowerUps {
    fn begin_frame(&mut self, dt: f32) {
        self.activations_this_frame = 0;
        self.effect_timer_secs = (self.effect_timer_secs - dt.max(0.0)).max(0.0);
    }

    fn collect(&mut self, name: &'static str, duration_secs: f32) {
        if self.collected.insert(name) {
            self.activations_this_frame += 1;
            self.total_activations += 1;
            self.effect_timer_secs = self.effect_timer_secs.max(duration_secs);
        }
    }

    fn is_collected(&self, name: &'static str) -> bool {
        self.collected.contains(name)
    }

    fn collected_count(&self) -> usize {
        self.collected.len()
    }

    fn visible_count(&self) -> usize {
        AERIAL_POWER_UP_ROUTE
            .len()
            .saturating_sub(self.collected.len())
    }

    fn active_effects(&self) -> usize {
        usize::from(self.effect_timer_secs > 0.0)
    }
}

#[derive(Clone, Debug)]
struct ObjectiveState {
    target_island_name: Option<&'static str>,
    completed_count: usize,
    total_count: usize,
    current_label: &'static str,
    current_distance_m: f32,
    complete: bool,
}

impl ObjectiveState {
    fn for_route(route: &SkyRoute, target_island_name: Option<&'static str>) -> Self {
        let mut state = Self {
            target_island_name,
            completed_count: 0,
            total_count: 0,
            current_label: "none",
            current_distance_m: 0.0,
            complete: false,
        };
        state.update(
            route,
            target_island_name,
            START_POSITION,
            FlightMode::Grounded,
        );
        state
    }

    fn update(
        &mut self,
        route: &SkyRoute,
        target_island_name: Option<&'static str>,
        position: Vec3,
        mode: FlightMode,
    ) {
        if self.target_island_name != target_island_name {
            *self = Self::for_route(route, target_island_name);
        }

        let objectives = route.route_objectives(target_island_name);
        self.total_count = objectives.len();
        self.completed_count = self.completed_count.min(objectives.len());

        while let Some(objective) = objectives.get(self.completed_count).copied() {
            if !objective.is_complete(route, position, mode) {
                break;
            }
            self.completed_count += 1;
        }

        if let Some(objective) = objectives.get(self.completed_count).copied() {
            self.current_label = objective.label;
            self.current_distance_m = objective.horizontal_distance(position);
            self.complete = false;
        } else {
            self.current_label = "complete";
            self.current_distance_m = 0.0;
            self.complete = !objectives.is_empty();
        }
    }

    fn current_step(&self) -> usize {
        if self.total_count == 0 {
            0
        } else {
            (self.completed_count + 1).min(self.total_count)
        }
    }
}

#[derive(Clone, Debug)]
struct SimSample {
    frame: u32,
    time_secs: f32,
    position: Vec3,
    velocity: Vec3,
    speed_mps: f32,
    altitude_m: f32,
    mode: &'static str,
    desired_body_yaw_error_degrees: f32,
    desired_body_heading_error_degrees: f32,
    body_roll_degrees: f32,
    desired_heading_alignment_mps: f32,
    lateral_response_mps: f32,
    lateral_input_active: bool,
    movement_input_lateral_axis: f32,
    movement_input_forward_axis: f32,
    camera_distance_m: f32,
    camera_surface_clearance_m: f32,
    camera_player_angle_degrees: f32,
    camera_pitch_degrees: f32,
    camera_yaw_offset_degrees: f32,
    camera_pitch_offset_degrees: f32,
    camera_step_distance_m: f32,
    camera_rotation_delta_degrees: f32,
    camera_orbit_alignment_degrees: f32,
    camera_follow_direction_error_degrees: f32,
    camera_view_yaw_degrees: f32,
    camera_world_yaw_degrees: f32,
    camera_obstruction_adjustment_m: f32,
    camera_obstruction_hits: usize,
    visible_wind_fields: usize,
    wind_field_count: usize,
    active_lift_fields: usize,
    readable_lift_fields: usize,
    lift_field_count: usize,
    target_distance_m: f32,
    on_landing_target: bool,
    objective: ObjectiveState,
    sky_island_count: usize,
    active_chunk_count: usize,
    active_island_count: usize,
    near_lod_islands: usize,
    mid_lod_islands: usize,
    far_lod_islands: usize,
    power_up_count: usize,
    visible_power_up_count: usize,
    collected_power_up_count: usize,
    active_power_up_effects: usize,
    total_power_up_activations: usize,
}

impl SimSample {
    #[allow(clippy::too_many_arguments)]
    fn new(
        scenario: EvalScenario,
        frame: u32,
        state: FlightState,
        player_rotation: Quat,
        orbit: nau_engine::camera::CameraOrbit,
        camera: CameraDiagnosticsSample,
        input: FlightInput,
        facing: Facing,
        route: &SkyRoute,
        lift_fields: &[LiftField],
        visual_fields: &[WindField],
        objective: &ObjectiveState,
        power_ups: &SimPowerUps,
    ) -> Self {
        let movement_axis = input.planar_axis();
        let desired_movement_direction = if input.forward || input.left || input.right {
            desired_planar_movement_direction(input, facing)
        } else {
            None
        };
        let desired_body_yaw_error_degrees = desired_movement_direction
            .map(|direction| body_yaw_error_degrees(player_rotation, direction))
            .unwrap_or(f32::NAN);
        let desired_heading_alignment_mps = desired_movement_direction
            .map(|direction| desired_heading_alignment_speed(state.velocity, direction))
            .unwrap_or(f32::NAN);
        let lateral_axis_active = input.has_lateral_axis();
        let lateral_input_active =
            lateral_axis_active && state.controller.mode != FlightMode::Grounded;
        let lateral_response_mps = if lateral_axis_active {
            lateral_response_speed(state.velocity, input, facing)
        } else {
            0.0
        };
        let streaming_lod = route.streaming_lod_stats(state.position);

        Self {
            frame,
            time_secs: frame as f32 * scenario.fixed_dt,
            position: state.position,
            velocity: state.velocity,
            speed_mps: state.velocity.length(),
            altitude_m: state.position.y,
            mode: state.controller.mode.label(),
            desired_body_yaw_error_degrees,
            desired_body_heading_error_degrees: desired_body_yaw_error_degrees.abs(),
            body_roll_degrees: body_roll_degrees(player_rotation),
            desired_heading_alignment_mps,
            lateral_response_mps,
            lateral_input_active,
            movement_input_lateral_axis: movement_axis.x,
            movement_input_forward_axis: movement_axis.y,
            camera_distance_m: camera.distance_m,
            camera_surface_clearance_m: camera.surface_clearance_m,
            camera_player_angle_degrees: camera.player_angle_degrees,
            camera_pitch_degrees: camera.pitch_degrees,
            camera_yaw_offset_degrees: orbit.yaw_degrees(),
            camera_pitch_offset_degrees: orbit.pitch_degrees(),
            camera_step_distance_m: camera.step_distance_m,
            camera_rotation_delta_degrees: camera.rotation_delta_degrees,
            camera_orbit_alignment_degrees: camera.orbit_alignment_degrees,
            camera_follow_direction_error_degrees: camera.follow_direction_error_degrees,
            camera_view_yaw_degrees: camera.view_yaw_degrees,
            camera_world_yaw_degrees: camera.world_yaw_degrees,
            camera_obstruction_adjustment_m: camera.obstruction_adjustment_m,
            camera_obstruction_hits: camera.obstruction_hits,
            visible_wind_fields: visible_fields_at(state.position, visual_fields.iter().copied()),
            wind_field_count: visual_fields.len(),
            active_lift_fields: active_lift_fields_at(state.position, lift_fields.iter().copied()),
            readable_lift_fields: readable_lift_fields_at(
                state.position,
                lift_fields.iter().copied(),
                visual_fields.iter().copied(),
            ),
            lift_field_count: lift_fields.len(),
            target_distance_m: route
                .target_distance_to(state.position, scenario.target_island_name),
            on_landing_target: route.on_landing_target_named(
                state.position,
                state.controller.mode,
                scenario.target_island_name,
            ),
            objective: objective.clone(),
            sky_island_count: route.islands().len(),
            active_chunk_count: streaming_lod.active_chunk_count,
            active_island_count: streaming_lod.active_island_count,
            near_lod_islands: streaming_lod.near_lod_islands,
            mid_lod_islands: streaming_lod.mid_lod_islands,
            far_lod_islands: streaming_lod.far_lod_islands,
            power_up_count: AERIAL_POWER_UP_ROUTE.len(),
            visible_power_up_count: power_ups.visible_count(),
            collected_power_up_count: power_ups.collected_count(),
            active_power_up_effects: power_ups.active_effects(),
            total_power_up_activations: power_ups.total_activations,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "frame": self.frame,
            "time_secs": round4(self.time_secs),
            "position": vec3_json(self.position),
            "velocity": vec3_json(self.velocity),
            "speed_mps": round4(self.speed_mps),
            "altitude_m": round4(self.altitude_m),
            "mode": self.mode,
            "desired_body_yaw_error_degrees": finite_json(self.desired_body_yaw_error_degrees),
            "desired_body_heading_error_degrees": finite_json(self.desired_body_heading_error_degrees),
            "body_roll_degrees": round4(self.body_roll_degrees),
            "desired_heading_alignment_mps": finite_json(self.desired_heading_alignment_mps),
            "lateral_response_mps": round4(self.lateral_response_mps),
            "lateral_input_active": self.lateral_input_active,
            "movement_input_lateral_axis": round4(self.movement_input_lateral_axis),
            "movement_input_forward_axis": round4(self.movement_input_forward_axis),
            "camera_distance_m": round4(self.camera_distance_m),
            "camera_surface_clearance_m": round4(self.camera_surface_clearance_m),
            "camera_player_angle_degrees": round4(self.camera_player_angle_degrees),
            "camera_pitch_degrees": round4(self.camera_pitch_degrees),
            "camera_yaw_offset_degrees": round4(self.camera_yaw_offset_degrees),
            "camera_pitch_offset_degrees": round4(self.camera_pitch_offset_degrees),
            "camera_step_distance_m": round4(self.camera_step_distance_m),
            "camera_rotation_delta_degrees": round4(self.camera_rotation_delta_degrees),
            "camera_orbit_alignment_degrees": round4(self.camera_orbit_alignment_degrees),
            "camera_follow_direction_error_degrees": round4(self.camera_follow_direction_error_degrees),
            "camera_view_yaw_degrees": round4(self.camera_view_yaw_degrees),
            "camera_world_yaw_degrees": round4(self.camera_world_yaw_degrees),
            "camera_obstruction_adjustment_m": round4(self.camera_obstruction_adjustment_m),
            "camera_obstruction_hits": self.camera_obstruction_hits,
            "visible_wind_fields": self.visible_wind_fields,
            "wind_field_count": self.wind_field_count,
            "active_lift_fields": self.active_lift_fields,
            "readable_lift_fields": self.readable_lift_fields,
            "lift_field_count": self.lift_field_count,
            "target_distance_m": round4(self.target_distance_m),
            "on_landing_target": self.on_landing_target,
            "objective": {
                "completed_count": self.objective.completed_count,
                "total_count": self.objective.total_count,
                "current_step": self.objective.current_step(),
                "current_label": self.objective.current_label,
                "current_distance_m": round4(self.objective.current_distance_m),
                "complete": self.objective.complete,
            },
            "sky_island_count": self.sky_island_count,
            "active_chunk_count": self.active_chunk_count,
            "active_island_count": self.active_island_count,
            "near_lod_islands": self.near_lod_islands,
            "mid_lod_islands": self.mid_lod_islands,
            "far_lod_islands": self.far_lod_islands,
            "power_up_count": self.power_up_count,
            "visible_power_up_count": self.visible_power_up_count,
            "collected_power_up_count": self.collected_power_up_count,
            "active_power_up_effects": self.active_power_up_effects,
            "total_power_up_activations": self.total_power_up_activations,
            "visual_foot_gap_m": GROUND_VISUAL_FOOT_GAP_M,
        })
    }
}

fn vec3_json(value: Vec3) -> Value {
    json!([round4(value.x), round4(value.y), round4(value.z)])
}

fn finite_json(value: f32) -> Value {
    if value.is_finite() {
        json!(round4(value))
    } else {
        Value::Null
    }
}

fn round4(value: f32) -> f32 {
    if value.is_finite() {
        (value * 10_000.0).round() / 10_000.0
    } else {
        value
    }
}

fn round4_f64(value: f64) -> f64 {
    if value.is_finite() {
        (value * 10_000.0).round() / 10_000.0
    } else {
        value
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "traversal_sim_eval/tests.rs"]
mod tests;
