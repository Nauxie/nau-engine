use crate::camera::CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M;

use super::{EvalAccumulator, EvalSample};

pub(super) struct ContinuityMetrics {
    pub(super) camera_distance_m: f32,
    pub(super) camera_step_distance_m: f32,
    pub(super) camera_player_relative_step_m: f32,
    pub(super) camera_rotation_delta_degrees: f32,
    pub(super) camera_player_relative_linear_velocity_mps: f32,
    pub(super) camera_player_relative_linear_acceleration_mps2: f32,
    pub(super) camera_player_relative_angular_velocity_degrees_per_sec: f32,
    pub(super) camera_player_relative_angular_acceleration_degrees_per_sec2: f32,
    pub(super) camera_obstruction_adjustment_m: f32,
    pub(super) camera_obstruction_hits: usize,
    pub(super) camera_correction_source: &'static str,
    pub(super) camera_continuity_offset_limited: bool,
    pub(super) camera_continuity_rotation_limited: bool,
    pub(super) player_integration_residual_m: f32,
    pub(super) player_world_correction_m: f32,
    pub(super) player_collision_correction_m: f32,
    pub(super) world_collision_correction_m: Option<f32>,
}

pub(super) fn observe_continuity(
    accumulator: &mut EvalAccumulator,
    frame: u32,
    metrics: ContinuityMetrics,
) {
    if accumulator.last_continuity_frame == Some(frame) {
        return;
    }
    if accumulator
        .last_continuity_frame
        .is_some_and(|last_frame| frame < last_frame)
    {
        accumulator.previous_camera_obstructed_sample = false;
    }
    accumulator.last_continuity_frame = Some(frame);

    observe_max(
        &mut accumulator.max_camera_step_distance_m,
        metrics.camera_step_distance_m,
    );
    observe_max(
        &mut accumulator.max_camera_rotation_delta_degrees,
        metrics.camera_rotation_delta_degrees,
    );
    if metrics.camera_correction_source != "input" {
        observe_max(
            &mut accumulator.max_camera_player_relative_step_m,
            metrics.camera_player_relative_step_m,
        );
        observe_max(
            &mut accumulator.max_camera_player_relative_linear_velocity_mps,
            metrics.camera_player_relative_linear_velocity_mps,
        );
        observe_max(
            &mut accumulator.max_camera_player_relative_linear_acceleration_mps2,
            metrics.camera_player_relative_linear_acceleration_mps2,
        );
        observe_max(
            &mut accumulator.max_camera_player_relative_angular_velocity_degrees_per_sec,
            metrics.camera_player_relative_angular_velocity_degrees_per_sec,
        );
        observe_max(
            &mut accumulator.max_camera_player_relative_angular_acceleration_degrees_per_sec2,
            metrics.camera_player_relative_angular_acceleration_degrees_per_sec2,
        );
    }
    observe_max(
        &mut accumulator.max_player_integration_residual_m,
        metrics.player_integration_residual_m,
    );
    observe_max(
        &mut accumulator.max_player_world_correction_m,
        metrics.player_world_correction_m,
    );
    observe_max(
        &mut accumulator.max_player_collision_correction_m,
        metrics.player_collision_correction_m,
    );
    if metrics.world_collision_correction_m.is_none()
        && metrics.player_world_correction_m <= f32::EPSILON
        && metrics.player_collision_correction_m <= f32::EPSILON
    {
        observe_max(
            &mut accumulator.max_player_integration_residual_without_world_collision_m,
            metrics.player_integration_residual_m,
        );
    }
    if let Some(correction_m) = metrics.world_collision_correction_m {
        observe_max(&mut accumulator.max_world_collision_push_m, correction_m);
    }

    observe_max(
        &mut accumulator.max_camera_obstruction_adjustment_m,
        metrics.camera_obstruction_adjustment_m,
    );
    accumulator.max_camera_obstruction_hits = accumulator
        .max_camera_obstruction_hits
        .max(metrics.camera_obstruction_hits);
    if metrics.camera_obstruction_hits > 0 {
        if metrics.camera_distance_m.is_finite() {
            accumulator.min_camera_obstructed_distance_m = Some(
                accumulator
                    .min_camera_obstructed_distance_m
                    .map_or(metrics.camera_distance_m, |distance| {
                        distance.min(metrics.camera_distance_m)
                    }),
            );
        }
        if accumulator.previous_camera_obstructed_sample
            && metrics.camera_step_distance_m > CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M
        {
            accumulator.camera_obstruction_snap_count += 1;
        }
        accumulator.previous_camera_obstructed_sample = true;
    } else {
        accumulator.previous_camera_obstructed_sample = false;
    }

    match metrics.camera_correction_source {
        "input" => (),
        "follow" => accumulator.camera_follow_correction_frames += 1,
        "floor" => accumulator.camera_floor_correction_frames += 1,
        "obstruction" => accumulator.camera_obstruction_correction_frames += 1,
        "distance" => accumulator.camera_distance_correction_frames += 1,
        "scripted" => accumulator.camera_scripted_correction_frames += 1,
        _ if metrics.camera_player_relative_step_m > 0.001
            || metrics.camera_rotation_delta_degrees > 0.001 =>
        {
            accumulator.camera_unclassified_correction_frames += 1;
        }
        _ => {}
    }
    accumulator.camera_continuity_offset_limited_frames +=
        u32::from(metrics.camera_continuity_offset_limited);
    accumulator.camera_continuity_rotation_limited_frames +=
        u32::from(metrics.camera_continuity_rotation_limited);
}

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    observe_continuity(
        accumulator,
        sample.frame,
        ContinuityMetrics {
            camera_distance_m: sample.camera_distance_m,
            camera_step_distance_m: sample.camera_step_distance_m,
            camera_player_relative_step_m: sample.camera_player_relative_step_m,
            camera_rotation_delta_degrees: sample.camera_rotation_delta_degrees,
            camera_player_relative_linear_velocity_mps: sample
                .camera_player_relative_linear_velocity_mps,
            camera_player_relative_linear_acceleration_mps2: sample
                .camera_player_relative_linear_acceleration_mps2,
            camera_player_relative_angular_velocity_degrees_per_sec: sample
                .camera_player_relative_angular_velocity_degrees_per_sec,
            camera_player_relative_angular_acceleration_degrees_per_sec2: sample
                .camera_player_relative_angular_acceleration_degrees_per_sec2,
            camera_obstruction_adjustment_m: sample.camera_obstruction_adjustment_m,
            camera_obstruction_hits: sample.camera_obstruction_hits,
            camera_correction_source: sample.camera_correction_source,
            camera_continuity_offset_limited: sample.camera_continuity_offset_limited,
            camera_continuity_rotation_limited: sample.camera_continuity_rotation_limited,
            player_integration_residual_m: sample.player_integration_residual_m,
            player_world_correction_m: 0.0,
            player_collision_correction_m: if sample.world_collision_resolved_count > 0 {
                sample.max_world_collision_push_m
            } else {
                0.0
            },
            world_collision_correction_m: (sample.world_collision_resolved_count > 0)
                .then_some(sample.max_world_collision_push_m),
        },
    );

    accumulator.max_camera_distance_m = accumulator
        .max_camera_distance_m
        .max(sample.camera_distance_m);
    accumulator.min_camera_surface_clearance_m = accumulator
        .min_camera_surface_clearance_m
        .min(sample.camera_surface_clearance_m);
    accumulator.max_camera_player_angle_degrees = accumulator
        .max_camera_player_angle_degrees
        .max(sample.camera_player_angle_degrees);
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

