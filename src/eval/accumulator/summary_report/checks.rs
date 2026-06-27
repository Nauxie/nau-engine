#[path = "checks/assets.rs"]
mod assets;
#[path = "checks/content.rs"]
mod content;
#[path = "checks/control.rs"]
mod control;

use super::super::EvalAccumulator;
use super::derived::SummaryDerivedMetrics;
use crate::eval::{
    scenarios::{
        EvalScenario, GROUND_TAXI_CONTROL, TERRAIN_RIM_COLLISION_CONTACT, WORLD_COLLISION_CONTACT,
    },
    summary::EvalCheck,
    thresholds::*,
};

pub(super) fn build_checks(
    acc: &EvalAccumulator,
    scenario: EvalScenario,
    derived: &SummaryDerivedMetrics,
) -> Vec<EvalCheck> {
    let thresholds = scenario.thresholds;
    let mut checks = vec![
        EvalCheck::at_least(
            "sample_count",
            acc.sample_count as f32,
            thresholds.min_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "horizontal_distance",
            derived.horizontal_distance_m,
            thresholds.min_horizontal_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_altitude",
            acc.max_altitude_m,
            thresholds.min_max_altitude_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_speed",
            acc.max_speed_mps,
            thresholds.min_max_speed_mps,
            "m/s",
        ),
        EvalCheck::at_least(
            "gliding_samples",
            acc.gliding_samples as f32,
            thresholds.min_gliding_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "grounded_samples",
            acc.grounded_samples as f32,
            thresholds.min_grounded_samples as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "grounded_visual_foot_gap",
            acc.max_grounded_visual_foot_gap_m,
            MAX_GROUNDED_VISUAL_FOOT_GAP_M,
            "m",
        ),
        EvalCheck::at_least(
            "lifted_samples",
            acc.lifted_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sky_island_count",
            acc.max_sky_island_count as f32,
            thresholds.min_sky_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "active_island_count",
            acc.max_active_island_count as f32,
            thresholds.min_active_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "active_chunk_count",
            acc.max_active_chunk_count as f32,
            thresholds.max_active_chunk_count as f32,
            "chunks",
        ),
        EvalCheck::at_least(
            "near_lod_island_count",
            acc.max_near_lod_islands as f32,
            thresholds.min_near_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "mid_lod_island_count",
            acc.max_mid_lod_islands as f32,
            thresholds.min_mid_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "far_lod_island_count",
            acc.max_far_lod_islands as f32,
            thresholds.min_far_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "visible_island_terrain_count",
            acc.max_visible_island_terrain_count as f32,
            thresholds.max_visible_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_terrain_count",
            acc.max_hidden_island_terrain_count as f32,
            thresholds.min_hidden_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_island_impostor_count",
            acc.max_visible_island_impostor_count as f32,
            thresholds.min_visible_island_impostor_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "visible_island_detail_count",
            acc.max_visible_island_detail_count as f32,
            thresholds.max_visible_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_detail_count",
            acc.max_hidden_island_detail_count as f32,
            thresholds.min_hidden_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_route_beacon_count",
            acc.max_visible_route_beacon_count as f32,
            thresholds.min_visible_route_beacon_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "weather_cloud_count",
            acc.max_weather_cloud_count as f32,
            thresholds.min_weather_cloud_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_visual_count",
            acc.max_environment_motion_visual_count as f32,
            thresholds.min_environment_motion_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_offset",
            acc.max_environment_motion_offset_m,
            thresholds.min_environment_motion_offset_m,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_guide_visual_count",
            acc.max_updraft_guide_visual_count as f32,
            MIN_UPDRAFT_GUIDE_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "updraft_ribbon_visual_count",
            acc.max_updraft_ribbon_visual_count as f32,
            MIN_UPDRAFT_RIBBON_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_guide_visual_count",
            acc.max_crosswind_guide_visual_count as f32,
            MIN_CROSSWIND_GUIDE_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_visual_count",
            acc.max_crosswind_ribbon_visual_count as f32,
            MIN_CROSSWIND_RIBBON_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "updraft_visual_motion",
            acc.max_updraft_visual_motion_m,
            MIN_UPDRAFT_VISUAL_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_rise",
            acc.max_updraft_visual_rise_m,
            MIN_UPDRAFT_VISUAL_RISE_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_visual_swirl_displacement",
            acc.max_updraft_visual_swirl_displacement_m,
            MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_visual_motion",
            acc.max_crosswind_visual_motion_m,
            MIN_CROSSWIND_VISUAL_MOTION_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_guide_flow_displacement",
            acc.max_crosswind_guide_flow_displacement_m,
            MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "crosswind_ribbon_flow_displacement",
            acc.max_crosswind_ribbon_flow_displacement_m,
            MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
            "m",
        ),
        EvalCheck::at_least(
            "updraft_flow_coherent_visual_count",
            acc.max_updraft_flow_coherent_visual_count as f32,
            MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "crosswind_flow_coherent_visual_count",
            acc.max_crosswind_flow_coherent_visual_count as f32,
            MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "updraft_visual_flow_alignment",
            acc.max_updraft_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "crosswind_visual_flow_alignment",
            acc.max_crosswind_visual_flow_alignment,
            MIN_WIND_VISUAL_FLOW_ALIGNMENT,
            "dot",
        ),
        EvalCheck::at_least(
            "world_collision_proxy_count",
            acc.max_world_collision_proxy_count as f32,
            MIN_WORLD_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
        EvalCheck::at_least(
            "terrain_rim_collision_proxy_count",
            acc.max_terrain_rim_collision_proxy_count as f32,
            MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT as f32,
            "proxies",
        ),
    ];

    if scenario.name == WORLD_COLLISION_CONTACT {
        checks.extend([
            EvalCheck::at_least(
                "world_collision_contact_samples",
                acc.world_collision_contact_samples as f32,
                MIN_WORLD_COLLISION_CONTACT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "world_collision_push",
                acc.max_world_collision_push_m,
                MIN_WORLD_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
        ]);
    }
    if scenario.name == TERRAIN_RIM_COLLISION_CONTACT {
        checks.extend([
            EvalCheck::at_least(
                "terrain_rim_collision_contact_samples",
                acc.terrain_rim_collision_contact_samples as f32,
                MIN_WORLD_COLLISION_CONTACT_SAMPLES as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "terrain_rim_collision_push",
                acc.max_terrain_rim_collision_push_m,
                MIN_WORLD_COLLISION_CONTACT_PUSH_M,
                "m",
            ),
        ]);
    }
    if scenario.name == GROUND_TAXI_CONTROL {
        checks.push(EvalCheck::at_most(
            "ground_taxi_terrain_rim_contact_samples",
            acc.terrain_rim_collision_resolved_samples as f32,
            0.0,
            "samples",
        ));
    }

    content::append_content_checks(&mut checks, acc, &thresholds);

    checks.extend([
        EvalCheck::at_most(
            "resident_island_visual_count",
            acc.max_resident_island_visual_count as f32,
            thresholds.max_resident_island_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "stream_visibility_changes_per_frame",
            acc.max_stream_visibility_changes_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_least(
            "hidden_island_visual_count",
            acc.max_hidden_island_visual_count as f32,
            (thresholds.min_hidden_island_terrain_count + thresholds.min_hidden_island_detail_count)
                as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "resident_island_visual_fraction",
            acc.max_resident_island_visual_fraction,
            MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
            "ratio",
        ),
        EvalCheck::at_most(
            "stream_spawned_visuals_per_frame",
            acc.max_stream_spawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_most(
            "stream_despawned_visuals_per_frame",
            acc.max_stream_despawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_least(
            "entity_count",
            acc.max_entity_count as f32,
            thresholds.min_entity_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "objective_total_count",
            acc.max_objective_total_count as f32,
            thresholds.min_objective_total_count as f32,
            "objectives",
        ),
        EvalCheck::at_least(
            "completed_objective_count",
            acc.max_completed_objective_count as f32,
            thresholds.min_completed_objective_count as f32,
            "objectives",
        ),
    ]);

    assets::append_asset_checks(&mut checks, acc, &thresholds);

    checks.extend([
        EvalCheck::at_most(
            "max_camera_distance",
            acc.max_camera_distance_m,
            thresholds.max_camera_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "min_camera_surface_clearance",
            acc.min_camera_surface_clearance_m,
            thresholds.min_camera_surface_clearance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_player_angle",
            acc.max_camera_player_angle_degrees,
            thresholds.max_camera_player_angle_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_step_distance",
            acc.max_camera_step_distance_m,
            thresholds.max_camera_step_distance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            thresholds.max_camera_rotation_delta_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_orbit_alignment",
            acc.max_camera_orbit_alignment_degrees,
            thresholds.max_camera_orbit_alignment_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_abs_camera_view_yaw",
            acc.max_abs_camera_view_yaw_degrees,
            thresholds.max_abs_camera_view_yaw_degrees,
            "deg",
        ),
        EvalCheck::at_least(
            "max_camera_obstruction_adjustment",
            acc.max_camera_obstruction_adjustment_m,
            thresholds.min_camera_obstruction_adjustment_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_abs_camera_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            thresholds.min_abs_camera_yaw_degrees,
            "deg",
        ),
        if thresholds.min_camera_pitch_offset_degrees < 0.0 {
            EvalCheck::at_most(
                "min_camera_pitch_offset",
                acc.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            )
        } else {
            EvalCheck::at_least(
                "min_camera_pitch_offset",
                acc.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            )
        },
        if thresholds.max_camera_pitch_offset_degrees > 0.0 {
            EvalCheck::at_least(
                "max_camera_pitch_offset",
                acc.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            )
        } else {
            EvalCheck::at_most(
                "max_camera_pitch_offset",
                acc.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            )
        },
    ]);

    control::append_scenario_checks(&mut checks, acc, scenario, derived, &thresholds);
    checks
}
