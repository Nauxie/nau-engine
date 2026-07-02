use bevy::prelude::*;

use super::types::{CameraFrame, CameraObstruction, CameraObstructionResolution};

pub const CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M: f32 = 6.5;
pub const CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M: f32 = 4.8;
pub const CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M: f32 = 2.4;
pub const CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M: f32 = 2.4;
pub const CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M: f32 = 1.5;
pub const CAMERA_OBSTRUCTION_RELEASE_HOLD_SECS: f32 = 0.22;
const CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M: f32 = 0.08;
const CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_HORIZONTAL_HALF_EXTENT_M: f32 = 2.0;
const CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_VERTICAL_HALF_EXTENT_M: f32 = 6.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct CameraObstructionSmoothingState {
    held_offset: Vec3,
    release_remaining_secs: f32,
    obstructed_last_frame: bool,
}

impl CameraObstructionSmoothingState {
    pub fn readable_offset(self) -> Option<Vec3> {
        (self.held_offset.length_squared() > 0.001).then_some(self.held_offset)
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
        frame.rotation = Transform::from_translation(frame.position)
            .looking_at(frame.look_target, Vec3::Y)
            .rotation;
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

pub fn smooth_camera_obstruction(
    mut frame: CameraFrame,
    state: &mut CameraObstructionSmoothingState,
    obstruction_hits: usize,
    obstruction_adjustment_m: f32,
    dt: f32,
) -> CameraFrame {
    if obstruction_hits > 0 && obstruction_adjustment_m > 0.0 {
        let target_offset = frame.position - frame.look_target;

        state.held_offset = target_offset;
        state.release_remaining_secs = CAMERA_OBSTRUCTION_RELEASE_HOLD_SECS;
        state.obstructed_last_frame = true;
        return frame;
    }

    state.obstructed_last_frame = false;
    if state.release_remaining_secs <= 0.0 || state.held_offset.length_squared() <= 0.001 {
        state.release_remaining_secs = 0.0;
        return frame;
    }

    state.release_remaining_secs = (state.release_remaining_secs - dt.max(0.0))
        .clamp(0.0, CAMERA_OBSTRUCTION_RELEASE_HOLD_SECS);
    let hold_weight =
        (state.release_remaining_secs / CAMERA_OBSTRUCTION_RELEASE_HOLD_SECS).clamp(0.0, 1.0);
    let held_position = frame.look_target + state.held_offset;
    frame.position = frame.position.lerp(held_position, hold_weight);
    frame.rotation = Transform::from_translation(frame.position)
        .looking_at(frame.look_target, Vec3::Y)
        .rotation;
    frame
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
    let mut nearest_obstruction = None;
    let mut hit_count = 0;

    for obstruction in obstructions.iter().copied() {
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
            nearest_obstruction = Some(obstruction);
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

    if adjusted_distance >= CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M {
        return CameraObstructionResolution {
            frame: adjusted,
            adjusted_distance_m: frame.position.distance(adjusted.position),
            hit_count,
        };
    }

    if let Some(fallback) =
        readable_obstruction_fallback(frame, &obstructions, clearance, preferred_offset)
    {
        return CameraObstructionResolution {
            frame: fallback,
            adjusted_distance_m: frame.position.distance(fallback.position),
            hit_count,
        };
    }

    if nearest_hit_distance < CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M
        && nearest_obstruction.is_some_and(is_local_prop_blocker)
    {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count,
        };
    }

    CameraObstructionResolution {
        frame: adjusted,
        adjusted_distance_m: frame.position.distance(adjusted.position),
        hit_count,
    }
}

fn is_local_prop_blocker(obstruction: CameraObstruction) -> bool {
    obstruction.half_extents.x.max(obstruction.half_extents.z)
        <= CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_HORIZONTAL_HALF_EXTENT_M
        && obstruction.half_extents.y <= CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_VERTICAL_HALF_EXTENT_M
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

fn readable_obstruction_fallback(
    frame: CameraFrame,
    obstructions: &[CameraObstruction],
    clearance: f32,
    preferred_offset: Option<Vec3>,
) -> Option<CameraFrame> {
    let target_to_camera = frame.position - frame.look_target;
    let boom_distance = target_to_camera.length();
    if boom_distance < CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M || !boom_distance.is_finite() {
        return None;
    }

    let direction = target_to_camera / boom_distance;
    let lateral = direction.cross(Vec3::Y).normalize_or_zero();
    if lateral.length_squared() <= 0.0001 {
        return None;
    }

    let offsets = [
        Vec3::Y * CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M,
        lateral * CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M,
        -lateral * CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M,
        lateral * CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
        -lateral * CAMERA_OBSTRUCTION_SOFT_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
        lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M,
        -lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M,
        lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
        -lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
    ];

    let mut best_candidate = None;
    let mut best_preferred_distance = f32::MAX;

    for offset in offsets {
        let mut candidate = frame;
        candidate.position = frame.position + offset;
        if camera_segment_is_blocked(candidate, obstructions.iter().copied(), clearance) {
            continue;
        }
        candidate.rotation = Transform::from_translation(candidate.position)
            .looking_at(candidate.look_target, Vec3::Y)
            .rotation;

        let Some(preferred_offset) = preferred_offset else {
            return Some(candidate);
        };
        let preferred_distance =
            (candidate.position - candidate.look_target).distance(preferred_offset);
        if preferred_distance < best_preferred_distance {
            best_preferred_distance = preferred_distance;
            best_candidate = Some(candidate);
        }
    }

    best_candidate
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
    fn obstruction_release_holds_last_readable_offset_briefly() {
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
            released.position.x > 2.0,
            "camera should not immediately snap off the readable obstruction fallback"
        );
        assert!(released.position.distance(clear.position) > 2.0);
    }

    #[test]
    fn obstruction_resolution_prefers_previous_readable_fallback() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 5.0), Vec3::new(1.0, 0.8, 1.0));
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
    fn obstruction_resolution_prefers_centered_shortening_when_still_readable() {
        let blocker = CameraObstruction::new(Vec3::new(0.0, 2.0, 9.0), Vec3::new(1.0, 1.0, 1.0));
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
    fn active_obstruction_returns_clear_preferred_frame_without_extra_lag() {
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

        assert_eq!(smoothed.position, opposite_blocked.position);
        assert_eq!(
            state.readable_offset(),
            Some(opposite_blocked.position - look_target)
        );
    }
}
