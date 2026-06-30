use crate::movement::{
    FlightInput, FlightMode, body_forward,
    landing_recovery_strength as movement_landing_recovery_strength, smoothing_factor,
};
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Component, Clone, Copy, Debug)]
pub struct AnimationState {
    pub phase: f32,
    pub last_input: FlightInput,
    pub height_above_ground_m: f32,
    pub wind_lateral_load: f32,
    pub pose_intent: PlayerPoseIntent,
    pub pose_intent_hold_remaining_secs: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            last_input: FlightInput::default(),
            height_above_ground_m: f32::INFINITY,
            wind_lateral_load: 0.0,
            pose_intent: PlayerPoseIntent::GroundedIdle,
            pose_intent_hold_remaining_secs: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct CharacterPart {
    pub role: CharacterPartRole,
    pub base_translation: Vec3,
    pub base_rotation: Quat,
}

impl CharacterPart {
    pub fn new(role: CharacterPartRole, base_translation: Vec3, base_rotation: Quat) -> Self {
        Self {
            role,
            base_translation,
            base_rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterPartRole {
    Hips,
    Torso,
    Head,
    Arm(Side),
    Forearm(Side),
    Hand(Side),
    Leg(Side),
    LowerLeg(Side),
    Foot(Side),
    Wing(Side),
    Scarf(ScarfSegment),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScarfSegment {
    Anchor,
    Trail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartVisibility {
    Inherited,
    Hidden,
    Visible,
}

#[derive(Clone, Copy, Debug)]
pub struct PartPose {
    pub translation: Vec3,
    pub rotation: Quat,
    pub visibility: PartVisibility,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PoseReadabilityMetrics {
    pub torso_pitch_degrees: f32,
    pub arm_spread_degrees: f32,
    pub leg_tuck_degrees: f32,
    pub lateral_lean_degrees: f32,
    pub signed_lateral_lean_degrees: f32,
    pub grounded_stride_foot_travel_m: f32,
    pub grounded_stride_leg_opposition_degrees: f32,
    pub landing_crouch_m: f32,
    pub landing_foot_forward_m: f32,
    pub landing_foot_split_m: f32,
    pub landing_recovery_flip_degrees: f32,
    pub wing_airflow_strength: f32,
    pub scarf_stream_m: f32,
    pub scarf_lateral_sway_m: f32,
    pub scarf_tail_flex_degrees: f32,
    pub launch_overhead_arm_score: f32,
    pub key_pose_readability_score: f32,
}

pub const MIN_KEY_POSE_READABILITY_SCORE: f32 = 0.9;
pub const GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M: f32 = 0.08;
pub const GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M: f32 = 0.16;
pub const GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES: f32 = 20.0;
pub const GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES: f32 = 34.0;
pub const GROUNDED_STRIDE_MIN_FOOT_TRAVEL_M: f32 = GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M;
pub const GROUNDED_STRIDE_MIN_LEG_OPPOSITION_DEGREES: f32 =
    GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES;
pub const WIND_LOAD_FULL_RESPONSE_DELTA_MPS: f32 = 0.10;
pub const LANDING_MIN_FOOT_FORWARD_READABILITY_M: f32 = 0.32;
pub const LANDING_MIN_FOOT_SPLIT_READABILITY_M: f32 = 0.14;
const DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES: f32 = 82.0;
const DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES: f32 = 74.0;
const DIVE_MIN_LEG_TUCK_READABILITY_DEGREES: f32 = 68.0;
const CONNECTED_LIMB_MAX_TRANSLATION_M: f32 = 0.015;
const LANDING_ANTICIPATION_BASE_HEIGHT_M: f32 = 6.0;
const LANDING_ANTICIPATION_MAX_HEIGHT_M: f32 = 36.0;
const LANDING_ANTICIPATION_SINK_LOOKAHEAD_SECS: f32 = 0.95;

#[derive(Clone, Copy, Debug)]
pub struct PoseReadabilityPartTransforms {
    pub torso_rotation: Quat,
    pub left_arm_rotation: Quat,
    pub right_arm_rotation: Quat,
    pub left_leg_rotation: Quat,
    pub right_leg_rotation: Quat,
    pub left_leg_translation: Vec3,
    pub right_leg_translation: Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerPoseIntent {
    #[default]
    GroundedIdle,
    GroundedStride,
    GroundedWalk,
    GroundedRun,
    Launching,
    Falling,
    Gliding,
    AirTurn,
    Diving,
    AirBrake,
    LandingAnticipation,
    LandingRecovery,
}

impl PlayerPoseIntent {
    pub fn label(self) -> &'static str {
        match self {
            Self::GroundedIdle => "grounded_idle",
            Self::GroundedStride => "grounded_stride",
            Self::GroundedWalk => "grounded_walk",
            Self::GroundedRun => "grounded_run",
            Self::Launching => "launching",
            Self::Falling => "falling",
            Self::Gliding => "gliding",
            Self::AirTurn => "air_turn",
            Self::Diving => "diving",
            Self::AirBrake => "air_brake",
            Self::LandingAnticipation => "landing_anticipation",
            Self::LandingRecovery => "landing_recovery",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerPoseContext {
    pub mode: FlightMode,
    pub velocity: Vec3,
    pub input: FlightInput,
    pub height_above_ground_m: f32,
    pub wind_lateral_load: f32,
    pub landing_recovery_remaining_secs: f32,
    pub landing_impact_speed_mps: f32,
    pub resolved_intent: Option<PlayerPoseIntent>,
}

impl PlayerPoseContext {
    pub fn new(
        mode: FlightMode,
        velocity: Vec3,
        input: FlightInput,
        height_above_ground_m: f32,
    ) -> Self {
        Self {
            mode,
            velocity,
            input,
            height_above_ground_m,
            wind_lateral_load: 0.0,
            landing_recovery_remaining_secs: 0.0,
            landing_impact_speed_mps: 0.0,
            resolved_intent: None,
        }
    }

    pub fn with_wind_lateral_load(mut self, wind_lateral_load: f32) -> Self {
        self.wind_lateral_load = wind_lateral_load.clamp(-1.0, 1.0);
        self
    }

    pub fn with_landing_recovery(mut self, remaining_secs: f32, impact_speed_mps: f32) -> Self {
        self.landing_recovery_remaining_secs = remaining_secs.max(0.0);
        self.landing_impact_speed_mps = impact_speed_mps.max(0.0);
        self
    }

    pub fn with_resolved_intent(mut self, intent: PlayerPoseIntent) -> Self {
        self.resolved_intent = Some(intent);
        self
    }

    pub fn intent(self) -> PlayerPoseIntent {
        self.resolved_intent
            .unwrap_or_else(|| player_pose_intent(self.without_resolved_intent()))
    }

    pub fn landing_recovery_strength(self) -> f32 {
        movement_landing_recovery_strength(
            self.landing_recovery_remaining_secs,
            self.landing_impact_speed_mps,
        )
    }

    fn without_resolved_intent(mut self) -> Self {
        self.resolved_intent = None;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseIntentResolution {
    pub intent: PlayerPoseIntent,
    pub hold_remaining_secs: f32,
}

pub fn advance_phase(phase: f32, speed: f32, dt: f32) -> f32 {
    (phase + (5.0 + speed.max(0.0) * 0.08) * dt.max(0.0)).rem_euclid(TAU)
}

pub fn pose_blend_for_intent(intent: PlayerPoseIntent, dt: f32) -> f32 {
    let rate = match intent {
        PlayerPoseIntent::LandingAnticipation => 60.0,
        PlayerPoseIntent::LandingRecovery => 30.0,
        PlayerPoseIntent::Gliding | PlayerPoseIntent::AirTurn => 28.0,
        PlayerPoseIntent::Diving => 14.0,
        PlayerPoseIntent::AirBrake => 24.0,
        _ => 18.0,
    };
    smoothing_factor(rate, dt)
}

pub fn wing_airflow_strength(mode: FlightMode, velocity: Vec3) -> f32 {
    if mode != FlightMode::Gliding {
        return 0.0;
    }

    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    let speed_pressure = ((horizontal_speed - 18.0) / 44.0).clamp(0.0, 1.0);
    let sink_pressure = (-velocity.y / 28.0).clamp(0.0, 1.0) * 0.18;
    (speed_pressure + sink_pressure).clamp(0.0, 1.0)
}

pub fn wind_lateral_load_from_delta(wind_delta: Vec3, player_rotation: Quat) -> f32 {
    let forward = body_forward(player_rotation);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    if right.length_squared() <= f32::EPSILON {
        return 0.0;
    }

    (wind_delta.dot(right) / WIND_LOAD_FULL_RESPONSE_DELTA_MPS).clamp(-1.0, 1.0)
}

pub fn glider_deployment_for_mode(mode: FlightMode) -> f32 {
    match mode {
        FlightMode::Launching => 0.52,
        FlightMode::Gliding => 1.0,
        _ => 0.0,
    }
}

pub fn glider_deployment_for_context(context: PlayerPoseContext) -> f32 {
    if context.mode == FlightMode::Gliding && context.intent() == PlayerPoseIntent::Diving {
        0.42
    } else {
        glider_deployment_for_mode(context.mode)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GliderTraversalPose {
    pub translation_offset: Vec3,
    pub rotation_offset: Quat,
}

impl Default for GliderTraversalPose {
    fn default() -> Self {
        Self {
            translation_offset: Vec3::ZERO,
            rotation_offset: Quat::IDENTITY,
        }
    }
}

impl GliderTraversalPose {
    pub fn response_degrees(self) -> f32 {
        self.rotation_offset
            .angle_between(Quat::IDENTITY)
            .to_degrees()
    }

    pub fn motion_m(self) -> f32 {
        self.translation_offset.length()
    }
}

pub fn glider_traversal_pose(context: PlayerPoseContext, phase: f32) -> GliderTraversalPose {
    if context.mode == FlightMode::Launching {
        let unfurl = (phase * 1.7).sin() * 0.010;
        return GliderTraversalPose {
            translation_offset: Vec3::new(0.0, 2.950 + unfurl, -0.260),
            rotation_offset: Quat::from_rotation_x(-0.18 + unfurl * 0.8)
                * Quat::from_rotation_z(0.045),
        };
    }

    if context.mode != FlightMode::Gliding {
        return GliderTraversalPose::default();
    }

    let intent = context.intent();
    let turn_weight = pose_turn_weight(context);
    let airflow = wing_airflow_strength(context.mode, context.velocity);
    let dive_pressure = dive_pose_pressure(context, intent);
    let brake_pressure = air_brake_pose_pressure(context, intent);
    let rearward_brake_pressure = rearward_air_brake_pressure(context, intent);
    let brake_turn_pressure = if intent == PlayerPoseIntent::AirBrake {
        turn_weight
    } else {
        0.0
    };
    let turn_pressure = if intent == PlayerPoseIntent::AirTurn {
        turn_weight
    } else if intent == PlayerPoseIntent::AirBrake {
        turn_weight * 0.72
    } else {
        turn_weight * 0.45
    };
    let flutter = (phase * 2.2).sin() * (0.010 + airflow * 0.035);
    let roll = (-turn_pressure * 0.22 - brake_turn_pressure * 0.09).clamp(-0.31, 0.31);
    let yaw = (turn_pressure * 0.08 + brake_turn_pressure * 0.06).clamp(-0.16, 0.16);
    let pitch = (-airflow * 0.06 - dive_pressure * 0.24
        + brake_pressure * 0.18
        + rearward_brake_pressure * 0.08
        + flutter)
        .clamp(-0.24, 0.30);

    GliderTraversalPose {
        translation_offset: Vec3::new(
            turn_pressure * 0.060 + brake_turn_pressure * 0.045,
            0.780
                + airflow * 0.050
                + dive_pressure * 0.012
                + brake_pressure * 0.035
                + flutter * 0.50,
            0.300 + airflow * 0.080 + dive_pressure * 0.125
                - brake_pressure * 0.100
                - rearward_brake_pressure * 0.225,
        ),
        rotation_offset: Quat::from_rotation_z(roll)
            * Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(pitch),
    }
}

pub fn body_local_pose_velocity(world_velocity: Vec3, player_rotation: Quat) -> Vec3 {
    let forward = body_forward(player_rotation);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    Vec3::new(
        world_velocity.dot(right),
        world_velocity.y,
        -world_velocity.dot(forward),
    )
}

pub fn player_pose_intent(context: PlayerPoseContext) -> PlayerPoseIntent {
    let context = context.without_resolved_intent();
    let horizontal_speed = Vec2::new(context.velocity.x, context.velocity.z).length();
    if context.mode == FlightMode::Grounded && context.landing_recovery_strength() > 0.0 {
        return PlayerPoseIntent::LandingRecovery;
    }

    let near_landing = context.mode != FlightMode::Grounded
        && context.height_above_ground_m <= landing_anticipation_height_m(context.velocity.y)
        && context.velocity.y < -1.2;

    if near_landing {
        return PlayerPoseIntent::LandingAnticipation;
    }

    match context.mode {
        FlightMode::Grounded => {
            if horizontal_speed > 6.0 {
                PlayerPoseIntent::GroundedRun
            } else if horizontal_speed > 1.0 {
                PlayerPoseIntent::GroundedWalk
            } else {
                PlayerPoseIntent::GroundedIdle
            }
        }
        FlightMode::Launching => PlayerPoseIntent::Launching,
        FlightMode::Gliding if context.input.backward => PlayerPoseIntent::AirBrake,
        FlightMode::Gliding if context.input.dive || context.velocity.y < -14.0 => {
            PlayerPoseIntent::Diving
        }
        FlightMode::Gliding if airborne_turn_input(context) => PlayerPoseIntent::AirTurn,
        FlightMode::Gliding => PlayerPoseIntent::Gliding,
        FlightMode::Airborne if context.input.dive || context.velocity.y < -18.0 => {
            PlayerPoseIntent::Diving
        }
        FlightMode::Airborne if airborne_turn_input(context) => PlayerPoseIntent::AirTurn,
        FlightMode::Airborne => PlayerPoseIntent::Falling,
    }
}

pub fn resolve_pose_intent(
    previous_intent: PlayerPoseIntent,
    previous_hold_remaining_secs: f32,
    context: PlayerPoseContext,
    dt: f32,
) -> PoseIntentResolution {
    let raw_intent = player_pose_intent(context);
    let decayed_hold = (previous_hold_remaining_secs - dt.max(0.0)).max(0.0);

    if raw_intent == previous_intent {
        return PoseIntentResolution {
            intent: raw_intent,
            hold_remaining_secs: pose_intent_hold_secs(raw_intent),
        };
    }

    if pose_intent_is_immediate(raw_intent) {
        return PoseIntentResolution {
            intent: raw_intent,
            hold_remaining_secs: pose_intent_hold_secs(raw_intent),
        };
    }

    if pose_intent_can_hold(previous_intent)
        && pose_intent_can_be_held_over(raw_intent)
        && decayed_hold > 0.0
    {
        return PoseIntentResolution {
            intent: previous_intent,
            hold_remaining_secs: decayed_hold,
        };
    }

    PoseIntentResolution {
        intent: raw_intent,
        hold_remaining_secs: pose_intent_hold_secs(raw_intent),
    }
}

pub fn resolve_pose_input(
    previous_intent: PlayerPoseIntent,
    resolved_intent: PlayerPoseIntent,
    raw_intent: PlayerPoseIntent,
    previous_input: FlightInput,
    current_input: FlightInput,
) -> FlightInput {
    let holding_previous_intent = resolved_intent == previous_intent
        && raw_intent != resolved_intent
        && pose_intent_can_hold(resolved_intent)
        && pose_intent_can_be_held_over(raw_intent);

    if !holding_previous_intent {
        return current_input;
    }

    match resolved_intent {
        PlayerPoseIntent::AirTurn => carry_lateral_pose_input(previous_input, current_input),
        PlayerPoseIntent::AirBrake => carry_air_brake_pose_input(previous_input, current_input),
        _ => current_input,
    }
}

fn carry_lateral_pose_input(
    previous_input: FlightInput,
    current_input: FlightInput,
) -> FlightInput {
    if !previous_input.has_lateral_axis() || current_input.has_lateral_axis() {
        return current_input;
    }

    FlightInput {
        left: previous_input.left,
        right: previous_input.right,
        ..current_input
    }
}

fn carry_air_brake_pose_input(
    previous_input: FlightInput,
    current_input: FlightInput,
) -> FlightInput {
    if !previous_input.backward {
        return current_input;
    }

    let mut pose_input = current_input;
    if !pose_input.backward {
        pose_input.backward = true;
    }
    if previous_input.has_lateral_axis() && !pose_input.has_lateral_axis() {
        pose_input.left = previous_input.left;
        pose_input.right = previous_input.right;
    }
    pose_input
}

fn pose_intent_hold_secs(intent: PlayerPoseIntent) -> f32 {
    match intent {
        PlayerPoseIntent::AirTurn => 0.12,
        PlayerPoseIntent::AirBrake => 0.16,
        _ => 0.0,
    }
}

fn pose_intent_can_hold(intent: PlayerPoseIntent) -> bool {
    pose_intent_hold_secs(intent) > 0.0
}

fn pose_intent_can_be_held_over(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::Falling | PlayerPoseIntent::Gliding
    )
}

fn pose_intent_is_immediate(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::GroundedIdle
            | PlayerPoseIntent::GroundedStride
            | PlayerPoseIntent::GroundedWalk
            | PlayerPoseIntent::GroundedRun
            | PlayerPoseIntent::Launching
            | PlayerPoseIntent::LandingAnticipation
            | PlayerPoseIntent::LandingRecovery
    )
}

fn airborne_turn_input(context: PlayerPoseContext) -> bool {
    context.input.planar_axis().x.abs() >= 0.25
}

fn side_cycle(phase: f32, side: Side) -> f32 {
    let offset = if side == Side::Left { 0.0 } else { TAU * 0.5 };
    (phase + offset).sin()
}

fn breath_cycle(phase: f32) -> f32 {
    (phase * 0.42).sin()
}

fn airflow_micro_cycle(phase: f32, side: Side) -> f32 {
    (phase * 1.7 + side.sign() * 0.55).sin()
}

fn dive_airflow_flutter(phase: f32, dive_pressure: f32) -> f32 {
    (phase * 2.35).sin() * dive_pressure.clamp(0.0, 1.0)
}

fn airborne_pose_intent(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::Falling
            | PlayerPoseIntent::Gliding
            | PlayerPoseIntent::AirTurn
            | PlayerPoseIntent::Diving
            | PlayerPoseIntent::AirBrake
            | PlayerPoseIntent::LandingAnticipation
    )
}

fn pose_turn_weight(context: PlayerPoseContext) -> f32 {
    let intent = context.intent();
    let divisor = if airborne_pose_intent(intent) {
        18.0
    } else {
        12.0
    };
    let velocity_weight = (context.velocity.x / divisor).clamp(-1.0, 1.0);
    let input_weight = context.input.planar_axis().x.clamp(-1.0, 1.0);
    let base_weight = if airborne_pose_intent(intent) && input_weight.abs() > f32::EPSILON {
        (velocity_weight * 0.35 + input_weight * 0.65).clamp(-1.0, 1.0)
    } else {
        velocity_weight
    };
    if airborne_pose_intent(intent) {
        let wind_weight = context.wind_lateral_load * (1.0 - input_weight.abs()).clamp(0.0, 1.0);
        (base_weight + wind_weight * 0.55).clamp(-1.0, 1.0)
    } else {
        base_weight
    }
}

fn pose_lateral_lean_radians(context: PlayerPoseContext) -> f32 {
    let intent = context.intent();
    let max_lean = if intent == PlayerPoseIntent::AirTurn {
        0.30
    } else if intent == PlayerPoseIntent::AirBrake {
        0.26
    } else if airborne_pose_intent(intent) {
        0.22
    } else {
        0.08
    };
    -pose_turn_weight(context) * max_lean
}

fn dive_pose_pressure(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::Diving {
        return 0.0;
    }

    let sink_pressure = ((-context.velocity.y - 6.0) / 18.0).clamp(0.0, 1.0);
    let input_pressure = if context.input.dive { 0.26 } else { 0.0 };
    (0.28 + input_pressure + sink_pressure * 0.52).clamp(0.0, 1.0)
}

fn air_brake_pose_pressure(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::AirBrake {
        return 0.0;
    }

    let speed_pressure = ((Vec2::new(context.velocity.x, context.velocity.z).length() - 12.0)
        / 28.0)
        .clamp(0.0, 1.0);
    (0.78 + speed_pressure * 0.22).clamp(0.0, 1.0)
}

fn rearward_air_brake_pressure(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::AirBrake {
        return 0.0;
    }

    (context.velocity.z / 14.0).clamp(0.0, 1.0)
}

fn landing_anticipation_strength(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::LandingAnticipation {
        return 0.0;
    }

    let anticipation_height = landing_anticipation_height_m(context.velocity.y);
    let ground_proximity = ((anticipation_height - context.height_above_ground_m)
        / anticipation_height.max(0.1))
    .clamp(0.0, 1.0);
    let sink_rate = ((-context.velocity.y - 1.2) / 6.0).clamp(0.0, 1.0);
    (0.65 + ground_proximity.max(sink_rate) * 0.35).clamp(0.0, 1.0)
}

fn landing_flip_strength(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::LandingAnticipation {
        return 0.0;
    }

    let anticipation_height = landing_anticipation_height_m(context.velocity.y);
    let proximity = ((anticipation_height - context.height_above_ground_m)
        / anticipation_height.max(0.1))
    .clamp(0.0, 1.0);
    let fast_sink = ((-context.velocity.y - 6.0) / 12.0).clamp(0.0, 1.0);
    ((proximity * 0.45) + (fast_sink * 0.75)).clamp(0.0, 1.0)
}

fn landing_anticipation_height_m(vertical_velocity_mps: f32) -> f32 {
    (LANDING_ANTICIPATION_BASE_HEIGHT_M
        + (-vertical_velocity_mps).max(0.0) * LANDING_ANTICIPATION_SINK_LOOKAHEAD_SECS)
        .min(LANDING_ANTICIPATION_MAX_HEIGHT_M)
}

fn landing_recovery_strength(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::LandingRecovery {
        return 0.0;
    }

    context.landing_recovery_strength()
}

fn landing_lead_side_sign(context: PlayerPoseContext) -> f32 {
    let lateral_bias = context.input.planar_axis().x + context.velocity.x / 24.0;
    if lateral_bias < -0.08 { -1.0 } else { 1.0 }
}

fn landing_lead_weight(context: PlayerPoseContext, side: Side) -> f32 {
    if side.sign() == landing_lead_side_sign(context) {
        1.0
    } else {
        0.0
    }
}

pub fn key_pose_readability_score(
    intent: PlayerPoseIntent,
    torso_pitch_degrees: f32,
    arm_spread_degrees: f32,
    leg_tuck_degrees: f32,
    landing_crouch_m: f32,
    landing_foot_forward_m: f32,
    landing_foot_split_m: f32,
) -> f32 {
    match intent {
        PlayerPoseIntent::Diving => readable_pair_score(
            torso_pitch_degrees,
            DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES / MIN_KEY_POSE_READABILITY_SCORE,
            leg_tuck_degrees,
            DIVE_MIN_LEG_TUCK_READABILITY_DEGREES / MIN_KEY_POSE_READABILITY_SCORE,
        )
        .min(readable_at_most_score(
            arm_spread_degrees,
            DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES,
        )),
        PlayerPoseIntent::AirBrake => {
            readable_pair_score(torso_pitch_degrees, 4.0, arm_spread_degrees, 160.0)
        }
        PlayerPoseIntent::Launching => readable_triple_score(
            torso_pitch_degrees,
            16.0,
            arm_spread_degrees,
            28.0,
            leg_tuck_degrees,
            18.0,
        ),
        PlayerPoseIntent::Falling => {
            readable_pair_score(torso_pitch_degrees, 58.0, arm_spread_degrees, 136.0)
        }
        PlayerPoseIntent::LandingAnticipation => readable_triple_score(
            torso_pitch_degrees,
            40.0,
            leg_tuck_degrees,
            48.0,
            landing_crouch_m,
            0.07,
        )
        .min(landing_foot_forward_m / LANDING_MIN_FOOT_FORWARD_READABILITY_M)
        .min(landing_foot_split_m / LANDING_MIN_FOOT_SPLIT_READABILITY_M)
        .clamp(0.0, 1.0),
        PlayerPoseIntent::LandingRecovery => readable_triple_score(
            torso_pitch_degrees,
            38.0,
            leg_tuck_degrees,
            32.0,
            landing_crouch_m,
            0.055,
        )
        .min(landing_foot_split_m / LANDING_MIN_FOOT_SPLIT_READABILITY_M)
        .clamp(0.0, 1.0),
        PlayerPoseIntent::Gliding => {
            readable_pair_score(torso_pitch_degrees, 16.0, arm_spread_degrees, 120.0)
        }
        _ => 1.0,
    }
}

fn torso_signed_lateral_lean_degrees(rotation: Quat) -> f32 {
    let local_up = rotation * Vec3::Y;
    -local_up.x.atan2(local_up.y).to_degrees()
}

fn torso_lateral_lean_degrees(rotation: Quat) -> f32 {
    torso_signed_lateral_lean_degrees(rotation).abs()
}

fn launch_overhead_arm_score(
    intent: PlayerPoseIntent,
    left_arm_rotation: Quat,
    right_arm_rotation: Quat,
) -> f32 {
    if intent != PlayerPoseIntent::Launching {
        return 0.0;
    }

    let left_raise = (left_arm_rotation * Vec3::NEG_Y).y;
    let right_raise = (right_arm_rotation * Vec3::NEG_Y).y;
    ((left_raise + right_raise) * 0.5).clamp(0.0, 1.0)
}

pub fn pose_readability_metrics_from_part_transforms(
    context: PlayerPoseContext,
    parts: PoseReadabilityPartTransforms,
) -> PoseReadabilityMetrics {
    let landing_pose = matches!(
        context.intent(),
        PlayerPoseIntent::LandingAnticipation | PlayerPoseIntent::LandingRecovery
    );
    let average_leg_rotation_radians = if landing_pose {
        (parts.left_leg_rotation.angle_between(Quat::IDENTITY)
            + parts.right_leg_rotation.angle_between(Quat::IDENTITY))
            * 0.5
    } else {
        0.0
    };
    let leg_rotation_split_m = if landing_pose {
        parts
            .left_leg_rotation
            .angle_between(parts.right_leg_rotation)
            * 0.42
    } else {
        0.0
    };
    let (average_leg_lift, average_forward_tuck) = if landing_pose {
        (
            ((parts.left_leg_translation.y + parts.right_leg_translation.y) * 0.5).max(0.0),
            ((parts.left_leg_translation.z + parts.right_leg_translation.z) * 0.5).max(0.0)
                + average_leg_rotation_radians * 0.26,
        )
    } else {
        (0.0, 0.0)
    };
    let landing_crouch_m = if landing_pose {
        average_leg_lift + average_forward_tuck * 0.08 + average_leg_rotation_radians * 0.045
    } else {
        0.0
    };
    let landing_foot_forward_m = if context.intent() == PlayerPoseIntent::LandingAnticipation {
        average_forward_tuck
    } else {
        0.0
    };
    let landing_foot_split_m = if landing_pose {
        (parts.left_leg_translation.z - parts.right_leg_translation.z).abs() + leg_rotation_split_m
    } else {
        0.0
    };

    let torso_pitch_degrees = parts
        .torso_rotation
        .angle_between(Quat::IDENTITY)
        .to_degrees();
    let arm_spread_degrees = parts
        .left_arm_rotation
        .angle_between(parts.right_arm_rotation)
        .to_degrees();
    let raw_leg_tuck_degrees = (parts
        .left_leg_rotation
        .angle_between(Quat::IDENTITY)
        .to_degrees()
        + parts
            .right_leg_rotation
            .angle_between(Quat::IDENTITY)
            .to_degrees())
        * 0.5;
    let leg_tuck_degrees = if context.intent() == PlayerPoseIntent::Diving {
        (torso_pitch_degrees - raw_leg_tuck_degrees * 1.1).max(0.0)
    } else {
        raw_leg_tuck_degrees
    };
    let grounded_stride_pose = matches!(
        context.intent(),
        PlayerPoseIntent::GroundedStride
            | PlayerPoseIntent::GroundedWalk
            | PlayerPoseIntent::GroundedRun
    );
    let grounded_stride_foot_travel_m = if grounded_stride_pose {
        (parts.left_leg_translation.z - parts.right_leg_translation.z).abs()
    } else {
        0.0
    };
    let grounded_stride_leg_opposition_degrees = if grounded_stride_pose {
        parts
            .left_leg_rotation
            .angle_between(parts.right_leg_rotation)
            .to_degrees()
    } else {
        0.0
    };

    PoseReadabilityMetrics {
        torso_pitch_degrees,
        arm_spread_degrees,
        leg_tuck_degrees,
        lateral_lean_degrees: torso_lateral_lean_degrees(parts.torso_rotation),
        signed_lateral_lean_degrees: torso_signed_lateral_lean_degrees(parts.torso_rotation),
        grounded_stride_foot_travel_m,
        grounded_stride_leg_opposition_degrees,
        landing_crouch_m,
        landing_foot_forward_m,
        landing_foot_split_m,
        landing_recovery_flip_degrees: if context.intent() == PlayerPoseIntent::LandingRecovery {
            torso_pitch_degrees
        } else {
            0.0
        },
        wing_airflow_strength: wing_airflow_strength(context.mode, context.velocity),
        scarf_stream_m: 0.0,
        scarf_lateral_sway_m: 0.0,
        scarf_tail_flex_degrees: 0.0,
        launch_overhead_arm_score: launch_overhead_arm_score(
            context.intent(),
            parts.left_arm_rotation,
            parts.right_arm_rotation,
        ),
        key_pose_readability_score: key_pose_readability_score(
            context.intent(),
            torso_pitch_degrees,
            arm_spread_degrees,
            leg_tuck_degrees,
            landing_crouch_m,
            landing_foot_forward_m,
            landing_foot_split_m,
        ),
    }
}

fn readable_pair_score(first: f32, first_target: f32, second: f32, second_target: f32) -> f32 {
    (first / first_target)
        .min(second / second_target)
        .clamp(0.0, 1.0)
}

fn readable_at_most_score(value: f32, max_target: f32) -> f32 {
    (max_target / value.max(f32::EPSILON)).clamp(0.0, 1.0)
}

fn readable_triple_score(
    first: f32,
    first_target: f32,
    second: f32,
    second_target: f32,
    third: f32,
    third_target: f32,
) -> f32 {
    readable_pair_score(first, first_target, second, second_target)
        .min(third / third_target)
        .clamp(0.0, 1.0)
}

pub fn pose_readability_metrics(context: PlayerPoseContext, phase: f32) -> PoseReadabilityMetrics {
    let hips = CharacterPart::new(CharacterPartRole::Hips, Vec3::ZERO, Quat::IDENTITY);
    let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
    let left_arm = CharacterPart::new(
        CharacterPartRole::Arm(Side::Left),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let right_arm = CharacterPart::new(
        CharacterPartRole::Arm(Side::Right),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let left_leg = CharacterPart::new(
        CharacterPartRole::Leg(Side::Left),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let right_leg = CharacterPart::new(
        CharacterPartRole::Leg(Side::Right),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let scarf_anchor = CharacterPart::new(
        CharacterPartRole::Scarf(ScarfSegment::Anchor),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let scarf_tail = CharacterPart::new(
        CharacterPartRole::Scarf(ScarfSegment::Trail),
        Vec3::ZERO,
        Quat::IDENTITY,
    );

    let hips_pose = part_pose_with_context(&hips, context, phase);
    let torso_pose = part_pose_with_context(&torso, context, phase);
    let left_arm_pose = part_pose_with_context(&left_arm, context, phase);
    let right_arm_pose = part_pose_with_context(&right_arm, context, phase);
    let left_leg_pose = part_pose_with_context(&left_leg, context, phase);
    let right_leg_pose = part_pose_with_context(&right_leg, context, phase);
    let scarf_anchor_pose = part_pose_with_context(&scarf_anchor, context, phase);
    let scarf_tail_pose = part_pose_with_context(&scarf_tail, context, phase);
    let mut metrics = pose_readability_metrics_from_part_transforms(
        context,
        PoseReadabilityPartTransforms {
            torso_rotation: hips_pose.rotation * torso_pose.rotation,
            left_arm_rotation: left_arm_pose.rotation,
            right_arm_rotation: right_arm_pose.rotation,
            left_leg_rotation: left_leg_pose.rotation,
            right_leg_rotation: right_leg_pose.rotation,
            left_leg_translation: left_leg_pose.translation,
            right_leg_translation: right_leg_pose.translation,
        },
    );
    metrics.signed_lateral_lean_degrees = pose_lateral_lean_radians(context).to_degrees();
    metrics.lateral_lean_degrees = metrics.signed_lateral_lean_degrees.abs();
    metrics.scarf_stream_m = scarf_tail_pose.translation.z.max(0.0);
    metrics.scarf_lateral_sway_m = scarf_tail_pose.translation.x.abs();
    metrics.scarf_tail_flex_degrees = scarf_tail_pose
        .rotation
        .angle_between(scarf_anchor_pose.rotation)
        .to_degrees();
    metrics
}

pub fn part_pose(part: &CharacterPart, mode: FlightMode, velocity: Vec3, phase: f32) -> PartPose {
    part_pose_with_context(
        part,
        PlayerPoseContext::new(mode, velocity, FlightInput::default(), f32::INFINITY),
        phase,
    )
}

pub fn part_pose_with_context(
    part: &CharacterPart,
    context: PlayerPoseContext,
    phase: f32,
) -> PartPose {
    let cycle = phase.sin();
    let intent = context.intent();
    let horizontal_speed = Vec2::new(context.velocity.x, context.velocity.z).length();
    let gait_weight = (horizontal_speed / 16.0).clamp(0.0, 1.0);
    let walk_weight = ((horizontal_speed - 1.0) / 7.0).clamp(0.0, 1.0);
    let run_weight = ((horizontal_speed - 6.0) / 8.0).clamp(0.0, 1.0);
    let turn_weight = pose_turn_weight(context);
    let airborne_pose = airborne_pose_intent(intent);
    let roll = pose_lateral_lean_radians(context);
    let turn_reach = match intent {
        PlayerPoseIntent::AirTurn => turn_weight.abs(),
        PlayerPoseIntent::AirBrake => turn_weight.abs() * 0.72,
        _ => 0.0,
    };
    let air_turn_yaw = if intent == PlayerPoseIntent::AirTurn {
        turn_weight * 0.24
    } else if intent == PlayerPoseIntent::AirBrake {
        turn_weight * 0.16
    } else if airborne_pose {
        turn_weight * 0.08
    } else {
        0.0
    };
    let dive_pressure = dive_pose_pressure(context, intent);
    let dive_extension = if intent == PlayerPoseIntent::Diving {
        ((dive_pressure - 0.45) / 0.55).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let dive_flutter = dive_airflow_flutter(phase, dive_pressure);
    let brake_pressure = air_brake_pose_pressure(context, intent);
    let rearward_brake_pressure = rearward_air_brake_pressure(context, intent);
    let vertical_pitch = (-context.velocity.y * 0.004).clamp(-0.1, 0.1);
    let mut translation = part.base_translation;
    let mut rotation = part.base_rotation;
    let mut visibility = PartVisibility::Inherited;
    let landing_strength = landing_anticipation_strength(context, intent);
    let landing_flip = landing_flip_strength(context, intent);
    let recovery_strength = landing_recovery_strength(context, intent);
    let landing_lead_sign = landing_lead_side_sign(context);
    let breath = breath_cycle(phase);

    match part.role {
        CharacterPartRole::Hips => {
            let pitch = match intent {
                PlayerPoseIntent::GroundedIdle => breath * 0.006,
                PlayerPoseIntent::GroundedStride => -0.012 * gait_weight,
                PlayerPoseIntent::GroundedWalk => -0.010 * walk_weight,
                PlayerPoseIntent::GroundedRun => -0.025 - run_weight * 0.018,
                PlayerPoseIntent::Launching => -0.22 + vertical_pitch * 0.12,
                PlayerPoseIntent::Falling => -1.18 + vertical_pitch * 0.10,
                PlayerPoseIntent::Gliding => -0.04 + vertical_pitch * 0.12,
                PlayerPoseIntent::AirTurn => -0.06 + vertical_pitch * 0.10,
                PlayerPoseIntent::Diving => {
                    -2.45 - dive_pressure * 0.18 + vertical_pitch * 0.03 + dive_flutter * 0.024
                }
                PlayerPoseIntent::AirBrake => 0.02 + rearward_brake_pressure * 0.06,
                PlayerPoseIntent::LandingAnticipation => {
                    0.38 + landing_strength * 0.14 + landing_flip * 0.34
                }
                PlayerPoseIntent::LandingRecovery => 0.32 + recovery_strength * 0.34,
            };
            translation.y += match intent {
                PlayerPoseIntent::GroundedWalk => cycle.abs() * 0.006,
                PlayerPoseIntent::GroundedRun => cycle.abs() * (0.010 + run_weight * 0.010),
                PlayerPoseIntent::Launching => 0.030,
                PlayerPoseIntent::Falling => dive_flutter * 0.003,
                PlayerPoseIntent::Diving => dive_pressure * 0.020 + dive_flutter * 0.006,
                PlayerPoseIntent::LandingAnticipation => 0.040 + landing_flip * 0.040,
                PlayerPoseIntent::LandingRecovery => -0.030 - recovery_strength * 0.040,
                _ => 0.0,
            };
            translation.z += match intent {
                PlayerPoseIntent::Launching => 0.035,
                PlayerPoseIntent::Diving => dive_pressure * 0.020,
                PlayerPoseIntent::LandingAnticipation => 0.070 + landing_flip * 0.080,
                PlayerPoseIntent::LandingRecovery => 0.045 + recovery_strength * 0.065,
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_x(pitch)
                * Quat::from_rotation_y(air_turn_yaw * 0.35)
                * Quat::from_rotation_z(roll * 0.45);
        }
        CharacterPartRole::Torso => {
            let pitch = match intent {
                PlayerPoseIntent::GroundedIdle => 0.018 + breath * 0.018,
                PlayerPoseIntent::GroundedStride => -0.04 * gait_weight,
                PlayerPoseIntent::GroundedWalk => -0.035 * walk_weight,
                PlayerPoseIntent::GroundedRun => -0.08 - run_weight * 0.07,
                PlayerPoseIntent::Falling => -0.12 + vertical_pitch * 0.04,
                PlayerPoseIntent::Gliding => -0.24 + vertical_pitch * 0.35,
                PlayerPoseIntent::AirTurn => -0.24 + vertical_pitch * 0.32,
                PlayerPoseIntent::Diving => {
                    -0.36 - dive_pressure * 0.14 + vertical_pitch * 0.03 + dive_flutter * 0.035
                }
                PlayerPoseIntent::AirBrake => {
                    0.08 + brake_pressure * 0.07
                        + rearward_brake_pressure * 0.12
                        + turn_weight.abs() * 0.05
                        + vertical_pitch * 0.35
                }
                PlayerPoseIntent::LandingAnticipation => {
                    0.28 + landing_strength * 0.10 + landing_flip * 0.22
                }
                PlayerPoseIntent::LandingRecovery => 0.28 + recovery_strength * 0.28,
                PlayerPoseIntent::Launching => -0.20 + vertical_pitch * 0.24,
            };
            translation.y += match intent {
                PlayerPoseIntent::GroundedIdle => 0.018 + breath * 0.012,
                PlayerPoseIntent::GroundedWalk => cycle.abs() * (0.018 + walk_weight * 0.018),
                PlayerPoseIntent::GroundedRun => cycle.abs() * (0.032 + run_weight * 0.030),
                PlayerPoseIntent::Gliding | PlayerPoseIntent::AirTurn => {
                    cycle.abs() * (0.010 + gait_weight * 0.010) + breath * 0.004
                }
                _ => cycle.abs() * (0.014 + gait_weight * 0.018),
            };
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.y += 0.13 + landing_strength * 0.07 + landing_flip * 0.08;
                translation.z += 0.18 + landing_strength * 0.10 + landing_flip * 0.16;
            } else if intent == PlayerPoseIntent::LandingRecovery {
                translation.y -= 0.08 + recovery_strength * 0.10;
                translation.z += 0.10 + recovery_strength * 0.15;
            }
            translation.x += turn_weight * turn_reach * 0.035;
            if intent == PlayerPoseIntent::Diving {
                translation.y += dive_pressure * 0.035;
                translation.z += dive_pressure * 0.045;
                translation.y += dive_flutter * 0.010;
                translation.z += dive_flutter.abs() * 0.012;
            } else if intent == PlayerPoseIntent::AirBrake {
                translation.y += brake_pressure * 0.025;
                translation.z += rearward_brake_pressure * 0.035;
            }
            let landing_counter_roll = match intent {
                PlayerPoseIntent::LandingAnticipation => {
                    -landing_lead_sign * (0.045 + landing_flip * 0.040)
                }
                PlayerPoseIntent::LandingRecovery => {
                    -landing_lead_sign * (0.030 + recovery_strength * 0.025)
                }
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_x(pitch)
                * Quat::from_rotation_y(air_turn_yaw)
                * Quat::from_rotation_z(roll + landing_counter_roll);
        }
        CharacterPartRole::Head => {
            translation.y += cycle.abs() * (0.01 + gait_weight * 0.006);
            translation.x += turn_weight * turn_reach * 0.025;
            let pitch = match intent {
                PlayerPoseIntent::AirTurn => -0.10,
                PlayerPoseIntent::Falling => 0.20 + vertical_pitch * 0.35,
                PlayerPoseIntent::Diving => -0.12 + dive_pressure * 0.06 - dive_flutter * 0.035,
                PlayerPoseIntent::AirBrake => -0.14 - rearward_brake_pressure * 0.08,
                PlayerPoseIntent::LandingAnticipation => -0.42 - landing_flip * 0.24,
                PlayerPoseIntent::LandingRecovery => -0.26 - recovery_strength * 0.18,
                PlayerPoseIntent::GroundedIdle => -0.05 + breath * 0.018,
                PlayerPoseIntent::GroundedWalk => -0.04,
                PlayerPoseIntent::GroundedRun => -0.02 + run_weight * 0.04,
                PlayerPoseIntent::Launching => 0.10,
                _ => -0.05,
            };
            rotation *= Quat::from_rotation_x(pitch)
                * Quat::from_rotation_y(air_turn_yaw * 1.45)
                * Quat::from_rotation_z(roll * 0.35);
        }
        CharacterPartRole::Arm(side) => {
            let sign = side.sign();
            let same_side_turn = (sign * turn_weight).max(0.0);
            let opposite_side_turn = (-sign * turn_weight).max(0.0);
            let gait = -side_cycle(phase, side);
            let airflow = airflow_micro_cycle(phase, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let spread = match intent {
                PlayerPoseIntent::GroundedIdle => 0.08,
                PlayerPoseIntent::GroundedStride => 0.08 + gait.abs() * 0.06 * gait_weight,
                PlayerPoseIntent::GroundedWalk => 0.10 + gait.abs() * 0.08 * walk_weight,
                PlayerPoseIntent::GroundedRun => 0.16 + gait.abs() * 0.14,
                PlayerPoseIntent::Falling => 1.43 + airflow * 0.018,
                PlayerPoseIntent::Gliding => 1.22 + airflow * 0.035,
                PlayerPoseIntent::AirTurn => 1.18 + airflow * 0.040,
                PlayerPoseIntent::Diving => 0.075 + dive_extension * 0.025 + airflow * 0.014,
                PlayerPoseIntent::AirBrake => 1.52 + brake_pressure * 0.045 + same_side_turn * 0.08,
                PlayerPoseIntent::LandingAnticipation => {
                    1.06 + landing_strength * 0.16 + landing_flip * 0.28
                }
                PlayerPoseIntent::LandingRecovery => 0.88 + recovery_strength * 0.34,
                PlayerPoseIntent::Launching => 2.52 + run_weight * 0.02,
            };
            let sweep = match intent {
                PlayerPoseIntent::GroundedIdle => cycle * 0.025,
                PlayerPoseIntent::GroundedStride => gait * 0.48 * gait_weight,
                PlayerPoseIntent::GroundedWalk => gait * 0.34 * walk_weight,
                PlayerPoseIntent::GroundedRun => gait * (0.58 + run_weight * 0.18),
                PlayerPoseIntent::Gliding => -0.58 + airflow * 0.035,
                PlayerPoseIntent::AirTurn => -0.46 + turn_weight.abs() * 0.10,
                PlayerPoseIntent::Diving => 0.08 + dive_extension * 0.06 + airflow * 0.012,
                PlayerPoseIntent::AirBrake => {
                    0.36 + brake_pressure * 0.06
                        + rearward_brake_pressure * 0.16
                        + same_side_turn * 0.12
                }
                PlayerPoseIntent::LandingAnticipation => {
                    1.22 + landing_strength * 0.24 + landing_flip * 0.32
                }
                PlayerPoseIntent::LandingRecovery => 0.70 + recovery_strength * 0.40,
                PlayerPoseIntent::Launching => -0.72,
                PlayerPoseIntent::Falling => -0.50 + airflow * 0.018,
            };
            translation.z += gait * 0.08 * gait_weight;
            translation.y += match context.mode {
                _ if intent == PlayerPoseIntent::Launching => -0.04,
                _ if intent == PlayerPoseIntent::Diving => 0.12 + dive_pressure * 0.08,
                _ if intent == PlayerPoseIntent::AirBrake => {
                    0.08 + brake_pressure * 0.02
                        + rearward_brake_pressure * 0.035
                        + same_side_turn * 0.045
                }
                _ if intent == PlayerPoseIntent::LandingAnticipation => -0.15 - landing_flip * 0.05,
                _ if intent == PlayerPoseIntent::LandingRecovery => -0.12,
                FlightMode::Gliding => 0.04,
                FlightMode::Airborne => -0.02,
                _ => 0.0,
            };
            if airborne_pose {
                let airflow_z = if intent == PlayerPoseIntent::AirTurn {
                    airflow * 0.004
                } else {
                    airflow * 0.012
                };
                translation.x += sign * turn_reach * 0.060;
                translation.y +=
                    sign * turn_weight * 0.045 + same_side_turn * 0.035 + airflow * 0.012;
                translation.z += turn_weight.abs() * 0.025 - same_side_turn * 0.05
                    + opposite_side_turn * 0.025
                    + rearward_brake_pressure * 0.045
                    + airflow_z;
            }
            if intent == PlayerPoseIntent::Diving {
                translation.x += sign * (0.040 + dive_extension * 0.020);
                translation.x += sign * dive_side_flutter * 0.010;
                translation.y += dive_side_flutter * 0.014;
                translation.z += dive_extension * 0.070;
                translation.z += dive_side_flutter.abs() * 0.010;
            }
            rotation *= Quat::from_rotation_z(sign * spread)
                * Quat::from_rotation_x(sweep - same_side_turn * 0.16 + opposite_side_turn * 0.06)
                * Quat::from_rotation_y(
                    sign * turn_weight * 0.12
                        + same_side_turn * 0.10
                        + sign * dive_side_flutter * 0.05,
                );
        }
        CharacterPartRole::Forearm(side) => {
            let sign = side.sign();
            let same_side_turn = (sign * turn_weight).max(0.0);
            let opposite_side_turn = (-sign * turn_weight).max(0.0);
            let gait = -side_cycle(phase, side);
            let airflow = airflow_micro_cycle(phase + 0.45, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let elbow_pitch = match intent {
                PlayerPoseIntent::GroundedIdle => -0.08 + breath * 0.020,
                PlayerPoseIntent::GroundedStride => gait * 0.18 * gait_weight,
                PlayerPoseIntent::GroundedWalk => gait * 0.20 * walk_weight,
                PlayerPoseIntent::GroundedRun => gait * (0.28 + run_weight * 0.10),
                PlayerPoseIntent::Launching => -0.26,
                PlayerPoseIntent::Falling => -0.28 + airflow * 0.035,
                PlayerPoseIntent::Gliding => -0.18 + airflow * 0.035,
                PlayerPoseIntent::AirTurn => {
                    -0.22 - same_side_turn * 0.18 + opposite_side_turn * 0.06 + airflow * 0.040
                }
                PlayerPoseIntent::Diving => {
                    0.12 + dive_extension * 0.05 + dive_side_flutter * 0.035
                }
                PlayerPoseIntent::AirBrake => {
                    -0.54 - brake_pressure * 0.16 - rearward_brake_pressure * 0.18
                        + same_side_turn * 0.06
                }
                PlayerPoseIntent::LandingAnticipation => {
                    -0.76 - landing_strength * 0.14 - landing_flip * 0.20
                }
                PlayerPoseIntent::LandingRecovery => -0.54 - recovery_strength * 0.18,
            };
            let elbow_roll = match intent {
                PlayerPoseIntent::Falling
                | PlayerPoseIntent::Gliding
                | PlayerPoseIntent::AirTurn => sign * (0.10 + turn_weight.abs() * 0.04),
                PlayerPoseIntent::Diving => sign * (0.025 + dive_side_flutter * 0.025),
                PlayerPoseIntent::AirBrake => sign * (-0.12 - brake_pressure * 0.04),
                PlayerPoseIntent::LandingAnticipation => sign * 0.12,
                _ => sign * 0.04,
            };
            translation.x += sign * turn_reach * 0.012;
            translation.y += airflow * 0.006;
            translation.z += match intent {
                PlayerPoseIntent::Diving => {
                    dive_extension * 0.020 + dive_side_flutter.abs() * 0.006
                }
                PlayerPoseIntent::AirBrake => rearward_brake_pressure * 0.018,
                _ => airflow * 0.006,
            };
            rotation *= Quat::from_rotation_x(elbow_pitch)
                * Quat::from_rotation_y(
                    sign * turn_weight * 0.050 + sign * dive_side_flutter * 0.040,
                )
                * Quat::from_rotation_z(elbow_roll);
        }
        CharacterPartRole::Hand(side) => {
            let sign = side.sign();
            let same_side_turn = (sign * turn_weight).max(0.0);
            let airflow = airflow_micro_cycle(phase + 1.1, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let wrist_pitch = match intent {
                PlayerPoseIntent::GroundedIdle => 0.04 + breath * 0.018,
                PlayerPoseIntent::GroundedStride => -side_cycle(phase, side) * 0.10 * gait_weight,
                PlayerPoseIntent::GroundedWalk => -side_cycle(phase, side) * 0.12 * walk_weight,
                PlayerPoseIntent::GroundedRun => -side_cycle(phase, side) * 0.18,
                PlayerPoseIntent::Launching => 0.12,
                PlayerPoseIntent::Falling => -0.10 + airflow * 0.045,
                PlayerPoseIntent::Gliding => 0.06 + airflow * 0.050,
                PlayerPoseIntent::AirTurn => 0.02 + same_side_turn * 0.10 + airflow * 0.045,
                PlayerPoseIntent::Diving => {
                    0.04 + dive_extension * 0.04 + dive_side_flutter * 0.040
                }
                PlayerPoseIntent::AirBrake => {
                    -0.28 - brake_pressure * 0.10 - rearward_brake_pressure * 0.12
                }
                PlayerPoseIntent::LandingAnticipation => -0.34 - landing_strength * 0.10,
                PlayerPoseIntent::LandingRecovery => -0.22 - recovery_strength * 0.10,
            };
            translation.x += sign * turn_reach * 0.008 + sign * dive_side_flutter * 0.004;
            translation.y += airflow * 0.006;
            translation.z += match intent {
                PlayerPoseIntent::Diving => 0.012 + dive_side_flutter.abs() * 0.006,
                PlayerPoseIntent::AirBrake => rearward_brake_pressure * 0.012,
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_x(wrist_pitch)
                * Quat::from_rotation_y(sign * (0.08 + turn_weight * 0.035))
                * Quat::from_rotation_z(sign * (0.08 + airflow * 0.030));
        }
        CharacterPartRole::Leg(side) => {
            let sign = side.sign();
            let same_side_turn = (sign * turn_weight).max(0.0);
            let opposite_side_turn = (-sign * turn_weight).max(0.0);
            let landing_lead = landing_lead_weight(context, side);
            let landing_trail = 1.0 - landing_lead;
            let gait = side_cycle(phase, side);
            let airflow = airflow_micro_cycle(phase + 0.9, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let spread = match intent {
                PlayerPoseIntent::GroundedIdle => 0.04,
                PlayerPoseIntent::GroundedStride => 0.04 + gait.abs() * 0.05 * gait_weight,
                PlayerPoseIntent::GroundedWalk => 0.06 + gait.abs() * 0.055 * walk_weight,
                PlayerPoseIntent::GroundedRun => 0.08 + gait.abs() * 0.08,
                PlayerPoseIntent::Falling => 0.28 + airflow.abs() * 0.012,
                PlayerPoseIntent::Gliding => 0.20 + airflow.abs() * 0.025,
                PlayerPoseIntent::AirTurn => 0.24 + airflow.abs() * 0.030,
                PlayerPoseIntent::Diving => 0.10 + dive_pressure * 0.040 + airflow.abs() * 0.008,
                PlayerPoseIntent::AirBrake => 0.30 + brake_pressure * 0.04 + same_side_turn * 0.08,
                PlayerPoseIntent::LandingAnticipation => {
                    0.58 + landing_strength * 0.14 + landing_flip * 0.24
                }
                PlayerPoseIntent::LandingRecovery => 0.40 + recovery_strength * 0.20,
                PlayerPoseIntent::Launching => 0.18,
            };
            let trail = match intent {
                PlayerPoseIntent::GroundedIdle => 0.02,
                PlayerPoseIntent::GroundedStride => gait * 0.52 * gait_weight,
                PlayerPoseIntent::GroundedWalk => gait * 0.46 * walk_weight,
                PlayerPoseIntent::GroundedRun => gait * (0.68 + run_weight * 0.22),
                PlayerPoseIntent::Gliding => 0.50 + cycle * 0.04 + airflow * 0.025,
                PlayerPoseIntent::AirTurn => 0.54 + cycle * 0.04 + airflow * 0.025,
                PlayerPoseIntent::Diving => {
                    0.04 + dive_pressure * 0.06 + cycle * 0.004 + airflow * 0.004
                }
                PlayerPoseIntent::AirBrake => {
                    -0.30
                        - brake_pressure * 0.04
                        - rearward_brake_pressure * 0.22
                        - same_side_turn * 0.16
                }
                PlayerPoseIntent::LandingAnticipation => {
                    -1.14 - landing_strength * 0.28 - landing_flip * 0.52 - landing_lead * 0.24
                        + landing_trail * 0.10
                }
                PlayerPoseIntent::LandingRecovery => {
                    -0.64 - recovery_strength * 0.42 - landing_lead * 0.10 + landing_trail * 0.08
                }
                PlayerPoseIntent::Falling => 0.58 + vertical_pitch * 0.5,
                PlayerPoseIntent::Launching => -0.44,
            };
            let locomotion_gait_weight = if matches!(
                intent,
                PlayerPoseIntent::LandingAnticipation | PlayerPoseIntent::LandingRecovery
            ) {
                0.0
            } else {
                gait_weight
            };
            translation.z += gait * 0.18 * locomotion_gait_weight;
            translation.y += gait.max(0.0) * 0.045 * locomotion_gait_weight;
            if intent == PlayerPoseIntent::GroundedRun {
                translation.y += gait.max(0.0) * 0.045 * (0.5 + run_weight);
                translation.z += gait * 0.045 * run_weight;
            }
            if intent == PlayerPoseIntent::Launching {
                translation.z += 0.08;
                translation.y += 0.05;
            }
            if intent == PlayerPoseIntent::Diving {
                translation.z += dive_pressure * 0.035;
                translation.x += sign * dive_side_flutter * 0.010;
                translation.y -= dive_pressure * 0.015;
                translation.y += dive_side_flutter * 0.006;
                translation.z += dive_side_flutter * 0.004;
            } else if intent == PlayerPoseIntent::AirBrake {
                translation.z += rearward_brake_pressure * 0.12;
                translation.y += rearward_brake_pressure * 0.035;
            }
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.z += 0.42
                    + landing_strength * 0.16
                    + landing_flip * 0.34
                    + landing_lead * (0.20 + landing_strength * 0.06 + landing_flip * 0.12)
                    - landing_trail * (0.08 + landing_flip * 0.04);
                translation.y += 0.11
                    + landing_strength * 0.07
                    + landing_flip * 0.08
                    + landing_lead * (0.04 + landing_strength * 0.02)
                    - landing_trail * (0.025 + landing_flip * 0.015);
            } else if intent == PlayerPoseIntent::LandingRecovery {
                translation.z += 0.17
                    + recovery_strength * 0.17
                    + landing_lead * (0.12 + recovery_strength * 0.05)
                    - landing_trail * (0.04 + recovery_strength * 0.02);
                translation.y += 0.06
                    + recovery_strength * 0.08
                    + landing_lead * 0.015
                    + landing_trail * (0.035 + recovery_strength * 0.02);
            }
            if airborne_pose {
                translation.x += sign * turn_weight * 0.035 + sign * turn_reach * 0.025;
                translation.y += turn_weight.abs() * 0.018 + same_side_turn * 0.035;
                translation.z += same_side_turn * 0.08 - opposite_side_turn * 0.035;
            }
            rotation *= Quat::from_rotation_z(sign * spread)
                * Quat::from_rotation_x(trail - same_side_turn * 0.18 + opposite_side_turn * 0.05)
                * Quat::from_rotation_y(
                    sign * turn_weight * 0.08
                        + same_side_turn * 0.08
                        + sign * dive_side_flutter * 0.065,
                );
        }
        CharacterPartRole::LowerLeg(side) => {
            let sign = side.sign();
            let same_side_turn = (sign * turn_weight).max(0.0);
            let gait = side_cycle(phase, side);
            let airflow = airflow_micro_cycle(phase + 1.4, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let knee_pitch = match intent {
                PlayerPoseIntent::GroundedIdle => 0.08 + breath * 0.014,
                PlayerPoseIntent::GroundedStride => -gait * 0.30 * gait_weight,
                PlayerPoseIntent::GroundedWalk => -gait * 0.34 * walk_weight,
                PlayerPoseIntent::GroundedRun => -gait * (0.48 + run_weight * 0.18),
                PlayerPoseIntent::Launching => 0.24,
                PlayerPoseIntent::Falling => 0.24 + airflow * 0.040,
                PlayerPoseIntent::Gliding => 0.18 + airflow * 0.030,
                PlayerPoseIntent::AirTurn => 0.22 + same_side_turn * 0.10 + airflow * 0.035,
                PlayerPoseIntent::Diving => {
                    0.055 + dive_pressure * 0.025 + dive_side_flutter * 0.015
                }
                PlayerPoseIntent::AirBrake => {
                    -0.36 - brake_pressure * 0.10 - rearward_brake_pressure * 0.18
                }
                PlayerPoseIntent::LandingAnticipation => {
                    0.62 + landing_strength * 0.22 + landing_flip * 0.26
                }
                PlayerPoseIntent::LandingRecovery => 0.44 + recovery_strength * 0.26,
            };
            translation.x += sign * turn_reach * 0.006 + sign * dive_side_flutter * 0.004;
            translation.y += airflow * 0.005;
            translation.z += match intent {
                PlayerPoseIntent::Diving => 0.014 + dive_side_flutter.abs() * 0.004,
                PlayerPoseIntent::LandingAnticipation => 0.040 + landing_flip * 0.016,
                _ => 0.0,
            };
            let lower_leg_roll = match intent {
                PlayerPoseIntent::LandingAnticipation => {
                    0.36 + landing_strength * 0.14 + landing_flip * 0.10
                }
                PlayerPoseIntent::LandingRecovery => 0.22 + recovery_strength * 0.10,
                PlayerPoseIntent::Diving => 0.11 + dive_pressure * 0.030,
                _ => 0.035 + same_side_turn * 0.050,
            };
            rotation *= Quat::from_rotation_x(knee_pitch)
                * Quat::from_rotation_y(sign * (turn_weight * 0.035 + dive_side_flutter * 0.040))
                * Quat::from_rotation_z(sign * lower_leg_roll);
        }
        CharacterPartRole::Foot(side) => {
            let sign = side.sign();
            let gait = side_cycle(phase, side);
            let airflow = airflow_micro_cycle(phase + 1.9, side);
            let dive_side_flutter = if intent == PlayerPoseIntent::Diving {
                airflow * dive_pressure
            } else {
                0.0
            };
            let ankle_pitch = match intent {
                PlayerPoseIntent::GroundedIdle => -0.03 + breath * 0.010,
                PlayerPoseIntent::GroundedStride => -0.10 + gait * 0.20 * gait_weight,
                PlayerPoseIntent::GroundedWalk => -0.08 + gait * 0.22 * walk_weight,
                PlayerPoseIntent::GroundedRun => -0.12 + gait * (0.30 + run_weight * 0.08),
                PlayerPoseIntent::Launching => -0.18,
                PlayerPoseIntent::Falling => -0.08 + airflow * 0.030,
                PlayerPoseIntent::Gliding => -0.04 + airflow * 0.030,
                PlayerPoseIntent::AirTurn => -0.06 + airflow * 0.035,
                PlayerPoseIntent::Diving => {
                    0.02 + dive_extension * 0.025 + dive_side_flutter * 0.015
                }
                PlayerPoseIntent::AirBrake => 0.18 + rearward_brake_pressure * 0.18,
                PlayerPoseIntent::LandingAnticipation => {
                    -0.34 - landing_strength * 0.12 - landing_flip * 0.18
                }
                PlayerPoseIntent::LandingRecovery => -0.18 - recovery_strength * 0.16,
            };
            translation.x += sign * turn_reach * 0.004;
            translation.z += match intent {
                PlayerPoseIntent::Diving => 0.002,
                PlayerPoseIntent::LandingAnticipation => 0.018 + landing_flip * 0.016,
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_x(ankle_pitch)
                * Quat::from_rotation_y(sign * (0.030 + turn_weight * 0.025))
                * Quat::from_rotation_z(sign * airflow * 0.030);
        }
        CharacterPartRole::Wing(side) => {
            visibility = if context.mode == FlightMode::Gliding {
                PartVisibility::Visible
            } else {
                PartVisibility::Hidden
            };

            let sign = side.sign();
            let bank =
                (context.velocity.x * 0.008 + pose_turn_weight(context) * 0.18).clamp(-0.26, 0.26);
            let airflow = wing_airflow_strength(context.mode, context.velocity);
            let flutter = ((phase * 2.4) + sign * 0.6).sin() * (0.018 + airflow * 0.038)
                + breath * airflow * 0.010;
            let air_brake_cup = if intent == PlayerPoseIntent::AirBrake {
                0.12 + brake_pressure * 0.04 + rearward_brake_pressure * 0.04
            } else {
                0.0
            };
            let dive_wing_sweep = if intent == PlayerPoseIntent::Diving {
                dive_pressure
            } else {
                0.0
            };
            translation.y +=
                flutter * 0.5 + airflow * 0.050 + air_brake_cup * 0.2 - dive_wing_sweep * 0.035;
            translation.z += airflow * 0.06 - air_brake_cup * 0.12 + dive_wing_sweep * 0.16;
            rotation *= Quat::from_rotation_z(
                sign * (bank + airflow * 0.05 + air_brake_cup - dive_wing_sweep * 0.035),
            ) * Quat::from_rotation_y(
                sign * (airflow * 0.08 + air_brake_cup * 0.25 + dive_wing_sweep * 0.12),
            ) * Quat::from_rotation_x(
                flutter - airflow * 0.09 + air_brake_cup * 0.55 - dive_wing_sweep * 0.22,
            );
        }
        CharacterPartRole::Scarf(segment) => {
            let segment_weight = match segment {
                ScarfSegment::Anchor => 0.34,
                ScarfSegment::Trail => 1.0,
            };
            let speed_pressure = (horizontal_speed / 38.0).clamp(0.0, 1.0);
            let sink_pressure = (-context.velocity.y / 34.0).clamp(0.0, 1.0);
            let stream_pressure = (speed_pressure + sink_pressure * 0.45).clamp(0.0, 1.0);
            let lateral_sway =
                (context.wind_lateral_load * 0.20 + turn_weight * 0.16).clamp(-0.36, 0.36);
            let flutter_phase = phase * (2.2 + segment_weight * 0.8) + segment_weight * 1.7;
            let flutter = flutter_phase.sin() * (0.012 + stream_pressure * 0.034)
                + breath * (0.004 + stream_pressure * 0.008);
            let trailing_stream = match intent {
                PlayerPoseIntent::GroundedIdle => 0.03 + breath.abs() * 0.012,
                PlayerPoseIntent::GroundedStride
                | PlayerPoseIntent::GroundedWalk
                | PlayerPoseIntent::GroundedRun => 0.04 + gait_weight * 0.045 + cycle.abs() * 0.018,
                PlayerPoseIntent::Launching => 0.12 + stream_pressure * 0.08,
                PlayerPoseIntent::Falling => 0.16 + stream_pressure * 0.15,
                PlayerPoseIntent::Gliding => 0.20 + stream_pressure * 0.22,
                PlayerPoseIntent::AirTurn => {
                    0.22 + stream_pressure * 0.22 + turn_weight.abs() * 0.05
                }
                PlayerPoseIntent::Diving => 0.34 + dive_pressure * 0.28 + stream_pressure * 0.18,
                PlayerPoseIntent::AirBrake => {
                    0.10 + brake_pressure * 0.06 + rearward_brake_pressure * 0.12
                }
                PlayerPoseIntent::LandingAnticipation => {
                    0.12 + landing_strength * 0.05 + landing_flip * 0.04
                }
                PlayerPoseIntent::LandingRecovery => 0.08 + recovery_strength * 0.08,
            };

            translation.x += (lateral_sway * 0.20 + flutter * 0.34) * segment_weight;
            translation.y += (flutter * 0.55 - stream_pressure * 0.020) * segment_weight;
            translation.z += trailing_stream * segment_weight;
            rotation *= Quat::from_rotation_x(
                -trailing_stream * segment_weight * (0.70 + segment_weight * 0.22) + flutter * 1.7,
            ) * Quat::from_rotation_y(lateral_sway * (0.80 + segment_weight * 0.35))
                * Quat::from_rotation_z(-lateral_sway * 0.42 * segment_weight);
        }
    }

    if let Some(limit_m) = connected_limb_translation_limit_m(part.role, intent) {
        let offset = translation - part.base_translation;
        if offset.length_squared() > limit_m * limit_m {
            translation = part.base_translation + offset.normalize_or_zero() * limit_m;
        }
    }

    PartPose {
        translation,
        rotation,
        visibility,
    }
}

fn connected_limb_translation_limit_m(
    role: CharacterPartRole,
    intent: PlayerPoseIntent,
) -> Option<f32> {
    if !matches!(
        role,
        CharacterPartRole::Head
            | CharacterPartRole::Arm(_)
            | CharacterPartRole::Forearm(_)
            | CharacterPartRole::Hand(_)
            | CharacterPartRole::Leg(_)
            | CharacterPartRole::LowerLeg(_)
            | CharacterPartRole::Foot(_)
    ) {
        return None;
    }

    if matches!(
        intent,
        PlayerPoseIntent::Launching
            | PlayerPoseIntent::Falling
            | PlayerPoseIntent::Gliding
            | PlayerPoseIntent::AirTurn
            | PlayerPoseIntent::Diving
            | PlayerPoseIntent::AirBrake
            | PlayerPoseIntent::LandingAnticipation
            | PlayerPoseIntent::LandingRecovery
    ) {
        Some(CONNECTED_LIMB_MAX_TRANSLATION_M)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_advances_by_delta_instead_of_recomputing_from_elapsed_time() {
        let first = advance_phase(0.0, 10.0, 1.0 / 60.0);
        let second = advance_phase(first, 60.0, 1.0 / 60.0);

        assert!(second > first);
        assert!(second - first < 0.2);
    }

    #[test]
    fn landing_anticipation_pose_blend_is_immediate_enough_for_first_contact_readability() {
        let frame_dt = 1.0 / 60.0;
        let landing_blend = pose_blend_for_intent(PlayerPoseIntent::LandingAnticipation, frame_dt);
        let dive_blend = pose_blend_for_intent(PlayerPoseIntent::Diving, frame_dt);

        assert!(landing_blend > 0.6);
        assert!(dive_blend > 0.20);
        assert!(landing_blend > dive_blend * 2.6);
    }

    #[test]
    fn wing_visibility_tracks_glide_mode() {
        let wing = CharacterPart::new(
            CharacterPartRole::Wing(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        assert_eq!(
            part_pose(&wing, FlightMode::Airborne, Vec3::ZERO, 0.0).visibility,
            PartVisibility::Hidden
        );
        assert_eq!(
            part_pose(&wing, FlightMode::Gliding, Vec3::ZERO, 0.0).visibility,
            PartVisibility::Visible
        );
    }

    #[test]
    fn wing_airflow_strength_requires_fast_gliding_motion() {
        let fast_glide = wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, -18.0, -55.0));
        let slow_glide = wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, 0.0, -8.0));
        let fast_ground = wing_airflow_strength(FlightMode::Grounded, Vec3::new(0.0, -18.0, -55.0));

        assert!(fast_glide > 0.9);
        assert_eq!(slow_glide, 0.0);
        assert_eq!(fast_ground, 0.0);
    }

    #[test]
    fn fast_gliding_wings_flex_under_airflow() {
        let wing = CharacterPart::new(
            CharacterPartRole::Wing(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let slow = part_pose(&wing, FlightMode::Gliding, Vec3::new(0.0, 0.0, -8.0), 0.0);
        let fast = part_pose(
            &wing,
            FlightMode::Gliding,
            Vec3::new(0.0, -10.0, -55.0),
            0.0,
        );

        assert!(fast.translation.y > slow.translation.y + 0.035);
        assert!(fast.translation.z > slow.translation.z);
    }

    #[test]
    fn deployed_glider_dive_sweeps_wings_into_streamline() {
        let wing = CharacterPart::new(
            CharacterPartRole::Wing(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -4.0, -32.0),
            FlightInput::default(),
            50.0,
        );
        let dive_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -22.0, -44.0),
            FlightInput {
                dive: true,
                ..default()
            },
            50.0,
        );

        let glide = part_pose_with_context(&wing, glide_context, 0.0);
        let dive = part_pose_with_context(&wing, dive_context, 0.0);
        let glide_traversal = glider_traversal_pose(glide_context, 0.0);
        let dive_traversal = glider_traversal_pose(dive_context, 0.0);

        assert_eq!(dive.visibility, PartVisibility::Visible);
        assert!(dive.translation.z > glide.translation.z + 0.10);
        assert!(
            dive.rotation.angle_between(glide.rotation).to_degrees() > 10.0,
            "expected deployed dive wing to visibly pitch/sweep, glide={:?}, dive={:?}",
            glide.rotation,
            dive.rotation
        );
        assert!(dive_traversal.motion_m() > glide_traversal.motion_m() + 0.08);
        assert!(dive_traversal.response_degrees() > glide_traversal.response_degrees() + 8.0);
    }

    #[test]
    fn glider_traversal_pose_is_neutral_when_not_gliding() {
        let pose = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -12.0, -36.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert_eq!(pose.motion_m(), 0.0);
        assert_eq!(pose.response_degrees(), 0.0);
    }

    #[test]
    fn glider_deployment_releases_for_head_down_dive() {
        let glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -5.0, -34.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Gliding);
        let dive_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -28.0, -42.0),
            FlightInput {
                dive: true,
                ..default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Diving);

        assert_eq!(glider_deployment_for_context(glide_context), 1.0);
        assert!(glider_deployment_for_context(dive_context) > 0.25);
        assert!(glider_deployment_for_context(dive_context) < 0.60);
    }

    #[test]
    fn glider_traversal_pose_has_launch_takeout_motion() {
        let launch_pose = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Launching,
                Vec3::new(0.0, 8.0, -20.0),
                FlightInput::default(),
                80.0,
            ),
            0.0,
        );
        let glide_pose = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -36.0),
                FlightInput::default(),
                80.0,
            ),
            0.0,
        );

        assert!(launch_pose.motion_m() > 0.18);
        assert!(launch_pose.response_degrees() > 8.0);
        assert!(launch_pose.motion_m() > glide_pose.motion_m() + 0.08);
    }

    #[test]
    fn glider_traversal_pose_banks_with_lateral_turn_input() {
        let right_pose = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -36.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let left_pose = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -36.0),
                FlightInput {
                    left: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert!(right_pose.response_degrees() > 8.0);
        assert!(left_pose.response_degrees() > 8.0);
        assert!((right_pose.rotation_offset * Vec3::Y).x > 0.09);
        assert!((left_pose.rotation_offset * Vec3::Y).x < -0.09);
    }

    #[test]
    fn glider_traversal_pose_distinguishes_dive_and_air_brake() {
        let dive = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -22.0, -44.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );
        let air_brake = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -34.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );

        assert!(dive.response_degrees() > 6.0);
        assert!(air_brake.response_degrees() > 8.0);
        assert!(dive.translation_offset.z > air_brake.translation_offset.z + 0.10);
        assert!(dive.response_degrees() < 18.0);
        assert!(air_brake.response_degrees() < 18.0);
    }

    #[test]
    fn rearward_air_brake_deepens_cupped_pose_and_glider_response() {
        let input = FlightInput {
            backward: true,
            ..default()
        };
        let forward_brake_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -4.0, -34.0),
            input,
            50.0,
        );
        let rearward_brake_context =
            PlayerPoseContext::new(FlightMode::Gliding, Vec3::new(0.0, -4.0, 16.0), input, 50.0);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let forward_metrics = pose_readability_metrics(forward_brake_context, 0.0);
        let rearward_metrics = pose_readability_metrics(rearward_brake_context, 0.0);
        let forward_arm = part_pose_with_context(&left_arm, forward_brake_context, 0.0);
        let rearward_arm = part_pose_with_context(&left_arm, rearward_brake_context, 0.0);
        let forward_leg = part_pose_with_context(&left_leg, forward_brake_context, 0.0);
        let rearward_leg = part_pose_with_context(&left_leg, rearward_brake_context, 0.0);
        let forward_glider = glider_traversal_pose(forward_brake_context, 0.0);
        let rearward_glider = glider_traversal_pose(rearward_brake_context, 0.0);

        assert_eq!(rearward_brake_context.intent(), PlayerPoseIntent::AirBrake);
        assert!(rearward_metrics.torso_pitch_degrees > forward_metrics.torso_pitch_degrees + 5.0);
        assert!(rearward_metrics.leg_tuck_degrees > forward_metrics.leg_tuck_degrees + 6.0);
        assert!(rearward_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
        assert!(rearward_arm.rotation.angle_between(forward_arm.rotation) > 0.10);
        assert!(rearward_leg.rotation.angle_between(forward_leg.rotation) > 0.12);
        assert!(rearward_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(rearward_leg.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(rearward_glider.translation_offset.z < forward_glider.translation_offset.z - 0.035);
        assert!(rearward_glider.response_degrees() > forward_glider.response_degrees() + 2.5);
    }

    #[test]
    fn glider_traversal_pose_skids_directionally_while_air_braking() {
        let pure_brake = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -34.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );
        let right_brake = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -34.0),
                FlightInput {
                    backward: true,
                    right: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );
        let left_brake = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -34.0),
                FlightInput {
                    backward: true,
                    left: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );

        assert!(right_brake.response_degrees() > pure_brake.response_degrees() + 4.0);
        assert!(right_brake.translation_offset.x > pure_brake.translation_offset.x + 0.045);
        assert!(left_brake.translation_offset.x < pure_brake.translation_offset.x - 0.045);
        assert!((right_brake.rotation_offset * Vec3::Y).x > 0.13);
        assert!((left_brake.rotation_offset * Vec3::Y).x < -0.13);
    }

    #[test]
    fn glider_traversal_pose_keeps_moderate_deployed_dive_readable() {
        let dive = glider_traversal_pose(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -13.5, -26.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                50.0,
            ),
            0.0,
        );

        assert!(dive.response_degrees() > 4.0);
        assert!(dive.motion_m() > 0.04);
    }

    #[test]
    fn grounded_idle_pose_breathes_without_stride_input() {
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let head = CharacterPart::new(CharacterPartRole::Head, Vec3::ZERO, Quat::IDENTITY);
        let context = PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::ZERO,
            FlightInput::default(),
            0.0,
        );

        let neutral_torso = part_pose_with_context(&torso, context, 0.0);
        let inhale_torso = part_pose_with_context(&torso, context, std::f32::consts::PI);
        let neutral_head = part_pose_with_context(&head, context, 0.0);
        let inhale_head = part_pose_with_context(&head, context, std::f32::consts::PI);

        assert!(inhale_torso.translation.y > neutral_torso.translation.y + 0.010);
        assert!(inhale_torso.rotation.angle_between(neutral_torso.rotation) > 0.01);
        assert!(inhale_head.rotation.angle_between(neutral_head.rotation) > 0.01);
    }

    #[test]
    fn gliding_pose_has_subtle_limb_airflow_motion() {
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -6.0, -40.0),
            FlightInput::default(),
            40.0,
        );

        let arm_a = part_pose_with_context(&left_arm, context, 0.0);
        let arm_b = part_pose_with_context(&left_arm, context, 1.0);
        let leg_a = part_pose_with_context(&left_leg, context, 0.0);
        let leg_b = part_pose_with_context(&left_leg, context, 1.0);

        assert!(arm_a.rotation.angle_between(arm_b.rotation) > 0.02);
        assert!(arm_a.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(arm_b.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(leg_a.rotation.angle_between(leg_b.rotation) > 0.02);
    }

    #[test]
    fn grounded_stride_moves_left_and_right_legs_opposite_each_other() {
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let left_pose = part_pose(
            &left_leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );
        let right_pose = part_pose(
            &right_leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );

        assert!(left_pose.translation.z > 0.1);
        assert!(right_pose.translation.z < -0.1);
    }

    #[test]
    fn grounded_pose_intent_splits_walk_and_run_speed_bands() {
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -3.0),
                FlightInput::default(),
                0.0,
            )),
            PlayerPoseIntent::GroundedWalk
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -10.0),
                FlightInput::default(),
                0.0,
            )),
            PlayerPoseIntent::GroundedRun
        );
    }

    #[test]
    fn grounded_run_has_larger_stride_than_walk() {
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let walk = part_pose(
            &leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -3.0),
            TAU * 0.25,
        );
        let run = part_pose(
            &leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -10.0),
            TAU * 0.25,
        );

        assert!(run.translation.z > walk.translation.z + 0.08);
        assert!(run.translation.y > walk.translation.y + 0.03);
        assert!(
            run.rotation.angle_between(Quat::IDENTITY)
                > walk.rotation.angle_between(Quat::IDENTITY)
        );
    }

