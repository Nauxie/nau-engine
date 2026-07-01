use bevy::prelude::*;

use super::types::{CameraFrame, CameraObstruction, CameraObstructionResolution};

pub const CAMERA_MIN_READABLE_OBSTRUCTION_DISTANCE_M: f32 = 6.5;
pub const CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M: f32 = 4.8;
pub const CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M: f32 = 2.4;
pub const CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M: f32 = 1.5;
const CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M: f32 = 0.08;
const CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_HORIZONTAL_HALF_EXTENT_M: f32 = 2.0;
const CAMERA_TRANSPARENT_NEAR_BLOCKER_MAX_VERTICAL_HALF_EXTENT_M: f32 = 6.0;

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

pub fn avoid_camera_obstructions(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
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

    if let Some(fallback) = readable_obstruction_fallback(frame, &obstructions, clearance) {
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

    let adjusted_distance = if nearest_hit_distance > CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M {
        nearest_hit_distance - CAMERA_OBSTRUCTION_FRONT_CLEARANCE_M
    } else {
        nearest_hit_distance * 0.5
    };
    let mut adjusted = frame;
    adjusted.position = frame.look_target + direction * adjusted_distance;
    adjusted.rotation = Transform::from_translation(adjusted.position)
        .looking_at(adjusted.look_target, Vec3::Y)
        .rotation;

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

fn readable_obstruction_fallback(
    frame: CameraFrame,
    obstructions: &[CameraObstruction],
    clearance: f32,
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
        lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M,
        -lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M,
        lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
        -lateral * CAMERA_OBSTRUCTION_SHOULDER_OFFSET_M
            + Vec3::Y * (CAMERA_OBSTRUCTION_VERTICAL_OFFSET_M * 0.65),
    ];

    offsets.into_iter().find_map(|offset| {
        let mut candidate = frame;
        candidate.position = frame.position + offset;
        if camera_segment_is_blocked(candidate, obstructions.iter().copied(), clearance) {
            return None;
        }
        candidate.rotation = Transform::from_translation(candidate.position)
            .looking_at(candidate.look_target, Vec3::Y)
            .rotation;
        Some(candidate)
    })
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
