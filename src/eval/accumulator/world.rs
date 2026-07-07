use bevy::prelude::Vec3;

use super::{EvalAccumulator, EvalSample};
use crate::eval::thresholds::{
    MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT, MIN_CROSSWIND_FORCE_DELTA_MPS,
    MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M, MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
    MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT, MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
    MIN_CROSSWIND_RIBBON_VISUAL_COUNT, MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M,
    MIN_CROSSWIND_VISUAL_MOTION_M, MIN_CROSSWIND_VISUAL_SCALE_PULSE,
    MIN_OBSERVED_CROSSWIND_GUIDE_FRAME_FLOW_DISPLACEMENT_M,
    MIN_OBSERVED_CROSSWIND_RIBBON_FRAME_FLOW_DISPLACEMENT_M,
    MIN_OBSERVED_CROSSWIND_VISUAL_FRAME_MOTION_M, MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_MOTION_M,
    MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_RISE_M,
    MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_SWIRL_DISPLACEMENT_M, MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
    MIN_UPDRAFT_GUIDE_VISUAL_COUNT, MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
    MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M, MIN_UPDRAFT_VISUAL_MOTION_M, MIN_UPDRAFT_VISUAL_RISE_M,
    MIN_UPDRAFT_VISUAL_SCALE_PULSE, MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
    MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS, MIN_WIND_FORCE_FLOW_ALIGNMENT,
    MIN_WIND_FORCE_VARIATION, MIN_WIND_LOAD_LATERAL_LOAD, MIN_WIND_VISUAL_FLOW_ALIGNMENT,
    MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M, SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO,
};

