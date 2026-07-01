use crate::camera::CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M;

use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_camera_distance_m = accumulator
        .max_camera_distance_m
        .max(sample.camera_distance_m);
    accumulator.min_camera_surface_clearance_m = accumulator
        .min_camera_surface_clearance_m
        .min(sample.camera_surface_clearance_m);
    accumulator.max_camera_player_angle_degrees = accumulator
        .max_camera_player_angle_degrees
        .max(sample.camera_player_angle_degrees);
    accumulator.max_camera_step_distance_m = accumulator
        .max_camera_step_distance_m
        .max(sample.camera_step_distance_m);
    accumulator.max_camera_rotation_delta_degrees = accumulator
        .max_camera_rotation_delta_degrees
        .max(sample.camera_rotation_delta_degrees);
    accumulator.max_camera_orbit_alignment_degrees = accumulator
        .max_camera_orbit_alignment_degrees
        .max(sample.camera_orbit_alignment_degrees);

    if sample.camera_follow_direction_error_degrees.is_finite() {
        accumulator.camera_follow_direction_error_sum_degrees +=
            sample.camera_follow_direction_error_degrees;
        accumulator.camera_follow_direction_error_samples += 1;
        accumulator
            .camera_follow_direction_error_values_degrees
            .push(sample.camera_follow_direction_error_degrees);
        accumulator.max_camera_follow_direction_error_degrees = accumulator
            .max_camera_follow_direction_error_degrees
            .max(sample.camera_follow_direction_error_degrees);
    }

    accumulator.max_abs_camera_view_yaw_degrees = accumulator
        .max_abs_camera_view_yaw_degrees
        .max(sample.camera_view_yaw_degrees.abs());
    if sample.camera_view_yaw_degrees.is_finite() {
        let first_yaw = accumulator
            .first_camera_view_yaw_degrees
            .get_or_insert(sample.camera_view_yaw_degrees);
        accumulator.max_camera_view_yaw_drift_degrees = accumulator
            .max_camera_view_yaw_drift_degrees
            .max((sample.camera_view_yaw_degrees - *first_yaw).abs());
    }

    if sample.camera_world_yaw_degrees.is_finite() {
        let first_world_yaw = accumulator
            .first_camera_world_yaw_degrees
            .get_or_insert(sample.camera_world_yaw_degrees);
        accumulator.max_camera_world_yaw_drift_degrees = accumulator
            .max_camera_world_yaw_drift_degrees
            .max((sample.camera_world_yaw_degrees - *first_world_yaw).abs());
    }

    accumulator.max_camera_obstruction_adjustment_m = accumulator
        .max_camera_obstruction_adjustment_m
        .max(sample.camera_obstruction_adjustment_m);
    accumulator.max_camera_obstruction_hits = accumulator
        .max_camera_obstruction_hits
        .max(sample.camera_obstruction_hits);
    if sample.camera_obstruction_hits > 0 {
        accumulator.min_camera_obstructed_distance_m = Some(
            accumulator
                .min_camera_obstructed_distance_m
                .map_or(sample.camera_distance_m, |distance| {
                    distance.min(sample.camera_distance_m)
                }),
        );
        if accumulator.previous_camera_obstructed_sample
            && sample.camera_step_distance_m > CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M
        {
            accumulator.camera_obstruction_snap_count += 1;
        }
        accumulator.previous_camera_obstructed_sample = true;
    } else {
        accumulator.previous_camera_obstructed_sample = false;
    }
    accumulator.min_camera_pitch_degrees = accumulator
        .min_camera_pitch_degrees
        .min(sample.camera_pitch_degrees);
    accumulator.max_camera_pitch_degrees = accumulator
        .max_camera_pitch_degrees
        .max(sample.camera_pitch_degrees);
    accumulator.max_abs_camera_yaw_offset_degrees = accumulator
        .max_abs_camera_yaw_offset_degrees
        .max(sample.camera_yaw_offset_degrees.abs());
    accumulator.min_camera_pitch_offset_degrees = accumulator
        .min_camera_pitch_offset_degrees
        .min(sample.camera_pitch_offset_degrees);
    accumulator.max_camera_pitch_offset_degrees = accumulator
        .max_camera_pitch_offset_degrees
        .max(sample.camera_pitch_offset_degrees);
}
