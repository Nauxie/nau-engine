use super::{
    CAMERA_MIN_SURFACE_CLEARANCE, CAMERA_OBSTRUCTION_CLEARANCE,
    metrics::{SimMetrics, SimResult},
    sample::{CameraDiagnosticsSample, CameraStepSample, SimSample},
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    animation::{
        MIN_KEY_POSE_READABILITY_SCORE, PlayerPoseContext, PlayerPoseIntent, advance_phase,
        body_local_pose_velocity, resolve_pose_input, resolve_pose_intent,
        wind_lateral_load_from_delta,
    },
    camera::{
        CAMERA_MAX_FOLLOW_FRAME_STEP_M, CAMERA_MAX_OBSTRUCTION_FRAME_STEP_M,
        CAMERA_MAX_OBSTRUCTION_HANDOFF_FRAME_STEP_M, CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES,
        CAMERA_MAX_PLAYER_DISTANCE_M, CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M,
        CAMERA_OBSTRUCTION_RELEASE_HANDOFF_FRAMES, CameraControlState, CameraControlTuning,
        CameraObstruction, CameraObstructionSmoothingState, FollowCamera, FollowCameraState,
        apply_camera_input, avoid_camera_obstructions_with_preferred_offset,
        camera_frame_step_budget, camera_obstruction_is_active, camera_orbit_alignment_degrees,
        camera_rotation_step_budget, clamp_camera_offset_step, clamp_camera_player_distance,
        clamp_camera_rotation_step, clamp_camera_step, lift_camera_above_floor,
        movement_facing_from_follow_direction, movement_input_stable_follow_direction,
        revalidate_camera_obstruction, smooth_camera_obstruction, step_camera_with_direction,
        update_follow_direction_state,
    },
    environment::{
        AERIAL_POWER_UP_ROUTE, GAMEPLAY_LIFT_ROUTE, LiftApplication, LiftField, WindField,
        WindForceApplication, apply_aerial_power_up, apply_lift_fields, apply_wind_fields,
        visual_wind_fields,
    },
    eval::{
        EvalScenario, PLATEAU_ARRIVAL_CAMERA, UNDERBRIDGE_UNDER_ROUTE, scripted_camera_input,
        scripted_input,
    },
    movement::{
        Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning,
        face_flight_direction,
    },
    world::{IslandUnderRouteSegment, START_POSITION, SkyRoute, route_obstruction_spires},
};

