use nau_engine::eval::EvalScenario;

use super::{super::SimMetrics, SimCheck};

pub(super) fn append_checks(
    checks: &mut Vec<SimCheck>,
    metrics: &SimMetrics,
    scenario: EvalScenario,
) {
    let thresholds = scenario.thresholds;
    checks.extend([
        SimCheck::at_least(
            "sample_count",
            metrics.sample_count as f32,
            thresholds.min_samples as f32,
            "samples",
        ),
        SimCheck::at_least(
            "horizontal_distance",
            metrics.horizontal_distance_m,
            thresholds.min_horizontal_distance_m,
            "m",
        ),
        SimCheck::at_least(
            "max_altitude",
            metrics.max_altitude_m,
            thresholds.min_max_altitude_m,
            "m",
        ),
        SimCheck::at_least(
            "max_speed",
            metrics.max_speed_mps,
            thresholds.min_max_speed_mps,
            "mps",
        ),
        SimCheck::at_least(
            "gliding_samples",
            metrics.gliding_samples as f32,
            thresholds.min_gliding_samples as f32,
            "samples",
        ),
        SimCheck::at_least(
            "grounded_samples",
            metrics.grounded_samples as f32,
            thresholds.min_grounded_samples as f32,
            "samples",
        ),
        SimCheck::at_least(
            "lifted_samples",
            metrics.lifted_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ),
        SimCheck::at_least(
            "sky_island_count",
            metrics.max_sky_island_count as f32,
            thresholds.min_sky_island_count as f32,
            "islands",
        ),
        SimCheck::at_least(
            "active_island_count",
            metrics.max_active_island_count as f32,
            thresholds.min_active_island_count as f32,
            "islands",
        ),
        SimCheck::at_most(
            "active_chunk_count",
            metrics.max_active_chunk_count as f32,
            thresholds.max_active_chunk_count as f32,
            "chunks",
        ),
        SimCheck::at_least(
            "near_lod_island_count",
            metrics.max_near_lod_islands as f32,
            thresholds.min_near_lod_island_count as f32,
            "islands",
        ),
        SimCheck::at_least(
            "mid_lod_island_count",
            metrics.max_mid_lod_islands as f32,
            thresholds.min_mid_lod_island_count as f32,
            "islands",
        ),
        SimCheck::at_least(
            "far_lod_island_count",
            metrics.max_far_lod_islands as f32,
            thresholds.min_far_lod_island_count as f32,
            "islands",
        ),
        SimCheck::at_most(
            "camera_distance",
            metrics.max_camera_distance_m,
            thresholds.max_camera_distance_m,
            "m",
        ),
        SimCheck::at_least(
            "camera_surface_clearance",
            metrics.min_camera_surface_clearance_m,
            thresholds.min_camera_surface_clearance_m,
            "m",
        ),
        SimCheck::at_most(
            "camera_player_angle",
            metrics.max_camera_player_angle_degrees,
            thresholds.max_camera_player_angle_degrees,
            "deg",
        ),
        SimCheck::at_most(
            "camera_step_distance",
            metrics.max_camera_step_distance_m,
            thresholds.max_camera_step_distance_m,
            "m",
        ),
        SimCheck::at_most(
            "camera_rotation_delta",
            metrics.max_camera_rotation_delta_degrees,
            thresholds.max_camera_rotation_delta_degrees,
            "deg",
        ),
        SimCheck::at_most(
            "camera_orbit_alignment",
            metrics.max_camera_orbit_alignment_degrees,
            thresholds.max_camera_orbit_alignment_degrees,
            "deg",
        ),
        SimCheck::at_most(
            "camera_view_yaw",
            metrics.max_abs_camera_view_yaw_degrees,
            thresholds.max_abs_camera_view_yaw_degrees,
            "deg",
        ),
        SimCheck::at_least(
            "camera_obstruction_adjustment",
            metrics.max_camera_obstruction_adjustment_m,
            thresholds.min_camera_obstruction_adjustment_m,
            "m",
        ),
        SimCheck::at_least(
            "camera_yaw_input",
            metrics.max_abs_camera_yaw_offset_degrees,
            thresholds.min_abs_camera_yaw_degrees,
            "deg",
        ),
        camera_pitch_min_check(metrics, scenario),
        camera_pitch_max_check(metrics, scenario),
        SimCheck::at_least(
            "objective_total_count",
            metrics.objective_total_count as f32,
            thresholds.min_objective_total_count as f32,
            "objectives",
        ),
        SimCheck::at_least(
            "completed_objective_count",
            metrics.max_completed_objective_count as f32,
            thresholds.min_completed_objective_count as f32,
            "objectives",
        ),
        SimCheck::at_most(
            "final_target_distance",
            metrics.final_target_distance_m,
            thresholds.max_final_target_distance_m,
            "m",
        ),
        SimCheck::at_least(
            "target_landing_samples",
            metrics.target_landing_samples as f32,
            thresholds.min_target_landing_samples as f32,
            "samples",
        ),
        SimCheck::at_least(
            "power_up_count",
            metrics.max_power_up_count as f32,
            thresholds.min_power_up_count as f32,
            "powerups",
        ),
        SimCheck::at_least(
            "collected_power_up_count",
            metrics.max_collected_power_up_count as f32,
            thresholds.min_collected_power_up_count as f32,
            "powerups",
        ),
        SimCheck::at_least(
            "power_up_effect_samples",
            metrics.power_up_effect_samples as f32,
            thresholds.min_power_up_effect_samples as f32,
            "samples",
        ),
    ]);

    if thresholds.min_camera_obstruction_adjustment_m > 0.0
        || thresholds.min_camera_obstructed_distance_m > 0.0
    {
        checks.extend([
            SimCheck::at_least(
                "camera_obstructed_distance",
                metrics.min_camera_obstructed_distance_m.unwrap_or(0.0),
                thresholds.min_camera_obstructed_distance_m,
                "m",
            ),
            SimCheck::at_most(
                "camera_obstruction_snap_count",
                metrics.camera_obstruction_snap_count as f32,
                thresholds.max_camera_obstruction_snap_count as f32,
                "samples",
            ),
        ]);
    }
}

fn camera_pitch_min_check(metrics: &SimMetrics, scenario: EvalScenario) -> SimCheck {
    let threshold = scenario.thresholds.min_camera_pitch_offset_degrees;
    if threshold < 0.0 {
        SimCheck::at_most(
            "camera_pitch_input_min",
            metrics.min_camera_pitch_offset_degrees,
            threshold,
            "deg",
        )
    } else {
        SimCheck::at_least(
            "camera_pitch_input_min",
            metrics.min_camera_pitch_offset_degrees,
            threshold,
            "deg",
        )
    }
}

fn camera_pitch_max_check(metrics: &SimMetrics, scenario: EvalScenario) -> SimCheck {
    let threshold = scenario.thresholds.max_camera_pitch_offset_degrees;
    if threshold > 0.0 {
        SimCheck::at_least(
            "camera_pitch_input_max",
            metrics.max_camera_pitch_offset_degrees,
            threshold,
            "deg",
        )
    } else {
        SimCheck::at_most(
            "camera_pitch_input_max",
            metrics.max_camera_pitch_offset_degrees,
            threshold,
            "deg",
        )
    }
}
