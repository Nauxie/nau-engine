use super::{
    BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS, CAMERA_PLAYER_FOCUS_HEIGHT, GROUND_VISUAL_FOOT_GAP_M,
    state::{ObjectiveState, SimPowerUps},
};
use bevy::prelude::{Quat, Transform, Vec3};
use nau_engine::{
    animation::{
        PlayerPoseContext, PlayerPoseIntent, body_local_pose_velocity, glider_traversal_pose,
        pose_readability_metrics, wind_lateral_load_from_delta,
    },
    camera::{
        CameraOrbit, camera_distance, camera_pitch_degrees, camera_surface_clearance,
        camera_target_angle_degrees, camera_view_yaw_degrees,
    },
    environment::{
        AERIAL_POWER_UP_ROUTE, LiftApplication, LiftField, WindField, WindForceApplication,
        readable_lift_fields_at, visible_fields_at, wind_flow_metrics_at,
    },
    eval::EvalScenario,
    movement::{
        Facing, FlightInput, FlightMode, FlightState, body_roll_degrees, body_yaw_error_degrees,
        desired_heading_alignment_speed, desired_planar_movement_direction,
        desired_planar_travel_heading_error_degrees, lateral_response_speed,
    },
    world::SkyRoute,
};
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub(crate) struct CameraStepSample {
    pub(crate) position: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) orbit_alignment_degrees: f32,
    pub(crate) obstruction_adjustment_m: f32,
    pub(crate) obstruction_hits: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CameraDiagnosticsSample {
    pub(crate) distance_m: f32,
    pub(crate) surface_clearance_m: f32,
    pub(crate) player_angle_degrees: f32,
    pub(crate) pitch_degrees: f32,
    pub(crate) step_distance_m: f32,
    pub(crate) rotation_delta_degrees: f32,
    pub(crate) orbit_alignment_degrees: f32,
    pub(crate) follow_direction_error_degrees: f32,
    pub(crate) view_yaw_degrees: f32,
    pub(crate) world_yaw_degrees: f32,
    pub(crate) obstruction_adjustment_m: f32,
    pub(crate) obstruction_hits: usize,
}