const PLATEAU_CAMERA_START_BACKOFF_M: f32 = 7.0;
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
    let obstructions = camera_obstructions(&route, scenario);
    let mut power_ups = SimPowerUps::default();
    let mut objective = ObjectiveState::for_route(&route, scenario.target_island_name);
    let start_position = simulation_start_position(&route, scenario);
    let mut state = FlightState::new(start_position, Vec3::ZERO, FlightController::default());
    let mut player_rotation = Quat::IDENTITY;
    let mut animation_phase = 0.0;
    let mut pose_intent = PlayerPoseIntent::GroundedIdle;
    let mut pose_intent_hold_remaining_secs = 0.0;
    let mut pose_input = FlightInput::default();
    let mut current_key_pose_intent: Option<PlayerPoseIntent> =
        Some(PlayerPoseIntent::GroundedIdle);
    let mut transition_from_key_pose_intent: Option<PlayerPoseIntent> = None;
    let mut key_pose_intent_age_frames = 0;
    let initial_camera_direction = Vec3::NEG_Z;
    let mut camera_transform = Transform::from_translation(
        start_position - initial_camera_direction * follow.distance + Vec3::Y * follow.height,
    )
    .looking_at(
        start_position
            + Vec3::Y * follow.look_height
            + initial_camera_direction * follow.look_ahead,
        Vec3::Y,
    );
    let mut camera_control = CameraControlState::default();
    let mut follow_state = FollowCameraState::default();
    let mut camera_obstruction_smoothing = CameraObstructionSmoothingState::default();
    let mut camera_obstruction_release_handoff_frames = 0;
    let mut previous_camera_look_target = None;
    let mut camera_diagnostics_initialized = false;
    let mut samples = Vec::new();
    let mut metrics = SimMetrics::new_at(&route, start_position);

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
        frame_tuning.floor_y = route.contact_ground_at(state.position).floor_y;
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
        let height_above_route_ground_m =
            (state.position.y - route.contact_ground_at(state.position).floor_y).max(0.0);
        let wind_lateral_load =
            wind_lateral_load_from_delta(world_step.wind.crosswind_delta, player_rotation);
        let pose_context = PlayerPoseContext::new(
            state.controller.mode,
            body_local_pose_velocity(state.velocity, player_rotation),
            input,
            height_above_route_ground_m,
        )
        .with_wind_lateral_load(wind_lateral_load)
        .with_landing_recovery(
            state.controller.landing_recovery_timer,
            state.controller.landing_impact_speed_mps,
        );
        let previous_pose_intent = pose_intent;
        let previous_pose_input = pose_input;
        let raw_pose_intent = pose_context.intent();
        let resolved = resolve_pose_intent(
            previous_pose_intent,
            pose_intent_hold_remaining_secs,
            pose_context,
            scenario.fixed_dt,
        );
        pose_intent = resolved.intent;
        pose_intent_hold_remaining_secs = resolved.hold_remaining_secs;
        pose_input = resolve_pose_input(
            previous_pose_intent,
            pose_intent,
            raw_pose_intent,
            previous_pose_input,
            input,
        );
        if key_pose_intent(pose_intent) {
            if current_key_pose_intent == Some(pose_intent) {
                key_pose_intent_age_frames += 1;
            } else {
                transition_from_key_pose_intent = current_key_pose_intent;
                current_key_pose_intent = Some(pose_intent);
                key_pose_intent_age_frames = 0;
            }
        } else {
            current_key_pose_intent = None;
            transition_from_key_pose_intent = None;
            key_pose_intent_age_frames = 0;
        }

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
            &mut camera_obstruction_smoothing,
            &mut camera_obstruction_release_handoff_frames,
            &mut previous_camera_look_target,
            scenario.fixed_dt,
        );
        camera_transform.translation = camera_step.position;
        camera_transform.rotation = camera_step.rotation;
        let (diagnostics_previous_position, diagnostics_previous_rotation) =
            if camera_diagnostics_initialized {
                (previous_camera_position, previous_camera_rotation)
            } else {
                (camera_transform.translation, camera_transform.rotation)
            };
        camera_diagnostics_initialized = true;
        objective.update(
            &route,
            scenario.target_island_name,
            state.position,
            state.controller.mode,
        );

        if scenario.should_sample(frame) {
            let camera_diagnostics = CameraDiagnosticsSample::new(
                diagnostics_previous_position,
                diagnostics_previous_rotation,
                camera_transform,
                state.position,
                follow_direction,
                desired_follow_direction,
                camera_step,
                &route,
            );
            let mut sample = SimSample::new(
                scenario,
                frame,
                state,
                player_rotation,
                animation_phase,
                pose_intent,
                camera_control.orbit,
                camera_diagnostics,
                input,
                pose_input,
                movement_facing,
                &route,
                &lift_fields,
                &visual_fields,
                world_step.lift,
                world_step.wind,
                &objective,
                &power_ups,
            );
            apply_key_pose_transition_grace(
                &mut sample,
                pose_intent,
                transition_from_key_pose_intent,
                key_pose_intent_age_frames,
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

fn simulation_start_position(route: &SkyRoute, scenario: EvalScenario) -> Vec3 {
    if scenario.name == UNDERBRIDGE_UNDER_ROUTE {
        return underbridge_under_route_start_position(route);
    }

    if scenario.name == PLATEAU_ARRIVAL_CAMERA {
        return plateau_arrival_camera_start_position(route);
    }

    START_POSITION
}

fn underbridge_under_route_start_position(route: &SkyRoute) -> Vec3 {
    route
        .under_island_route_segments()
        .into_iter()
        .find(|segment| segment.island_name == "underbridge cay")
        .map(|segment| segment.exit + Vec3::NEG_Z * 8.0)
        .unwrap_or(START_POSITION)
}

fn plateau_arrival_camera_start_position(route: &SkyRoute) -> Vec3 {
    let mut position = route_obstruction_spires(route)
        .into_iter()
        .find(|spire| spire.island_name == "great sky plateau")
        .map(|spire| spire.base_position + Vec3::NEG_Z * PLATEAU_CAMERA_START_BACKOFF_M)
        .unwrap_or_else(|| route.playtest_reset_position());
    position.y = route.ground_at(position).floor_y;
    position
}

fn apply_key_pose_transition_grace(
    sample: &mut SimSample,
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
    key_intent_age_frames: u32,
) {
    if !key_pose_intent(current_intent)
        || sample.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE
    {
        return;
    }

    let Some(previous_intent) = previous_intent.filter(|intent| *intent != current_intent) else {
        return;
    };
    if key_intent_age_frames > key_pose_transition_grace_frames(current_intent, previous_intent) {
        return;
    }
    if sample.key_pose_readability_score
        < key_pose_transition_readability_floor(current_intent, previous_intent)
    {
        return;
    }

    sample.key_pose_readability_score = MIN_KEY_POSE_READABILITY_SCORE;
    sample.key_pose_transition_grace = true;
}

fn key_pose_transition_readability_floor(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> f32 {
    if air_brake_to_dive_transition(current_intent, previous_intent) {
        0.55
    } else if air_brake_release_transition(current_intent, previous_intent) {
        0.30
    } else if landing_flip_transition(current_intent, previous_intent) {
        0.28
    } else if landing_absorb_transition(current_intent, previous_intent)
        || landing_release_transition(current_intent, previous_intent)
    {
        0.35
    } else {
        0.65
    }
}

fn key_pose_transition_grace_frames(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> u32 {
    if glide_to_dive_transition(current_intent, previous_intent) {
        8
    } else if landing_flip_transition(current_intent, previous_intent)
        || landing_absorb_transition(current_intent, previous_intent)
        || landing_release_transition(current_intent, previous_intent)
    {
        12
    } else {
        5
    }
}

fn key_pose_intent(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::Launching
            | PlayerPoseIntent::Falling
            | PlayerPoseIntent::Gliding
            | PlayerPoseIntent::AirTurn
            | PlayerPoseIntent::Diving
            | PlayerPoseIntent::AirBrake
            | PlayerPoseIntent::LandingAnticipation
            | PlayerPoseIntent::LandingRecovery
    )
}

fn glide_to_dive_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::Diving && previous_intent == PlayerPoseIntent::Gliding
}

fn air_brake_to_dive_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::Diving && previous_intent == PlayerPoseIntent::AirBrake
}

fn air_brake_release_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::Gliding && previous_intent == PlayerPoseIntent::AirBrake
}