pub(super) fn observe(
    accumulator: &mut EvalAccumulator,
    sample: &EvalSample,
    min_updraft_swirl_force_delta_mps: f32,
) {
    accumulator.max_visible_wind_fields = accumulator
        .max_visible_wind_fields
        .max(sample.visible_wind_fields);
    accumulator.max_dynamic_wind_flow_fields = accumulator
        .max_dynamic_wind_flow_fields
        .max(sample.dynamic_wind_flow_fields);
    accumulator.max_wind_flow_speed_mps = accumulator
        .max_wind_flow_speed_mps
        .max(sample.max_wind_flow_speed_mps);
    accumulator.max_wind_flow_variation = accumulator
        .max_wind_flow_variation
        .max(sample.max_wind_flow_variation);
    accumulator.max_wind_flow_direction_change_degrees = accumulator
        .max_wind_flow_direction_change_degrees
        .max(sample.max_wind_flow_direction_change_degrees);
    if sample.active_wind_force_fields > 0 {
        accumulator.wind_force_samples += 1;
        let meaningful_delta = sample.max_wind_force_delta_mps >= MIN_WIND_FORCE_DELTA_MPS
            || sample.max_crosswind_force_delta_mps >= MIN_CROSSWIND_FORCE_DELTA_MPS
            || sample.max_updraft_swirl_force_delta_mps >= min_updraft_swirl_force_delta_mps;
        if meaningful_delta && sample.max_wind_force_variation >= MIN_WIND_FORCE_VARIATION {
            accumulator.meaningful_wind_force_samples += 1;
        }
        if sample.max_wind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_wind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
        {
            accumulator.aligned_wind_force_samples += 1;
        }
    }
    if sample.crosswind_force_fields > 0 {
        accumulator.crosswind_force_samples += 1;
        if sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
        {
            accumulator.aligned_crosswind_force_samples += 1;
        }
    }
    if sample.updraft_swirl_force_fields > 0 {
        accumulator.updraft_swirl_force_samples += 1;
        if sample.max_updraft_swirl_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_updraft_swirl_force_aligned_delta_mps >= min_updraft_swirl_force_delta_mps
        {
            accumulator.aligned_updraft_swirl_force_samples += 1;
        }
    }
    if sample.active_wind_force_fields >= 2 {
        accumulator.layered_wind_force_samples += 1;
        accumulator.max_layered_wind_force_fields = accumulator
            .max_layered_wind_force_fields
            .max(sample.active_wind_force_fields);
        accumulator.max_layered_wind_force_delta_mps = accumulator
            .max_layered_wind_force_delta_mps
            .max(sample.max_wind_force_delta_mps);
        accumulator.max_layered_wind_force_flow_alignment = accumulator
            .max_layered_wind_force_flow_alignment
            .max(sample.max_wind_force_flow_alignment);
        accumulator.max_layered_wind_force_aligned_delta_mps = accumulator
            .max_layered_wind_force_aligned_delta_mps
            .max(sample.max_wind_force_aligned_delta_mps);
        if sample.max_wind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_wind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
        {
            accumulator.aligned_layered_wind_force_samples += 1;
        }
    }
    if sample.active_wind_force_fields >= 2
        && sample.crosswind_force_fields > 0
        && sample.updraft_swirl_force_fields > 0
    {
        accumulator.crosswind_updraft_overlap_samples += 1;
        if sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            && sample.max_updraft_swirl_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
            && sample.max_updraft_swirl_force_aligned_delta_mps >= min_updraft_swirl_force_delta_mps
        {
            accumulator.aligned_crosswind_updraft_overlap_samples += 1;
        }
    }
    accumulator.max_active_wind_force_fields = accumulator
        .max_active_wind_force_fields
        .max(sample.active_wind_force_fields);
    accumulator.max_crosswind_force_fields = accumulator
        .max_crosswind_force_fields
        .max(sample.crosswind_force_fields);
    accumulator.max_updraft_swirl_force_fields = accumulator
        .max_updraft_swirl_force_fields
        .max(sample.updraft_swirl_force_fields);
    accumulator.max_wind_force_delta_mps = accumulator
        .max_wind_force_delta_mps
        .max(sample.max_wind_force_delta_mps);
    accumulator.max_crosswind_force_delta_mps = accumulator
        .max_crosswind_force_delta_mps
        .max(sample.max_crosswind_force_delta_mps);
    accumulator.max_updraft_swirl_force_delta_mps = accumulator
        .max_updraft_swirl_force_delta_mps
        .max(sample.max_updraft_swirl_force_delta_mps);
    accumulator.max_wind_force_flow_speed_mps = accumulator
        .max_wind_force_flow_speed_mps
        .max(sample.max_wind_force_flow_speed_mps);
    accumulator.max_wind_force_variation = accumulator
        .max_wind_force_variation
        .max(sample.max_wind_force_variation);
    accumulator.max_wind_force_flow_alignment = accumulator
        .max_wind_force_flow_alignment
        .max(sample.max_wind_force_flow_alignment);
    accumulator.max_crosswind_force_flow_alignment = accumulator
        .max_crosswind_force_flow_alignment
        .max(sample.max_crosswind_force_flow_alignment);
    accumulator.max_updraft_swirl_force_flow_alignment = accumulator
        .max_updraft_swirl_force_flow_alignment
        .max(sample.max_updraft_swirl_force_flow_alignment);
    accumulator.max_wind_force_aligned_delta_mps = accumulator
        .max_wind_force_aligned_delta_mps
        .max(sample.max_wind_force_aligned_delta_mps);
    accumulator.max_crosswind_force_aligned_delta_mps = accumulator
        .max_crosswind_force_aligned_delta_mps
        .max(sample.max_crosswind_force_aligned_delta_mps);
    accumulator.max_updraft_swirl_force_aligned_delta_mps = accumulator
        .max_updraft_swirl_force_aligned_delta_mps
        .max(sample.max_updraft_swirl_force_aligned_delta_mps);
    if wind_load_response_sample(sample) {
        accumulator.wind_load_response_samples += 1;
        accumulator.max_wind_load_lateral_load = accumulator
            .max_wind_load_lateral_load
            .max(sample.wind_lateral_load.abs());
        accumulator.max_wind_load_pose_lean_degrees = accumulator
            .max_wind_load_pose_lean_degrees
            .max(sample.pose_lateral_lean_degrees);
        accumulator.max_wind_load_glider_response_degrees = accumulator
            .max_wind_load_glider_response_degrees
            .max(sample.authored_glider_response_degrees);
    }
    observe_crosswind_neutral_drift(accumulator, sample);
    accumulator.max_active_lift_fields = accumulator
        .max_active_lift_fields
        .max(sample.active_lift_fields);
    accumulator.max_readable_lift_fields = accumulator
        .max_readable_lift_fields
        .max(sample.readable_lift_fields);
    if sample.active_lift_fields > 0
        && sample.readable_lift_fields > 0
        && sample.dynamic_wind_flow_fields > 0
        && sample.max_wind_flow_variation > 0.05
    {
        accumulator.dynamic_readable_lift_samples += 1;
        accumulator.max_dynamic_readable_wind_flow_variation = accumulator
            .max_dynamic_readable_wind_flow_variation
            .max(sample.max_wind_flow_variation);
        let min_variation = accumulator
            .min_dynamic_readable_wind_flow_variation
            .map_or(sample.max_wind_flow_variation, |current| {
                current.min(sample.max_wind_flow_variation)
            });
        accumulator.min_dynamic_readable_wind_flow_variation = Some(min_variation);
        accumulator.max_wind_flow_variation_range = accumulator
            .max_wind_flow_variation_range
            .max(accumulator.max_dynamic_readable_wind_flow_variation - min_variation);
    }
    accumulator.max_sky_island_count = accumulator
        .max_sky_island_count
        .max(sample.sky_island_count);
    accumulator.max_active_chunk_count = accumulator
        .max_active_chunk_count
        .max(sample.active_chunk_count);
    accumulator.max_active_island_count = accumulator
        .max_active_island_count
        .max(sample.active_island_count);
    accumulator.max_near_lod_islands = accumulator
        .max_near_lod_islands
        .max(sample.near_lod_islands);
    accumulator.max_mid_lod_islands = accumulator.max_mid_lod_islands.max(sample.mid_lod_islands);
    accumulator.max_far_lod_islands = accumulator.max_far_lod_islands.max(sample.far_lod_islands);
    accumulator.max_visible_island_terrain_count = accumulator
        .max_visible_island_terrain_count
        .max(sample.visible_island_terrain_count);
    accumulator.max_hidden_island_terrain_count = accumulator
        .max_hidden_island_terrain_count
        .max(sample.hidden_island_terrain_count);
    accumulator.max_visible_island_impostor_count = accumulator
        .max_visible_island_impostor_count
        .max(sample.visible_island_impostor_count);
    accumulator.max_hidden_island_impostor_count = accumulator
        .max_hidden_island_impostor_count
        .max(sample.hidden_island_impostor_count);
    accumulator.max_visible_island_detail_count = accumulator
        .max_visible_island_detail_count
        .max(sample.visible_island_detail_count);
    accumulator.max_hidden_island_detail_count = accumulator
        .max_hidden_island_detail_count
        .max(sample.hidden_island_detail_count);
    accumulator.max_visible_route_beacon_count = accumulator
        .max_visible_route_beacon_count
        .max(sample.visible_route_beacon_count);
    accumulator.max_weather_cloud_count = accumulator
        .max_weather_cloud_count
        .max(sample.weather_cloud_count);
    accumulator.max_environment_motion_visual_count = accumulator
        .max_environment_motion_visual_count
        .max(sample.environment_motion_visual_count);
    accumulator.max_environment_motion_offset_m = accumulator
        .max_environment_motion_offset_m
        .max(sample.max_environment_motion_offset_m);
    accumulator.max_updraft_guide_visual_count = accumulator
        .max_updraft_guide_visual_count
        .max(sample.updraft_guide_visual_count);
    accumulator.max_updraft_ribbon_visual_count = accumulator
        .max_updraft_ribbon_visual_count
        .max(sample.updraft_ribbon_visual_count);
    accumulator.max_crosswind_guide_visual_count = accumulator
        .max_crosswind_guide_visual_count
        .max(sample.crosswind_guide_visual_count);
    accumulator.max_crosswind_ribbon_visual_count = accumulator
        .max_crosswind_ribbon_visual_count
        .max(sample.crosswind_ribbon_visual_count);
    accumulator.max_updraft_visual_motion_m = accumulator
        .max_updraft_visual_motion_m
        .max(sample.max_updraft_visual_motion_m);
    accumulator.max_updraft_visual_rise_m = accumulator
        .max_updraft_visual_rise_m
        .max(sample.max_updraft_visual_rise_m);
    accumulator.max_updraft_visual_swirl_displacement_m = accumulator
        .max_updraft_visual_swirl_displacement_m
        .max(sample.max_updraft_visual_swirl_displacement_m);
    accumulator.max_updraft_visual_depth_span_m = accumulator
        .max_updraft_visual_depth_span_m
        .max(sample.max_updraft_visual_depth_span_m);
    accumulator.max_updraft_visual_scale_pulse = accumulator
        .max_updraft_visual_scale_pulse
        .max(sample.max_updraft_visual_scale_pulse);
    accumulator.max_crosswind_visual_motion_m = accumulator
        .max_crosswind_visual_motion_m
        .max(sample.max_crosswind_visual_motion_m);
    accumulator.max_crosswind_guide_flow_displacement_m = accumulator
        .max_crosswind_guide_flow_displacement_m
        .max(sample.max_crosswind_guide_flow_displacement_m);
    accumulator.max_crosswind_ribbon_flow_displacement_m = accumulator
        .max_crosswind_ribbon_flow_displacement_m
        .max(sample.max_crosswind_ribbon_flow_displacement_m);
    accumulator.max_crosswind_visual_lane_depth_span_m = accumulator
        .max_crosswind_visual_lane_depth_span_m
        .max(sample.max_crosswind_visual_lane_depth_span_m);
    accumulator.max_crosswind_visual_scale_pulse = accumulator
        .max_crosswind_visual_scale_pulse
        .max(sample.max_crosswind_visual_scale_pulse);
    accumulator.max_updraft_flow_coherent_visual_count = accumulator
        .max_updraft_flow_coherent_visual_count
        .max(sample.updraft_flow_coherent_visual_count);
    accumulator.max_crosswind_flow_coherent_visual_count = accumulator
        .max_crosswind_flow_coherent_visual_count
        .max(sample.crosswind_flow_coherent_visual_count);
    accumulator.max_crosswind_ribbon_flow_coherent_sample_count = accumulator
        .max_crosswind_ribbon_flow_coherent_sample_count
        .max(sample.crosswind_ribbon_flow_coherent_sample_count);
    accumulator.max_updraft_visual_flow_alignment = accumulator
        .max_updraft_visual_flow_alignment
        .max(sample.max_updraft_visual_flow_alignment);
    accumulator.max_crosswind_visual_flow_alignment = accumulator
        .max_crosswind_visual_flow_alignment
        .max(sample.max_crosswind_visual_flow_alignment);
    accumulator.max_crosswind_ribbon_visual_flow_alignment = accumulator
        .max_crosswind_ribbon_visual_flow_alignment
        .max(sample.max_crosswind_ribbon_visual_flow_alignment);
    accumulator.max_observed_updraft_flow_coherent_visual_count = accumulator
        .max_observed_updraft_flow_coherent_visual_count
        .max(sample.observed_updraft_flow_coherent_visual_count);
    accumulator.max_observed_crosswind_flow_coherent_visual_count = accumulator
        .max_observed_crosswind_flow_coherent_visual_count
        .max(sample.observed_crosswind_flow_coherent_visual_count);
    accumulator.max_observed_crosswind_ribbon_flow_coherent_sample_count = accumulator
        .max_observed_crosswind_ribbon_flow_coherent_sample_count
        .max(sample.observed_crosswind_ribbon_flow_coherent_sample_count);
    accumulator.max_observed_updraft_visual_frame_motion_m = accumulator
        .max_observed_updraft_visual_frame_motion_m
        .max(sample.max_observed_updraft_visual_frame_motion_m);
    accumulator.max_observed_updraft_visual_frame_rise_m = accumulator
        .max_observed_updraft_visual_frame_rise_m
        .max(sample.max_observed_updraft_visual_frame_rise_m);
    accumulator.max_observed_updraft_visual_frame_swirl_displacement_m = accumulator
        .max_observed_updraft_visual_frame_swirl_displacement_m
        .max(sample.max_observed_updraft_visual_frame_swirl_displacement_m);
    accumulator.max_observed_crosswind_visual_frame_motion_m = accumulator
        .max_observed_crosswind_visual_frame_motion_m
        .max(sample.max_observed_crosswind_visual_frame_motion_m);
    accumulator.max_observed_crosswind_guide_frame_flow_displacement_m = accumulator
        .max_observed_crosswind_guide_frame_flow_displacement_m
        .max(sample.max_observed_crosswind_guide_frame_flow_displacement_m);
    accumulator.max_observed_crosswind_ribbon_frame_flow_displacement_m = accumulator
        .max_observed_crosswind_ribbon_frame_flow_displacement_m
        .max(sample.max_observed_crosswind_ribbon_frame_flow_displacement_m);
    accumulator.max_observed_updraft_visual_speed_mps = accumulator
        .max_observed_updraft_visual_speed_mps
        .max(sample.max_observed_updraft_visual_speed_mps);
    accumulator.max_observed_crosswind_visual_speed_mps = accumulator
        .max_observed_crosswind_visual_speed_mps
        .max(sample.max_observed_crosswind_visual_speed_mps);
    accumulator.max_observed_wind_visual_acceleration_mps2 = accumulator
        .max_observed_wind_visual_acceleration_mps2
        .max(sample.max_observed_wind_visual_acceleration_mps2);
    accumulator.observed_wind_visual_jump_count += sample.observed_wind_visual_jump_count;
    accumulator.max_observed_updraft_visual_flow_alignment = accumulator
        .max_observed_updraft_visual_flow_alignment
        .max(sample.max_observed_updraft_visual_flow_alignment);
    accumulator.max_observed_crosswind_visual_flow_alignment = accumulator
        .max_observed_crosswind_visual_flow_alignment
        .max(sample.max_observed_crosswind_visual_flow_alignment);
    accumulator.max_observed_crosswind_ribbon_visual_flow_alignment = accumulator
        .max_observed_crosswind_ribbon_visual_flow_alignment
        .max(sample.max_observed_crosswind_ribbon_visual_flow_alignment);
    accumulator.max_updraft_field_count = accumulator
        .max_updraft_field_count
        .max(sample.updraft_field_count);
    accumulator.max_updraft_fields_with_guides_count = accumulator
        .max_updraft_fields_with_guides_count
        .max(sample.updraft_fields_with_guides_count);
    accumulator.max_updraft_fields_with_ribbons_count = accumulator
        .max_updraft_fields_with_ribbons_count
        .max(sample.updraft_fields_with_ribbons_count);
    accumulator.max_updraft_fields_with_guides_and_ribbons_count = accumulator
        .max_updraft_fields_with_guides_and_ribbons_count
        .max(sample.updraft_fields_with_guides_and_ribbons_count);
    accumulator.max_updraft_flow_coherent_field_count = accumulator
        .max_updraft_flow_coherent_field_count
        .max(sample.updraft_flow_coherent_field_count);
    accumulator.max_crosswind_field_count = accumulator
        .max_crosswind_field_count
        .max(sample.crosswind_field_count);
    accumulator.max_crosswind_fields_with_guides_count = accumulator
        .max_crosswind_fields_with_guides_count
        .max(sample.crosswind_fields_with_guides_count);
    accumulator.max_crosswind_fields_with_ribbons_count = accumulator
        .max_crosswind_fields_with_ribbons_count
        .max(sample.crosswind_fields_with_ribbons_count);
    accumulator.max_crosswind_fields_with_guides_and_ribbons_count = accumulator
        .max_crosswind_fields_with_guides_and_ribbons_count
        .max(sample.crosswind_fields_with_guides_and_ribbons_count);
    accumulator.max_crosswind_flow_coherent_field_count = accumulator
        .max_crosswind_flow_coherent_field_count
        .max(sample.crosswind_flow_coherent_field_count);
    let sustained_updraft_visual_flow = has_sustained_updraft_visual_flow(sample);
    let sustained_crosswind_visual_flow = has_sustained_crosswind_visual_flow(sample);
    let sustained_crosswind_ribbon_advected_flow =
        has_sustained_crosswind_ribbon_advected_flow(sample);
    if sustained_updraft_visual_flow {
        accumulator.sustained_updraft_visual_flow_samples += 1;
    }
    if sustained_crosswind_visual_flow {
        accumulator.sustained_crosswind_visual_flow_samples += 1;
    }
    if sustained_crosswind_ribbon_advected_flow {
        accumulator.sustained_crosswind_ribbon_advected_flow_samples += 1;
    }
    if sustained_updraft_visual_flow || sustained_crosswind_visual_flow {
        accumulator.sustained_wind_visual_flow_samples += 1;
    }
    accumulator.max_world_collision_proxy_count = accumulator
        .max_world_collision_proxy_count
        .max(sample.world_collision_proxy_count);
    accumulator.max_terrain_rim_collision_proxy_count = accumulator
        .max_terrain_rim_collision_proxy_count
        .max(sample.terrain_rim_collision_proxy_count);
    accumulator.max_terrain_body_collision_proxy_count = accumulator
        .max_terrain_body_collision_proxy_count
        .max(sample.terrain_body_collision_proxy_count);
    accumulator.max_solid_world_collision_proxy_count = accumulator
        .max_solid_world_collision_proxy_count
        .max(sample.solid_world_collision_proxy_count);
    accumulator.max_tree_world_collision_proxy_count = accumulator
        .max_tree_world_collision_proxy_count
        .max(sample.tree_world_collision_proxy_count);
    accumulator.max_rock_world_collision_proxy_count = accumulator
        .max_rock_world_collision_proxy_count
        .max(sample.rock_world_collision_proxy_count);
    accumulator.max_landmark_world_collision_proxy_count = accumulator
        .max_landmark_world_collision_proxy_count
        .max(sample.landmark_world_collision_proxy_count);
    if sample.world_collision_resolved_count > 0 {
        accumulator.world_collision_resolved_samples += 1;
        if sample.max_world_collision_push_m >= MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M {
            accumulator.world_collision_contact_samples += 1;
        }
    }
    if sample.terrain_rim_collision_resolved_count > 0 {
        accumulator.terrain_rim_collision_resolved_samples += 1;
        if sample.max_terrain_rim_collision_push_m >= MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M {
            accumulator.terrain_rim_collision_contact_samples += 1;
        }
    }
    if sample.terrain_body_collision_resolved_count > 0 {
        accumulator.terrain_body_collision_resolved_samples += 1;
        if sample.max_terrain_body_collision_push_m >= MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M {
            accumulator.terrain_body_collision_contact_samples += 1;
        }
    }
    accumulator.max_world_collision_push_m = accumulator
        .max_world_collision_push_m
        .max(sample.max_world_collision_push_m);
    accumulator.max_terrain_rim_collision_push_m = accumulator
        .max_terrain_rim_collision_push_m
        .max(sample.max_terrain_rim_collision_push_m);
    accumulator.max_terrain_body_collision_push_m = accumulator
        .max_terrain_body_collision_push_m
        .max(sample.max_terrain_body_collision_push_m);
    if sample.near_island_edge {
        accumulator.near_island_edge_samples += 1;
    }
    if sample.outside_island_footprint {
        accumulator.outside_island_footprint_samples += 1;
    }
    if sample.near_island_edge && sample.outside_island_footprint {
        accumulator.outside_near_island_edge_samples += 1;
    }
    accumulator.max_resident_island_visual_count = accumulator
        .max_resident_island_visual_count
        .max(sample.resident_island_visual_count);
    accumulator.max_stream_visibility_changes_per_frame = accumulator
        .max_stream_visibility_changes_per_frame
        .max(sample.max_stream_visibility_changes_per_frame);
    accumulator.total_stream_visibility_changes = accumulator
        .total_stream_visibility_changes
        .max(sample.total_stream_visibility_changes);
    accumulator.max_catalog_island_visual_count = accumulator
        .max_catalog_island_visual_count
        .max(sample.catalog_island_visual_count);
    accumulator.max_hidden_island_visual_count = accumulator
        .max_hidden_island_visual_count
        .max(sample.hidden_island_visual_count);
    accumulator.max_resident_island_visual_fraction = accumulator
        .max_resident_island_visual_fraction
        .max(sample.resident_island_visual_fraction);
    accumulator.max_stream_spawned_visuals_per_frame = accumulator
        .max_stream_spawned_visuals_per_frame
        .max(sample.max_stream_spawned_visuals_per_frame);
    accumulator.max_stream_despawned_visuals_per_frame = accumulator
        .max_stream_despawned_visuals_per_frame
        .max(sample.max_stream_despawned_visuals_per_frame);
    accumulator.total_stream_spawned_visuals = accumulator
        .total_stream_spawned_visuals
        .max(sample.total_stream_spawned_visuals);
    accumulator.total_stream_despawned_visuals = accumulator
        .total_stream_despawned_visuals
        .max(sample.total_stream_despawned_visuals);
    accumulator.max_entity_count = accumulator.max_entity_count.max(sample.entity_count);
}

