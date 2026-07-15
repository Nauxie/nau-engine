use bevy::prelude::*;

use super::{
    follow::clamp_camera_player_distance,
    types::{CameraFrame, CameraObstruction, CameraObstructionResolution},
};

pub const CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M: f32 = 7.1;
pub const CAMERA_OBSTRUCTION_WIDE_SHOULDER_OFFSET_M: f32 = 10.0;
pub const CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M: f32 = 2.4;
pub const CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M: f32 = CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M;
pub const CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M: f32 = 1.5;
pub const CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M: f32 = 0.35;
pub const CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS: f32 = 24.0;
pub const CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND: f32 = 120.0;
pub const CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS: f32 = 14.4;
pub const CAMERA_MAX_OBSTRUCTION_HANDOFF_OFFSET_SPEED_MPS: f32 = 18.0;
pub const CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND: f32 = 88.8;
pub const CAMERA_MAX_PLAYER_DISTANCE_M: f32 = 16.45;
pub const CAMERA_OBSTRUCTION_RELEASE_HANDOFF_SECS: f32 = 1.0 / 6.0;
const CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M: f32 = 0.08;
const CAMERA_MIN_COMPACT_OBSTRUCTION_DISTANCE_M: f32 = 5.0;
const CAMERA_OBSTRUCTION_NEAR_TARGET_FALLBACK_CUTOFF_M: f32 = 3.2;
const CAMERA_OBSTRUCTION_RADIAL_OFFSET_SPEED_MPS: f32 = 12.0;
const CAMERA_OBSTRUCTION_LATERAL_OFFSET_SPEED_MPS: f32 = 8.0;
const CAMERA_OBSTRUCTION_RELEASE_OFFSET_SPEED_MPS: f32 = 18.0;
const CAMERA_OBSTRUCTION_LATERAL_SPEED_YAW_DELTA_DEGREES: f32 = 2.0;
const CAMERA_OBSTRUCTION_RELEASE_PREFERENCE_SECS: f32 = 0.24;
const CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M: f32 = 0.05;
const CAMERA_OBSTRUCTION_SIDE_SWITCH_PENALTY_M: f32 = 100.0;
const CAMERA_OBSTRUCTION_FALLBACK_YAW_STEP_DEGREES: f32 = 2.0;
const CAMERA_MAX_OBSTRUCTION_ORBIT_DEVIATION_DEGREES: f32 = 5.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CameraObstructionDistancePolicy {
    #[default]
    Undecided,
    Compact,
    PreserveReadable,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CameraObstructionSmoothingState {
    held_offset: Vec3,
    obstructed_last_frame: bool,
    preference_hold_remaining_secs: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CameraObstructionHandoffState {
    smoothing: CameraObstructionSmoothingState,
    distance_policy: CameraObstructionDistancePolicy,
    preferred_side: f32,
    release_handoff_remaining_secs: f32,
    previous_look_target: Option<Vec3>,
    previous_intent_offset: Option<Vec3>,
    intentional_camera_motion: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct CameraObstructionStep {
    pub frame: CameraFrame,
    pub obstruction_adjustment_m: f32,
    pub obstruction_hits: usize,
    pub distance_clamped: bool,
    pub continuity_offset_limited: bool,
    pub continuity_rotation_limited: bool,
}

impl CameraObstructionHandoffState {
    pub fn set_intentional_camera_motion(&mut self, active: bool) {
        self.intentional_camera_motion = active;
    }
}

impl CameraObstructionSmoothingState {
    fn carry_intent_delta(&mut self, delta: Vec3) {
        if self.readable_offset().is_some() && delta.is_finite() {
            self.held_offset += delta;
        }
    }

    pub fn readable_offset(self) -> Option<Vec3> {
        (self.held_offset.length_squared() > 0.001
            && (self.obstructed_last_frame || self.preference_hold_remaining_secs > 0.0))
            .then_some(self.held_offset)
    }

    pub fn sync_resolved_frame(
        &mut self,
        frame: CameraFrame,
        obstruction_hits: usize,
        obstruction_adjustment_m: f32,
    ) {
        if camera_obstruction_is_active(obstruction_hits, obstruction_adjustment_m) {
            self.held_offset = frame.position - frame.look_target;
            self.obstructed_last_frame = true;
            self.preference_hold_remaining_secs = CAMERA_OBSTRUCTION_RELEASE_PREFERENCE_SECS;
        } else if self.readable_offset().is_some() {
            self.held_offset = frame.position - frame.look_target;
        }
    }
}

pub fn lift_camera_above_floor(
    mut frame: CameraFrame,
    floor_y: f32,
    min_clearance: f32,
) -> CameraFrame {
    let min_y = floor_y + min_clearance.max(0.0);
    if frame.position.y < min_y {
        frame.position.y = min_y;
    }

    frame
}

pub fn clamp_camera_step(
    mut frame: CameraFrame,
    previous_position: Vec3,
    max_step_m: f32,
) -> CameraFrame {
    let max_step_m = max_step_m.max(0.0);
    let step = frame.position - previous_position;
    let step_distance = step.length();
    if step_distance <= max_step_m || step_distance <= 0.001 || !step_distance.is_finite() {
        return frame;
    }

    frame.position = previous_position + step / step_distance * max_step_m;
    frame.rotation = Transform::from_translation(frame.position)
        .looking_at(frame.look_target, Vec3::Y)
        .rotation;
    frame
}

pub fn clamp_camera_offset_step(
    mut frame: CameraFrame,
    previous_position: Vec3,
    previous_look_target: Option<Vec3>,
    max_offset_step_m: f32,
) -> CameraFrame {
    let Some(previous_look_target) = previous_look_target else {
        return clamp_camera_step(frame, previous_position, max_offset_step_m);
    };
    let previous_offset = previous_position - previous_look_target;
    let target_offset = frame.position - frame.look_target;
    if !previous_offset.is_finite() || !target_offset.is_finite() {
        return clamp_camera_step(frame, previous_position, max_offset_step_m);
    }

    let offset_delta = target_offset - previous_offset;
    let offset_delta_distance = offset_delta.length();
    let max_offset_step_m = max_offset_step_m.max(0.0);
    if offset_delta_distance <= max_offset_step_m
        || offset_delta_distance <= 0.001
        || !offset_delta_distance.is_finite()
    {
        return frame;
    }

    frame.position = frame.look_target
        + previous_offset
        + offset_delta / offset_delta_distance * max_offset_step_m;
    frame.rotation = Transform::from_translation(frame.position)
        .looking_at(frame.look_target, Vec3::Y)
        .rotation;
    frame
}

pub fn clamp_camera_rotation_step(
    mut frame: CameraFrame,
    previous_rotation: Quat,
    max_step_degrees: f32,
) -> CameraFrame {
    let max_step_radians = max_step_degrees.max(0.0).to_radians();
    let rotation_delta = previous_rotation.angle_between(frame.rotation);
    if rotation_delta <= max_step_radians || rotation_delta <= 0.001 || !rotation_delta.is_finite()
    {
        return frame;
    }

    frame.rotation = previous_rotation.slerp(frame.rotation, max_step_radians / rotation_delta);
    frame
}

#[allow(clippy::too_many_arguments)]
pub fn enforce_camera_continuity(
    frame: CameraFrame,
    previous_position: Vec3,
    previous_look_target: Option<Vec3>,
    previous_rotation: Quat,
    dt: f32,
    max_offset_speed_mps: f32,
    max_rotation_speed_degrees_per_second: f32,
) -> (CameraFrame, bool, bool) {
    let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
    let target_position = frame.position;
    let target_rotation = frame.rotation;
    let frame = clamp_camera_offset_step(
        frame,
        previous_position,
        previous_look_target,
        max_offset_speed_mps.max(0.0) * dt,
    );
    let offset_limited = frame.position.distance(target_position) > 0.0001;
    let frame = clamp_camera_rotation_step(
        frame,
        previous_rotation,
        max_rotation_speed_degrees_per_second.max(0.0) * dt,
    );
    let rotation_limited = frame.rotation.angle_between(target_rotation) > 0.0001;
    (frame, offset_limited, rotation_limited)
}

pub fn camera_obstruction_is_active(
    obstruction_hits: usize,
    obstruction_adjustment_m: f32,
) -> bool {
    obstruction_hits > 0 && obstruction_adjustment_m >= CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M
}

pub fn smooth_camera_obstruction(
    mut frame: CameraFrame,
    state: &mut CameraObstructionSmoothingState,
    obstruction_hits: usize,
    obstruction_adjustment_m: f32,
    dt: f32,
) -> CameraFrame {
    if camera_obstruction_is_active(obstruction_hits, obstruction_adjustment_m) {
        let mut target_offset = frame.position - frame.look_target;
        if state.readable_offset().is_some() {
            let max_speed_mps = obstruction_offset_speed_mps(state.held_offset, target_offset);
            target_offset =
                step_obstruction_offset_toward(state.held_offset, target_offset, dt, max_speed_mps);
            frame.position = frame.look_target + target_offset;
            frame.rotation = Transform::from_translation(frame.position)
                .looking_at(frame.look_target, Vec3::Y)
                .rotation;
        }

        state.held_offset = target_offset;
        state.obstructed_last_frame = true;
        state.preference_hold_remaining_secs = CAMERA_OBSTRUCTION_RELEASE_PREFERENCE_SECS;
        return frame;
    }

    state.obstructed_last_frame = false;
    let held_offset = (state.held_offset.length_squared() > 0.001).then_some(state.held_offset);
    state.preference_hold_remaining_secs =
        (state.preference_hold_remaining_secs - dt.max(0.0)).max(0.0);

    if let Some(held_offset) = held_offset {
        let clear_offset = frame.position - frame.look_target;
        let eased_offset = step_obstruction_offset_toward(
            held_offset,
            clear_offset,
            dt,
            CAMERA_OBSTRUCTION_RELEASE_OFFSET_SPEED_MPS,
        );
        if eased_offset.distance(clear_offset) > 0.001 {
            frame.position = frame.look_target + eased_offset;
            frame.rotation = Transform::from_translation(frame.position)
                .looking_at(frame.look_target, Vec3::Y)
                .rotation;
            state.held_offset = eased_offset;
            return frame;
        }
    }

    if state.preference_hold_remaining_secs <= 0.0 {
        state.held_offset = Vec3::ZERO;
    } else {
        state.held_offset = frame.position - frame.look_target;
    }
    frame
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_camera_obstruction_handoff(
    frame: CameraFrame,
    previous_position: Vec3,
    previous_rotation: Quat,
    player_position: Vec3,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
    dt: f32,
    state: &mut CameraObstructionHandoffState,
    lift_frame: impl Fn(CameraFrame) -> CameraFrame,
) -> CameraObstructionStep {
    let intentional_camera_motion = std::mem::take(&mut state.intentional_camera_motion);
    let intent_offset = frame.position - frame.look_target;
    if intentional_camera_motion && let Some(previous_intent_offset) = state.previous_intent_offset
    {
        state
            .smoothing
            .carry_intent_delta(intent_offset - previous_intent_offset);
    }
    let obstructions = obstructions.into_iter().collect::<Vec<_>>();
    let preferred_obstruction_offset = state.smoothing.readable_offset();
    let preferred_side = (state.preferred_side.abs() > CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M)
        .then_some(state.preferred_side);
    let mut obstruction = avoid_camera_obstructions_with_distance_policy(
        frame,
        obstructions.iter().copied(),
        clearance,
        preferred_obstruction_offset,
        state.distance_policy,
        preferred_side,
    );
    let mut active_obstruction =
        camera_obstruction_is_active(obstruction.hit_count, obstruction.adjusted_distance_m);
    if active_obstruction
        && state.distance_policy != CameraObstructionDistancePolicy::Compact
        && camera_obstruction_orbit_deviation_degrees(frame, obstruction.frame)
            > CAMERA_MAX_OBSTRUCTION_ORBIT_DEVIATION_DEGREES
    {
        state.distance_policy = CameraObstructionDistancePolicy::Compact;
        state.preferred_side = 0.0;
        obstruction = avoid_camera_obstructions_with_distance_policy(
            frame,
            obstructions.iter().copied(),
            clearance,
            preferred_obstruction_offset,
            state.distance_policy,
            None,
        );
        active_obstruction =
            camera_obstruction_is_active(obstruction.hit_count, obstruction.adjusted_distance_m);
    }
    if active_obstruction && state.distance_policy == CameraObstructionDistancePolicy::Undecided {
        state.distance_policy = if obstruction
            .frame
            .position
            .distance(obstruction.frame.look_target)
            >= CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
        {
            CameraObstructionDistancePolicy::PreserveReadable
        } else {
            CameraObstructionDistancePolicy::Compact
        };
    }
    if active_obstruction {
        let lateral = intent_offset
            .normalize_or_zero()
            .cross(Vec3::Y)
            .normalize_or_zero();
        let correction_side = (obstruction.frame.position - frame.position).dot(lateral);
        if correction_side.abs() > CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M {
            state.preferred_side = correction_side.signum();
        }
    }
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

    let frame = lift_frame(obstruction_frame);
    let pre_smoothing_frame = frame;
    let frame = smooth_camera_obstruction(
        frame,
        &mut state.smoothing,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
        dt,
    );
    let revalidated_obstruction = revalidate_camera_obstruction_with_distance_policy(
        frame,
        obstructions.iter().copied(),
        clearance,
        preferred_obstruction_offset,
        state.distance_policy,
        (state.preferred_side.abs() > CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M)
            .then_some(state.preferred_side),
    );
    let revalidated_active = camera_obstruction_is_active(
        revalidated_obstruction.hit_count,
        revalidated_obstruction.adjusted_distance_m,
    );
    let (frame, active_obstruction_hits, active_obstruction_adjustment_m) = if revalidated_active {
        (
            lift_frame(revalidated_obstruction.frame),
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
        active_obstruction_hits == 0 && state.release_handoff_remaining_secs > 0.0;
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
    let pre_distance_frame = frame;
    let frame = clamp_camera_player_distance(frame, player_position, CAMERA_MAX_PLAYER_DISTANCE_M);
    let distance_clamped = frame.position.distance(pre_distance_frame.position) > 0.0001;
    let max_offset_speed_mps = if active_obstruction_hits > 0 || release_smoothing_active {
        CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS
    } else if release_handoff_active {
        CAMERA_MAX_OBSTRUCTION_HANDOFF_OFFSET_SPEED_MPS
    } else {
        CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS
    };
    let max_rotation_speed_degrees_per_second = if reported_obstruction_hits > 0 {
        CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND
    } else {
        CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND
    };
    let frame = if reported_obstruction_hits > 0 {
        CameraFrame {
            rotation: Transform::from_translation(frame.position)
                .looking_at(frame.look_target, Vec3::Y)
                .rotation,
            ..frame
        }
    } else {
        frame
    };
    let continuity_active =
        !intentional_camera_motion && (reported_obstruction_hits > 0 || distance_clamped);
    let (frame, continuity_offset_limited, continuity_rotation_limited) = if continuity_active {
        enforce_camera_continuity(
            frame,
            previous_position,
            state.previous_look_target,
            previous_rotation,
            dt,
            max_offset_speed_mps,
            max_rotation_speed_degrees_per_second,
        )
    } else {
        (frame, false, false)
    };
    state.smoothing.sync_resolved_frame(
        frame,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
    );
    let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
    if active_obstruction_hits > 0 || release_smoothing_active {
        state.release_handoff_remaining_secs = CAMERA_OBSTRUCTION_RELEASE_HANDOFF_SECS;
    } else {
        state.release_handoff_remaining_secs = (state.release_handoff_remaining_secs - dt).max(0.0);
    }
    if reported_obstruction_hits == 0
        && state.smoothing.readable_offset().is_none()
        && state.release_handoff_remaining_secs <= 0.0
    {
        state.distance_policy = CameraObstructionDistancePolicy::Undecided;
        state.preferred_side = 0.0;
    }
    state.previous_look_target = Some(frame.look_target);
    state.previous_intent_offset = Some(intent_offset);

    CameraObstructionStep {
        frame,
        obstruction_adjustment_m: reported_obstruction_adjustment_m,
        obstruction_hits: reported_obstruction_hits,
        distance_clamped,
        continuity_offset_limited,
        continuity_rotation_limited,
    }
}

fn obstruction_offset_speed_mps(current_offset: Vec3, target_offset: Vec3) -> f32 {
    let current_horizontal = Vec3::new(current_offset.x, 0.0, current_offset.z).normalize_or_zero();
    let target_horizontal = Vec3::new(target_offset.x, 0.0, target_offset.z).normalize_or_zero();
    if current_horizontal.length_squared() <= 0.0001 || target_horizontal.length_squared() <= 0.0001
    {
        return CAMERA_OBSTRUCTION_RADIAL_OFFSET_SPEED_MPS;
    }

    let yaw_delta_degrees = current_horizontal
        .angle_between(target_horizontal)
        .to_degrees();
    if yaw_delta_degrees > CAMERA_OBSTRUCTION_LATERAL_SPEED_YAW_DELTA_DEGREES {
        CAMERA_OBSTRUCTION_LATERAL_OFFSET_SPEED_MPS
    } else {
        CAMERA_OBSTRUCTION_RADIAL_OFFSET_SPEED_MPS
    }
}

fn step_obstruction_offset_toward(
    current_offset: Vec3,
    target_offset: Vec3,
    dt: f32,
    max_speed_mps: f32,
) -> Vec3 {
    let delta = target_offset - current_offset;
    let distance = delta.length();
    let max_step = (max_speed_mps * dt.max(0.0)).max(0.0);
    if distance <= max_step || distance <= 0.001 || !distance.is_finite() {
        return target_offset;
    }
    if max_step <= 0.0 {
        return current_offset;
    }

    current_offset + delta / distance * max_step
}

pub fn avoid_camera_obstructions(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
) -> CameraObstructionResolution {
    avoid_camera_obstructions_with_preferred_offset(frame, obstructions, clearance, None)
}

pub fn avoid_camera_obstructions_with_preferred_offset(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
    preferred_offset: Option<Vec3>,
) -> CameraObstructionResolution {
    avoid_camera_obstructions_with_distance_policy(
        frame,
        obstructions,
        clearance,
        preferred_offset,
        CameraObstructionDistancePolicy::Undecided,
        None,
    )
}

fn avoid_camera_obstructions_with_distance_policy(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
    preferred_offset: Option<Vec3>,
    distance_policy: CameraObstructionDistancePolicy,
    preferred_side: Option<f32>,
) -> CameraObstructionResolution {
    let obstructions = obstructions.into_iter().collect::<Vec<_>>();
    let segment = frame.position - frame.look_target;
    let segment_length = segment.length();
    if segment_length <= 0.001 || !segment_length.is_finite() {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count: 0,
        };
    }

    let direction = segment / segment_length;
    let mut nearest_hit_distance = segment_length;
    let mut hit_count = 0;

    for obstruction in obstructions.iter().copied() {
        if !camera_obstruction_blocks_boom(obstruction) {
            continue;
        }
        let obstruction = obstruction.expanded(clearance);
        if obstruction.contains(frame.look_target) {
            continue;
        }
        let Some(hit_distance) =
            segment_aabb_hit_distance(frame.look_target, direction, segment_length, obstruction)
        else {
            continue;
        };
        hit_count += 1;
        if hit_distance < nearest_hit_distance {
            nearest_hit_distance = hit_distance;
        }
    }

    if hit_count == 0 || nearest_hit_distance >= segment_length {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count,
        };
    }

    let adjusted_distance = if nearest_hit_distance > CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M {
        nearest_hit_distance - CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M
    } else {
        nearest_hit_distance * 0.5
    };
    let adjusted = obstruction_shortened_frame(frame, direction, adjusted_distance);

    let minimum_centered_distance = match distance_policy {
        CameraObstructionDistancePolicy::PreserveReadable => {
            CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
        }
        CameraObstructionDistancePolicy::Compact => {
            CAMERA_OBSTRUCTION_NEAR_TARGET_FALLBACK_CUTOFF_M
        }
        CameraObstructionDistancePolicy::Undecided => CAMERA_MIN_COMPACT_OBSTRUCTION_DISTANCE_M,
    };
    if adjusted_distance >= minimum_centered_distance {
        return CameraObstructionResolution {
            frame: adjusted,
            adjusted_distance_m: frame.position.distance(adjusted.position),
            hit_count,
        };
    }

    if adjusted_distance < CAMERA_OBSTRUCTION_NEAR_TARGET_FALLBACK_CUTOFF_M {
        let readable_distance = CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M.min(segment_length);
        let readable = obstruction_shortened_frame(frame, direction, readable_distance);
        return CameraObstructionResolution {
            frame: readable,
            adjusted_distance_m: frame.position.distance(readable.position),
            hit_count,
        };
    }

    if let Some(fallback) = readable_obstruction_fallback(
        frame,
        &obstructions,
        clearance,
        preferred_offset,
        preferred_side,
    ) {
        return CameraObstructionResolution {
            frame: fallback,
            adjusted_distance_m: frame.position.distance(fallback.position),
            hit_count,
        };
    }

    let readable_distance = CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M.min(segment_length);
    let readable = obstruction_shortened_frame(frame, direction, readable_distance);
    CameraObstructionResolution {
        frame: readable,
        adjusted_distance_m: frame.position.distance(readable.position),
        hit_count,
    }
}

pub fn revalidate_camera_obstruction(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
    preferred_offset: Option<Vec3>,
) -> CameraObstructionResolution {
    revalidate_camera_obstruction_with_distance_policy(
        frame,
        obstructions,
        clearance,
        preferred_offset,
        CameraObstructionDistancePolicy::Undecided,
        None,
    )
}

fn revalidate_camera_obstruction_with_distance_policy(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
    preferred_offset: Option<Vec3>,
    distance_policy: CameraObstructionDistancePolicy,
    preferred_side: Option<f32>,
) -> CameraObstructionResolution {
    let Some(preferred_offset) = preferred_offset else {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count: 0,
        };
    };
    let resolution = avoid_camera_obstructions_with_distance_policy(
        frame,
        obstructions,
        clearance,
        Some(preferred_offset),
        distance_policy,
        preferred_side,
    );
    if camera_obstruction_is_active(resolution.hit_count, resolution.adjusted_distance_m) {
        resolution
    } else {
        CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count: 0,
        }
    }
}

fn camera_obstruction_blocks_boom(obstruction: CameraObstruction) -> bool {
    !obstruction.is_local_prop()
}

fn obstruction_shortened_frame(
    frame: CameraFrame,
    direction: Vec3,
    adjusted_distance: f32,
) -> CameraFrame {
    let mut adjusted = frame;
    adjusted.position = frame.look_target + direction * adjusted_distance;
    adjusted.rotation = Transform::from_translation(adjusted.position)
        .looking_at(adjusted.look_target, Vec3::Y)
        .rotation;
    adjusted
}

fn camera_obstruction_orbit_deviation_degrees(intended: CameraFrame, resolved: CameraFrame) -> f32 {
    let intended_horizontal = Vec2::new(
        intended.position.x - intended.look_target.x,
        intended.position.z - intended.look_target.z,
    )
    .normalize_or_zero();
    let resolved_horizontal = Vec2::new(
        resolved.position.x - resolved.look_target.x,
        resolved.position.z - resolved.look_target.z,
    )
    .normalize_or_zero();
    if intended_horizontal.length_squared() <= 0.0001
        || resolved_horizontal.length_squared() <= 0.0001
    {
        return 0.0;
    }

    intended_horizontal
        .dot(resolved_horizontal)
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

fn readable_obstruction_fallback(
    frame: CameraFrame,
    obstructions: &[CameraObstruction],
    clearance: f32,
    preferred_offset: Option<Vec3>,
    preferred_side: Option<f32>,
) -> Option<CameraFrame> {
    let target_to_camera = frame.position - frame.look_target;
    let boom_distance = target_to_camera.length();
    if boom_distance < CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M || !boom_distance.is_finite() {
        return None;
    }

    let direction = target_to_camera / boom_distance;
    let lateral = direction.cross(Vec3::Y).normalize_or_zero();
    let horizontal_distance = Vec2::new(target_to_camera.x, target_to_camera.z).length();
    if lateral.length_squared() <= 0.0001 || horizontal_distance <= 0.001 {
        return None;
    }

    let max_yaw = (CAMERA_OBSTRUCTION_WIDE_SHOULDER_OFFSET_M / horizontal_distance).atan();
    let yaw_step = CAMERA_OBSTRUCTION_FALLBACK_YAW_STEP_DEGREES.to_radians();
    let yaw_step_count = (max_yaw / yaw_step).ceil() as usize;

    let mut best_candidate = None;
    let mut best_preferred_distance = f32::MAX;

    for vertical_scale in [0.0, 0.65, 1.0] {
        let vertical_offset = Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * vertical_scale);
        for yaw_step_index in 0..=yaw_step_count {
            let yaw = ((yaw_step_index as f32) * yaw_step).min(max_yaw);
            for yaw_sign in [-1.0, 1.0] {
                let candidate_offset =
                    Quat::from_rotation_y(yaw * yaw_sign) * target_to_camera + vertical_offset;
                let candidate_side = (candidate_offset - target_to_camera).dot(lateral);
                if preferred_side.is_some_and(|preferred_side| {
                    candidate_side.abs() <= CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M
                        || candidate_side.signum() != preferred_side.signum()
                }) {
                    continue;
                }
                let mut candidate = frame;
                candidate.position = frame.look_target + candidate_offset;
                if camera_segment_is_blocked(candidate, obstructions.iter().copied(), clearance) {
                    continue;
                }
                candidate.rotation = Transform::from_translation(candidate.position)
                    .looking_at(candidate.look_target, Vec3::Y)
                    .rotation;

                let preferred_distance = preferred_offset.map_or_else(
                    || candidate_offset.distance(target_to_camera),
                    |preferred_offset| {
                        obstruction_candidate_preference_score(
                            candidate_offset,
                            preferred_offset,
                            lateral,
                        )
                    },
                );
                if preferred_distance < best_preferred_distance {
                    best_preferred_distance = preferred_distance;
                    best_candidate = Some(candidate);
                }
            }
        }
    }

    best_candidate.or_else(|| {
        preferred_side.and_then(|_| {
            readable_obstruction_fallback(frame, obstructions, clearance, preferred_offset, None)
        })
    })
}

fn obstruction_candidate_preference_score(
    candidate_offset: Vec3,
    preferred_offset: Vec3,
    lateral: Vec3,
) -> f32 {
    let preferred_distance = candidate_offset.distance(preferred_offset);
    let preferred_lateral = preferred_offset.dot(lateral);
    if preferred_lateral.abs() <= CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M {
        return preferred_distance;
    }

    let candidate_lateral = candidate_offset.dot(lateral);
    let same_side = candidate_lateral.abs() > CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M
        && candidate_lateral.signum() == preferred_lateral.signum();
    if same_side {
        preferred_distance
    } else {
        preferred_distance + CAMERA_OBSTRUCTION_SIDE_SWITCH_PENALTY_M
    }
}

fn camera_segment_is_blocked(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
) -> bool {
    let segment = frame.position - frame.look_target;
    let segment_length = segment.length();
    if segment_length <= 0.001 || !segment_length.is_finite() {
        return false;
    }

    let direction = segment / segment_length;
    obstructions.into_iter().any(|obstruction| {
        if !camera_obstruction_blocks_boom(obstruction) {
            return false;
        }
        let obstruction = obstruction.expanded(clearance);
        !obstruction.contains(frame.look_target)
            && segment_aabb_hit_distance(frame.look_target, direction, segment_length, obstruction)
                .is_some()
    })
}

fn segment_aabb_hit_distance(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    obstruction: CameraObstruction,
) -> Option<f32> {
    let min = obstruction.center - obstruction.half_extents;
    let max = obstruction.center + obstruction.half_extents;
    let mut t_min = 0.0;
    let mut t_max = max_distance;

    update_slab_interval(origin.x, direction.x, min.x, max.x, &mut t_min, &mut t_max)?;
    update_slab_interval(origin.y, direction.y, min.y, max.y, &mut t_min, &mut t_max)?;
    update_slab_interval(origin.z, direction.z, min.z, max.z, &mut t_min, &mut t_max)?;

    if t_min <= max_distance && t_max >= 0.0 {
        Some(t_min.max(0.0))
    } else {
        None
    }
}

fn update_slab_interval(
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
    t_min: &mut f32,
    t_max: &mut f32,
) -> Option<()> {
    if direction.abs() <= 0.0001 {
        return (origin >= min && origin <= max).then_some(());
    }

    let inverse_direction = direction.recip();
    let mut near = (min - origin) * inverse_direction;
    let mut far = (max - origin) * inverse_direction;
    if near > far {
        std::mem::swap(&mut near, &mut far);
    }

    *t_min = (*t_min).max(near);
    *t_max = (*t_max).min(far);
    (*t_min <= *t_max).then_some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obstruction_release_eases_large_clear_frame_and_expires_preference() {
        let mut state = CameraObstructionSmoothingState::default();
        let blocked = CameraFrame {
            position: Vec3::new(3.0, 6.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        smooth_camera_obstruction(blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        let clear = CameraFrame {
            position: Vec3::new(0.0, 5.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: blocked.look_target,
        };
        let released = smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 60.0);

        assert!(
            released.position.distance(blocked.position) <= 0.301,
            "first clear frame after obstruction should ease out of held offset instead of snapping"
        );
        assert!(
            released.position.distance(clear.position) < blocked.position.distance(clear.position),
            "release smoothing should still move toward the clear follow frame"
        );
        assert!(
            state.readable_offset().is_some(),
            "brief clear gaps should keep the previous readable side for obstruction flicker"
        );

        for _ in 0..15 {
            smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 60.0);
        }
        assert!(state.readable_offset().is_none());
    }

    #[test]
    fn obstruction_release_keeps_easing_after_preferred_side_expires() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let blocked = CameraFrame {
            position: look_target + Vec3::new(12.0, 4.0, 18.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        let clear = CameraFrame {
            position: look_target + Vec3::new(-12.0, 3.0, -18.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let mut previous = blocked;
        for _ in 0..15 {
            previous = smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 60.0);
        }
        assert!(
            state.readable_offset().is_none(),
            "preferred obstruction side should still expire for future blockers"
        );

        let continued = smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 60.0);

        assert!(
            continued.position.distance(clear.position) > 0.001,
            "release easing should continue after preferred-side expiry when the clear frame is still far"
        );
        assert!(
            continued.position.distance(clear.position)
                < previous.position.distance(clear.position),
            "post-expiry release smoothing should still move toward the clear frame"
        );
    }

    #[test]
    fn brief_obstruction_flicker_reuses_previous_readable_offset() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let first_blocked = CameraFrame {
            position: look_target + Vec3::new(-4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(first_blocked, &mut state, 1, 4.0, 1.0 / 60.0);
        let preferred_offset = state.readable_offset().expect("held offset");

        let clear = CameraFrame {
            position: look_target + Vec3::new(0.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 60.0);

        let opposite_blocked = CameraFrame {
            position: look_target + Vec3::new(4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let smoothed = smooth_camera_obstruction(opposite_blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        assert!(
            smoothed.position.distance(look_target + preferred_offset) < 1.0,
            "camera should not flip shoulders after a one-frame obstruction miss"
        );
    }

    #[test]
    fn tiny_obstruction_adjustments_do_not_hold_or_smooth_offset() {
        let mut state = CameraObstructionSmoothingState::default();
        let frame = CameraFrame {
            position: Vec3::new(1.0, 5.0, 8.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let smoothed = smooth_camera_obstruction(frame, &mut state, 1, 0.1, 1.0 / 60.0);
        state.sync_resolved_frame(frame, 1, 0.1);

        assert_eq!(smoothed.position, frame.position);
        assert!(
            state.readable_offset().is_none(),
            "tiny obstruction contacts should not create sticky camera memory"
        );
    }

    #[test]
    fn obstruction_resolution_prefers_previous_readable_fallback() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 5.0), Vec3::new(2.05, 0.8, 0.2));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let default_resolution = avoid_camera_obstructions(frame, [blocker], 0.0);
        assert!(
            default_resolution.frame.position.y > frame.position.y,
            "default fallback should keep the existing readable fallback order"
        );

        let preferred_resolution = avoid_camera_obstructions_with_preferred_offset(
            frame,
            [blocker],
            0.0,
            Some(Vec3::new(
                -CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M,
                0.0,
                10.0,
            )),
        );

        assert!(
            preferred_resolution.frame.position.x < -2.0,
            "active obstruction should keep using the prior readable shoulder when it remains clear"
        );
    }

    #[test]
    fn readable_fallback_uses_smallest_clear_orbit_instead_of_wide_shoulder_jump() {
        let look_target = Vec3::new(0.0, 1.4, 0.0);
        let frame = CameraFrame {
            position: look_target + Vec3::new(0.0, 3.6, 12.5),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let blocker = CameraObstruction::new(Vec3::new(0.0, 4.0, 6.0), Vec3::new(1.75, 3.0, 2.0));

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);
        let intended_view = Vec3::new(
            frame.look_target.x - frame.position.x,
            0.0,
            frame.look_target.z - frame.position.z,
        )
        .normalize();
        let resolved_view = Vec3::new(
            resolved.frame.look_target.x - resolved.frame.position.x,
            0.0,
            resolved.frame.look_target.z - resolved.frame.position.z,
        )
        .normalize();
        let orbit_deviation_degrees = intended_view.angle_between(resolved_view).to_degrees();

        assert_eq!(resolved.hit_count, 1);
        assert!(
            !camera_segment_is_blocked(resolved.frame, [blocker], 0.0),
            "fallback should restore a clear target-to-camera segment"
        );
        assert!(
            orbit_deviation_degrees <= 26.0,
            "fallback should search between shoulder presets instead of jumping wide; deviation was {orbit_deviation_degrees}"
        );
        assert!(
            (resolved.frame.position.distance(look_target) - frame.position.distance(look_target))
                .abs()
                <= 0.001,
            "yaw fallback should preserve boom distance"
        );
    }

    #[test]
    fn moderate_close_obstruction_shortens_before_orbiting_sideways() {
        let look_target = Vec3::new(0.0, 1.4, 0.0);
        let frame = CameraFrame {
            position: look_target + Vec3::new(0.0, 3.6, 12.5),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let blocker = CameraObstruction::new(Vec3::new(0.0, 4.0, 7.0), Vec3::new(2.1, 3.0, 2.0));

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);
        let resolved_distance = resolved.frame.position.distance(look_target);

        assert_eq!(resolved.hit_count, 1);
        assert!(
            resolved.frame.position.x.abs() <= 0.001,
            "a readable centered zoom should not steer the camera around the blocker"
        );
        assert!(
            (CAMERA_MIN_COMPACT_OBSTRUCTION_DISTANCE_M..CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M)
                .contains(&resolved_distance),
            "moderate obstruction should use the compact readable range; distance was {resolved_distance}"
        );
        assert!(
            !camera_segment_is_blocked(resolved.frame, [blocker], 0.0),
            "shortened camera should stop in front of the blocker"
        );
    }

    #[test]
    fn obstruction_distance_policy_does_not_flip_mid_encounter() {
        let look_target = Vec3::new(0.0, 1.4, 0.0);
        let frame = CameraFrame {
            position: look_target + Vec3::new(0.0, 3.6, 12.5),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let readable_blocker =
            CameraObstruction::new(Vec3::new(0.0, 4.0, 9.0), Vec3::new(2.1, 3.0, 1.0));
        let close_blocker =
            CameraObstruction::new(Vec3::new(0.0, 4.0, 7.0), Vec3::new(2.1, 3.0, 1.8));
        let mut state = CameraObstructionHandoffState::default();
        state.set_intentional_camera_motion(true);

        let readable = resolve_camera_obstruction_handoff(
            frame,
            frame.position,
            frame.rotation,
            Vec3::ZERO,
            [readable_blocker],
            0.0,
            1.0,
            &mut state,
            |frame| frame,
        );

        assert_eq!(
            state.distance_policy,
            CameraObstructionDistancePolicy::PreserveReadable
        );
        assert!(
            readable.frame.position.distance(look_target)
                >= CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
        );

        state.set_intentional_camera_motion(true);
        let compacted = resolve_camera_obstruction_handoff(
            frame,
            readable.frame.position,
            readable.frame.rotation,
            Vec3::ZERO,
            [close_blocker],
            0.0,
            1.0,
            &mut state,
            |frame| frame,
        );

        assert_eq!(
            state.distance_policy,
            CameraObstructionDistancePolicy::Compact,
            "an encounter should switch once to compact zoom when preserving distance would exceed the orbit cap"
        );
        assert!(
            compacted.frame.position.distance(look_target)
                < CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M,
            "orbit-capped correction should compress the boom instead of steering around the blocker"
        );
        assert!(
            compacted.frame.position.x.abs() <= 0.001,
            "compact correction should remain centered on the requested orbit"
        );

        let mut compact_state = CameraObstructionHandoffState::default();
        compact_state.set_intentional_camera_motion(true);
        let compact = resolve_camera_obstruction_handoff(
            frame,
            frame.position,
            frame.rotation,
            Vec3::ZERO,
            [close_blocker],
            0.0,
            1.0,
            &mut compact_state,
            |frame| frame,
        );

        assert_eq!(
            compact_state.distance_policy,
            CameraObstructionDistancePolicy::Compact
        );
        assert!(
            compact.frame.position.x.abs() <= 0.001,
            "an encounter that begins close should stay centered instead of jumping shoulders"
        );
    }

    #[test]
    fn approaching_launch_spire_does_not_oscillate_camera_yaw() {
        let route = crate::world::SkyRoute::default();
        let spire = crate::world::route_obstruction_spire(0, route.islands()[0]);
        let blocker = CameraObstruction::new(spire.center, spire.half_extents);
        let mut state = CameraObstructionHandoffState::default();
        let mut previous_position = Vec3::new(0.0, 33.0, 12.0);
        let mut previous_rotation = Transform::from_translation(previous_position)
            .looking_at(Vec3::new(0.0, 29.4, -0.5), Vec3::Y)
            .rotation;
        let mut yaw_samples = Vec::new();

        for frame_index in 0..=70 {
            let player_position = Vec3::new(0.0, 28.0, frame_index as f32 * (7.2 / 70.0));
            let look_target = player_position + Vec3::new(0.0, 1.4, -0.5);
            let desired = CameraFrame {
                position: player_position + Vec3::new(0.0, 5.0, 12.0),
                rotation: Transform::from_translation(player_position + Vec3::new(0.0, 5.0, 12.0))
                    .looking_at(look_target, Vec3::Y)
                    .rotation,
                look_target,
            };
            if frame_index == 0 {
                state.set_intentional_camera_motion(true);
            }

            let resolved = resolve_camera_obstruction_handoff(
                desired,
                previous_position,
                previous_rotation,
                player_position,
                [blocker],
                0.45,
                1.0 / 60.0,
                &mut state,
                |frame| frame,
            );
            let horizontal_view = Vec3::new(
                resolved.frame.rotation.mul_vec3(Vec3::NEG_Z).x,
                0.0,
                resolved.frame.rotation.mul_vec3(Vec3::NEG_Z).z,
            )
            .normalize_or_zero();
            let yaw = Vec3::NEG_Z
                .cross(horizontal_view)
                .y
                .atan2(Vec3::NEG_Z.dot(horizontal_view))
                .to_degrees();
            yaw_samples.push(yaw);
            previous_position = resolved.frame.position;
            previous_rotation = resolved.frame.rotation;
        }

        let mut previous_direction = 0.0_f32;
        let mut direction_reversals = 0;
        for pair in yaw_samples.windows(2) {
            let direction = pair[1] - pair[0];
            if direction.abs() <= 0.2 {
                continue;
            }
            if previous_direction.abs() > 0.2 && direction.signum() != previous_direction.signum() {
                direction_reversals += 1;
            }
            previous_direction = direction;
        }

        assert!(
            direction_reversals <= 3,
            "obstruction correction should commit to a stable path; reversals={direction_reversals}, yaw={yaw_samples:?}"
        );
        let max_abs_yaw = yaw_samples
            .iter()
            .copied()
            .map(f32::abs)
            .fold(0.0, f32::max);
        assert!(
            max_abs_yaw <= CAMERA_MAX_OBSTRUCTION_ORBIT_DEVIATION_DEGREES + 0.25,
            "environmental correction should zoom instead of steering beyond the orbit contract; max yaw was {max_abs_yaw}"
        );
    }

    #[test]
    fn obstruction_resolution_keeps_clamped_shoulder_preference() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 5.0), Vec3::new(2.05, 0.8, 0.2));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let preferred_resolution = avoid_camera_obstructions_with_preferred_offset(
            frame,
            [blocker],
            0.0,
            Some(Vec3::new(
                -CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M * 2.0,
                0.0,
                10.0,
            )),
        );

        assert!(
            preferred_resolution.frame.position.x < -CAMERA_OBSTRUCTION_SIDE_PREFERENCE_MIN_M,
            "a partially clamped shoulder offset should not oscillate back through center; x was {}",
            preferred_resolution.frame.position.x
        );
    }

    #[test]
    fn soft_local_prop_obstruction_does_not_preserve_previous_readable_offset() {
        let blocker =
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.0, 5.0), Vec3::new(2.0, 6.0, 1.0));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };
        let preferred_offset = Vec3::new(-5.5, 3.0, 9.0);

        let resolved = avoid_camera_obstructions_with_preferred_offset(
            frame,
            [blocker],
            0.0,
            Some(preferred_offset),
        );

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(
            resolved.frame.position, frame.position,
            "soft props should not inherit stale obstruction shoulders"
        );
    }

    #[test]
    fn soft_local_prop_obstruction_drops_stale_blocked_readable_offset() {
        let blocker =
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.0, 4.0), Vec3::new(2.0, 2.0, 2.0));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };
        let stale_blocked_offset = Vec3::new(1.5, 2.0, 10.0);

        let resolved = avoid_camera_obstructions_with_preferred_offset(
            frame,
            [blocker],
            0.0,
            Some(stale_blocked_offset),
        );

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(
            resolved.frame.position, frame.position,
            "close local prop transparency should not hold a stale shoulder that is now blocked"
        );
    }

    #[test]
    fn soft_local_prop_obstruction_does_not_create_new_readable_fallback() {
        let blocker =
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.0, 4.0), Vec3::new(0.6, 2.0, 0.6));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 12.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(
            resolved.frame.position, frame.position,
            "tree-sized soft props should not create a fresh shoulder/vertical camera fallback"
        );
    }

    #[test]
    fn far_soft_local_prop_obstruction_does_not_shorten_camera_boom() {
        let blocker =
            CameraObstruction::soft_local_prop(Vec3::new(0.0, 2.0, 8.0), Vec3::splat(0.8));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 14.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(
            resolved.frame.position, frame.position,
            "soft tree/canopy props should not zoom the camera even when the hit is readable"
        );
    }

    #[test]
    fn local_prop_obstruction_does_not_shorten_camera_boom() {
        let blocker =
            CameraObstruction::local_prop(Vec3::new(0.0, 2.0, 8.0), Vec3::new(1.1, 3.0, 1.1));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 14.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(
            resolved.frame.position, frame.position,
            "local marker props should not steer or zoom the camera"
        );
    }

    #[test]
    fn obstruction_resolution_prefers_centered_shortening_when_still_readable() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 9.0), Vec3::new(2.05, 1.0, 1.0));
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 14.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);

        assert_eq!(resolved.hit_count, 1);
        assert!(resolved.frame.position.distance(frame.look_target) > 7.0);
        assert_eq!(resolved.frame.position.x, frame.position.x);
        assert_eq!(resolved.frame.position.y, frame.position.y);
        assert!(resolved.frame.position.z < blocker.center.z);
    }

    #[test]
    fn obstruction_handoff_clamps_first_hard_fallback_frame() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_position = Vec3::new(0.0, 2.0, 10.0);
        let previous_rotation = Transform::from_translation(previous_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let frame = CameraFrame {
            position: previous_position,
            rotation: previous_rotation,
            look_target,
        };
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 5.0), Vec3::new(2.05, 0.8, 1.0));
        let mut handoff = CameraObstructionHandoffState::default();

        let step = resolve_camera_obstruction_handoff(
            frame,
            previous_position,
            previous_rotation,
            Vec3::ZERO,
            [blocker],
            0.0,
            1.0 / 60.0,
            &mut handoff,
            |frame| frame,
        );

        assert!(step.obstruction_hits > 0);
        assert!(step.obstruction_adjustment_m >= CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M);
        assert!(
            previous_position.distance(step.frame.position)
                <= CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS / 60.0 + 0.001,
            "first hard-obstruction fallback should be capped instead of snapping"
        );
        let rotation_delta_degrees = previous_rotation
            .angle_between(step.frame.rotation)
            .to_degrees();
        assert!(
            rotation_delta_degrees
                <= CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0 + 0.001,
            "first hard-obstruction fallback should stay inside the camera jitter gate; delta was {rotation_delta_degrees}"
        );
        assert!(step.continuity_offset_limited);
        assert!(step.continuity_rotation_limited);
    }

    #[test]
    fn near_target_obstruction_never_collapses_below_readable_distance() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let frame = CameraFrame {
            position: look_target + Vec3::Z * 13.0,
            rotation: Quat::IDENTITY,
            look_target,
        };
        let blocker =
            CameraObstruction::new(look_target + Vec3::Z * 2.0, Vec3::new(100.0, 100.0, 0.75));

        let resolved = avoid_camera_obstructions(frame, [blocker], 0.0);

        assert_eq!(resolved.hit_count, 1);
        assert!(
            (resolved.frame.position.distance(look_target)
                - CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M)
                .abs()
                <= 0.001
        );
    }

    #[test]
    fn release_handoff_stays_active_for_post_offset_rotation_debt() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_position = look_target + Vec3::Z * 14.0;
        let previous_rotation = Transform::from_translation(previous_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let target_position = look_target
            + Quat::from_rotation_y(4.15_f32.to_radians()) * (previous_position - look_target);
        let first_target = CameraFrame {
            position: target_position,
            rotation: previous_rotation,
            look_target,
        };
        let mut handoff = CameraObstructionHandoffState {
            release_handoff_remaining_secs: CAMERA_OBSTRUCTION_RELEASE_HANDOFF_SECS,
            previous_look_target: Some(look_target),
            ..Default::default()
        };

        let first = resolve_camera_obstruction_handoff(
            first_target,
            previous_position,
            previous_rotation,
            look_target - Vec3::Y * 1.4,
            [],
            0.0,
            1.0 / 60.0,
            &mut handoff,
            |frame| frame,
        );
        assert!(first.obstruction_hits > 0);
        assert!(
            previous_position.distance(first.frame.position)
                <= CAMERA_MAX_OBSTRUCTION_HANDOFF_OFFSET_SPEED_MPS / 60.0 + 0.001
        );
        assert!(
            previous_rotation
                .angle_between(first.frame.rotation)
                .to_degrees()
                <= CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0 + 0.001
        );

        let clear_target = CameraFrame {
            position: target_position,
            rotation: Transform::from_translation(target_position)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        let second = resolve_camera_obstruction_handoff(
            clear_target,
            first.frame.position,
            first.frame.rotation,
            look_target - Vec3::Y * 1.4,
            [],
            0.0,
            1.0 / 60.0,
            &mut handoff,
            |frame| frame,
        );
        let second_rotation_delta = first
            .frame
            .rotation
            .angle_between(second.frame.rotation)
            .to_degrees();

        assert!(
            second.obstruction_hits > 0,
            "handoff must remain active until post-offset rotation debt is settled"
        );
        assert!(
            second_rotation_delta
                <= CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0 + 0.001,
            "release rotation delta exceeded jitter gate: {second_rotation_delta}"
        );
    }

    #[test]
    fn clear_follow_handoff_does_not_world_step_clamp_fast_player_motion() {
        let previous_look_target = Vec3::new(0.0, 120.0, 0.0);
        let previous_position = previous_look_target + Vec3::new(0.0, 5.0, 12.0);
        let look_target = Vec3::new(0.0, 96.0, 0.0);
        let frame = CameraFrame {
            position: look_target + Vec3::new(0.0, 5.0, 12.0),
            rotation: Transform::from_translation(look_target + Vec3::new(0.0, 5.0, 12.0))
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        let previous_rotation = Transform::from_translation(previous_position)
            .looking_at(previous_look_target, Vec3::Y)
            .rotation;
        let mut handoff = CameraObstructionHandoffState {
            previous_look_target: Some(previous_look_target),
            ..Default::default()
        };

        let step = resolve_camera_obstruction_handoff(
            frame,
            previous_position,
            previous_rotation,
            look_target - Vec3::Y * 1.4,
            [],
            0.0,
            1.0 / 60.0,
            &mut handoff,
            |frame| frame,
        );

        assert_eq!(step.obstruction_hits, 0);
        assert_eq!(step.obstruction_adjustment_m, 0.0);
        assert!(
            step.frame.position.distance(frame.position) < 0.001,
            "clear follow frames should inherit fast player motion instead of consuming it in the obstruction step cap"
        );
    }

    #[test]
    fn obstruction_offset_clamp_preserves_target_motion_while_smoothing_boom() {
        let previous_look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_position = previous_look_target + Vec3::new(0.0, 4.0, 12.0);
        let look_target = Vec3::new(18.0, 2.0, -8.0);
        let centered_target = CameraFrame {
            position: look_target + Vec3::new(0.0, 4.0, 7.0),
            rotation: Transform::from_translation(look_target + Vec3::new(0.0, 4.0, 7.0))
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };

        let clamped = clamp_camera_offset_step(
            centered_target,
            previous_position,
            Some(previous_look_target),
            CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS / 60.0,
        );
        let target_forward = Vec3::new(
            clamped.look_target.x - clamped.position.x,
            0.0,
            clamped.look_target.z - clamped.position.z,
        )
        .normalize();

        assert!(
            target_forward.angle_between(Vec3::NEG_Z).to_degrees() <= 0.1,
            "target-relative obstruction clamp should inherit player motion instead of leaving yaw drift"
        );
        assert_eq!(clamped.position.x, look_target.x);
        assert_eq!(clamped.position.y, look_target.y + 4.0);
        let expected_boom_z = 12.0 - CAMERA_MAX_OBSTRUCTION_OFFSET_SPEED_MPS / 60.0;
        assert!(
            (clamped.position.z - (look_target.z + expected_boom_z)).abs() <= 0.001,
            "boom z offset should move by the obstruction cap, not consume the player target movement"
        );
    }

    #[test]
    fn active_obstruction_transitions_toward_new_readable_offset_without_snapping() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let first_blocked = CameraFrame {
            position: look_target + Vec3::new(4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(first_blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        let opposite_blocked = CameraFrame {
            position: look_target + Vec3::new(-4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let smoothed = smooth_camera_obstruction(opposite_blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        assert!(smoothed.position.x < first_blocked.position.x);
        assert!(smoothed.position.x > opposite_blocked.position.x);
        assert!(smoothed.position.distance(first_blocked.position) < 1.0);
        assert_eq!(
            state.readable_offset(),
            Some(smoothed.position - look_target)
        );
    }

    #[test]
    fn active_obstruction_transition_keeps_rotation_delta_below_jitter_gate() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let first_position = look_target + Vec3::new(0.0, 3.0, 5.4);
        let first_blocked = CameraFrame {
            position: first_position,
            rotation: Transform::from_translation(first_position)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        smooth_camera_obstruction(first_blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        let next_position = look_target + Vec3::new(-4.0, 3.0, 5.4);
        let next_blocked = CameraFrame {
            position: next_position,
            rotation: Transform::from_translation(next_position)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        let smoothed = smooth_camera_obstruction(next_blocked, &mut state, 1, 4.0, 1.0 / 60.0);
        let rotation_delta_degrees = first_blocked
            .rotation
            .angle_between(smoothed.rotation)
            .to_degrees();

        assert!(
            rotation_delta_degrees <= 1.5,
            "active obstruction easing should stay below the camera jitter gate; delta was {rotation_delta_degrees}"
        );
    }

    #[test]
    fn obstruction_rotation_step_clamp_limits_final_camera_turn() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_rotation = Transform::from_translation(Vec3::new(0.0, 5.0, 8.0))
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let frame = CameraFrame {
            position: Vec3::new(-5.0, 5.0, 6.0),
            rotation: Transform::from_translation(Vec3::new(-5.0, 5.0, 6.0))
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };

        let clamped = clamp_camera_rotation_step(
            frame,
            previous_rotation,
            CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0,
        );
        let rotation_delta_degrees = previous_rotation
            .angle_between(clamped.rotation)
            .to_degrees();

        assert!(
            rotation_delta_degrees
                <= CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0 + 0.001,
            "obstruction rotation clamp should bound final camera rotation; delta was {rotation_delta_degrees}"
        );
        assert_eq!(clamped.position, frame.position);
        assert_eq!(clamped.look_target, frame.look_target);
    }

    #[test]
    fn environmental_continuity_contract_is_frame_rate_independent() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let initial_position = look_target + Vec3::Z * 12.0;
        let target_position = look_target + Vec3::X * 12.0;
        let target_frame = CameraFrame {
            position: target_position,
            rotation: Transform::from_translation(target_position)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        let initial_rotation = Transform::from_translation(initial_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let mut final_offsets = Vec::new();

        for frame_rate in [30.0, 60.0, 120.0, 144.0] {
            let dt = 1.0 / frame_rate;
            let mut position = initial_position;
            let mut rotation = initial_rotation;
            for _ in 0..(frame_rate as usize / 2) {
                let (frame, _, _) = enforce_camera_continuity(
                    target_frame,
                    position,
                    Some(look_target),
                    rotation,
                    dt,
                    CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS,
                    CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND,
                );
                position = frame.position;
                rotation = frame.rotation;
            }
            final_offsets.push(position - look_target);
        }

        for offset in final_offsets.iter().skip(1) {
            assert!(
                offset.distance(final_offsets[0]) <= 0.001,
                "equal wall-clock camera correction must not depend on render frame rate"
            );
        }
    }

    #[test]
    fn continuity_contract_matches_substeps_across_hitches() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let initial_position = look_target + Vec3::Z * 12.0;
        let target_position = look_target + Vec3::X * 12.0;
        let target_frame = CameraFrame {
            position: target_position,
            rotation: Transform::from_translation(target_position)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };
        let initial_rotation = Transform::from_translation(initial_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;

        for hitch_secs in [0.05, 0.1] {
            let (single_step, _, _) = enforce_camera_continuity(
                target_frame,
                initial_position,
                Some(look_target),
                initial_rotation,
                hitch_secs,
                CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS,
                CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND,
            );
            let substep_count = (hitch_secs * 60.0).round() as usize;
            let mut substep_position = initial_position;
            let mut substep_rotation = initial_rotation;
            for _ in 0..substep_count {
                let (frame, _, _) = enforce_camera_continuity(
                    target_frame,
                    substep_position,
                    Some(look_target),
                    substep_rotation,
                    1.0 / 60.0,
                    CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS,
                    CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND,
                );
                substep_position = frame.position;
                substep_rotation = frame.rotation;
            }

            assert!(single_step.position.distance(substep_position) <= 0.001);
            let rotation_partition_error_degrees = single_step
                .rotation
                .angle_between(substep_rotation)
                .to_degrees();
            assert!(
                rotation_partition_error_degrees <= 0.25,
                "hitch partition changed camera rotation by {rotation_partition_error_degrees} degrees"
            );
        }
    }

    #[test]
    fn zero_delta_time_never_advances_camera_correction() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_position = look_target + Vec3::Z * 12.0;
        let previous_rotation = Transform::from_translation(previous_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let frame = CameraFrame {
            position: look_target + Vec3::X * 12.0,
            rotation: Quat::from_rotation_y(1.0) * previous_rotation,
            look_target,
        };

        let (resolved, offset_limited, rotation_limited) = enforce_camera_continuity(
            frame,
            previous_position,
            Some(look_target),
            previous_rotation,
            0.0,
            CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS,
            CAMERA_MAX_ROTATION_SPEED_DEGREES_PER_SECOND,
        );

        assert_eq!(resolved.position, previous_position);
        assert_eq!(resolved.rotation, previous_rotation);
        assert!(offset_limited);
        assert!(rotation_limited);
    }

    #[test]
    fn continuity_contract_never_escapes_rotation_limit_to_recenter() {
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let previous_position = look_target + Vec3::Z * 12.0;
        let previous_rotation = Transform::from_translation(previous_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let frame = CameraFrame {
            position: look_target + Vec3::X * 12.0,
            rotation: Transform::from_translation(look_target + Vec3::X * 12.0)
                .looking_at(look_target, Vec3::Y)
                .rotation,
            look_target,
        };

        let (resolved, _, rotation_limited) = enforce_camera_continuity(
            frame,
            previous_position,
            Some(look_target),
            previous_rotation,
            1.0 / 60.0,
            CAMERA_MAX_RELATIVE_OFFSET_SPEED_MPS,
            CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND,
        );
        let rotation_delta = previous_rotation
            .angle_between(resolved.rotation)
            .to_degrees();

        assert!(rotation_limited);
        assert!(
            rotation_delta
                <= CAMERA_MAX_OBSTRUCTION_ROTATION_SPEED_DEGREES_PER_SECOND / 60.0 + 0.001
        );
    }

    #[test]
    fn revalidation_rejects_blocked_smoothed_camera_frame() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 5.0), Vec3::new(2.05, 0.8, 1.0));
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let smoothed = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        assert!(
            camera_segment_is_blocked(smoothed, [blocker], 0.0),
            "test setup should model a smoothed frame that still crosses the blocker"
        );

        let revalidated = revalidate_camera_obstruction(
            smoothed,
            [blocker],
            0.0,
            Some(Vec3::new(
                -CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M,
                0.0,
                10.0,
            )),
        );

        assert_eq!(revalidated.hit_count, 1);
        assert!(
            camera_obstruction_is_active(revalidated.hit_count, revalidated.adjusted_distance_m),
            "blocked smoothed frames should remain active obstruction samples"
        );
        assert!(
            !camera_segment_is_blocked(revalidated.frame, [blocker], 0.0),
            "revalidated camera frame should restore a clear line of sight"
        );
    }

    #[test]
    fn resolved_frame_sync_keeps_obstruction_memory_at_clamped_camera_offset() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let target = CameraFrame {
            position: look_target + Vec3::new(6.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(target, &mut state, 1, 4.0, 1.0 / 60.0);

        let clamped = CameraFrame {
            position: look_target + Vec3::new(1.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        state.sync_resolved_frame(clamped, 1, 4.0);

        assert_eq!(
            state.readable_offset(),
            Some(clamped.position - look_target)
        );
    }

    #[test]
    fn clear_release_sync_keeps_obstruction_memory_at_clamped_camera_offset() {
        let mut state = CameraObstructionSmoothingState::default();
        let look_target = Vec3::new(0.0, 2.0, 0.0);
        let blocked = CameraFrame {
            position: look_target + Vec3::new(4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        smooth_camera_obstruction(blocked, &mut state, 1, 4.0, 1.0 / 60.0);

        let clear = CameraFrame {
            position: look_target + Vec3::new(-4.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        let release_target = smooth_camera_obstruction(clear, &mut state, 0, 0.0, 1.0 / 30.0);
        assert_eq!(
            state.readable_offset(),
            Some(release_target.position - look_target),
            "release smoothing should first remember the eased target"
        );

        let clamped_release = CameraFrame {
            position: look_target + Vec3::new(3.0, 3.0, 9.0),
            rotation: Quat::IDENTITY,
            look_target,
        };
        state.sync_resolved_frame(clamped_release, 0, 0.0);

        assert_eq!(
            state.readable_offset(),
            Some(clamped_release.position - look_target),
            "clear release frames should keep obstruction memory at the actual post-clamp camera offset"
        );
    }
}