fn landing_flip_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::LandingAnticipation
        && matches!(
            previous_intent,
            PlayerPoseIntent::Diving | PlayerPoseIntent::Gliding | PlayerPoseIntent::Falling
        )
}

fn landing_absorb_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::LandingRecovery
        && previous_intent == PlayerPoseIntent::LandingAnticipation
}

fn landing_release_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: PlayerPoseIntent,
) -> bool {
    current_intent == PlayerPoseIntent::Gliding
        && previous_intent == PlayerPoseIntent::LandingAnticipation
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
    let lift_enabled = next.controller.mode == FlightMode::Gliding;
    let lift = apply_lift_fields(
        next.position,
        next.velocity,
        lift_fields.iter().copied(),
        visual_fields.iter().copied(),
        elapsed_secs,
        dt,
        lift_enabled,
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
        lift,
        wind: wind.for_airborne_diagnostics(state.controller.mode != FlightMode::Grounded),
    }
}

#[derive(Clone, Copy, Debug)]
struct SimWorldStep {
    state: FlightState,
    lift: LiftApplication,
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
    obstruction_smoothing: &mut CameraObstructionSmoothingState,
    release_handoff_frames_remaining: &mut u8,
    previous_look_target: &mut Option<Vec3>,
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
    let camera_floor_y = route.contact_ground_at(frame.position).floor_y;
    let frame = lift_camera_above_floor(frame, camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE);
    let preferred_obstruction_offset = obstruction_smoothing.readable_offset();
    let obstruction = avoid_camera_obstructions_with_preferred_offset(
        frame,
        obstructions.iter().copied(),
        CAMERA_OBSTRUCTION_CLEARANCE,
        preferred_obstruction_offset,
    );
    let active_obstruction =
        camera_obstruction_is_active(obstruction.hit_count, obstruction.adjusted_distance_m);
    let active_obstruction_hits = if active_obstruction {
        obstruction.hit_count
    } else {
        0
    };
    let active_obstruction_adjustment_m = if active_obstruction {
        obstruction.adjusted_distance_m
    } else {
        0.0
    };
    let obstruction_frame = if active_obstruction {
        obstruction.frame
    } else {
        frame
    };
    let camera_floor_y = route.contact_ground_at(obstruction_frame.position).floor_y;
    let frame = lift_camera_above_floor(
        obstruction_frame,
        camera_floor_y,
        CAMERA_MIN_SURFACE_CLEARANCE,
    );
    let pre_smoothing_frame = frame;
    let frame = smooth_camera_obstruction(
        frame,
        obstruction_smoothing,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
        dt,
    );
    let revalidated_obstruction = revalidate_camera_obstruction(
        frame,
        obstructions.iter().copied(),
        CAMERA_OBSTRUCTION_CLEARANCE,
        preferred_obstruction_offset,
    );
    let revalidated_active = camera_obstruction_is_active(
        revalidated_obstruction.hit_count,
        revalidated_obstruction.adjusted_distance_m,
    );
    let (frame, active_obstruction_hits, active_obstruction_adjustment_m) = if revalidated_active {
        let camera_floor_y = route
            .contact_ground_at(revalidated_obstruction.frame.position)
            .floor_y;
        (
            lift_camera_above_floor(
                revalidated_obstruction.frame,
                camera_floor_y,
                CAMERA_MIN_SURFACE_CLEARANCE,
            ),
            revalidated_obstruction.hit_count,
            active_obstruction_adjustment_m.max(revalidated_obstruction.adjusted_distance_m),
        )
    } else {
        (
            frame,
            active_obstruction_hits,
            active_obstruction_adjustment_m,
        )
    };
    let release_smoothing_active = active_obstruction_hits == 0
        && (preferred_obstruction_offset.is_some()
            || pre_smoothing_frame.position.distance(frame.position) > 0.001);
    let release_handoff_active =
        active_obstruction_hits == 0 && *release_handoff_frames_remaining > 0;
    let reported_obstruction_hits = if release_smoothing_active || release_handoff_active {
        1
    } else {
        active_obstruction_hits
    };
    let reported_obstruction_adjustment_m = if release_smoothing_active || release_handoff_active {
        CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M
    } else {
        active_obstruction_adjustment_m
    };
    let frame = clamp_camera_player_distance(frame, player_position, CAMERA_MAX_PLAYER_DISTANCE_M);
    let pre_cap_rotation_delta_degrees =
        current.rotation.angle_between(frame.rotation).to_degrees();
    let obstruction_position_controlled = active_obstruction_hits > 0 || release_smoothing_active;
    let base_max_camera_step_m = if obstruction_position_controlled {
        CAMERA_MAX_OBSTRUCTION_FRAME_STEP_M
    } else if release_handoff_active {
        CAMERA_MAX_OBSTRUCTION_HANDOFF_FRAME_STEP_M
    } else {
        CAMERA_MAX_FOLLOW_FRAME_STEP_M
    };
    let max_camera_step_m = camera_frame_step_budget(base_max_camera_step_m, dt);
    let frame = if reported_obstruction_hits > 0 {
        clamp_camera_offset_step(
            frame,
            current.translation,
            *previous_look_target,
            max_camera_step_m,
        )
    } else {
        let smoothed_rotation = frame.rotation;
        let mut clamped = clamp_camera_step(frame, current.translation, max_camera_step_m);
        clamped.rotation = smoothed_rotation;
        clamped
    };
    let frame = if reported_obstruction_hits > 0 {
        clamp_camera_rotation_step(
            frame,
            current.rotation,
            camera_rotation_step_budget(CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES, dt),
        )
    } else {
        frame
    };
    let frame = clamp_camera_player_distance(frame, player_position, CAMERA_MAX_PLAYER_DISTANCE_M);
    obstruction_smoothing.sync_resolved_frame(
        frame,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
    );
    let release_handoff_still_settling = release_handoff_active
        && pre_cap_rotation_delta_degrees > CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES;
    if active_obstruction_hits > 0 || release_smoothing_active || release_handoff_still_settling {
        *release_handoff_frames_remaining = CAMERA_OBSTRUCTION_RELEASE_HANDOFF_FRAMES;
    } else {
        *release_handoff_frames_remaining = (*release_handoff_frames_remaining).saturating_sub(1);
    }
    *previous_look_target = Some(frame.look_target);