fn wind_load_response_sample(sample: &EvalSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.movement_input_lateral_axis.abs() < 0.25
        && sample.crosswind_force_fields > 0
        && sample.max_crosswind_force_delta_mps >= MIN_CROSSWIND_FORCE_DELTA_MPS
        && sample.max_wind_force_variation >= MIN_WIND_FORCE_VARIATION
        && sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
        && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
        && sample.wind_lateral_load.abs() >= MIN_WIND_LOAD_LATERAL_LOAD
}

fn observe_crosswind_neutral_drift(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if !crosswind_neutral_response_sample(sample) {
        accumulator.crosswind_neutral_previous_position = None;
        return;
    }

    accumulator.crosswind_neutral_drift_samples += 1;
    let current_position = horizontal_position(sample);
    let Some(previous_position) = accumulator
        .crosswind_neutral_previous_position
        .replace(current_position)
    else {
        return;
    };

    let Some(direction) = crosswind_horizontal_direction(sample) else {
        return;
    };
    let step = (current_position - previous_position)
        .dot(direction)
        .max(0.0);
    accumulator.crosswind_neutral_horizontal_drift_m += step;
    accumulator.max_crosswind_neutral_horizontal_step_m = accumulator
        .max_crosswind_neutral_horizontal_step_m
        .max(step);
}