    #[test]
    fn grounded_stride_readability_metrics_require_leg_opposition_and_foot_travel() {
        let walk_metrics = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -5.5),
                FlightInput::default(),
                0.0,
            ),
            TAU * 0.25,
        );
        let run_metrics = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -10.0),
                FlightInput::default(),
                0.0,
            ),
            TAU * 0.25,
        );

        assert!(
            walk_metrics.grounded_stride_foot_travel_m > GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M
        );
        assert!(
            walk_metrics.grounded_stride_leg_opposition_degrees
                > GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES
        );
        assert!(run_metrics.grounded_stride_foot_travel_m > GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M);
        assert!(
            run_metrics.grounded_stride_leg_opposition_degrees
                > GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES
        );
        assert!(
            run_metrics.grounded_stride_foot_travel_m > walk_metrics.grounded_stride_foot_travel_m
        );
        assert!(
            run_metrics.grounded_stride_leg_opposition_degrees
                > walk_metrics.grounded_stride_leg_opposition_degrees
        );
    }

    #[test]
    fn gliding_pose_lifts_arms_into_glider_posture() {
        let arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::new(-0.58, 1.05, 0.0),
            Quat::IDENTITY,
        );

        let grounded = part_pose(
            &arm,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );
        let gliding = part_pose(&arm, FlightMode::Gliding, Vec3::ZERO, TAU * 0.25);

        assert!(gliding.translation.y > grounded.translation.y);
    }

    #[test]
    fn pose_intent_classifies_dive_air_brake_and_landing_anticipation() {
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Launching,
                Vec3::new(0.0, 12.0, -8.0),
                FlightInput::default(),
                40.0,
            )),
            PlayerPoseIntent::Launching
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -8.0, -12.0),
                FlightInput::default(),
                40.0,
            )),
            PlayerPoseIntent::Falling
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -8.0, -32.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::Diving
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -26.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirBrake
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirTurn
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -2.0, -24.0),
                FlightInput {
                    left: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirTurn
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -26.0),
                FlightInput {
                    backward: true,
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirBrake
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -18.0, -32.0),
                FlightInput {
                    dive: true,
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::Diving
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -3.0, -18.0),
                FlightInput::default(),
                4.5,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -24.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                10.5,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(
                PlayerPoseContext::new(
                    FlightMode::Grounded,
                    Vec3::new(0.0, 0.0, -6.0),
                    FlightInput::default(),
                    0.0,
                )
                .with_landing_recovery(0.22, 12.0)
            ),
            PlayerPoseIntent::LandingRecovery
        );
    }

    #[test]
    fn pose_intent_resolution_holds_air_turn_through_short_input_gaps() {
        let turn_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -32.0),
            FlightInput {
                right: true,
                ..default()
            },
            40.0,
        );
        let neutral_glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -32.0),
            FlightInput::default(),
            40.0,
        );

        let initial = resolve_pose_intent(PlayerPoseIntent::Gliding, 0.0, turn_context, 1.0 / 60.0);
        let held = resolve_pose_intent(
            initial.intent,
            initial.hold_remaining_secs,
            neutral_glide_context,
            1.0 / 60.0,
        );
        let released = resolve_pose_intent(
            held.intent,
            held.hold_remaining_secs,
            neutral_glide_context,
            0.20,
        );

        assert_eq!(initial.intent, PlayerPoseIntent::AirTurn);
        assert_eq!(held.intent, PlayerPoseIntent::AirTurn);
        assert!(held.hold_remaining_secs < initial.hold_remaining_secs);
        assert_eq!(released.intent, PlayerPoseIntent::Gliding);
        assert_eq!(released.hold_remaining_secs, 0.0);
    }

    #[test]
    fn pose_input_resolution_preserves_held_air_turn_direction() {
        let previous_input = FlightInput {
            right: true,
            glide: true,
            ..default()
        };
        let current_input = FlightInput {
            glide: true,
            ..default()
        };

        let pose_input = resolve_pose_input(
            PlayerPoseIntent::AirTurn,
            PlayerPoseIntent::AirTurn,
            PlayerPoseIntent::Gliding,
            previous_input,
            current_input,
        );

        assert!(pose_input.right);
        assert!(!pose_input.left);
        assert!(pose_input.glide);
    }

    #[test]
    fn pose_input_resolution_uses_current_input_for_active_air_turn() {
        let previous_input = FlightInput {
            right: true,
            glide: true,
            ..default()
        };
        let current_input = FlightInput {
            left: true,
            glide: true,
            ..default()
        };

        let pose_input = resolve_pose_input(
            PlayerPoseIntent::AirTurn,
            PlayerPoseIntent::AirTurn,
            PlayerPoseIntent::AirTurn,
            previous_input,
            current_input,
        );

        assert!(pose_input.left);
        assert!(!pose_input.right);
    }

    #[test]
    fn pose_input_resolution_preserves_held_air_brake_pressure() {
        let previous_input = FlightInput {
            backward: true,
            left: true,
            glide: true,
            ..default()
        };
        let current_input = FlightInput {
            glide: true,
            ..default()
        };

        let pose_input = resolve_pose_input(
            PlayerPoseIntent::AirBrake,
            PlayerPoseIntent::AirBrake,
            PlayerPoseIntent::Gliding,
            previous_input,
            current_input,
        );

        assert!(pose_input.backward);
        assert!(pose_input.left);
        assert!(!pose_input.right);
        assert!(pose_input.glide);
    }

    #[test]
    fn pose_input_resolution_uses_current_input_for_immediate_landing() {
        let previous_input = FlightInput {
            backward: true,
            right: true,
            glide: true,
            ..default()
        };
        let current_input = FlightInput {
            glide: true,
            ..default()
        };

        let pose_input = resolve_pose_input(
            PlayerPoseIntent::AirBrake,
            PlayerPoseIntent::LandingAnticipation,
            PlayerPoseIntent::LandingAnticipation,
            previous_input,
            current_input,
        );

        assert_eq!(pose_input, current_input);
    }

    #[test]
    fn landing_anticipation_interrupts_held_air_pose_intent() {
        let near_ground = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -28.0),
            FlightInput {
                dive: true,
                ..default()
            },
            4.0,
        );

        let resolved = resolve_pose_intent(PlayerPoseIntent::Diving, 0.12, near_ground, 1.0 / 60.0);

        assert_eq!(resolved.intent, PlayerPoseIntent::LandingAnticipation);
        assert_eq!(resolved.hold_remaining_secs, 0.0);
    }

    #[test]
    fn pose_intent_resolution_does_not_hold_dive_over_readable_glide() {
        let neutral_glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -7.5, -32.0),
            FlightInput::default(),
            40.0,
        );

        let resolved = resolve_pose_intent(
            PlayerPoseIntent::Diving,
            0.12,
            neutral_glide_context,
            1.0 / 60.0,
        );

        assert_eq!(resolved.intent, PlayerPoseIntent::Gliding);
        assert_eq!(resolved.hold_remaining_secs, 0.0);
    }

    #[test]
    fn resolved_pose_context_drives_generated_pose_metrics() {
        let raw_glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -32.0),
            FlightInput::default(),
            40.0,
        );
        let resolved_dive_context =
            raw_glide_context.with_resolved_intent(PlayerPoseIntent::Diving);

        let glide_metrics = pose_readability_metrics(raw_glide_context, 0.0);
        let dive_metrics = pose_readability_metrics(resolved_dive_context, 0.0);

        assert_eq!(raw_glide_context.intent(), PlayerPoseIntent::Gliding);
        assert_eq!(resolved_dive_context.intent(), PlayerPoseIntent::Diving);
        assert!(dive_metrics.torso_pitch_degrees > glide_metrics.torso_pitch_degrees + 18.0);
        assert!(dive_metrics.arm_spread_degrees < glide_metrics.arm_spread_degrees - 40.0);
        assert!(dive_metrics.leg_tuck_degrees > glide_metrics.leg_tuck_degrees + 25.0);
    }

    #[test]
    fn high_sink_landing_anticipation_looks_ahead_before_dive_pose() {
        assert!((landing_anticipation_height_m(-42.0) - 36.0).abs() < 0.001);
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -42.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                19.0,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -42.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                37.0,
            )),
            PlayerPoseIntent::Diving
        );
    }

    #[test]
    fn dive_pose_streamlines_torso_and_arms() {
        let hips = CharacterPart::new(CharacterPartRole::Hips, Vec3::ZERO, Quat::IDENTITY);
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let gliding_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -28.0),
            FlightInput::default(),
            40.0,
        );
        let diving_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -42.0),
            FlightInput {
                dive: true,
                ..default()
            },
            40.0,
        );

        let gliding_hips = part_pose_with_context(&hips, gliding_context, 0.0);
        let gliding_torso = part_pose_with_context(&torso, gliding_context, 0.0);
        let diving_hips = part_pose_with_context(&hips, diving_context, 0.0);
        let diving_torso = part_pose_with_context(&torso, diving_context, 0.0);
        let gliding_arm = part_pose_with_context(&left_arm, gliding_context, 0.0);
        let diving_arm = part_pose_with_context(&left_arm, diving_context, 0.0);
        let gliding_body_rotation = gliding_hips.rotation * gliding_torso.rotation;
        let diving_body_rotation = diving_hips.rotation * diving_torso.rotation;

        assert!(diving_hips.rotation.angle_between(Quat::IDENTITY) > 1.8);
        assert!(diving_torso.rotation.angle_between(Quat::IDENTITY) < 0.75);
        assert!(diving_body_rotation.angle_between(Quat::IDENTITY) > 0.8);
        assert!(
            diving_body_rotation.angle_between(Quat::IDENTITY)
                > gliding_body_rotation.angle_between(Quat::IDENTITY) + 0.45
        );
        assert!(diving_arm.rotation.angle_between(gliding_arm.rotation) > 0.20);
        assert!(diving_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
    }

    #[test]
    fn dive_pose_pressure_scales_from_shallow_to_fast_sink() {
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let shallow_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -7.0, -28.0),
            FlightInput {
                dive: true,
                ..default()
            },
            40.0,
        );
        let fast_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -28.0, -48.0),
            FlightInput {
                dive: true,
                ..default()
            },
            40.0,
        );

        let shallow_metrics = pose_readability_metrics(shallow_context, 0.0);
        let fast_metrics = pose_readability_metrics(fast_context, 0.0);
        let shallow_leg = part_pose_with_context(&leg, shallow_context, 0.0);
        let fast_leg = part_pose_with_context(&leg, fast_context, 0.0);

        assert_eq!(shallow_context.intent(), PlayerPoseIntent::Diving);
        assert_eq!(fast_context.intent(), PlayerPoseIntent::Diving);
        assert!(shallow_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
        assert!(fast_metrics.torso_pitch_degrees > shallow_metrics.torso_pitch_degrees + 7.0);
        assert!(fast_metrics.arm_spread_degrees < DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES);
        assert!(
            fast_metrics.leg_tuck_degrees > shallow_metrics.leg_tuck_degrees + 4.0,
            "expected fast dive leg streamlining metric to rise; shallow={}, fast={}",
            shallow_metrics.leg_tuck_degrees,
            fast_metrics.leg_tuck_degrees
        );
        assert!(
            fast_leg
                .rotation
                .angle_between(shallow_leg.rotation)
                .to_degrees()
                > 2.0
        );
        assert!(fast_leg.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
    }

    #[test]
    fn scarf_trail_streams_harder_in_deployed_glider_dive() {
        let scarf = CharacterPart::new(
            CharacterPartRole::Scarf(ScarfSegment::Trail),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let glide_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -24.0),
            FlightInput {
                glide: true,
                ..default()
            },
            40.0,
        );
        let dive_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -24.0, -48.0),
            FlightInput {
                glide: true,
                dive: true,
                ..default()
            },
            40.0,
        );

        let glide = part_pose_with_context(&scarf, glide_context, 0.0);
        let dive = part_pose_with_context(&scarf, dive_context, 0.0);
        let glide_metrics = pose_readability_metrics(glide_context, 0.0);
        let dive_metrics = pose_readability_metrics(dive_context, 0.0);

        assert_eq!(glide.visibility, PartVisibility::Inherited);
        assert_eq!(dive_context.intent(), PlayerPoseIntent::Diving);
        assert!(
            dive.translation.z > glide.translation.z + 0.32,
            "expected dive scarf stream to read behind the player, glide {}, dive {}",
            glide.translation.z,
            dive.translation.z
        );
        assert!(
            dive.rotation.angle_between(Quat::IDENTITY)
                > glide.rotation.angle_between(Quat::IDENTITY) + 0.28
        );
        assert!(dive_metrics.scarf_stream_m > glide_metrics.scarf_stream_m + 0.32);
        assert!(dive_metrics.scarf_tail_flex_degrees > glide_metrics.scarf_tail_flex_degrees + 9.0);
        assert!(dive_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn scarf_trail_reacts_to_crosswind_and_anchors_less_than_tail() {
        let anchor = CharacterPart::new(
            CharacterPartRole::Scarf(ScarfSegment::Anchor),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let tail = CharacterPart::new(
            CharacterPartRole::Scarf(ScarfSegment::Trail),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let base_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -5.0, -34.0),
            FlightInput {
                right: true,
                glide: true,
                ..default()
            },
            40.0,
        );
        let left_wind = base_context.with_wind_lateral_load(-1.0);
        let right_wind = base_context.with_wind_lateral_load(1.0);

        let left_tail = part_pose_with_context(&tail, left_wind, 0.4);
        let right_tail = part_pose_with_context(&tail, right_wind, 0.4);
        let right_anchor = part_pose_with_context(&anchor, right_wind, 0.4);
        let left_metrics = pose_readability_metrics(left_wind, 0.4);
        let right_metrics = pose_readability_metrics(right_wind, 0.4);

        assert!(
            right_tail.translation.x > left_tail.translation.x + 0.06,
            "expected scarf tail to sway with crosswind, left {}, right {}",
            left_tail.translation.x,
            right_tail.translation.x
        );
        assert!(right_tail.translation.z > right_anchor.translation.z + 0.14);
        assert!(
            right_tail.rotation.angle_between(right_anchor.rotation) > 0.25,
            "expected scarf tail to flex more than anchor"
        );
        assert!(right_metrics.scarf_lateral_sway_m > left_metrics.scarf_lateral_sway_m + 0.02);
        assert!(right_metrics.scarf_stream_m > 0.25);
        assert!(right_metrics.scarf_tail_flex_degrees > 14.0);
    }

    #[test]
    fn dive_readability_requires_head_down_streamlined_shape() {
        let weak_torso_score = key_pose_readability_score(
            PlayerPoseIntent::Diving,
            DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES - 8.0,
            DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES - 12.0,
            DIVE_MIN_LEG_TUCK_READABILITY_DEGREES + 8.0,
            0.0,
            0.0,
            0.0,
        );
        let weak_arm_score = key_pose_readability_score(
            PlayerPoseIntent::Diving,
            DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES + 8.0,
            DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES + 24.0,
            DIVE_MIN_LEG_TUCK_READABILITY_DEGREES + 8.0,
            0.0,
            0.0,
            0.0,
        );
        let weak_leg_score = key_pose_readability_score(
            PlayerPoseIntent::Diving,
            DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES + 8.0,
            DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES - 12.0,
            DIVE_MIN_LEG_TUCK_READABILITY_DEGREES - 12.0,
            0.0,
            0.0,
            0.0,
        );
        let readable_score = key_pose_readability_score(
            PlayerPoseIntent::Diving,
            DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES + 6.0,
            DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES - 18.0,
            DIVE_MIN_LEG_TUCK_READABILITY_DEGREES + 6.0,
            0.0,
            0.0,
            0.0,
        );

        assert!(weak_torso_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(weak_arm_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(weak_leg_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(readable_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn freefall_pose_flattens_and_dive_pose_streamlines() {
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let falling_context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -12.0, -14.0),
            FlightInput::default(),
            50.0,
        );
        let dive_context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -30.0, -30.0),
            FlightInput {
                dive: true,
                ..default()
            },
            50.0,
        );

        let falling_metrics = pose_readability_metrics(falling_context, 0.0);
        let dive_metrics = pose_readability_metrics(dive_context, 0.0);
        let left_dive_arm = part_pose_with_context(&left_arm, dive_context, 0.0);
        let right_dive_arm = part_pose_with_context(&right_arm, dive_context, 0.0);

        assert_eq!(dive_context.intent(), PlayerPoseIntent::Diving);
        assert!(falling_metrics.torso_pitch_degrees > 72.0);
        assert!(falling_metrics.arm_spread_degrees > 150.0);
        assert!(dive_metrics.torso_pitch_degrees > DIVE_MIN_TORSO_PITCH_READABILITY_DEGREES);
        assert!(dive_metrics.arm_spread_degrees < DIVE_MAX_ARM_SPREAD_READABILITY_DEGREES);
        assert!(dive_metrics.torso_pitch_degrees > falling_metrics.torso_pitch_degrees + 20.0);
        assert!(dive_metrics.leg_tuck_degrees > falling_metrics.leg_tuck_degrees + 30.0);
        assert!(left_dive_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(right_dive_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(dive_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn falling_to_dive_pose_blend_is_bounded_and_directional() {
        let hips = CharacterPart::new(CharacterPartRole::Hips, Vec3::ZERO, Quat::IDENTITY);
        let falling_context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -18.0, -24.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Falling);
        let dive_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -30.0, -42.0),
            FlightInput {
                dive: true,
                ..default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Diving);
        let mut smoothed = part_pose_with_context(&hips, falling_context, 0.0).rotation;
        let dive_target = part_pose_with_context(&hips, dive_context, 0.0).rotation;
        let blend = pose_blend_for_intent(PlayerPoseIntent::Diving, 1.0 / 60.0);
        let mut max_step_degrees: f32 = 0.0;

        for _ in 0..11 {
            let previous = smoothed;
            smoothed = smoothed.slerp(dive_target, blend);
            max_step_degrees = max_step_degrees.max(previous.angle_between(smoothed).to_degrees());
        }

        assert!(
            max_step_degrees < 18.0,
            "max dive blend step was {max_step_degrees}"
        );
        assert!(smoothed.angle_between(dive_target).to_degrees() < 8.0);
    }

    #[test]
    fn dive_pose_has_phase_staggered_airflow_micro_motion() {
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let head = CharacterPart::new(CharacterPartRole::Head, Vec3::ZERO, Quat::IDENTITY);
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -28.0, -48.0),
            FlightInput {
                dive: true,
                ..default()
            },
            60.0,
        );

        let arm_a = part_pose_with_context(&left_arm, context, 0.0);
        let arm_b = part_pose_with_context(&left_arm, context, 1.1);
        let leg_a = part_pose_with_context(&left_leg, context, 0.0);
        let leg_b = part_pose_with_context(&left_leg, context, 1.1);
        let torso_a = part_pose_with_context(&torso, context, 0.0);
        let torso_b = part_pose_with_context(&torso, context, 1.1);
        let head_a = part_pose_with_context(&head, context, 0.0);
        let head_b = part_pose_with_context(&head, context, 1.1);
        let metrics = pose_readability_metrics(context, 1.1);

        assert_eq!(context.intent(), PlayerPoseIntent::Diving);
        assert!(arm_a.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(arm_b.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(leg_a.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(leg_b.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(arm_a.rotation.angle_between(arm_b.rotation).to_degrees() > 3.0);
        assert!(leg_a.rotation.angle_between(leg_b.rotation).to_degrees() > 2.0);
        assert!(
            torso_a
                .rotation
                .angle_between(torso_b.rotation)
                .to_degrees()
                > 1.0,
            "expected dive torso to subtly flex under airflow"
        );
        assert!(
            head_a.rotation.angle_between(head_b.rotation).to_degrees() > 1.0,
            "expected dive head tuck to subtly flex under airflow"
        );
        assert!(metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn launching_and_falling_key_poses_are_readable() {
        let launch_metrics = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Launching,
                Vec3::new(0.0, 12.0, -8.0),
                FlightInput::default(),
                40.0,
            ),
            0.0,
        );
        let falling_metrics = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -12.0, -14.0),
                FlightInput::default(),
                40.0,
            ),
            0.0,
        );

        assert!(launch_metrics.torso_pitch_degrees >= 16.0);
        assert!(launch_metrics.arm_spread_degrees >= 28.0);
        assert!(launch_metrics.leg_tuck_degrees >= 18.0);
        assert!(launch_metrics.launch_overhead_arm_score >= 0.60);
        assert!(launch_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
        assert!(falling_metrics.torso_pitch_degrees >= 8.0);
        assert!(falling_metrics.arm_spread_degrees >= 64.0);
        assert!(falling_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn launch_takeout_pose_lifts_both_arms_overhead() {
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let context = PlayerPoseContext::new(
            FlightMode::Launching,
            Vec3::new(0.0, 24.0, -18.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Launching);

        let metrics = pose_readability_metrics(context, 0.75);
        let left_pose = part_pose_with_context(&left_arm, context, 0.75);
        let right_pose = part_pose_with_context(&right_arm, context, 0.75);
        let left_arm_raise = (left_pose.rotation * Vec3::NEG_Y).y;
        let right_arm_raise = (right_pose.rotation * Vec3::NEG_Y).y;

        assert!(metrics.launch_overhead_arm_score >= 0.60);
        assert!(left_arm_raise >= 0.60);
        assert!(right_arm_raise >= 0.60);
    }

    #[test]
    fn landing_anticipation_pose_flares_torso_and_tucks_legs_forward() {
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let falling_context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            20.0,
        );
        let gliding_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            20.0,
        );
        let landing_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            4.5,
        );

        let falling_metrics = pose_readability_metrics(falling_context, 0.0);
        let gliding_metrics = pose_readability_metrics(gliding_context, 0.0);
        let landing_metrics = pose_readability_metrics(landing_context, 0.0);
        let falling = part_pose_with_context(&leg, falling_context, 0.0);
        let landing = part_pose_with_context(&leg, landing_context, 0.0);

        assert!(landing_metrics.torso_pitch_degrees > 32.0);
        assert!(falling_metrics.torso_pitch_degrees > 36.0);
        assert!(landing_metrics.torso_pitch_degrees > gliding_metrics.torso_pitch_degrees + 12.0);
        assert!(landing.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(falling.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(landing.rotation.angle_between(falling.rotation) > 1.0);
    }

    #[test]
    fn fast_sink_landing_anticipation_flips_more_than_slow_approach() {
        let head = CharacterPart::new(CharacterPartRole::Head, Vec3::ZERO, Quat::IDENTITY);
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let lead_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let slow_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            4.5,
        );
        let fast_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -14.0, -28.0),
            FlightInput::default(),
            7.0,
        );

        assert_eq!(slow_context.intent(), PlayerPoseIntent::LandingAnticipation);
        assert_eq!(fast_context.intent(), PlayerPoseIntent::LandingAnticipation);

        let slow_metrics = pose_readability_metrics(slow_context, 0.0);
        let fast_metrics = pose_readability_metrics(fast_context, 0.0);
        let slow_head = part_pose_with_context(&head, slow_context, 0.0);
        let fast_head = part_pose_with_context(&head, fast_context, 0.0);
        let slow_leg = part_pose_with_context(&leg, slow_context, 0.0);
        let fast_leg = part_pose_with_context(&leg, fast_context, 0.0);
        let fast_lead_leg = part_pose_with_context(&lead_leg, fast_context, 0.0);

        assert!(fast_metrics.torso_pitch_degrees > slow_metrics.torso_pitch_degrees + 8.0);
        assert!(fast_metrics.leg_tuck_degrees > slow_metrics.leg_tuck_degrees + 8.0);
        assert!(fast_metrics.landing_crouch_m > slow_metrics.landing_crouch_m + 0.02);
        assert!(fast_metrics.landing_foot_forward_m > slow_metrics.landing_foot_forward_m + 0.04);
        assert!(fast_metrics.landing_foot_split_m >= LANDING_MIN_FOOT_SPLIT_READABILITY_M);
        assert!(
            fast_head
                .rotation
                .angle_between(slow_head.rotation)
                .to_degrees()
                > 5.0
        );
        assert!(
            fast_leg
                .rotation
                .angle_between(slow_leg.rotation)
                .to_degrees()
                > 8.0
        );
        assert!(
            fast_lead_leg
                .rotation
                .angle_between(fast_leg.rotation)
                .to_degrees()
                > 8.0
        );
    }

    #[test]
    fn landing_anticipation_readability_requires_torso_flare() {
        let weak_torso_score = key_pose_readability_score(
            PlayerPoseIntent::LandingAnticipation,
            18.0,
            0.0,
            90.0,
            0.16,
            0.40,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M,
        );
        let weak_feet_score = key_pose_readability_score(
            PlayerPoseIntent::LandingAnticipation,
            44.0,
            0.0,
            90.0,
            0.16,
            0.08,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M,
        );
        let weak_split_score = key_pose_readability_score(
            PlayerPoseIntent::LandingAnticipation,
            44.0,
            0.0,
            90.0,
            0.16,
            0.40,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M * 0.5,
        );
        let readable_score = key_pose_readability_score(
            PlayerPoseIntent::LandingAnticipation,
            44.0,
            0.0,
            90.0,
            0.16,
            0.40,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M,
        );

        assert!(weak_torso_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(weak_feet_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(weak_split_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(readable_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn landing_recovery_pose_absorbs_impact_after_touchdown() {
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let stride_context = PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -6.0),
            FlightInput {
                forward: true,
                ..default()
            },
            0.0,
        );
        let recovery_context = stride_context.with_landing_recovery(0.24, 14.0);

        let stride_torso = part_pose_with_context(&torso, stride_context, 0.0);
        let recovery_torso = part_pose_with_context(&torso, recovery_context, 0.0);
        let stride_leg = part_pose_with_context(&leg, stride_context, 0.0);
        let recovery_leg = part_pose_with_context(&leg, recovery_context, 0.0);
        let recovery_metrics = pose_readability_metrics(recovery_context, 0.0);

        assert!(recovery_torso.translation.y < stride_torso.translation.y - 0.06);
        assert!(recovery_leg.rotation.angle_between(stride_leg.rotation) > 0.55);
        assert!(recovery_leg.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001);
        assert!(recovery_metrics.landing_recovery_flip_degrees > 48.0);
        assert!(recovery_metrics.landing_foot_split_m >= LANDING_MIN_FOOT_SPLIT_READABILITY_M);
        assert!(recovery_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn landing_recovery_readability_requires_flip_silhouette() {
        let weak_flip_score = key_pose_readability_score(
            PlayerPoseIntent::LandingRecovery,
            18.0,
            0.0,
            48.0,
            0.12,
            0.0,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M,
        );
        let weak_split_score = key_pose_readability_score(
            PlayerPoseIntent::LandingRecovery,
            42.0,
            0.0,
            48.0,
            0.12,
            0.0,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M * 0.5,
        );
        let readable_score = key_pose_readability_score(
            PlayerPoseIntent::LandingRecovery,
            42.0,
            0.0,
            48.0,
            0.12,
            0.0,
            LANDING_MIN_FOOT_SPLIT_READABILITY_M,
        );

        assert!(weak_flip_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(weak_split_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(readable_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn body_local_pose_velocity_uses_player_facing_axes() {
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::X, Vec3::Y)
            .rotation;
        let pose_velocity = body_local_pose_velocity(Vec3::NEG_Z * 14.0, rotation);

        assert!(pose_velocity.x < -13.9);
        assert!(pose_velocity.z.abs() < 0.001);
    }

    #[test]
    fn airborne_turn_pose_exposes_readable_lateral_lean() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(24.0, -2.0, -32.0),
            FlightInput {
                right: true,
                ..default()
            },
            40.0,
        );
        let metrics = pose_readability_metrics(context, 0.0);

        assert_eq!(context.intent(), PlayerPoseIntent::AirTurn);
        assert!(metrics.lateral_lean_degrees > 11.0);
        assert!(metrics.arm_spread_degrees > 100.0);
        assert!(metrics.wing_airflow_strength > 0.25);
    }

    #[test]
    fn airborne_turn_pose_responds_to_lateral_input_before_velocity_builds() {
        let neutral = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput::default(),
                40.0,
            ),
            0.0,
        );
        let turning = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert_eq!(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            )
            .intent(),
            PlayerPoseIntent::AirTurn
        );
        assert!(neutral.lateral_lean_degrees < 0.5);
        assert!(turning.lateral_lean_degrees > 10.0);
    }

    #[test]
    fn airborne_turn_pose_preserves_signed_lean_direction() {
        let right = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let left = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    left: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert!(right.signed_lateral_lean_degrees < -10.0);
        assert!(left.signed_lateral_lean_degrees > 10.0);
        assert!((right.lateral_lean_degrees - left.lateral_lean_degrees).abs() < 0.1);
    }

    #[test]
    fn neutral_gliding_pose_reacts_to_wind_lateral_load() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -34.0),
            FlightInput {
                forward: true,
                ..default()
            },
            40.0,
        )
        .with_wind_lateral_load(0.8);
        let metrics = pose_readability_metrics(context, 0.0);
        let glider = glider_traversal_pose(context, 0.0);

        assert!(metrics.signed_lateral_lean_degrees < -5.0);
        assert!(metrics.lateral_lean_degrees > 5.0);
        assert!(glider.response_degrees() > 2.0);
    }

    #[test]
    fn manual_lateral_input_dominates_wind_lateral_load() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -34.0),
            FlightInput {
                right: true,
                ..default()
            },
            40.0,
        )
        .with_wind_lateral_load(-1.0);
        let metrics = pose_readability_metrics(context, 0.0);

        assert!(metrics.signed_lateral_lean_degrees < -10.0);
    }

    #[test]
    fn airborne_turn_pose_leads_with_head_and_same_side_limbs() {
        let right_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -32.0),
            FlightInput {
                right: true,
                ..default()
            },
            40.0,
        );
        let left_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -32.0),
            FlightInput {
                left: true,
                ..default()
            },
            40.0,
        );
        let head = CharacterPart::new(CharacterPartRole::Head, Vec3::ZERO, Quat::IDENTITY);
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let right_head = part_pose_with_context(&head, right_context, 0.0);
        let left_head = part_pose_with_context(&head, left_context, 0.0);
        let right_torso = part_pose_with_context(&torso, right_context, 0.0);
        let left_torso = part_pose_with_context(&torso, left_context, 0.0);
        let right_turn_left_arm = part_pose_with_context(&left_arm, right_context, 0.0);
        let right_turn_right_arm = part_pose_with_context(&right_arm, right_context, 0.0);
        let right_turn_left_leg = part_pose_with_context(&left_leg, right_context, 0.0);
        let right_turn_right_leg = part_pose_with_context(&right_leg, right_context, 0.0);

        assert!(
            right_head
                .rotation
                .angle_between(left_head.rotation)
                .to_degrees()
                > 15.0
        );
        assert!(right_torso.translation.x > 0.01);
        assert!(left_torso.translation.x < -0.01);
        assert!(
            right_turn_right_arm
                .rotation
                .angle_between(right_turn_left_arm.rotation)
                .to_degrees()
                > 8.0
        );
        assert!(
            right_turn_right_leg
                .rotation
                .angle_between(right_turn_left_leg.rotation)
                .to_degrees()
                > 4.0
        );
        assert!(
            right_turn_right_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001
        );
        assert!(
            right_turn_right_leg.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001
        );
    }

    #[test]
    fn diagonal_air_brake_pose_cups_same_side_limbs_into_skid() {
        let right_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -34.0),
            FlightInput {
                backward: true,
                right: true,
                ..default()
            },
            40.0,
        );
        let left_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -34.0),
            FlightInput {
                backward: true,
                left: true,
                ..default()
            },
            40.0,
        );
        let neutral_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -34.0),
            FlightInput {
                backward: true,
                ..default()
            },
            40.0,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let neutral_metrics = pose_readability_metrics(neutral_context, 0.0);
        let right_metrics = pose_readability_metrics(right_context, 0.0);
        let left_metrics = pose_readability_metrics(left_context, 0.0);
        let right_brake_right_arm = part_pose_with_context(&right_arm, right_context, 0.0);
        let right_brake_left_arm = part_pose_with_context(&left_arm, right_context, 0.0);
        let right_brake_right_leg = part_pose_with_context(&right_leg, right_context, 0.0);
        let right_brake_left_leg = part_pose_with_context(&left_leg, right_context, 0.0);

        assert_eq!(right_context.intent(), PlayerPoseIntent::AirBrake);
        assert!(neutral_metrics.lateral_lean_degrees < 0.5);
        assert!(right_metrics.signed_lateral_lean_degrees < -9.0);
        assert!(left_metrics.signed_lateral_lean_degrees > 9.0);
        assert!(
            right_brake_right_arm
                .rotation
                .angle_between(right_brake_left_arm.rotation)
                .to_degrees()
                > 8.0
        );
        assert!(
            right_brake_right_leg
                .rotation
                .angle_between(right_brake_left_leg.rotation)
                .to_degrees()
                > 4.0
        );
        assert!(
            right_brake_right_arm.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001
        );
        assert!(
            right_brake_right_leg.translation.length() <= CONNECTED_LIMB_MAX_TRANSLATION_M + 0.001
        );
        assert!(right_metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
    }

    #[test]
    fn pose_readability_metrics_distinguish_dive_air_brake_and_landing() {
        let dive = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -18.0, -42.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let air_brake = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let landing = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -18.0),
                FlightInput::default(),
                4.0,
            ),
            0.0,
        );
        let recovery = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -5.0),
                FlightInput::default(),
                0.0,
            )
            .with_landing_recovery(0.24, 14.0),
            0.0,
        );

        assert!(dive.torso_pitch_degrees > 50.0);
        assert!(air_brake.arm_spread_degrees > dive.arm_spread_degrees);
        assert!(landing.torso_pitch_degrees > 32.0);
        assert!(landing.landing_crouch_m > 0.05);
        assert!(landing.landing_foot_forward_m > 0.32);
        assert!(landing.landing_foot_split_m >= LANDING_MIN_FOOT_SPLIT_READABILITY_M);
        assert!(recovery.landing_crouch_m > 0.055);
        assert!(recovery.landing_foot_split_m >= LANDING_MIN_FOOT_SPLIT_READABILITY_M);
        assert!(dive.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
        assert!(air_brake.key_pose_readability_score >= 0.98);
        assert!(landing.key_pose_readability_score >= 0.98);
        assert!(recovery.key_pose_readability_score >= 0.98);
    }

    #[test]
    fn pose_readability_can_be_measured_from_rendered_part_transforms() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(20.0, -3.0, -34.0),
            FlightInput {
                right: true,
                ..default()
            },
            30.0,
        );
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg_pose = part_pose_with_context(&left_leg, context, 0.0);
        let right_leg_pose = part_pose_with_context(&right_leg, context, 0.0);

        let metrics = pose_readability_metrics_from_part_transforms(
            context,
            PoseReadabilityPartTransforms {
                torso_rotation: part_pose_with_context(&torso, context, 0.0).rotation,
                left_arm_rotation: part_pose_with_context(&left_arm, context, 0.0).rotation,
                right_arm_rotation: part_pose_with_context(&right_arm, context, 0.0).rotation,
                left_leg_rotation: left_leg_pose.rotation,
                right_leg_rotation: right_leg_pose.rotation,
                left_leg_translation: left_leg_pose.translation,
                right_leg_translation: right_leg_pose.translation,
            },
        );

        assert!(metrics.lateral_lean_degrees > 8.0);
        assert!(metrics.arm_spread_degrees > 100.0);
        assert!(metrics.key_pose_readability_score >= 0.9);
    }
}