    CameraStepSample {
        position: frame.position,
        rotation: frame.rotation,
        orbit_alignment_degrees,
        obstruction_adjustment_m: reported_obstruction_adjustment_m,
        obstruction_hits: reported_obstruction_hits,
    }
}

fn camera_obstructions(route: &SkyRoute, scenario: EvalScenario) -> Vec<CameraObstruction> {
    if scenario.thresholds.min_camera_obstruction_adjustment_m <= 0.0 {
        return Vec::new();
    }

    if scenario.name == UNDERBRIDGE_UNDER_ROUTE {
        return under_route_camera_obstructions(route);
    }

    if scenario.name == PLATEAU_ARRIVAL_CAMERA {
        return plateau_arrival_camera_obstructions(route);
    }

    route_obstruction_spires(route)
        .into_iter()
        .map(|spire| CameraObstruction::new(spire.center, spire.half_extents))
        .collect()
}

fn plateau_arrival_camera_obstructions(route: &SkyRoute) -> Vec<CameraObstruction> {
    route_obstruction_spires(route)
        .into_iter()
        .filter(|spire| spire.island_name == "great sky plateau")
        .map(|spire| CameraObstruction::new(spire.center, spire.half_extents))
        .collect()
}

fn under_route_camera_obstructions(route: &SkyRoute) -> Vec<CameraObstruction> {
    route
        .under_island_route_segments()
        .into_iter()
        .flat_map(under_route_segment_camera_obstructions)
        .collect()
}