fn crosswind_neutral_response_sample(sample: &EvalSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.movement_input_lateral_axis.abs() < 0.25
        && sample.crosswind_force_fields > 0
        && sample.max_crosswind_force_delta_mps >= MIN_CROSSWIND_FORCE_DELTA_MPS
        && sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
        && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
}

fn horizontal_position(sample: &EvalSample) -> Vec3 {
    Vec3::new(sample.position[0], 0.0, sample.position[2])
}

fn crosswind_horizontal_direction(sample: &EvalSample) -> Option<Vec3> {
    let direction = Vec3::new(
        sample.crosswind_force_delta[0],
        0.0,
        sample.crosswind_force_delta[2],
    );
    if direction.length_squared() > f32::EPSILON {
        Some(dominant_horizontal_axis(direction))
    } else {
        (sample.max_crosswind_force_delta_mps > f32::EPSILON).then_some(Vec3::X)
    }
}

fn dominant_horizontal_axis(direction: Vec3) -> Vec3 {
    if direction.x.abs() >= direction.z.abs() {
        Vec3::X * direction.x.signum()
    } else {
        Vec3::Z * direction.z.signum()
    }
}

fn has_sustained_updraft_visual_flow(sample: &EvalSample) -> bool {
    let has_enough_guides = sustained_visual_count(
        sample.updraft_guide_visual_count,
        MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
    ) && sustained_visual_count(
        sample.updraft_ribbon_visual_count,
        MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
    );
    let has_strong_motion = sustained_visual_floor(
        sample.max_updraft_visual_motion_m,
        MIN_UPDRAFT_VISUAL_MOTION_M,
    ) && sustained_visual_floor(
        sample.max_updraft_visual_rise_m,
        MIN_UPDRAFT_VISUAL_RISE_M,
    ) && sustained_visual_floor(
        sample.max_updraft_visual_swirl_displacement_m,
        MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
    ) && sustained_visual_floor(
        sample.max_updraft_visual_depth_span_m,
        MIN_UPDRAFT_VISUAL_DEPTH_SPAN_M,
    ) && sustained_visual_floor(
        sample.max_updraft_visual_scale_pulse,
        MIN_UPDRAFT_VISUAL_SCALE_PULSE,
    );
    let has_flow_coherence = sustained_visual_count(
        sample.updraft_flow_coherent_visual_count,
        MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
    ) && sample.max_updraft_visual_flow_alignment
        >= MIN_WIND_VISUAL_FLOW_ALIGNMENT;
    let has_observed_frame_motion = sustained_visual_count(
        sample.observed_updraft_flow_coherent_visual_count,
        MIN_UPDRAFT_FLOW_COHERENT_VISUAL_COUNT,
    ) && sample.max_observed_updraft_visual_flow_alignment
        >= MIN_WIND_VISUAL_FLOW_ALIGNMENT
        && sample.max_observed_updraft_visual_frame_motion_m
            >= MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_MOTION_M
        && sample.max_observed_updraft_visual_frame_rise_m
            >= MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_RISE_M
        && sample.max_observed_updraft_visual_frame_swirl_displacement_m
            >= MIN_OBSERVED_UPDRAFT_VISUAL_FRAME_SWIRL_DISPLACEMENT_M;

    has_enough_guides && has_strong_motion && has_flow_coherence && has_observed_frame_motion
}