fn observe_max(maximum: &mut f32, value: f32) {
    if value.is_finite() {
        *maximum = maximum.max(value.max(0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{ISLAND_LAUNCH_TO_LANDING, scenario_named};

    #[test]
    fn full_rate_continuity_retains_an_unsampled_one_frame_snap() {
        let scenario = scenario_named(ISLAND_LAUNCH_TO_LANDING).expect("island traversal scenario");
        assert!(!scenario.should_sample(1));
        let mut accumulator = EvalAccumulator::default();

        observe_continuity(&mut accumulator, 0, continuity_metrics(0.1, 0.0));
        let mut snap = continuity_metrics(CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M + 0.5, 720.0);
        snap.camera_obstruction_hits = 1;
        snap.camera_correction_source = "obstruction";
        snap.camera_continuity_offset_limited = true;
        observe_continuity(&mut accumulator, 1, snap);
        observe_continuity(&mut accumulator, 2, continuity_metrics(0.1, 0.0));

        assert!(accumulator.max_camera_step_distance_m > CAMERA_OBSTRUCTION_SNAP_DISTANCE_DELTA_M);
        assert_eq!(
            accumulator.max_camera_player_relative_linear_acceleration_mps2,
            720.0
        );
        assert_eq!(accumulator.camera_obstruction_correction_frames, 1);
        assert_eq!(accumulator.camera_continuity_offset_limited_frames, 1);
        assert_eq!(accumulator.sample_count, 0);
    }

    #[test]
    fn player_world_corrections_are_attributed_without_becoming_proxy_pushes() {
        let mut accumulator = EvalAccumulator::default();
        let mut grounded = continuity_metrics(0.1, 0.0);
        grounded.player_integration_residual_m = 0.2;
        grounded.player_world_correction_m = 0.05;
        observe_continuity(&mut accumulator, 0, grounded);

        let mut continuous = continuity_metrics(0.1, 0.0);
        continuous.player_integration_residual_m = 0.01;
        observe_continuity(&mut accumulator, 1, continuous);

        assert_eq!(accumulator.max_player_world_correction_m, 0.05);
        assert_eq!(accumulator.max_player_collision_correction_m, 0.0);
        assert_eq!(accumulator.max_world_collision_push_m, 0.0);
        assert_eq!(
            accumulator.max_player_integration_residual_without_world_collision_m,
            0.01
        );
    }

    #[test]
    fn intentional_input_is_excluded_from_uncommanded_continuity_maxima() {
        let mut accumulator = EvalAccumulator::default();
        observe_continuity(&mut accumulator, 0, continuity_metrics(0.1, 12.0));

        let mut input = continuity_metrics(0.67, 920.0);
        input.camera_player_relative_linear_velocity_mps = 40.0;
        input.camera_correction_source = "input";
        observe_continuity(&mut accumulator, 1, input);

        assert_eq!(accumulator.max_camera_step_distance_m, 0.67);
        assert_eq!(accumulator.max_camera_player_relative_step_m, 0.1);
        assert_eq!(
            accumulator.max_camera_player_relative_linear_acceleration_mps2,
            12.0
        );
        assert_eq!(
            accumulator.max_camera_player_relative_linear_velocity_mps,
            0.0
        );
        assert_eq!(accumulator.camera_unclassified_correction_frames, 0);
    }

    fn continuity_metrics(
        camera_step_distance_m: f32,
        relative_linear_acceleration_mps2: f32,
    ) -> ContinuityMetrics {
        ContinuityMetrics {
            camera_distance_m: 12.0,
            camera_step_distance_m,
            camera_player_relative_step_m: camera_step_distance_m,
            camera_rotation_delta_degrees: 0.0,
            camera_player_relative_linear_velocity_mps: 0.0,
            camera_player_relative_linear_acceleration_mps2: relative_linear_acceleration_mps2,
            camera_player_relative_angular_velocity_degrees_per_sec: 0.0,
            camera_player_relative_angular_acceleration_degrees_per_sec2: 0.0,
            camera_obstruction_adjustment_m: 0.0,
            camera_obstruction_hits: 0,
            camera_correction_source: "follow",
            camera_continuity_offset_limited: false,
            camera_continuity_rotation_limited: false,
            player_integration_residual_m: 0.0,
            player_world_correction_m: 0.0,
            player_collision_correction_m: 0.0,
            world_collision_correction_m: None,
        }
    }
}
