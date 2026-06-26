use super::{
    CAMERA_MIN_SURFACE_CLEARANCE, CAMERA_OBSTRUCTION_CLEARANCE,
    metrics::{SimMetrics, SimResult},
    sample::{CameraDiagnosticsSample, CameraStepSample, SimSample},
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    animation::advance_phase,
    camera::{
        CameraControlState, CameraControlTuning, CameraObstruction, FollowCamera,
        FollowCameraState, apply_camera_input, avoid_camera_obstructions,
        camera_orbit_alignment_degrees, lift_camera_above_floor,
        movement_facing_from_follow_direction, movement_input_stable_follow_direction,
        step_camera_with_direction, update_follow_direction_state,
    },
    environment::{
        AERIAL_POWER_UP_ROUTE, GAMEPLAY_LIFT_ROUTE, LiftField, WindField, WindForceApplication,
        apply_aerial_power_up, apply_lift_fields, apply_wind_fields, visual_wind_fields,
    },
    eval::{EvalScenario, scripted_camera_input, scripted_input},
    movement::{
        Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning,
        face_flight_direction,
    },
    world::{START_POSITION, SkyRoute},
};

pub(crate) fn run_simulation(scenario: EvalScenario) -> SimResult {
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
    let mut animation_phase = 0.0;
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
        let (movement_forward, movement_right) =
            movement_facing_from_follow_direction(follow_state.direction, camera_control.orbit);
        let facing = Facing::new(movement_forward, movement_right);
        let movement_facing = facing;
        let was_grounded =
            route.is_grounded_at(state.position) && state.controller.mode == FlightMode::Grounded;
        let mut frame_tuning = tuning;
        frame_tuning.floor_y = route.ground_at(state.position).floor_y;
        let world_step = step_flight_with_world(
            state,
            input,
            facing,
            &frame_tuning,
            &route,
            &lift_fields,
            &visual_fields,
            frame as f32 * scenario.fixed_dt,
            &mut power_ups,
            scenario.fixed_dt,
            was_grounded,
        );
        state = world_step.state;
        player_rotation = face_flight_direction(
            player_rotation,
            state.velocity,
            input,
            facing,
            state.controller,
            &frame_tuning,
            scenario.fixed_dt,
        );
        animation_phase =
            advance_phase(animation_phase, state.velocity.length(), scenario.fixed_dt);

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
                animation_phase,
                camera_control.orbit,
                camera_diagnostics,
                input,
                movement_facing,
                &route,
                &lift_fields,
                &visual_fields,
                world_step.wind,
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
    visual_fields: &[WindField],
    elapsed_secs: f32,
    power_ups: &mut SimPowerUps,
    dt: f32,
    was_grounded: bool,
) -> SimWorldStep {
    let mut next = nau_engine::movement::step_flight(state, input, facing, tuning, dt);
    let lift = apply_lift_fields(
        next.position,
        next.velocity,
        lift_fields.iter().copied(),
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = lift.velocity;
    let wind = apply_wind_fields(
        next.position,
        next.velocity,
        visual_fields.iter().copied(),
        elapsed_secs,
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = wind.velocity;
    collect_aerial_power_ups(&mut next, power_ups);
    let state = route.resolve_ground_contact_after_step(next, was_grounded);
    SimWorldStep {
        state,
        wind: wind.for_airborne_diagnostics(state.controller.mode != FlightMode::Grounded),
    }
}

#[derive(Clone, Copy, Debug)]
struct SimWorldStep {
    state: FlightState,
    wind: WindForceApplication,
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