fn has_sustained_crosswind_visual_flow(sample: &EvalSample) -> bool {
    let has_enough_guides = sustained_visual_count(
        sample.crosswind_guide_visual_count,
        MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
    ) && sustained_visual_count(
        sample.crosswind_ribbon_visual_count,
        MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
    );
    let has_strong_motion = sustained_visual_floor(
        sample.max_crosswind_visual_motion_m,
        MIN_CROSSWIND_VISUAL_MOTION_M,
    ) && sustained_visual_floor(
        sample.max_crosswind_guide_flow_displacement_m,
        MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
    ) && sustained_visual_floor(
        sample.max_crosswind_ribbon_flow_displacement_m,
        MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
    ) && sustained_visual_floor(
        sample.max_crosswind_visual_lane_depth_span_m,
        MIN_CROSSWIND_VISUAL_LANE_DEPTH_SPAN_M,
    ) && sustained_visual_floor(
        sample.max_crosswind_visual_scale_pulse,
        MIN_CROSSWIND_VISUAL_SCALE_PULSE,
    );
    let has_flow_coherence = sustained_visual_count(
        sample.crosswind_flow_coherent_visual_count,
        MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
    ) && sample.max_crosswind_visual_flow_alignment
        >= MIN_WIND_VISUAL_FLOW_ALIGNMENT;
    let has_observed_frame_motion = sustained_visual_count(
        sample.observed_crosswind_flow_coherent_visual_count,
        MIN_CROSSWIND_FLOW_COHERENT_VISUAL_COUNT,
    ) && sample.max_observed_crosswind_visual_flow_alignment
        >= MIN_WIND_VISUAL_FLOW_ALIGNMENT
        && sample.max_observed_crosswind_visual_frame_motion_m
            >= MIN_OBSERVED_CROSSWIND_VISUAL_FRAME_MOTION_M
        && sample.max_observed_crosswind_guide_frame_flow_displacement_m
            >= MIN_OBSERVED_CROSSWIND_GUIDE_FRAME_FLOW_DISPLACEMENT_M
        && sample.max_observed_crosswind_ribbon_frame_flow_displacement_m
            >= MIN_OBSERVED_CROSSWIND_RIBBON_FRAME_FLOW_DISPLACEMENT_M;

    has_enough_guides && has_strong_motion && has_flow_coherence && has_observed_frame_motion
}

fn has_sustained_crosswind_ribbon_advected_flow(sample: &EvalSample) -> bool {
    sustained_visual_count(
        sample.crosswind_ribbon_flow_coherent_sample_count,
        MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT,
    ) && sample.max_crosswind_ribbon_visual_flow_alignment >= MIN_WIND_VISUAL_FLOW_ALIGNMENT
        && sustained_visual_count(
            sample.observed_crosswind_ribbon_flow_coherent_sample_count,
            MIN_CROSSWIND_RIBBON_FLOW_COHERENT_SAMPLE_COUNT,
        )
        && sample.max_observed_crosswind_ribbon_visual_flow_alignment
            >= MIN_WIND_VISUAL_FLOW_ALIGNMENT
}

fn sustained_visual_floor(value: f32, floor: f32) -> bool {
    value >= floor * SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO
}

fn sustained_visual_count(value: usize, floor: usize) -> bool {
    value >= ((floor as f32) * SUSTAINED_WIND_VISUAL_FLOW_FLOOR_RATIO).ceil() as usize
}