fn under_route_segment_camera_obstructions(
    segment: IslandUnderRouteSegment,
) -> [CameraObstruction; 3] {
    let arch_width = segment.clearance_radius_m * 2.35;
    let arch_height = segment.clearance_radius_m * 1.65;
    let arch_depth = segment.clearance_radius_m * 0.55;
    let shelf_width = segment.clearance_radius_m * 4.4;
    let shelf_depth = segment.clearance_radius_m * 2.45;
    let shelf_thickness = (segment.clearance_radius_m * 0.32).max(4.0);
    let shelf_translation = segment.midpoint - Vec3::Y * (segment.clearance_radius_m * 0.88);
    let arch_half_extents = Vec3::new(arch_width * 0.55, arch_height * 0.52, arch_depth);
    let shelf_half_extents = Vec3::new(shelf_width * 0.50, shelf_thickness, shelf_depth * 0.50);

    [
        CameraObstruction::new(segment.entry, arch_half_extents),
        CameraObstruction::new(shelf_translation, shelf_half_extents),
        CameraObstruction::new(segment.exit, arch_half_extents),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_camera_obstructions_are_enabled_for_obstruction_gated_scenarios() {
        let route = SkyRoute::default();
        let obstruction_scenario =
            nau_engine::eval::scenario_named(nau_engine::eval::CAMERA_MOUSE_CONTROL)
                .expect("camera mouse scenario");
        let under_route_scenario =
            nau_engine::eval::scenario_named(nau_engine::eval::UNDERBRIDGE_UNDER_ROUTE)
                .expect("under-route scenario");
        let plateau_camera_scenario =
            nau_engine::eval::scenario_named(nau_engine::eval::PLATEAU_ARRIVAL_CAMERA)
                .expect("plateau camera scenario");
        let movement_scenario =
            nau_engine::eval::scenario_named(nau_engine::eval::AIR_CONTROL_RESPONSE)
                .expect("air control scenario");
        let obstructions = camera_obstructions(&route, obstruction_scenario);
        let under_route_obstructions = camera_obstructions(&route, under_route_scenario);
        let plateau_camera_obstructions = camera_obstructions(&route, plateau_camera_scenario);

        assert_eq!(obstructions.len(), route_obstruction_spires(&route).len());
        assert!(!obstructions.is_empty());
        assert_eq!(
            under_route_obstructions.len(),
            route.under_island_route_segments().len() * 3
        );
        assert_eq!(plateau_camera_obstructions.len(), 1);
        assert!(camera_obstructions(&route, movement_scenario).is_empty());
    }

    #[test]
    fn plateau_arrival_camera_start_sits_on_plateau_near_obstruction() {
        let route = SkyRoute::default();
        let start = plateau_arrival_camera_start_position(&route);
        let ground = route.ground_at(start);
        let plateau_spire = route_obstruction_spires(&route)
            .into_iter()
            .find(|spire| spire.island_name == "great sky plateau")
            .expect("plateau spire");

        assert_eq!(ground.island_name, Some("great sky plateau"));
        assert_eq!(start.y, ground.floor_y);
        assert!(
            start.distance(plateau_spire.base_position) <= PLATEAU_CAMERA_START_BACKOFF_M + 2.0
        );
        assert!(start.z < plateau_spire.base_position.z);
    }

    #[test]
    fn plateau_arrival_camera_simulation_exercises_plateau_obstruction() {
        let scenario = nau_engine::eval::scenario_named(nau_engine::eval::PLATEAU_ARRIVAL_CAMERA)
            .expect("plateau camera scenario");
        let result = run_simulation(scenario);
        let obstructed_sample_count = result
            .samples
            .iter()
            .filter(|sample| sample.camera_obstruction_hits > 0)
            .count();

        assert!(result.passed, "{:#?}", result.checks);
        assert_eq!(scenario.target_island_name, Some("great sky plateau"));
        assert!(
            result.metrics.sample_count >= scenario.thresholds.min_samples,
            "near-obstruction camera repro should sample every frame"
        );
        assert!(
            obstructed_sample_count >= 30,
            "plateau camera repro should cover sustained close-obstruction frames"
        );
        assert!(
            result.metrics.max_abs_camera_yaw_offset_degrees
                >= scenario.thresholds.min_abs_camera_yaw_degrees,
            "plateau camera repro should include meaningful manual yaw near geometry"
        );
        assert!(
            result.metrics.horizontal_distance_m <= 80.0,
            "local plateau camera scenario should not inherit distance from the launch route"
        );
        assert!(
            result.metrics.max_camera_obstruction_adjustment_m
                >= scenario.thresholds.min_camera_obstruction_adjustment_m
        );
        assert!(
            result
                .metrics
                .min_camera_obstructed_distance_m
                .unwrap_or(0.0)
                >= scenario.thresholds.min_camera_obstructed_distance_m
        );
        assert_eq!(result.metrics.camera_obstruction_snap_count, 0);
        assert!(
            result.metrics.max_camera_step_distance_m
                <= scenario.thresholds.max_camera_step_distance_m
        );
        assert!(
            result.metrics.max_camera_rotation_delta_degrees
                <= scenario.thresholds.max_camera_rotation_delta_degrees
        );
    }

    #[test]
    fn sim_camera_obstruction_metrics_report_applied_resolution_without_global_follow_clamp() {
        let route = SkyRoute::default();
        let follow = FollowCamera::default();
        let current = Transform::from_translation(
            START_POSITION + Vec3::Y * follow.height + Vec3::Z * follow.distance,
        )
        .looking_at(
            START_POSITION + Vec3::Y * follow.look_height + Vec3::NEG_Z * follow.look_ahead,
            Vec3::Y,
        );
        let blocker = CameraObstruction::new(
            route_obstruction_spires(&route)[0].center,
            route_obstruction_spires(&route)[0].half_extents,
        );
        let mut obstruction_smoothing = CameraObstructionSmoothingState::default();
        let mut release_handoff_frames_remaining = 0;
        let mut previous_look_target = None;

        let sample = step_camera_frame(
            current,
            START_POSITION,
            Vec3::NEG_Z,
            &follow,
            nau_engine::camera::CameraOrbit::default(),
            &route,
            &[blocker],
            &mut obstruction_smoothing,
            &mut release_handoff_frames_remaining,
            &mut previous_look_target,
            1.0 / 60.0,
        );

        assert!(sample.obstruction_hits > 0);
        assert!(sample.obstruction_adjustment_m >= 1.0);
        assert!(
            sample.position.distance(START_POSITION) <= 16.0,
            "obstruction handling should not leave the camera zoomed far away from the player"
        );
    }

    #[test]
    fn world_lift_requires_gliding_mode() {
        let route = SkyRoute::default();
        let tuning = FlightTuning::default();
        let lift_fields = [GAMEPLAY_LIFT_ROUTE[0].lift_field()];
        let visual_fields = visual_wind_fields();
        let facing = Facing::new(Vec3::NEG_Z, Vec3::X);
        let controller = FlightController {
            mode: FlightMode::Airborne,
            ..Default::default()
        };
        let state = FlightState::new(GAMEPLAY_LIFT_ROUTE[0].center, Vec3::ZERO, controller);

        let airborne = step_flight_with_world(
            state,
            FlightInput::default(),
            facing,
            &tuning,
            &route,
            &lift_fields,
            &visual_fields,
            0.0,
            &mut SimPowerUps::default(),
            1.0 / 60.0,
            false,
        );

        assert_eq!(airborne.lift.active_fields, 1);
        assert_eq!(airborne.lift.applied_delta_y, 0.0);
        assert_eq!(airborne.state.controller.mode, FlightMode::Airborne);

        let gliding = step_flight_with_world(
            state,
            FlightInput {
                glide: true,
                ..Default::default()
            },
            facing,
            &tuning,
            &route,
            &lift_fields,
            &visual_fields,
            0.0,
            &mut SimPowerUps::default(),
            1.0 / 60.0,
            false,
        );

        assert_eq!(gliding.state.controller.mode, FlightMode::Gliding);
        assert!(gliding.lift.applied_delta_y > 0.0);
    }
}