impl CameraDiagnosticsSample {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
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

#[derive(Clone, Debug)]
pub(crate) struct SimSample {
    pub(crate) frame: u32,
    pub(crate) time_secs: f32,
    pub(crate) position: Vec3,
    pub(crate) velocity: Vec3,
    pub(crate) speed_mps: f32,
    pub(crate) altitude_m: f32,
    pub(crate) mode: &'static str,
    pub(crate) pose_intent_label: &'static str,
    pub(crate) pose_torso_pitch_degrees: f32,
    pub(crate) pose_arm_spread_degrees: f32,
    pub(crate) pose_leg_tuck_degrees: f32,
    pub(crate) pose_lateral_lean_degrees: f32,
    pub(crate) pose_signed_lateral_lean_degrees: f32,
    pub(crate) pose_grounded_stride_foot_travel_m: f32,
    pub(crate) pose_grounded_stride_leg_opposition_degrees: f32,
    pub(crate) pose_landing_crouch_m: f32,
    pub(crate) pose_landing_foot_forward_m: f32,
    pub(crate) pose_landing_recovery_flip_degrees: f32,
    pub(crate) pose_wing_airflow_strength: f32,
    pub(crate) pose_scarf_stream_m: f32,
    pub(crate) pose_scarf_lateral_sway_m: f32,
    pub(crate) pose_scarf_tail_flex_degrees: f32,
    pub(crate) key_pose_readability_score: f32,
    pub(crate) key_pose_transition_grace: bool,
    pub(crate) desired_body_yaw_error_degrees: f32,
    pub(crate) desired_body_heading_error_degrees: f32,
    pub(crate) body_travel_heading_error_degrees: f32,
    pub(crate) body_roll_degrees: f32,
    pub(crate) desired_heading_alignment_mps: f32,
    pub(crate) desired_travel_heading_error_degrees: f32,
    pub(crate) lateral_response_mps: f32,
    pub(crate) lateral_input_active: bool,
    pub(crate) movement_input_lateral_axis: f32,
    pub(crate) movement_input_forward_axis: f32,
    pub(crate) camera_distance_m: f32,
    pub(crate) camera_surface_clearance_m: f32,
    pub(crate) camera_player_angle_degrees: f32,
    pub(crate) camera_pitch_degrees: f32,
    pub(crate) camera_yaw_offset_degrees: f32,
    pub(crate) camera_pitch_offset_degrees: f32,
    pub(crate) camera_step_distance_m: f32,
    pub(crate) camera_rotation_delta_degrees: f32,
    pub(crate) camera_orbit_alignment_degrees: f32,
    pub(crate) camera_follow_direction_error_degrees: f32,
    pub(crate) camera_view_yaw_degrees: f32,
    pub(crate) camera_world_yaw_degrees: f32,
    pub(crate) camera_obstruction_adjustment_m: f32,
    pub(crate) camera_obstruction_hits: usize,
    pub(crate) visible_wind_fields: usize,
    pub(crate) wind_field_count: usize,
    pub(crate) dynamic_wind_flow_fields: usize,
    pub(crate) max_wind_flow_speed_mps: f32,
    pub(crate) max_wind_flow_variation: f32,
    pub(crate) max_wind_flow_direction_change_degrees: f32,
    pub(crate) active_wind_force_fields: usize,
    pub(crate) crosswind_force_fields: usize,
    pub(crate) updraft_swirl_force_fields: usize,
    pub(crate) max_wind_force_delta_mps: f32,
    pub(crate) max_crosswind_force_delta_mps: f32,
    pub(crate) max_updraft_swirl_force_delta_mps: f32,
    pub(crate) max_wind_force_flow_speed_mps: f32,
    pub(crate) max_wind_force_variation: f32,
    pub(crate) max_wind_force_flow_alignment: f32,
    pub(crate) max_crosswind_force_flow_alignment: f32,
    pub(crate) max_updraft_swirl_force_flow_alignment: f32,
    pub(crate) max_wind_force_aligned_delta_mps: f32,
    pub(crate) max_crosswind_force_aligned_delta_mps: f32,
    pub(crate) max_updraft_swirl_force_aligned_delta_mps: f32,
    pub(crate) wind_lateral_load: f32,
    pub(crate) wind_load_glider_response_degrees: f32,
    pub(crate) active_lift_fields: usize,
    pub(crate) readable_lift_fields: usize,
    pub(crate) paired_visual_lift_fields: usize,
    pub(crate) dynamic_lift_fields: usize,
    pub(crate) lift_applied_delta_mps: f32,
    pub(crate) min_lift_multiplier: f32,
    pub(crate) max_lift_multiplier: f32,
    pub(crate) lift_field_count: usize,
    pub(crate) target_distance_m: f32,
    pub(crate) on_landing_target: bool,
    pub(crate) objective: ObjectiveState,
    pub(crate) sky_island_count: usize,
    pub(crate) active_chunk_count: usize,
    pub(crate) active_island_count: usize,
    pub(crate) near_lod_islands: usize,
    pub(crate) mid_lod_islands: usize,
    pub(crate) far_lod_islands: usize,
    pub(crate) power_up_count: usize,
    pub(crate) visible_power_up_count: usize,
    pub(crate) collected_power_up_count: usize,
    pub(crate) active_power_up_effects: usize,
    pub(crate) total_power_up_activations: usize,
}

impl SimSample {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        scenario: EvalScenario,
        frame: u32,
        state: FlightState,
        player_rotation: Quat,
        pose_phase: f32,
        pose_intent: PlayerPoseIntent,
        orbit: CameraOrbit,
        camera: CameraDiagnosticsSample,
        input: FlightInput,
        pose_input: FlightInput,
        facing: Facing,
        route: &SkyRoute,
        lift_fields: &[LiftField],
        visual_fields: &[WindField],
        lift: LiftApplication,
        wind_force: WindForceApplication,
        objective: &ObjectiveState,
        power_ups: &SimPowerUps,
    ) -> Self {
        let movement_axis = input.planar_axis();
        let desired_movement_direction = desired_planar_movement_direction(input, facing);
        let desired_body_yaw_error_degrees = desired_movement_direction
            .map(|direction| body_yaw_error_degrees(player_rotation, direction))
            .unwrap_or(f32::NAN);
        let desired_heading_alignment_mps = desired_movement_direction
            .map(|direction| desired_heading_alignment_speed(state.velocity, direction))
            .unwrap_or(f32::NAN);
        let desired_travel_heading_error_degrees = desired_movement_direction
            .map(|direction| {
                desired_planar_travel_heading_error_degrees(
                    state.velocity,
                    direction,
                    BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS,
                )
            })
            .unwrap_or(f32::NAN);
        let lateral_axis_active = input.has_lateral_axis();
        let lateral_input_active =
            lateral_axis_active && state.controller.mode != FlightMode::Grounded;
        let body_travel_heading_error_degrees = body_travel_heading_error_degrees(
            player_rotation,
            state.velocity,
            state.controller.mode,
            lateral_input_active,
        );
        let lateral_response_mps = if lateral_axis_active {
            lateral_response_speed(state.velocity, input, facing)
        } else {
            0.0
        };
        let height_above_route_ground_m =
            (state.position.y - route.ground_at(state.position).floor_y).max(0.0);
        let time_secs = frame as f32 * scenario.fixed_dt;
        let wind_lateral_load =
            wind_lateral_load_from_delta(wind_force.crosswind_delta, player_rotation);
        let pose_context = PlayerPoseContext::new(
            state.controller.mode,
            body_local_pose_velocity(state.velocity, player_rotation),
            pose_input,
            height_above_route_ground_m,
        )
        .with_wind_lateral_load(wind_lateral_load)
        .with_landing_recovery(
            state.controller.landing_recovery_timer,
            state.controller.landing_impact_speed_mps,
        )
        .with_resolved_intent(pose_intent);
        let pose_intent_label = pose_context.intent().label();
        let pose_readability = pose_readability_metrics(pose_context, pose_phase);
        let wind_load_glider_response_degrees =
            glider_traversal_pose(pose_context, pose_phase).response_degrees();
        let streaming_lod = route.streaming_lod_stats(state.position);
        let wind_flow =
            wind_flow_metrics_at(state.position, time_secs, visual_fields.iter().copied());

        Self {
            frame,
            time_secs,
            position: state.position,
            velocity: state.velocity,
            speed_mps: state.velocity.length(),
            altitude_m: state.position.y,
            mode: state.controller.mode.label(),
            pose_intent_label,
            pose_torso_pitch_degrees: pose_readability.torso_pitch_degrees,
            pose_arm_spread_degrees: pose_readability.arm_spread_degrees,
            pose_leg_tuck_degrees: pose_readability.leg_tuck_degrees,
            pose_lateral_lean_degrees: pose_readability.lateral_lean_degrees,
            pose_signed_lateral_lean_degrees: pose_readability.signed_lateral_lean_degrees,
            pose_grounded_stride_foot_travel_m: pose_readability.grounded_stride_foot_travel_m,
            pose_grounded_stride_leg_opposition_degrees: pose_readability
                .grounded_stride_leg_opposition_degrees,
            pose_landing_crouch_m: pose_readability.landing_crouch_m,
            pose_landing_foot_forward_m: pose_readability.landing_foot_forward_m,
            pose_landing_recovery_flip_degrees: pose_readability.landing_recovery_flip_degrees,
            pose_wing_airflow_strength: pose_readability.wing_airflow_strength,
            pose_scarf_stream_m: pose_readability.scarf_stream_m,
            pose_scarf_lateral_sway_m: pose_readability.scarf_lateral_sway_m,
            pose_scarf_tail_flex_degrees: pose_readability.scarf_tail_flex_degrees,
            key_pose_readability_score: pose_readability.key_pose_readability_score,
            key_pose_transition_grace: false,
            desired_body_yaw_error_degrees,
            desired_body_heading_error_degrees: desired_body_yaw_error_degrees.abs(),
            body_travel_heading_error_degrees,
            body_roll_degrees: body_roll_degrees(player_rotation),
            desired_heading_alignment_mps,
            desired_travel_heading_error_degrees,
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
            dynamic_wind_flow_fields: wind_flow.active_fields,
            max_wind_flow_speed_mps: wind_flow.max_speed_mps,
            max_wind_flow_variation: wind_flow.max_variation,
            max_wind_flow_direction_change_degrees: wind_flow.max_direction_change_degrees,
            active_wind_force_fields: wind_force.active_fields,
            crosswind_force_fields: wind_force.crosswind_fields,
            updraft_swirl_force_fields: wind_force.updraft_swirl_fields,
            max_wind_force_delta_mps: wind_force.applied_delta_mps(),
            max_crosswind_force_delta_mps: wind_force.crosswind_delta_mps(),
            max_updraft_swirl_force_delta_mps: wind_force.updraft_swirl_delta_mps(),
            max_wind_force_flow_speed_mps: wind_force.max_flow_speed_mps,
            max_wind_force_variation: wind_force.max_variation,
            max_wind_force_flow_alignment: wind_force.max_flow_alignment,
            max_crosswind_force_flow_alignment: wind_force.max_crosswind_flow_alignment,
            max_updraft_swirl_force_flow_alignment: wind_force.max_updraft_swirl_flow_alignment,
            max_wind_force_aligned_delta_mps: wind_force.max_flow_aligned_delta_mps,
            max_crosswind_force_aligned_delta_mps: wind_force.max_crosswind_flow_aligned_delta_mps,
            max_updraft_swirl_force_aligned_delta_mps: wind_force
                .max_updraft_swirl_flow_aligned_delta_mps,
            wind_lateral_load,
            wind_load_glider_response_degrees,
            active_lift_fields: lift.active_fields,
            readable_lift_fields: readable_lift_fields_at(
                state.position,
                lift_fields.iter().copied(),
                visual_fields.iter().copied(),
            ),
            paired_visual_lift_fields: lift.paired_visual_fields,
            dynamic_lift_fields: lift.dynamic_lift_fields,
            lift_applied_delta_mps: lift.applied_delta_y,
            min_lift_multiplier: lift.min_lift_multiplier,
            max_lift_multiplier: lift.max_lift_multiplier,
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

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "frame": self.frame,
            "time_secs": round4(self.time_secs),
            "position": vec3_json(self.position),
            "velocity": vec3_json(self.velocity),
            "speed_mps": round4(self.speed_mps),
            "altitude_m": round4(self.altitude_m),
            "mode": self.mode,
            "pose_intent": self.pose_intent_label,
            "pose_torso_pitch_degrees": round4(self.pose_torso_pitch_degrees),
            "pose_arm_spread_degrees": round4(self.pose_arm_spread_degrees),
            "pose_leg_tuck_degrees": round4(self.pose_leg_tuck_degrees),
            "pose_lateral_lean_degrees": round4(self.pose_lateral_lean_degrees),
            "pose_signed_lateral_lean_degrees": round4(self.pose_signed_lateral_lean_degrees),
            "pose_grounded_stride_foot_travel_m": round4(self.pose_grounded_stride_foot_travel_m),
            "pose_grounded_stride_leg_opposition_degrees": round4(self.pose_grounded_stride_leg_opposition_degrees),
            "pose_landing_crouch_m": round4(self.pose_landing_crouch_m),
            "pose_landing_foot_forward_m": round4(self.pose_landing_foot_forward_m),
            "pose_landing_recovery_flip_degrees": round4(self.pose_landing_recovery_flip_degrees),
            "pose_wing_airflow_strength": round4(self.pose_wing_airflow_strength),
            "pose_scarf_stream_m": round4(self.pose_scarf_stream_m),
            "pose_scarf_lateral_sway_m": round4(self.pose_scarf_lateral_sway_m),
            "pose_scarf_tail_flex_degrees": round4(self.pose_scarf_tail_flex_degrees),
            "key_pose_readability_score": round4(self.key_pose_readability_score),
            "key_pose_transition_grace": self.key_pose_transition_grace,
            "desired_body_yaw_error_degrees": finite_json(self.desired_body_yaw_error_degrees),
            "desired_body_heading_error_degrees": finite_json(self.desired_body_heading_error_degrees),
            "body_travel_heading_error_degrees": finite_json(self.body_travel_heading_error_degrees),
            "body_roll_degrees": round4(self.body_roll_degrees),
            "desired_heading_alignment_mps": finite_json(self.desired_heading_alignment_mps),
            "desired_travel_heading_error_degrees": finite_json(self.desired_travel_heading_error_degrees),
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
            "dynamic_wind_flow_fields": self.dynamic_wind_flow_fields,
            "max_wind_flow_speed_mps": round4(self.max_wind_flow_speed_mps),
            "max_wind_flow_variation": round4(self.max_wind_flow_variation),
            "max_wind_flow_direction_change_degrees": round4(self.max_wind_flow_direction_change_degrees),
            "active_wind_force_fields": self.active_wind_force_fields,
            "crosswind_force_fields": self.crosswind_force_fields,
            "updraft_swirl_force_fields": self.updraft_swirl_force_fields,
            "max_wind_force_delta_mps": round4(self.max_wind_force_delta_mps),
            "max_crosswind_force_delta_mps": round4(self.max_crosswind_force_delta_mps),
            "max_updraft_swirl_force_delta_mps": round4(self.max_updraft_swirl_force_delta_mps),
            "max_wind_force_flow_speed_mps": round4(self.max_wind_force_flow_speed_mps),
            "max_wind_force_variation": round4(self.max_wind_force_variation),
            "max_wind_force_flow_alignment": round4(self.max_wind_force_flow_alignment),
            "max_crosswind_force_flow_alignment": round4(self.max_crosswind_force_flow_alignment),
            "max_updraft_swirl_force_flow_alignment": round4(self.max_updraft_swirl_force_flow_alignment),
            "max_wind_force_aligned_delta_mps": round4(self.max_wind_force_aligned_delta_mps),
            "max_crosswind_force_aligned_delta_mps": round4(self.max_crosswind_force_aligned_delta_mps),
            "max_updraft_swirl_force_aligned_delta_mps": round4(self.max_updraft_swirl_force_aligned_delta_mps),
            "wind_lateral_load": round4(self.wind_lateral_load),
            "wind_load_glider_response_degrees": round4(self.wind_load_glider_response_degrees),
            "active_lift_fields": self.active_lift_fields,
            "readable_lift_fields": self.readable_lift_fields,
            "paired_visual_lift_fields": self.paired_visual_lift_fields,
            "dynamic_lift_fields": self.dynamic_lift_fields,
            "lift_applied_delta_mps": round4(self.lift_applied_delta_mps),
            "min_lift_multiplier": round4(self.min_lift_multiplier),
            "max_lift_multiplier": round4(self.max_lift_multiplier),
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

pub(crate) fn vec3_json(value: Vec3) -> Value {
    json!([round4(value.x), round4(value.y), round4(value.z)])
}

fn body_travel_heading_error_degrees(
    player_rotation: Quat,
    velocity: Vec3,
    mode: FlightMode,
    lateral_input_active: bool,
) -> f32 {
    if !lateral_input_active || !matches!(mode, FlightMode::Airborne | FlightMode::Gliding) {
        return f32::NAN;
    }

    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length() < BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS {
        return f32::NAN;
    }

    body_yaw_error_degrees(player_rotation, horizontal_velocity).abs()
}

fn finite_json(value: f32) -> Value {
    if value.is_finite() {
        json!(round4(value))
    } else {
        Value::Null
    }
}

pub(crate) fn round4(value: f32) -> f32 {
    if value.is_finite() {
        (value * 10_000.0).round() / 10_000.0
    } else {
        value
    }
}

pub(crate) fn round4_f64(value: f64) -> f64 {
    if value.is_finite() {
        (value * 10_000.0).round() / 10_000.0
    } else {
        value
    }
}
