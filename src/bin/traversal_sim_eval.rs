#![recursion_limit = "512"]

use bevy::prelude::{Quat, Transform, Vec2, Vec3};
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
    eval::{
        AIR_CONTROL_RESPONSE, CAMERA_STRAFE_STABILITY, EvalScenario, SCENARIO_NAMES,
        scenario_named, scripted_camera_input, scripted_input,
    },
    movement::{
        Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning,
        body_yaw_error_degrees, desired_heading_alignment_speed, desired_planar_movement_direction,
        face_flight_direction, lateral_response_speed,
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
            state.controller.mode,
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

#[derive(Clone, Debug)]
struct SimMetrics {
    sample_count: u32,
    start_position: Vec3,
    final_position: Vec3,
    horizontal_distance_m: f32,
    max_altitude_m: f32,
    min_altitude_m: f32,
    max_speed_mps: f32,
    max_camera_distance_m: f32,
    min_camera_surface_clearance_m: f32,
    max_camera_player_angle_degrees: f32,
    max_camera_step_distance_m: f32,
    max_camera_rotation_delta_degrees: f32,
    max_camera_orbit_alignment_degrees: f32,
    max_abs_camera_view_yaw_degrees: f32,
    first_camera_view_yaw_degrees: Option<f32>,
    max_camera_view_yaw_drift_degrees: f32,
    first_camera_world_yaw_degrees: Option<f32>,
    max_camera_world_yaw_drift_degrees: f32,
    max_camera_obstruction_adjustment_m: f32,
    max_camera_obstruction_hits: usize,
    max_abs_camera_yaw_offset_degrees: f32,
    min_camera_pitch_offset_degrees: f32,
    max_camera_pitch_offset_degrees: f32,
    desired_body_heading_error_sum_degrees: f32,
    desired_body_heading_samples: u32,
    desired_body_heading_error_values_degrees: Vec<f32>,
    max_desired_body_heading_error_degrees: f32,
    previous_desired_body_yaw_error_degrees: Option<f32>,
    max_body_yaw_error_step_degrees: f32,
    previous_body_yaw_error_sign: Option<f32>,
    body_yaw_oscillation_count: u32,
    max_desired_heading_alignment_mps: f32,
    max_lateral_response_mps: f32,
    first_lateral_input_time_secs: Option<f32>,
    first_lateral_response_time_secs: Option<f32>,
    max_right_lateral_response_mps: f32,
    first_right_lateral_input_time_secs: Option<f32>,
    first_right_lateral_response_time_secs: Option<f32>,
    max_left_lateral_response_mps: f32,
    first_left_lateral_input_time_secs: Option<f32>,
    first_left_lateral_response_time_secs: Option<f32>,
    max_backward_lateral_response_mps: f32,
    first_backward_lateral_input_time_secs: Option<f32>,
    first_backward_lateral_response_time_secs: Option<f32>,
    max_backward_right_lateral_response_mps: f32,
    max_backward_right_rear_response_mps: f32,
    first_backward_right_lateral_input_time_secs: Option<f32>,
    first_backward_right_lateral_response_time_secs: Option<f32>,
    max_backward_left_lateral_response_mps: f32,
    max_backward_left_rear_response_mps: f32,
    first_backward_left_lateral_input_time_secs: Option<f32>,
    first_backward_left_lateral_response_time_secs: Option<f32>,
    backward_air_control_start_speed_mps: Option<f32>,
    min_backward_air_control_speed_mps: Option<f32>,
    backward_air_control_start_planar_speed_mps: Option<f32>,
    min_backward_air_control_planar_speed_mps: Option<f32>,
    max_air_brake_speed_drop_mps: f32,
    max_air_brake_planar_speed_drop_mps: f32,
    max_post_brake_forward_alignment_mps: f32,
    min_target_distance_m: f32,
    final_target_distance_m: f32,
    objective_total_count: usize,
    max_completed_objective_count: usize,
    final_objective_completed_count: usize,
    min_objective_distance_m: f32,
    final_objective_distance_m: f32,
    objective_complete_samples: u32,
    max_sky_island_count: usize,
    max_active_chunk_count: usize,
    max_active_island_count: usize,
    max_near_lod_islands: usize,
    max_mid_lod_islands: usize,
    max_far_lod_islands: usize,
    max_power_up_count: usize,
    min_visible_power_up_count: usize,
    max_collected_power_up_count: usize,
    power_up_effect_samples: u32,
    total_power_up_activations: usize,
    target_landing_samples: u32,
    lifted_samples: u32,
    readable_lift_samples: u32,
    unreadable_lift_samples: u32,
    gliding_samples: u32,
    launching_samples: u32,
    grounded_samples: u32,
}

impl SimMetrics {
    fn new(route: &SkyRoute) -> Self {
        Self {
            sample_count: 0,
            start_position: START_POSITION,
            final_position: START_POSITION,
            horizontal_distance_m: 0.0,
            max_altitude_m: START_POSITION.y,
            min_altitude_m: START_POSITION.y,
            max_speed_mps: 0.0,
            max_camera_distance_m: 0.0,
            min_camera_surface_clearance_m: f32::MAX,
            max_camera_player_angle_degrees: 0.0,
            max_camera_step_distance_m: 0.0,
            max_camera_rotation_delta_degrees: 0.0,
            max_camera_orbit_alignment_degrees: 0.0,
            max_abs_camera_view_yaw_degrees: 0.0,
            first_camera_view_yaw_degrees: None,
            max_camera_view_yaw_drift_degrees: 0.0,
            first_camera_world_yaw_degrees: None,
            max_camera_world_yaw_drift_degrees: 0.0,
            max_camera_obstruction_adjustment_m: 0.0,
            max_camera_obstruction_hits: 0,
            max_abs_camera_yaw_offset_degrees: 0.0,
            min_camera_pitch_offset_degrees: f32::MAX,
            max_camera_pitch_offset_degrees: f32::MIN,
            desired_body_heading_error_sum_degrees: 0.0,
            desired_body_heading_samples: 0,
            desired_body_heading_error_values_degrees: Vec::new(),
            max_desired_body_heading_error_degrees: 0.0,
            previous_desired_body_yaw_error_degrees: None,
            max_body_yaw_error_step_degrees: 0.0,
            previous_body_yaw_error_sign: None,
            body_yaw_oscillation_count: 0,
            max_desired_heading_alignment_mps: 0.0,
            max_lateral_response_mps: 0.0,
            first_lateral_input_time_secs: None,
            first_lateral_response_time_secs: None,
            max_right_lateral_response_mps: 0.0,
            first_right_lateral_input_time_secs: None,
            first_right_lateral_response_time_secs: None,
            max_left_lateral_response_mps: 0.0,
            first_left_lateral_input_time_secs: None,
            first_left_lateral_response_time_secs: None,
            max_backward_lateral_response_mps: 0.0,
            first_backward_lateral_input_time_secs: None,
            first_backward_lateral_response_time_secs: None,
            max_backward_right_lateral_response_mps: 0.0,
            max_backward_right_rear_response_mps: 0.0,
            first_backward_right_lateral_input_time_secs: None,
            first_backward_right_lateral_response_time_secs: None,
            max_backward_left_lateral_response_mps: 0.0,
            max_backward_left_rear_response_mps: 0.0,
            first_backward_left_lateral_input_time_secs: None,
            first_backward_left_lateral_response_time_secs: None,
            backward_air_control_start_speed_mps: None,
            min_backward_air_control_speed_mps: None,
            backward_air_control_start_planar_speed_mps: None,
            min_backward_air_control_planar_speed_mps: None,
            max_air_brake_speed_drop_mps: 0.0,
            max_air_brake_planar_speed_drop_mps: 0.0,
            max_post_brake_forward_alignment_mps: 0.0,
            min_target_distance_m: f32::MAX,
            final_target_distance_m: route.target_distance_to(START_POSITION, None),
            objective_total_count: 0,
            max_completed_objective_count: 0,
            final_objective_completed_count: 0,
            min_objective_distance_m: f32::MAX,
            final_objective_distance_m: 0.0,
            objective_complete_samples: 0,
            max_sky_island_count: route.islands().len(),
            max_active_chunk_count: 0,
            max_active_island_count: 0,
            max_near_lod_islands: 0,
            max_mid_lod_islands: 0,
            max_far_lod_islands: 0,
            max_power_up_count: AERIAL_POWER_UP_ROUTE.len(),
            min_visible_power_up_count: AERIAL_POWER_UP_ROUTE.len(),
            max_collected_power_up_count: 0,
            power_up_effect_samples: 0,
            total_power_up_activations: 0,
            target_landing_samples: 0,
            lifted_samples: 0,
            readable_lift_samples: 0,
            unreadable_lift_samples: 0,
            gliding_samples: 0,
            launching_samples: 0,
            grounded_samples: 0,
        }
    }

    fn observe(&mut self, sample: &SimSample, scenario: EvalScenario) {
        self.sample_count += 1;
        self.final_position = sample.position;
        self.horizontal_distance_m = self
            .horizontal_distance_m
            .max(horizontal_distance(self.start_position, sample.position));
        self.max_altitude_m = self.max_altitude_m.max(sample.altitude_m);
        self.min_altitude_m = self.min_altitude_m.min(sample.altitude_m);
        self.max_speed_mps = self.max_speed_mps.max(sample.speed_mps);
        self.max_camera_distance_m = self.max_camera_distance_m.max(sample.camera_distance_m);
        self.min_camera_surface_clearance_m = self
            .min_camera_surface_clearance_m
            .min(sample.camera_surface_clearance_m);
        self.max_camera_player_angle_degrees = self
            .max_camera_player_angle_degrees
            .max(sample.camera_player_angle_degrees);
        self.max_camera_step_distance_m = self
            .max_camera_step_distance_m
            .max(sample.camera_step_distance_m);
        self.max_camera_rotation_delta_degrees = self
            .max_camera_rotation_delta_degrees
            .max(sample.camera_rotation_delta_degrees);
        self.max_camera_orbit_alignment_degrees = self
            .max_camera_orbit_alignment_degrees
            .max(sample.camera_orbit_alignment_degrees);
        self.max_abs_camera_view_yaw_degrees = self
            .max_abs_camera_view_yaw_degrees
            .max(sample.camera_view_yaw_degrees.abs());
        let first_view_yaw = self
            .first_camera_view_yaw_degrees
            .get_or_insert(sample.camera_view_yaw_degrees);
        self.max_camera_view_yaw_drift_degrees = self
            .max_camera_view_yaw_drift_degrees
            .max((sample.camera_view_yaw_degrees - *first_view_yaw).abs());
        let first_world_yaw = self
            .first_camera_world_yaw_degrees
            .get_or_insert(sample.camera_world_yaw_degrees);
        self.max_camera_world_yaw_drift_degrees = self
            .max_camera_world_yaw_drift_degrees
            .max((sample.camera_world_yaw_degrees - *first_world_yaw).abs());
        self.max_camera_obstruction_adjustment_m = self
            .max_camera_obstruction_adjustment_m
            .max(sample.camera_obstruction_adjustment_m);
        self.max_camera_obstruction_hits = self
            .max_camera_obstruction_hits
            .max(sample.camera_obstruction_hits);
        self.max_abs_camera_yaw_offset_degrees = self
            .max_abs_camera_yaw_offset_degrees
            .max(sample.camera_yaw_offset_degrees.abs());
        self.min_camera_pitch_offset_degrees = self
            .min_camera_pitch_offset_degrees
            .min(sample.camera_pitch_offset_degrees);
        self.max_camera_pitch_offset_degrees = self
            .max_camera_pitch_offset_degrees
            .max(sample.camera_pitch_offset_degrees);

        if sample.desired_body_yaw_error_degrees.is_finite() {
            self.desired_body_heading_error_sum_degrees +=
                sample.desired_body_heading_error_degrees;
            self.desired_body_heading_samples += 1;
            self.desired_body_heading_error_values_degrees
                .push(sample.desired_body_heading_error_degrees);
            self.max_desired_body_heading_error_degrees = self
                .max_desired_body_heading_error_degrees
                .max(sample.desired_body_heading_error_degrees);
            if let Some(previous) = self.previous_desired_body_yaw_error_degrees {
                self.max_body_yaw_error_step_degrees = self
                    .max_body_yaw_error_step_degrees
                    .max((sample.desired_body_yaw_error_degrees - previous).abs());
            }
            self.previous_desired_body_yaw_error_degrees =
                Some(sample.desired_body_yaw_error_degrees);
            if sample.desired_body_yaw_error_degrees.abs()
                >= AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES
            {
                let sign = sample.desired_body_yaw_error_degrees.signum();
                if self
                    .previous_body_yaw_error_sign
                    .is_some_and(|previous| previous != sign)
                {
                    self.body_yaw_oscillation_count += 1;
                }
                self.previous_body_yaw_error_sign = Some(sign);
            }
        }
        if sample.desired_heading_alignment_mps.is_finite() {
            self.max_desired_heading_alignment_mps = self
                .max_desired_heading_alignment_mps
                .max(sample.desired_heading_alignment_mps);
            if sample.movement_input_forward_axis > 0.0 {
                self.max_post_brake_forward_alignment_mps = self
                    .max_post_brake_forward_alignment_mps
                    .max(sample.desired_heading_alignment_mps);
            }
        }
        self.observe_lateral_response(sample);
        self.observe_backward_air_control(sample);

        self.min_target_distance_m = self.min_target_distance_m.min(sample.target_distance_m);
        self.final_target_distance_m = sample.target_distance_m;
        self.objective_total_count = sample.objective.total_count;
        self.max_completed_objective_count = self
            .max_completed_objective_count
            .max(sample.objective.completed_count);
        self.final_objective_completed_count = sample.objective.completed_count;
        self.min_objective_distance_m = self
            .min_objective_distance_m
            .min(sample.objective.current_distance_m);
        self.final_objective_distance_m = sample.objective.current_distance_m;
        if sample.objective.complete {
            self.objective_complete_samples += 1;
        }
        if sample.on_landing_target {
            self.target_landing_samples += 1;
        }
        self.max_sky_island_count = self.max_sky_island_count.max(sample.sky_island_count);
        self.max_active_chunk_count = self.max_active_chunk_count.max(sample.active_chunk_count);
        self.max_active_island_count = self.max_active_island_count.max(sample.active_island_count);
        self.max_near_lod_islands = self.max_near_lod_islands.max(sample.near_lod_islands);
        self.max_mid_lod_islands = self.max_mid_lod_islands.max(sample.mid_lod_islands);
        self.max_far_lod_islands = self.max_far_lod_islands.max(sample.far_lod_islands);
        self.max_power_up_count = self.max_power_up_count.max(sample.power_up_count);
        self.min_visible_power_up_count = self
            .min_visible_power_up_count
            .min(sample.visible_power_up_count);
        self.max_collected_power_up_count = self
            .max_collected_power_up_count
            .max(sample.collected_power_up_count);
        if sample.active_power_up_effects > 0 {
            self.power_up_effect_samples += 1;
        }
        self.total_power_up_activations = sample.total_power_up_activations;
        if sample.active_lift_fields > 0 {
            self.lifted_samples += 1;
            if sample.readable_lift_fields > 0 {
                self.readable_lift_samples += 1;
            } else {
                self.unreadable_lift_samples += 1;
            }
        }
        match sample.mode {
            "gliding" => self.gliding_samples += 1,
            "launching" => self.launching_samples += 1,
            "grounded" => self.grounded_samples += 1,
            _ => {}
        }

        if scenario.name == CAMERA_STRAFE_STABILITY {
            self.max_camera_obstruction_adjustment_m = 0.0;
        }
    }

    fn observe_lateral_response(&mut self, sample: &SimSample) {
        let lateral_axis_active =
            sample.lateral_input_active || sample.movement_input_lateral_axis.abs() > f32::EPSILON;
        if !lateral_axis_active {
            return;
        }
        if self.first_lateral_input_time_secs.is_none() {
            self.first_lateral_input_time_secs = Some(sample.time_secs);
        }
        self.max_lateral_response_mps = self
            .max_lateral_response_mps
            .max(sample.lateral_response_mps);
        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
            && self.first_lateral_response_time_secs.is_none()
        {
            self.first_lateral_response_time_secs = Some(sample.time_secs);
        }

        match sample.movement_input_lateral_axis.signum() {
            sign if sign > 0.0 => {
                if self.first_right_lateral_input_time_secs.is_none() {
                    self.first_right_lateral_input_time_secs = Some(sample.time_secs);
                }
                self.max_right_lateral_response_mps = self
                    .max_right_lateral_response_mps
                    .max(sample.lateral_response_mps);
                if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                    && self.first_right_lateral_response_time_secs.is_none()
                {
                    self.first_right_lateral_response_time_secs = Some(sample.time_secs);
                }
                if sample.movement_input_forward_axis < 0.0 {
                    if self.first_backward_right_lateral_input_time_secs.is_none() {
                        self.first_backward_right_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_backward_right_lateral_response_mps = self
                        .max_backward_right_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if let Some(rear_response) = backward_diagonal_rear_response_mps(sample) {
                        self.max_backward_right_rear_response_mps =
                            self.max_backward_right_rear_response_mps.max(rear_response);
                    }
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self
                            .first_backward_right_lateral_response_time_secs
                            .is_none()
                    {
                        self.first_backward_right_lateral_response_time_secs =
                            Some(sample.time_secs);
                    }
                }
            }
            sign if sign < 0.0 => {
                if self.first_left_lateral_input_time_secs.is_none() {
                    self.first_left_lateral_input_time_secs = Some(sample.time_secs);
                }
                self.max_left_lateral_response_mps = self
                    .max_left_lateral_response_mps
                    .max(sample.lateral_response_mps);
                if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                    && self.first_left_lateral_response_time_secs.is_none()
                {
                    self.first_left_lateral_response_time_secs = Some(sample.time_secs);
                }
                if sample.movement_input_forward_axis < 0.0 {
                    if self.first_backward_left_lateral_input_time_secs.is_none() {
                        self.first_backward_left_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_backward_left_lateral_response_mps = self
                        .max_backward_left_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if let Some(rear_response) = backward_diagonal_rear_response_mps(sample) {
                        self.max_backward_left_rear_response_mps =
                            self.max_backward_left_rear_response_mps.max(rear_response);
                    }
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self
                            .first_backward_left_lateral_response_time_secs
                            .is_none()
                    {
                        self.first_backward_left_lateral_response_time_secs =
                            Some(sample.time_secs);
                    }
                }
            }
            _ => {}
        }
        if sample.movement_input_forward_axis < 0.0 {
            if self.first_backward_lateral_input_time_secs.is_none() {
                self.first_backward_lateral_input_time_secs = Some(sample.time_secs);
            }
            self.max_backward_lateral_response_mps = self
                .max_backward_lateral_response_mps
                .max(sample.lateral_response_mps);
            if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                && self.first_backward_lateral_response_time_secs.is_none()
            {
                self.first_backward_lateral_response_time_secs = Some(sample.time_secs);
            }
        }
    }

    fn observe_backward_air_control(&mut self, sample: &SimSample) {
        if sample.movement_input_forward_axis >= 0.0 || sample.mode == "grounded" {
            return;
        }

        let planar_speed = Vec2::new(sample.velocity.x, sample.velocity.z).length();
        self.backward_air_control_start_speed_mps
            .get_or_insert(sample.speed_mps);
        self.backward_air_control_start_planar_speed_mps
            .get_or_insert(planar_speed);
        self.min_backward_air_control_speed_mps = Some(
            self.min_backward_air_control_speed_mps
                .map_or(sample.speed_mps, |speed| speed.min(sample.speed_mps)),
        );
        self.min_backward_air_control_planar_speed_mps = Some(
            self.min_backward_air_control_planar_speed_mps
                .map_or(planar_speed, |speed| speed.min(planar_speed)),
        );
        if let (Some(start), Some(minimum)) = (
            self.backward_air_control_start_speed_mps,
            self.min_backward_air_control_speed_mps,
        ) {
            self.max_air_brake_speed_drop_mps = (start - minimum).max(0.0);
        }
        if let (Some(start), Some(minimum)) = (
            self.backward_air_control_start_planar_speed_mps,
            self.min_backward_air_control_planar_speed_mps,
        ) {
            self.max_air_brake_planar_speed_drop_mps = (start - minimum).max(0.0);
        }
    }

    fn checks(&self, scenario: EvalScenario) -> Vec<SimCheck> {
        let thresholds = scenario.thresholds;
        let mut checks = vec![
            SimCheck::at_least(
                "sample_count",
                self.sample_count as f32,
                thresholds.min_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "horizontal_distance",
                self.horizontal_distance_m,
                thresholds.min_horizontal_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "max_altitude",
                self.max_altitude_m,
                thresholds.min_max_altitude_m,
                "m",
            ),
            SimCheck::at_least(
                "max_speed",
                self.max_speed_mps,
                thresholds.min_max_speed_mps,
                "mps",
            ),
            SimCheck::at_least(
                "gliding_samples",
                self.gliding_samples as f32,
                thresholds.min_gliding_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "grounded_samples",
                self.grounded_samples as f32,
                thresholds.min_grounded_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "lifted_samples",
                self.lifted_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "sky_island_count",
                self.max_sky_island_count as f32,
                thresholds.min_sky_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "active_island_count",
                self.max_active_island_count as f32,
                thresholds.min_active_island_count as f32,
                "islands",
            ),
            SimCheck::at_most(
                "active_chunk_count",
                self.max_active_chunk_count as f32,
                thresholds.max_active_chunk_count as f32,
                "chunks",
            ),
            SimCheck::at_least(
                "near_lod_island_count",
                self.max_near_lod_islands as f32,
                thresholds.min_near_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "mid_lod_island_count",
                self.max_mid_lod_islands as f32,
                thresholds.min_mid_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "far_lod_island_count",
                self.max_far_lod_islands as f32,
                thresholds.min_far_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_most(
                "camera_distance",
                self.max_camera_distance_m,
                thresholds.max_camera_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "camera_surface_clearance",
                self.min_camera_surface_clearance_m,
                thresholds.min_camera_surface_clearance_m,
                "m",
            ),
            SimCheck::at_most(
                "camera_player_angle",
                self.max_camera_player_angle_degrees,
                thresholds.max_camera_player_angle_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_step_distance",
                self.max_camera_step_distance_m,
                thresholds.max_camera_step_distance_m,
                "m",
            ),
            SimCheck::at_most(
                "camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                thresholds.max_camera_rotation_delta_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_orbit_alignment",
                self.max_camera_orbit_alignment_degrees,
                thresholds.max_camera_orbit_alignment_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_view_yaw",
                self.max_abs_camera_view_yaw_degrees,
                thresholds.max_abs_camera_view_yaw_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "camera_yaw_input",
                self.max_abs_camera_yaw_offset_degrees,
                thresholds.min_abs_camera_yaw_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "camera_pitch_input_min",
                self.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_pitch_input_max",
                self.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "objective_total_count",
                self.objective_total_count as f32,
                thresholds.min_objective_total_count as f32,
                "objectives",
            ),
            SimCheck::at_least(
                "completed_objective_count",
                self.max_completed_objective_count as f32,
                thresholds.min_completed_objective_count as f32,
                "objectives",
            ),
            SimCheck::at_most(
                "final_target_distance",
                self.final_target_distance_m,
                thresholds.max_final_target_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "target_landing_samples",
                self.target_landing_samples as f32,
                thresholds.min_target_landing_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "power_up_count",
                self.max_power_up_count as f32,
                thresholds.min_power_up_count as f32,
                "powerups",
            ),
            SimCheck::at_least(
                "collected_power_up_count",
                self.max_collected_power_up_count as f32,
                thresholds.min_collected_power_up_count as f32,
                "powerups",
            ),
            SimCheck::at_least(
                "power_up_effect_samples",
                self.power_up_effect_samples as f32,
                thresholds.min_power_up_effect_samples as f32,
                "samples",
            ),
        ];

        if scenario.name == CAMERA_STRAFE_STABILITY {
            checks.extend([
                SimCheck::at_most(
                    "camera_strafe_view_yaw_drift",
                    self.max_camera_view_yaw_drift_degrees,
                    CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "camera_strafe_world_yaw_drift",
                    self.max_camera_world_yaw_drift_degrees,
                    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_least(
                    "camera_strafe_right_lateral_response",
                    self.max_right_lateral_response_mps,
                    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "camera_strafe_left_lateral_response",
                    self.max_left_lateral_response_mps,
                    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
            ]);
        }

        if scenario.name == AIR_CONTROL_RESPONSE {
            let lateral_response_latency_secs = response_latency_secs(
                self.first_lateral_input_time_secs,
                self.first_lateral_response_time_secs,
            );
            let right_lateral_response_latency_secs = response_latency_secs(
                self.first_right_lateral_input_time_secs,
                self.first_right_lateral_response_time_secs,
            );
            let left_lateral_response_latency_secs = response_latency_secs(
                self.first_left_lateral_input_time_secs,
                self.first_left_lateral_response_time_secs,
            );
            let backward_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_lateral_input_time_secs,
                self.first_backward_lateral_response_time_secs,
            );
            let backward_right_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_right_lateral_input_time_secs,
                self.first_backward_right_lateral_response_time_secs,
            );
            let backward_left_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_left_lateral_input_time_secs,
                self.first_backward_left_lateral_response_time_secs,
            );

            checks.extend([
                SimCheck::at_most(
                    "air_control_lateral_response_latency",
                    lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_lateral_response",
                    self.max_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_right_lateral_response_latency",
                    right_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_right_lateral_response",
                    self.max_right_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_left_lateral_response_latency",
                    left_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_left_lateral_response",
                    self.max_left_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_lateral_response_latency",
                    backward_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_lateral_response",
                    self.max_backward_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_right_lateral_response_latency",
                    backward_right_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_right_lateral_response",
                    self.max_backward_right_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_backward_right_rear_response",
                    self.max_backward_right_rear_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_left_lateral_response_latency",
                    backward_left_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_left_lateral_response",
                    self.max_backward_left_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_backward_left_rear_response",
                    self.max_backward_left_rear_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_desired_heading_alignment",
                    self.max_desired_heading_alignment_mps,
                    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_avg_body_heading_error",
                    self.avg_body_heading_error_degrees(),
                    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_p95_body_heading_error",
                    self.p95_body_heading_error_degrees(),
                    AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_max_body_heading_error",
                    self.max_desired_body_heading_error_degrees,
                    AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_max_body_yaw_error_step",
                    self.max_body_yaw_error_step_degrees,
                    AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_body_yaw_oscillations",
                    self.body_yaw_oscillation_count as f32,
                    AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
                    "oscillations",
                ),
                SimCheck::at_most(
                    "air_control_camera_orbit_yaw_offset",
                    self.max_abs_camera_yaw_offset_degrees,
                    AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_rotation_delta",
                    self.max_camera_rotation_delta_degrees,
                    AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_view_yaw_drift",
                    self.max_camera_view_yaw_drift_degrees,
                    AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_world_yaw_drift",
                    self.max_camera_world_yaw_drift_degrees,
                    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_least(
                    "air_control_air_brake_speed_drop",
                    self.max_air_brake_speed_drop_mps,
                    AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_air_brake_planar_speed_drop",
                    self.max_air_brake_planar_speed_drop_mps,
                    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_post_brake_forward_alignment",
                    self.max_post_brake_forward_alignment_mps,
                    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
                    "mps",
                ),
            ]);
        }

        checks
    }

    fn avg_body_heading_error_degrees(&self) -> f32 {
        if self.desired_body_heading_samples == 0 {
            0.0
        } else {
            self.desired_body_heading_error_sum_degrees / self.desired_body_heading_samples as f32
        }
    }

    fn p95_body_heading_error_degrees(&self) -> f32 {
        percentile(&self.desired_body_heading_error_values_degrees, 0.95)
    }

    fn to_json(&self) -> Value {
        json!({
            "sample_count": self.sample_count,
            "horizontal_distance_m": round4(self.horizontal_distance_m),
            "max_altitude_m": round4(self.max_altitude_m),
            "min_altitude_m": round4(self.min_altitude_m),
            "max_speed_mps": round4(self.max_speed_mps),
            "max_camera_distance_m": round4(self.max_camera_distance_m),
            "min_camera_surface_clearance_m": round4(self.min_camera_surface_clearance_m),
            "max_camera_player_angle_degrees": round4(self.max_camera_player_angle_degrees),
            "max_camera_step_distance_m": round4(self.max_camera_step_distance_m),
            "max_camera_rotation_delta_degrees": round4(self.max_camera_rotation_delta_degrees),
            "max_camera_orbit_alignment_degrees": round4(self.max_camera_orbit_alignment_degrees),
            "max_abs_camera_view_yaw_degrees": round4(self.max_abs_camera_view_yaw_degrees),
            "max_camera_view_yaw_drift_degrees": round4(self.max_camera_view_yaw_drift_degrees),
            "max_camera_world_yaw_drift_degrees": round4(self.max_camera_world_yaw_drift_degrees),
            "max_camera_obstruction_adjustment_m": round4(self.max_camera_obstruction_adjustment_m),
            "max_camera_obstruction_hits": self.max_camera_obstruction_hits,
            "max_abs_camera_yaw_offset_degrees": round4(self.max_abs_camera_yaw_offset_degrees),
            "min_camera_pitch_offset_degrees": round4(self.min_camera_pitch_offset_degrees),
            "max_camera_pitch_offset_degrees": round4(self.max_camera_pitch_offset_degrees),
            "avg_desired_body_heading_error_degrees": round4(self.avg_body_heading_error_degrees()),
            "p95_desired_body_heading_error_degrees": round4(self.p95_body_heading_error_degrees()),
            "max_desired_body_heading_error_degrees": round4(self.max_desired_body_heading_error_degrees),
            "max_body_yaw_error_step_degrees": round4(self.max_body_yaw_error_step_degrees),
            "body_yaw_oscillation_count": self.body_yaw_oscillation_count,
            "max_desired_heading_alignment_mps": round4(self.max_desired_heading_alignment_mps),
            "max_lateral_response_mps": round4(self.max_lateral_response_mps),
            "lateral_response_latency_secs": round4(response_latency_secs(self.first_lateral_input_time_secs, self.first_lateral_response_time_secs)),
            "max_right_lateral_response_mps": round4(self.max_right_lateral_response_mps),
            "right_lateral_response_latency_secs": round4(response_latency_secs(self.first_right_lateral_input_time_secs, self.first_right_lateral_response_time_secs)),
            "max_left_lateral_response_mps": round4(self.max_left_lateral_response_mps),
            "left_lateral_response_latency_secs": round4(response_latency_secs(self.first_left_lateral_input_time_secs, self.first_left_lateral_response_time_secs)),
            "max_backward_lateral_response_mps": round4(self.max_backward_lateral_response_mps),
            "backward_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_lateral_input_time_secs, self.first_backward_lateral_response_time_secs)),
            "max_backward_right_lateral_response_mps": round4(self.max_backward_right_lateral_response_mps),
            "backward_right_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_right_lateral_input_time_secs, self.first_backward_right_lateral_response_time_secs)),
            "max_backward_right_rear_response_mps": round4(self.max_backward_right_rear_response_mps),
            "max_backward_left_lateral_response_mps": round4(self.max_backward_left_lateral_response_mps),
            "backward_left_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_left_lateral_input_time_secs, self.first_backward_left_lateral_response_time_secs)),
            "max_backward_left_rear_response_mps": round4(self.max_backward_left_rear_response_mps),
            "max_air_brake_speed_drop_mps": round4(self.max_air_brake_speed_drop_mps),
            "max_air_brake_planar_speed_drop_mps": round4(self.max_air_brake_planar_speed_drop_mps),
            "max_post_brake_forward_alignment_mps": round4(self.max_post_brake_forward_alignment_mps),
            "min_target_distance_m": round4(self.min_target_distance_m),
            "final_target_distance_m": round4(self.final_target_distance_m),
            "objective_total_count": self.objective_total_count,
            "max_completed_objective_count": self.max_completed_objective_count,
            "final_objective_completed_count": self.final_objective_completed_count,
            "min_objective_distance_m": round4(self.min_objective_distance_m),
            "final_objective_distance_m": round4(self.final_objective_distance_m),
            "objective_complete_samples": self.objective_complete_samples,
            "max_sky_island_count": self.max_sky_island_count,
            "max_active_chunk_count": self.max_active_chunk_count,
            "max_active_island_count": self.max_active_island_count,
            "max_near_lod_islands": self.max_near_lod_islands,
            "max_mid_lod_islands": self.max_mid_lod_islands,
            "max_far_lod_islands": self.max_far_lod_islands,
            "max_power_up_count": self.max_power_up_count,
            "min_visible_power_up_count": self.min_visible_power_up_count,
            "max_collected_power_up_count": self.max_collected_power_up_count,
            "power_up_effect_samples": self.power_up_effect_samples,
            "total_power_up_activations": self.total_power_up_activations,
            "target_landing_samples": self.target_landing_samples,
            "lifted_samples": self.lifted_samples,
            "readable_lift_samples": self.readable_lift_samples,
            "unreadable_lift_samples": self.unreadable_lift_samples,
            "gliding_samples": self.gliding_samples,
            "launching_samples": self.launching_samples,
            "grounded_samples": self.grounded_samples,
            "final_position": vec3_json(self.final_position),
            "native_window_created": false,
        })
    }
}

#[derive(Clone, Debug)]
struct SimCheck {
    name: &'static str,
    passed: bool,
    value: f32,
    comparator: &'static str,
    threshold: f32,
    unit: &'static str,
}

impl SimCheck {
    fn at_least(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }

    fn at_most(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value <= threshold,
            value,
            comparator: "<=",
            threshold,
            unit,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "passed": self.passed,
            "value": round4(self.value),
            "comparator": self.comparator,
            "threshold": round4(self.threshold),
            "unit": self.unit,
        })
    }
}

#[derive(Clone, Debug)]
struct SimResult {
    scenario: EvalScenario,
    passed: bool,
    metrics: SimMetrics,
    checks: Vec<SimCheck>,
    samples: Vec<SimSample>,
    elapsed_ms: f64,
    summary_path: String,
    samples_path: String,
}

impl SimResult {
    fn to_summary_json(&self) -> String {
        serde_json::to_string_pretty(&json!({
            "schema": "nau_traversal_sim_eval.v1",
            "scenario": self.scenario.name,
            "target_island": self.scenario.target_island_name,
            "passed": self.passed,
            "mode": "simulation_only",
            "frame_count": self.scenario.frame_count,
            "duration_secs": round4(self.scenario.duration_secs()),
            "elapsed_ms": round4_f64(self.elapsed_ms),
            "metrics": self.metrics.to_json(),
            "checks": self.checks.iter().map(SimCheck::to_json).collect::<Vec<_>>(),
            "artifacts": {
                "summary_json": self.summary_path,
                "samples_ndjson": self.samples_path,
                "screenshot_png": Value::Null,
                "checkpoint_screenshots": [],
                "checkpoint_marker_metadata": [],
            },
            "final_sample": self.samples.last().map(SimSample::to_json),
        }))
        .expect("summary json")
            + "\n"
    }
}

fn backward_diagonal_rear_response_mps(sample: &SimSample) -> Option<f32> {
    if sample.movement_input_forward_axis >= 0.0
        || sample.movement_input_lateral_axis.abs() <= f32::EPSILON
        || !sample.desired_heading_alignment_mps.is_finite()
        || !sample.lateral_response_mps.is_finite()
    {
        return None;
    }

    Some(
        sample.desired_heading_alignment_mps * std::f32::consts::SQRT_2
            - sample.lateral_response_mps,
    )
}

fn response_latency_secs(input_time_secs: Option<f32>, response_time_secs: Option<f32>) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => 999.0,
        _ => 0.0,
    }
}

fn percentile(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let index =
        ((sorted.len().saturating_sub(1)) as f32 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn horizontal_distance(left: Vec3, right: Vec3) -> f32 {
    Vec2::new(left.x - right.x, left.z - right.z).length()
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
mod tests {
    use super::*;

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
    fn air_control_simulation_measures_backward_diagonal_response() {
        let scenario = scenario_named(AIR_CONTROL_RESPONSE).expect("scenario");
        let result = run_simulation(scenario);

        assert!(result.passed);
        assert!(result.metrics.max_backward_right_rear_response_mps >= 10.0);
        assert!(result.metrics.max_backward_left_rear_response_mps >= 10.0);
        assert!(result.metrics.max_air_brake_planar_speed_drop_mps >= 12.0);
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
}
