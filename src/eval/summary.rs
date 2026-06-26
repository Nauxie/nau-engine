use super::{EvalSample, EvalThresholds, json_number, json_string};

#[derive(Clone, Debug)]
pub struct EvalArtifacts {
    pub summary_json: String,
    pub samples_ndjson: String,
    pub screenshot_png: Option<String>,
    pub checkpoint_screenshots: Vec<String>,
    pub checkpoint_marker_metadata: Vec<String>,
}

impl EvalArtifacts {
    fn to_json(&self, indent: &str) -> String {
        let screenshot = self
            .screenshot_png
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".to_string());
        let checkpoint_screenshots = json_string_array(&self.checkpoint_screenshots);
        let checkpoint_marker_metadata = json_string_array(&self.checkpoint_marker_metadata);

        format!(
            "{{\n{indent}  \"summary_json\": {},\n{indent}  \"samples_ndjson\": {},\n{indent}  \"screenshot_png\": {},\n{indent}  \"checkpoint_screenshots\": {},\n{indent}  \"checkpoint_marker_metadata\": {}\n{indent}}}",
            json_string(&self.summary_json),
            json_string(&self.samples_ndjson),
            screenshot,
            checkpoint_screenshots,
            checkpoint_marker_metadata,
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvalMetricsSummary {
    pub sample_count: u32,
    pub avg_frame_time_ms: f32,
    pub p95_frame_time_ms: f32,
    pub p99_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub horizontal_distance_m: f32,
    pub max_altitude_m: f32,
    pub min_altitude_m: f32,
    pub max_grounded_visual_foot_gap_m: f32,
    pub max_speed_mps: f32,
    pub max_camera_distance_m: f32,
    pub min_camera_surface_clearance_m: f32,
    pub max_camera_player_angle_degrees: f32,
    pub max_camera_step_distance_m: f32,
    pub max_camera_rotation_delta_degrees: f32,
    pub max_camera_orbit_alignment_degrees: f32,
    pub avg_camera_follow_direction_error_degrees: f32,
    pub p95_camera_follow_direction_error_degrees: f32,
    pub max_camera_follow_direction_error_degrees: f32,
    pub max_abs_camera_view_yaw_degrees: f32,
    pub max_camera_view_yaw_drift_degrees: f32,
    pub max_camera_world_yaw_drift_degrees: f32,
    pub max_camera_obstruction_adjustment_m: f32,
    pub max_camera_obstruction_hits: usize,
    pub avg_desired_body_heading_error_degrees: f32,
    pub p95_desired_body_heading_error_degrees: f32,
    pub max_desired_body_heading_error_degrees: f32,
    pub max_body_yaw_error_step_degrees: f32,
    pub body_yaw_oscillation_count: u32,
    pub max_body_roll_step_degrees: f32,
    pub max_right_body_bank_degrees: f32,
    pub max_left_body_bank_degrees: f32,
    pub max_desired_heading_alignment_mps: f32,
    pub max_lateral_response_mps: f32,
    pub lateral_response_latency_secs: f32,
    pub max_right_lateral_response_mps: f32,
    pub right_lateral_response_latency_secs: f32,
    pub max_left_lateral_response_mps: f32,
    pub left_lateral_response_latency_secs: f32,
    pub max_backward_lateral_response_mps: f32,
    pub backward_lateral_response_latency_secs: f32,
    pub max_backward_right_lateral_response_mps: f32,
    pub backward_right_lateral_response_latency_secs: f32,
    pub max_backward_right_rear_response_mps: f32,
    pub max_backward_left_lateral_response_mps: f32,
    pub backward_left_lateral_response_latency_secs: f32,
    pub max_backward_left_rear_response_mps: f32,
    pub max_air_brake_speed_drop_mps: f32,
    pub max_air_brake_planar_speed_drop_mps: f32,
    pub max_post_brake_forward_alignment_mps: f32,
    pub max_pose_torso_pitch_degrees: f32,
    pub max_pose_arm_spread_degrees: f32,
    pub max_pose_leg_tuck_degrees: f32,
    pub max_pose_lateral_lean_degrees: f32,
    pub max_pose_landing_crouch_m: f32,
    pub max_pose_wing_airflow_strength: f32,
    pub min_key_pose_readability_score: f32,
    pub max_key_pose_readability_score: f32,
    pub unreadable_key_pose_samples: u32,
    pub min_target_distance_m: f32,
    pub final_target_distance_m: f32,
    pub min_camera_pitch_degrees: f32,
    pub max_camera_pitch_degrees: f32,
    pub max_abs_camera_yaw_offset_degrees: f32,
    pub min_camera_pitch_offset_degrees: f32,
    pub max_camera_pitch_offset_degrees: f32,
    pub max_visible_wind_fields: usize,
    pub max_dynamic_wind_flow_fields: usize,
    pub max_wind_flow_speed_mps: f32,
    pub max_wind_flow_variation: f32,
    pub max_wind_flow_variation_range: f32,
    pub wind_force_samples: u32,
    pub crosswind_force_samples: u32,
    pub updraft_swirl_force_samples: u32,
    pub max_active_wind_force_fields: usize,
    pub max_crosswind_force_fields: usize,
    pub max_updraft_swirl_force_fields: usize,
    pub max_wind_force_delta_mps: f32,
    pub max_crosswind_force_delta_mps: f32,
    pub max_updraft_swirl_force_delta_mps: f32,
    pub max_wind_force_flow_speed_mps: f32,
    pub max_wind_force_variation: f32,
    pub max_active_lift_fields: usize,
    pub max_readable_lift_fields: usize,
    pub max_sky_island_count: usize,
    pub max_active_chunk_count: usize,
    pub max_active_island_count: usize,
    pub max_near_lod_islands: usize,
    pub max_mid_lod_islands: usize,
    pub max_far_lod_islands: usize,
    pub max_visible_island_terrain_count: usize,
    pub max_hidden_island_terrain_count: usize,
    pub max_visible_island_impostor_count: usize,
    pub max_hidden_island_impostor_count: usize,
    pub max_visible_island_detail_count: usize,
    pub max_hidden_island_detail_count: usize,
    pub max_visible_route_beacon_count: usize,
    pub max_weather_cloud_count: usize,
    pub max_environment_motion_visual_count: usize,
    pub max_environment_motion_offset_m: f32,
    pub max_updraft_guide_visual_count: usize,
    pub max_updraft_ribbon_visual_count: usize,
    pub max_crosswind_guide_visual_count: usize,
    pub max_crosswind_ribbon_visual_count: usize,
    pub max_updraft_visual_motion_m: f32,
    pub max_crosswind_visual_motion_m: f32,
    pub max_world_collision_proxy_count: usize,
    pub world_collision_resolved_samples: u32,
    pub world_collision_contact_samples: u32,
    pub max_world_collision_push_m: f32,
    pub min_island_terrain_surface_count: usize,
    pub min_island_terrain_mesh_vertices: usize,
    pub min_island_terrain_color_bands: usize,
    pub min_island_terrain_material_weight_bands: usize,
    pub min_island_terrain_material_channels: usize,
    pub min_island_terrain_material_regions: usize,
    pub min_island_terrain_texture_detail_bands: usize,
    pub min_island_terrain_relief_range_m: f32,
    pub min_island_terrain_archetype_count: usize,
    pub min_island_cliff_color_bands: usize,
    pub min_island_impostor_mesh_vertices: usize,
    pub min_island_impostor_color_bands: usize,
    pub min_procedural_island_body_count: usize,
    pub max_primitive_island_body_count: usize,
    pub min_island_body_silhouette_segments: usize,
    pub max_avg_island_body_silhouette_segments: f32,
    pub min_island_body_mesh_vertices: usize,
    pub max_island_body_mesh_vertices: usize,
    pub min_generated_ground_cover_patch_count: usize,
    pub min_ground_cover_blade_count: usize,
    pub min_ground_cover_mesh_vertices: usize,
    pub min_generated_tree_trunk_count: usize,
    pub min_generated_tree_canopy_count: usize,
    pub min_tree_trunk_mesh_vertices: usize,
    pub min_tree_canopy_mesh_vertices: usize,
    pub min_detail_biome_palette_count: usize,
    pub min_generated_rock_count: usize,
    pub min_rock_mesh_vertices: usize,
    pub min_generated_landmark_count: usize,
    pub min_generated_route_cairn_count: usize,
    pub min_generated_launch_beacon_count: usize,
    pub min_generated_landing_garden_marker_count: usize,
    pub min_generated_pond_surface_count: usize,
    pub min_landmark_mesh_vertices: usize,
    pub min_generated_weather_cloud_count: usize,
    pub min_generated_weather_cloud_bank_count: usize,
    pub min_weather_cloud_bank_depth_m: f32,
    pub min_weather_cloud_lobe_count: usize,
    pub min_max_weather_cloud_lobe_count: usize,
    pub min_weather_cloud_mesh_vertices: usize,
    pub min_weather_cloud_filament_ribbon_detail_count: usize,
    pub max_resident_island_visual_count: usize,
    pub max_stream_visibility_changes_per_frame: usize,
    pub total_stream_visibility_changes: usize,
    pub max_catalog_island_visual_count: usize,
    pub max_hidden_island_visual_count: usize,
    pub max_resident_island_visual_fraction: f32,
    pub max_stream_spawned_visuals_per_frame: usize,
    pub max_stream_despawned_visuals_per_frame: usize,
    pub total_stream_spawned_visuals: usize,
    pub total_stream_despawned_visuals: usize,
    pub max_entity_count: usize,
    pub objective_total_count: usize,
    pub max_completed_objective_count: usize,
    pub final_objective_completed_count: usize,
    pub min_objective_distance_m: f32,
    pub final_objective_distance_m: f32,
    pub objective_complete_samples: u32,
    pub max_visual_asset_slot_count: usize,
    pub max_gltf_scene_asset_slot_count: usize,
    pub max_ready_visual_asset_slot_count: usize,
    pub max_placeholder_visual_asset_slot_count: usize,
    pub max_streaming_visual_asset_slot_count: usize,
    pub max_missing_visual_asset_slot_count: usize,
    pub max_deferred_visual_asset_scene_count: usize,
    pub max_queued_visual_asset_scene_count: usize,
    pub max_loading_visual_asset_scene_count: usize,
    pub max_loaded_visual_asset_scene_count: usize,
    pub max_dependency_loaded_visual_asset_scene_count: usize,
    pub max_preload_ready_visual_asset_scene_count: usize,
    pub max_failed_visual_asset_scene_count: usize,
    pub max_spawned_visual_asset_scene_count: usize,
    pub max_ready_visual_asset_scene_count: usize,
    pub max_visible_authored_world_fixture_count: usize,
    pub max_always_visual_asset_slot_count: usize,
    pub max_stream_window_visual_asset_slot_count: usize,
    pub max_near_lod_visual_asset_slot_count: usize,
    pub max_far_lod_visual_asset_slot_count: usize,
    pub max_weather_visual_asset_slot_count: usize,
    pub max_always_preload_ready_visual_asset_slot_count: usize,
    pub max_streaming_preload_ready_visual_asset_slot_count: usize,
    pub max_declared_animation_clip_count: usize,
    pub max_ready_animation_clip_count: usize,
    pub max_animation_player_count: usize,
    pub max_animation_graph_count: usize,
    pub max_power_up_count: usize,
    pub min_visible_power_up_count: usize,
    pub max_collected_power_up_count: usize,
    pub power_up_effect_samples: u32,
    pub total_power_up_activations: usize,
    pub target_landing_samples: u32,
    pub lifted_samples: u32,
    pub readable_lift_samples: u32,
    pub unreadable_lift_samples: u32,
    pub dynamic_readable_lift_samples: u32,
    pub pose_gliding_samples: u32,
    pub pose_diving_samples: u32,
    pub pose_air_brake_samples: u32,
    pub pose_landing_anticipation_samples: u32,
    pub pose_landing_recovery_samples: u32,
    pub gliding_samples: u32,
    pub launching_samples: u32,
    pub grounded_samples: u32,
}

impl EvalMetricsSummary {
    fn to_json(&self, indent: &str) -> String {
        let json = format!(
            "{{\n{indent}  \"sample_count\": {},\n{indent}  \"avg_frame_time_ms\": {},\n{indent}  \"p95_frame_time_ms\": {},\n{indent}  \"p99_frame_time_ms\": {},\n{indent}  \"max_frame_time_ms\": {},\n{indent}  \"horizontal_distance_m\": {},\n{indent}  \"max_altitude_m\": {},\n{indent}  \"min_altitude_m\": {},\n{indent}  \"max_grounded_visual_foot_gap_m\": {},\n{indent}  \"max_speed_mps\": {},\n{indent}  \"max_camera_distance_m\": {},\n{indent}  \"min_camera_surface_clearance_m\": {},\n{indent}  \"max_camera_player_angle_degrees\": {},\n{indent}  \"max_camera_step_distance_m\": {},\n{indent}  \"max_camera_rotation_delta_degrees\": {},\n{indent}  \"max_camera_orbit_alignment_degrees\": {},\n{indent}  \"avg_camera_follow_direction_error_degrees\": {},\n{indent}  \"p95_camera_follow_direction_error_degrees\": {},\n{indent}  \"max_camera_follow_direction_error_degrees\": {},\n{indent}  \"max_abs_camera_view_yaw_degrees\": {},\n{indent}  \"max_camera_view_yaw_drift_degrees\": {},\n{indent}  \"max_camera_world_yaw_drift_degrees\": {},\n{indent}  \"max_camera_obstruction_adjustment_m\": {},\n{indent}  \"max_camera_obstruction_hits\": {},\n{indent}  \"avg_desired_body_heading_error_degrees\": {},\n{indent}  \"p95_desired_body_heading_error_degrees\": {},\n{indent}  \"max_desired_body_heading_error_degrees\": {},\n{indent}  \"max_body_yaw_error_step_degrees\": {},\n{indent}  \"body_yaw_oscillation_count\": {},\n{indent}  \"max_body_roll_step_degrees\": {},\n{indent}  \"max_right_body_bank_degrees\": {},\n{indent}  \"max_left_body_bank_degrees\": {},\n{indent}  \"max_desired_heading_alignment_mps\": {},\n{indent}  \"max_lateral_response_mps\": {},\n{indent}  \"lateral_response_latency_secs\": {},\n{indent}  \"max_right_lateral_response_mps\": {},\n{indent}  \"right_lateral_response_latency_secs\": {},\n{indent}  \"max_left_lateral_response_mps\": {},\n{indent}  \"left_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_lateral_response_mps\": {},\n{indent}  \"backward_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_right_lateral_response_mps\": {},\n{indent}  \"backward_right_lateral_response_latency_secs\": {},\n{indent}  \"max_backward_left_lateral_response_mps\": {},\n{indent}  \"backward_left_lateral_response_latency_secs\": {},\n{indent}  \"max_air_brake_speed_drop_mps\": {},\n{indent}  \"max_post_brake_forward_alignment_mps\": {},\n{indent}  \"min_target_distance_m\": {},\n{indent}  \"final_target_distance_m\": {},\n{indent}  \"min_camera_pitch_degrees\": {},\n{indent}  \"max_camera_pitch_degrees\": {},\n{indent}  \"max_abs_camera_yaw_offset_degrees\": {},\n{indent}  \"min_camera_pitch_offset_degrees\": {},\n{indent}  \"max_camera_pitch_offset_degrees\": {},\n{indent}  \"max_visible_wind_fields\": {},\n{indent}  \"max_dynamic_wind_flow_fields\": {},\n{indent}  \"max_wind_flow_speed_mps\": {},\n{indent}  \"max_wind_flow_variation\": {},\n{indent}  \"max_wind_flow_variation_range\": {},\n{indent}  \"max_active_lift_fields\": {},\n{indent}  \"max_readable_lift_fields\": {},\n{indent}  \"max_sky_island_count\": {},\n{indent}  \"max_active_chunk_count\": {},\n{indent}  \"max_active_island_count\": {},\n{indent}  \"max_near_lod_islands\": {},\n{indent}  \"max_mid_lod_islands\": {},\n{indent}  \"max_far_lod_islands\": {},\n{indent}  \"max_visible_island_terrain_count\": {},\n{indent}  \"max_hidden_island_terrain_count\": {},\n{indent}  \"max_visible_island_impostor_count\": {},\n{indent}  \"max_hidden_island_impostor_count\": {},\n{indent}  \"max_visible_island_detail_count\": {},\n{indent}  \"max_hidden_island_detail_count\": {},\n{indent}  \"max_visible_route_beacon_count\": {},\n{indent}  \"max_weather_cloud_count\": {},\n{indent}  \"max_environment_motion_visual_count\": {},\n{indent}  \"max_environment_motion_offset_m\": {},\n{indent}  \"min_island_terrain_surface_count\": {},\n{indent}  \"min_island_terrain_mesh_vertices\": {},\n{indent}  \"min_island_terrain_color_bands\": {},\n{indent}  \"min_island_terrain_material_weight_bands\": {},\n{indent}  \"min_island_terrain_material_channels\": {},\n{indent}  \"min_island_terrain_material_regions\": {},\n{indent}  \"min_island_terrain_texture_detail_bands\": {},\n{indent}  \"min_island_terrain_relief_range_m\": {},\n{indent}  \"min_island_terrain_archetype_count\": {},\n{indent}  \"min_island_cliff_color_bands\": {},\n{indent}  \"min_procedural_island_body_count\": {},\n{indent}  \"max_primitive_island_body_count\": {},\n{indent}  \"min_island_body_silhouette_segments\": {},\n{indent}  \"max_avg_island_body_silhouette_segments\": {},\n{indent}  \"max_island_body_mesh_vertices\": {},\n{indent}  \"min_generated_ground_cover_patch_count\": {},\n{indent}  \"min_ground_cover_blade_count\": {},\n{indent}  \"min_ground_cover_mesh_vertices\": {},\n{indent}  \"min_generated_tree_trunk_count\": {},\n{indent}  \"min_generated_tree_canopy_count\": {},\n{indent}  \"min_tree_trunk_mesh_vertices\": {},\n{indent}  \"min_tree_canopy_mesh_vertices\": {},\n{indent}  \"min_detail_biome_palette_count\": {},\n{indent}  \"min_generated_rock_count\": {},\n{indent}  \"min_rock_mesh_vertices\": {},\n{indent}  \"min_generated_landmark_count\": {},\n{indent}  \"min_generated_route_cairn_count\": {},\n{indent}  \"min_generated_launch_beacon_count\": {},\n{indent}  \"min_generated_landing_garden_marker_count\": {},\n{indent}  \"min_generated_pond_surface_count\": {},\n{indent}  \"min_landmark_mesh_vertices\": {},\n{indent}  \"min_generated_weather_cloud_count\": {},\n{indent}  \"min_generated_weather_cloud_bank_count\": {},\n{indent}  \"min_weather_cloud_bank_depth_m\": {},\n{indent}  \"min_weather_cloud_lobe_count\": {},\n{indent}  \"min_max_weather_cloud_lobe_count\": {},\n{indent}  \"min_weather_cloud_mesh_vertices\": {},\n{indent}  \"min_weather_cloud_filament_ribbon_detail_count\": {},\n{indent}  \"max_resident_island_visual_count\": {},\n{indent}  \"max_stream_visibility_changes_per_frame\": {},\n{indent}  \"total_stream_visibility_changes\": {},\n{indent}  \"max_catalog_island_visual_count\": {},\n{indent}  \"max_hidden_island_visual_count\": {},\n{indent}  \"max_resident_island_visual_fraction\": {},\n{indent}  \"max_stream_spawned_visuals_per_frame\": {},\n{indent}  \"max_stream_despawned_visuals_per_frame\": {},\n{indent}  \"total_stream_spawned_visuals\": {},\n{indent}  \"total_stream_despawned_visuals\": {},\n{indent}  \"max_entity_count\": {},\n{indent}  \"objective_total_count\": {},\n{indent}  \"max_completed_objective_count\": {},\n{indent}  \"final_objective_completed_count\": {},\n{indent}  \"min_objective_distance_m\": {},\n{indent}  \"final_objective_distance_m\": {},\n{indent}  \"objective_complete_samples\": {},\n{indent}  \"max_visual_asset_slot_count\": {},\n{indent}  \"max_gltf_scene_asset_slot_count\": {},\n{indent}  \"max_ready_visual_asset_slot_count\": {},\n{indent}  \"max_placeholder_visual_asset_slot_count\": {},\n{indent}  \"max_streaming_visual_asset_slot_count\": {},\n{indent}  \"max_missing_visual_asset_slot_count\": {},\n{indent}  \"max_queued_visual_asset_scene_count\": {},\n{indent}  \"max_loading_visual_asset_scene_count\": {},\n{indent}  \"max_loaded_visual_asset_scene_count\": {},\n{indent}  \"max_dependency_loaded_visual_asset_scene_count\": {},\n{indent}  \"max_preload_ready_visual_asset_scene_count\": {},\n{indent}  \"max_failed_visual_asset_scene_count\": {},\n{indent}  \"max_spawned_visual_asset_scene_count\": {},\n{indent}  \"max_ready_visual_asset_scene_count\": {},\n{indent}  \"max_visible_authored_world_fixture_count\": {},\n{indent}  \"max_always_visual_asset_slot_count\": {},\n{indent}  \"max_stream_window_visual_asset_slot_count\": {},\n{indent}  \"max_near_lod_visual_asset_slot_count\": {},\n{indent}  \"max_far_lod_visual_asset_slot_count\": {},\n{indent}  \"max_weather_visual_asset_slot_count\": {},\n{indent}  \"max_always_preload_ready_visual_asset_slot_count\": {},\n{indent}  \"max_streaming_preload_ready_visual_asset_slot_count\": {},\n{indent}  \"max_declared_animation_clip_count\": {},\n{indent}  \"max_ready_animation_clip_count\": {},\n{indent}  \"max_animation_player_count\": {},\n{indent}  \"max_animation_graph_count\": {},\n{indent}  \"max_power_up_count\": {},\n{indent}  \"min_visible_power_up_count\": {},\n{indent}  \"max_collected_power_up_count\": {},\n{indent}  \"power_up_effect_samples\": {},\n{indent}  \"total_power_up_activations\": {},\n{indent}  \"target_landing_samples\": {},\n{indent}  \"lifted_samples\": {},\n{indent}  \"readable_lift_samples\": {},\n{indent}  \"unreadable_lift_samples\": {},\n{indent}  \"dynamic_readable_lift_samples\": {},\n{indent}  \"pose_gliding_samples\": {},\n{indent}  \"pose_diving_samples\": {},\n{indent}  \"pose_air_brake_samples\": {},\n{indent}  \"pose_landing_anticipation_samples\": {},\n{indent}  \"pose_landing_recovery_samples\": {},\n{indent}  \"gliding_samples\": {},\n{indent}  \"launching_samples\": {},\n{indent}  \"grounded_samples\": {}\n{indent}}}",
            self.sample_count,
            json_number(self.avg_frame_time_ms),
            json_number(self.p95_frame_time_ms),
            json_number(self.p99_frame_time_ms),
            json_number(self.max_frame_time_ms),
            json_number(self.horizontal_distance_m),
            json_number(self.max_altitude_m),
            json_number(self.min_altitude_m),
            json_number(self.max_grounded_visual_foot_gap_m),
            json_number(self.max_speed_mps),
            json_number(self.max_camera_distance_m),
            json_number(self.min_camera_surface_clearance_m),
            json_number(self.max_camera_player_angle_degrees),
            json_number(self.max_camera_step_distance_m),
            json_number(self.max_camera_rotation_delta_degrees),
            json_number(self.max_camera_orbit_alignment_degrees),
            json_number(self.avg_camera_follow_direction_error_degrees),
            json_number(self.p95_camera_follow_direction_error_degrees),
            json_number(self.max_camera_follow_direction_error_degrees),
            json_number(self.max_abs_camera_view_yaw_degrees),
            json_number(self.max_camera_view_yaw_drift_degrees),
            json_number(self.max_camera_world_yaw_drift_degrees),
            json_number(self.max_camera_obstruction_adjustment_m),
            self.max_camera_obstruction_hits,
            json_number(self.avg_desired_body_heading_error_degrees),
            json_number(self.p95_desired_body_heading_error_degrees),
            json_number(self.max_desired_body_heading_error_degrees),
            json_number(self.max_body_yaw_error_step_degrees),
            self.body_yaw_oscillation_count,
            json_number(self.max_body_roll_step_degrees),
            json_number(self.max_right_body_bank_degrees),
            json_number(self.max_left_body_bank_degrees),
            json_number(self.max_desired_heading_alignment_mps),
            json_number(self.max_lateral_response_mps),
            json_number(self.lateral_response_latency_secs),
            json_number(self.max_right_lateral_response_mps),
            json_number(self.right_lateral_response_latency_secs),
            json_number(self.max_left_lateral_response_mps),
            json_number(self.left_lateral_response_latency_secs),
            json_number(self.max_backward_lateral_response_mps),
            json_number(self.backward_lateral_response_latency_secs),
            json_number(self.max_backward_right_lateral_response_mps),
            json_number(self.backward_right_lateral_response_latency_secs),
            json_number(self.max_backward_left_lateral_response_mps),
            json_number(self.backward_left_lateral_response_latency_secs),
            json_number(self.max_air_brake_speed_drop_mps),
            json_number(self.max_post_brake_forward_alignment_mps),
            json_number(self.min_target_distance_m),
            json_number(self.final_target_distance_m),
            json_number(self.min_camera_pitch_degrees),
            json_number(self.max_camera_pitch_degrees),
            json_number(self.max_abs_camera_yaw_offset_degrees),
            json_number(self.min_camera_pitch_offset_degrees),
            json_number(self.max_camera_pitch_offset_degrees),
            self.max_visible_wind_fields,
            self.max_dynamic_wind_flow_fields,
            json_number(self.max_wind_flow_speed_mps),
            json_number(self.max_wind_flow_variation),
            json_number(self.max_wind_flow_variation_range),
            self.max_active_lift_fields,
            self.max_readable_lift_fields,
            self.max_sky_island_count,
            self.max_active_chunk_count,
            self.max_active_island_count,
            self.max_near_lod_islands,
            self.max_mid_lod_islands,
            self.max_far_lod_islands,
            self.max_visible_island_terrain_count,
            self.max_hidden_island_terrain_count,
            self.max_visible_island_impostor_count,
            self.max_hidden_island_impostor_count,
            self.max_visible_island_detail_count,
            self.max_hidden_island_detail_count,
            self.max_visible_route_beacon_count,
            self.max_weather_cloud_count,
            self.max_environment_motion_visual_count,
            json_number(self.max_environment_motion_offset_m),
            self.min_island_terrain_surface_count,
            self.min_island_terrain_mesh_vertices,
            self.min_island_terrain_color_bands,
            self.min_island_terrain_material_weight_bands,
            self.min_island_terrain_material_channels,
            self.min_island_terrain_material_regions,
            self.min_island_terrain_texture_detail_bands,
            json_number(self.min_island_terrain_relief_range_m),
            self.min_island_terrain_archetype_count,
            self.min_island_cliff_color_bands,
            self.min_procedural_island_body_count,
            self.max_primitive_island_body_count,
            self.min_island_body_silhouette_segments,
            json_number(self.max_avg_island_body_silhouette_segments),
            self.max_island_body_mesh_vertices,
            self.min_generated_ground_cover_patch_count,
            self.min_ground_cover_blade_count,
            self.min_ground_cover_mesh_vertices,
            self.min_generated_tree_trunk_count,
            self.min_generated_tree_canopy_count,
            self.min_tree_trunk_mesh_vertices,
            self.min_tree_canopy_mesh_vertices,
            self.min_detail_biome_palette_count,
            self.min_generated_rock_count,
            self.min_rock_mesh_vertices,
            self.min_generated_landmark_count,
            self.min_generated_route_cairn_count,
            self.min_generated_launch_beacon_count,
            self.min_generated_landing_garden_marker_count,
            self.min_generated_pond_surface_count,
            self.min_landmark_mesh_vertices,
            self.min_generated_weather_cloud_count,
            self.min_generated_weather_cloud_bank_count,
            json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_lobe_count,
            self.min_max_weather_cloud_lobe_count,
            self.min_weather_cloud_mesh_vertices,
            self.min_weather_cloud_filament_ribbon_detail_count,
            self.max_resident_island_visual_count,
            self.max_stream_visibility_changes_per_frame,
            self.total_stream_visibility_changes,
            self.max_catalog_island_visual_count,
            self.max_hidden_island_visual_count,
            json_number(self.max_resident_island_visual_fraction),
            self.max_stream_spawned_visuals_per_frame,
            self.max_stream_despawned_visuals_per_frame,
            self.total_stream_spawned_visuals,
            self.total_stream_despawned_visuals,
            self.max_entity_count,
            self.objective_total_count,
            self.max_completed_objective_count,
            self.final_objective_completed_count,
            json_number(self.min_objective_distance_m),
            json_number(self.final_objective_distance_m),
            self.objective_complete_samples,
            self.max_visual_asset_slot_count,
            self.max_gltf_scene_asset_slot_count,
            self.max_ready_visual_asset_slot_count,
            self.max_placeholder_visual_asset_slot_count,
            self.max_streaming_visual_asset_slot_count,
            self.max_missing_visual_asset_slot_count,
            self.max_queued_visual_asset_scene_count,
            self.max_loading_visual_asset_scene_count,
            self.max_loaded_visual_asset_scene_count,
            self.max_dependency_loaded_visual_asset_scene_count,
            self.max_preload_ready_visual_asset_scene_count,
            self.max_failed_visual_asset_scene_count,
            self.max_spawned_visual_asset_scene_count,
            self.max_ready_visual_asset_scene_count,
            self.max_visible_authored_world_fixture_count,
            self.max_always_visual_asset_slot_count,
            self.max_stream_window_visual_asset_slot_count,
            self.max_near_lod_visual_asset_slot_count,
            self.max_far_lod_visual_asset_slot_count,
            self.max_weather_visual_asset_slot_count,
            self.max_always_preload_ready_visual_asset_slot_count,
            self.max_streaming_preload_ready_visual_asset_slot_count,
            self.max_declared_animation_clip_count,
            self.max_ready_animation_clip_count,
            self.max_animation_player_count,
            self.max_animation_graph_count,
            self.max_power_up_count,
            self.min_visible_power_up_count,
            self.max_collected_power_up_count,
            self.power_up_effect_samples,
            self.total_power_up_activations,
            self.target_landing_samples,
            self.lifted_samples,
            self.readable_lift_samples,
            self.unreadable_lift_samples,
            self.dynamic_readable_lift_samples,
            self.pose_gliding_samples,
            self.pose_diving_samples,
            self.pose_air_brake_samples,
            self.pose_landing_anticipation_samples,
            self.pose_landing_recovery_samples,
            self.gliding_samples,
            self.launching_samples,
            self.grounded_samples,
        );
        let max_active_lift_fields_key = format!("{indent}  \"max_active_lift_fields\"");
        let wind_force_metrics = format!(
            "{indent}  \"wind_force_samples\": {},\n{indent}  \"crosswind_force_samples\": {},\n{indent}  \"updraft_swirl_force_samples\": {},\n{indent}  \"max_active_wind_force_fields\": {},\n{indent}  \"max_crosswind_force_fields\": {},\n{indent}  \"max_updraft_swirl_force_fields\": {},\n{indent}  \"max_wind_force_delta_mps\": {},\n{indent}  \"max_crosswind_force_delta_mps\": {},\n{indent}  \"max_updraft_swirl_force_delta_mps\": {},\n{indent}  \"max_wind_force_flow_speed_mps\": {},\n{indent}  \"max_wind_force_variation\": {},\n{}",
            self.wind_force_samples,
            self.crosswind_force_samples,
            self.updraft_swirl_force_samples,
            self.max_active_wind_force_fields,
            self.max_crosswind_force_fields,
            self.max_updraft_swirl_force_fields,
            json_number(self.max_wind_force_delta_mps),
            json_number(self.max_crosswind_force_delta_mps),
            json_number(self.max_updraft_swirl_force_delta_mps),
            json_number(self.max_wind_force_flow_speed_mps),
            json_number(self.max_wind_force_variation),
            max_active_lift_fields_key
        );
        let json = json.replacen(&max_active_lift_fields_key, &wind_force_metrics, 1);
        let air_brake_key = format!("{indent}  \"max_air_brake_speed_drop_mps\"");
        let rear_response_metrics = format!(
            "{indent}  \"max_backward_right_rear_response_mps\": {},\n{indent}  \"max_backward_left_rear_response_mps\": {},\n{}",
            json_number(self.max_backward_right_rear_response_mps),
            json_number(self.max_backward_left_rear_response_mps),
            air_brake_key
        );
        let json = json.replacen(&air_brake_key, &rear_response_metrics, 1);
        let post_brake_key = format!("{indent}  \"max_post_brake_forward_alignment_mps\"");
        let planar_brake_metrics = format!(
            "{indent}  \"max_air_brake_planar_speed_drop_mps\": {},\n{}",
            json_number(self.max_air_brake_planar_speed_drop_mps),
            post_brake_key
        );
        let json = json.replacen(&post_brake_key, &planar_brake_metrics, 1);
        let min_target_distance_key = format!("{indent}  \"min_target_distance_m\"");
        let pose_readability_metrics = format!(
            "{indent}  \"max_pose_torso_pitch_degrees\": {},\n{indent}  \"max_pose_arm_spread_degrees\": {},\n{indent}  \"max_pose_leg_tuck_degrees\": {},\n{indent}  \"max_pose_lateral_lean_degrees\": {},\n{indent}  \"max_pose_landing_crouch_m\": {},\n{indent}  \"max_pose_wing_airflow_strength\": {},\n{indent}  \"min_key_pose_readability_score\": {},\n{indent}  \"max_key_pose_readability_score\": {},\n{indent}  \"unreadable_key_pose_samples\": {},\n{}",
            json_number(self.max_pose_torso_pitch_degrees),
            json_number(self.max_pose_arm_spread_degrees),
            json_number(self.max_pose_leg_tuck_degrees),
            json_number(self.max_pose_lateral_lean_degrees),
            json_number(self.max_pose_landing_crouch_m),
            json_number(self.max_pose_wing_airflow_strength),
            json_number(self.min_key_pose_readability_score),
            json_number(self.max_key_pose_readability_score),
            self.unreadable_key_pose_samples,
            min_target_distance_key
        );
        let json = json.replacen(&min_target_distance_key, &pose_readability_metrics, 1);
        let terrain_surface_key = format!("{indent}  \"min_island_terrain_surface_count\"");
        let wind_visual_metrics = format!(
            "{indent}  \"max_updraft_guide_visual_count\": {},\n{indent}  \"max_updraft_ribbon_visual_count\": {},\n{indent}  \"max_crosswind_guide_visual_count\": {},\n{indent}  \"max_crosswind_ribbon_visual_count\": {},\n{indent}  \"max_updraft_visual_motion_m\": {},\n{indent}  \"max_crosswind_visual_motion_m\": {},\n{}",
            self.max_updraft_guide_visual_count,
            self.max_updraft_ribbon_visual_count,
            self.max_crosswind_guide_visual_count,
            self.max_crosswind_ribbon_visual_count,
            json_number(self.max_updraft_visual_motion_m),
            json_number(self.max_crosswind_visual_motion_m),
            terrain_surface_key
        );
        let json = json.replacen(&terrain_surface_key, &wind_visual_metrics, 1);
        let collision_metrics = format!(
            "{indent}  \"max_world_collision_proxy_count\": {},\n{indent}  \"world_collision_resolved_samples\": {},\n{indent}  \"world_collision_contact_samples\": {},\n{indent}  \"max_world_collision_push_m\": {},\n{}",
            self.max_world_collision_proxy_count,
            self.world_collision_resolved_samples,
            self.world_collision_contact_samples,
            json_number(self.max_world_collision_push_m),
            terrain_surface_key
        );
        let json = json.replacen(&terrain_surface_key, &collision_metrics, 1);
        let procedural_body_key = format!("{indent}  \"min_procedural_island_body_count\"");
        let impostor_metrics = format!(
            "{indent}  \"min_island_impostor_mesh_vertices\": {},\n{indent}  \"min_island_impostor_color_bands\": {},\n{}",
            self.min_island_impostor_mesh_vertices,
            self.min_island_impostor_color_bands,
            procedural_body_key
        );
        let json = json.replacen(&procedural_body_key, &impostor_metrics, 1);
        let body_mesh_key = format!("{indent}  \"max_island_body_mesh_vertices\"");
        let body_mesh_metrics = format!(
            "{indent}  \"min_island_body_mesh_vertices\": {},\n{}",
            self.min_island_body_mesh_vertices, body_mesh_key
        );
        let json = json.replacen(&body_mesh_key, &body_mesh_metrics, 1);
        let deferred_asset_key = format!("{indent}  \"max_queued_visual_asset_scene_count\"");
        let deferred_asset_metrics = format!(
            "{indent}  \"max_deferred_visual_asset_scene_count\": {},\n{}",
            self.max_deferred_visual_asset_scene_count, deferred_asset_key
        );
        json.replacen(&deferred_asset_key, &deferred_asset_metrics, 1)
    }
}

#[derive(Clone, Debug)]
pub struct EvalCheck {
    pub name: &'static str,
    pub passed: bool,
    pub value: f32,
    pub threshold: f32,
    pub comparator: &'static str,
    pub unit: &'static str,
}

impl EvalCheck {
    pub(super) fn at_least(
        name: &'static str,
        value: f32,
        threshold: f32,
        unit: &'static str,
    ) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            threshold,
            comparator: ">=",
            unit,
        }
    }

    pub(super) fn at_most(
        name: &'static str,
        value: f32,
        threshold: f32,
        unit: &'static str,
    ) -> Self {
        Self {
            name,
            passed: value <= threshold,
            value,
            threshold,
            comparator: "<=",
            unit,
        }
    }

    fn to_json(&self, indent: &str) -> String {
        format!(
            "{{\n{indent}  \"name\": {},\n{indent}  \"passed\": {},\n{indent}  \"value\": {},\n{indent}  \"comparator\": {},\n{indent}  \"threshold\": {},\n{indent}  \"unit\": {}\n{indent}}}",
            json_string(self.name),
            self.passed,
            json_number(self.value),
            json_string(self.comparator),
            json_number(self.threshold),
            json_string(self.unit),
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvalSummary {
    pub scenario_name: &'static str,
    pub target_island_name: Option<&'static str>,
    pub passed: bool,
    pub frame_count: u32,
    pub duration_secs: f32,
    pub thresholds: EvalThresholds,
    pub metrics: EvalMetricsSummary,
    pub checks: Vec<EvalCheck>,
    pub artifacts: EvalArtifacts,
    pub final_sample: Option<EvalSample>,
}

impl EvalSummary {
    pub fn to_json(&self) -> String {
        let checks = self
            .checks
            .iter()
            .map(|check| check.to_json("      "))
            .collect::<Vec<_>>()
            .join(",\n");
        let final_sample = self
            .final_sample
            .as_ref()
            .map(EvalSample::to_json)
            .unwrap_or_else(|| "null".to_string());
        let target_island = self
            .target_island_name
            .map(json_string)
            .unwrap_or_else(|| "null".to_string());

        format!(
            "{{\n  \"scenario\": {},\n  \"target_island\": {},\n  \"passed\": {},\n  \"frame_count\": {},\n  \"duration_secs\": {},\n  \"thresholds\": {},\n  \"metrics\": {},\n  \"checks\": [\n{}\n  ],\n  \"artifacts\": {},\n  \"final_sample\": {}\n}}\n",
            json_string(self.scenario_name),
            target_island,
            self.passed,
            self.frame_count,
            json_number(self.duration_secs),
            self.thresholds.to_json("  "),
            self.metrics.to_json("  "),
            checks,
            self.artifacts.to_json("  "),
            final_sample,
        )
    }
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}
