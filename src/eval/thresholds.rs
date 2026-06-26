use super::json_number;
use crate::world::IslandTerrainArchetype;

pub const MAX_RESIDENT_ISLAND_VISUAL_FRACTION: f32 = 0.70;
pub(super) const MIN_GENERATED_GROUND_COVER_PATCH_COUNT: usize = 500;
pub(super) const MIN_GROUND_COVER_BLADE_COUNT: usize = 200;
pub(super) const MIN_GROUND_COVER_MESH_VERTICES: usize = 1000;
pub(super) const MIN_GENERATED_TREE_TRUNK_COUNT: usize = 30;
pub(super) const MIN_GENERATED_TREE_CANOPY_COUNT: usize = 30;
pub(super) const MIN_TREE_TRUNK_MESH_VERTICES: usize = 190;
pub(super) const MIN_TREE_CANOPY_MESH_VERTICES: usize = 400;
pub(super) const MIN_DETAIL_BIOME_PALETTE_COUNT: usize = 5;
pub(super) const MIN_GENERATED_ROCK_COUNT: usize = 55;
pub(super) const MIN_ROCK_MESH_VERTICES: usize = 70;
pub(super) const MIN_GENERATED_LANDMARK_COUNT: usize = 27;
pub(super) const MIN_GENERATED_ROUTE_CAIRN_COUNT: usize = 10;
pub(super) const MIN_GENERATED_LAUNCH_BEACON_COUNT: usize = 1;
pub(super) const MIN_GENERATED_LANDING_GARDEN_MARKER_COUNT: usize = 4;
pub(super) const MIN_GENERATED_POND_SURFACE_COUNT: usize = 12;
pub(super) const MIN_LANDMARK_MESH_VERTICES: usize = 39;
pub(super) const MIN_GENERATED_WEATHER_CLOUD_COUNT: usize = 24;
pub(super) const MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT: usize = 12;
pub(super) const MIN_WEATHER_CLOUD_BANK_DEPTH_M: f32 = 5.8;
pub(super) const MIN_WEATHER_CLOUD_LOBE_COUNT: usize = 9;
pub(super) const MIN_MAX_WEATHER_CLOUD_LOBE_COUNT: usize = 18;
pub(super) const MIN_WEATHER_CLOUD_MESH_VERTICES: usize = 1458;
pub(super) const MIN_WEATHER_CLOUD_FILAMENT_RIBBON_DETAIL_COUNT: usize = 27;
pub(super) const MIN_ISLAND_TERRAIN_SURFACE_COUNT: usize = 15;
pub(super) const MIN_ISLAND_TERRAIN_MESH_VERTICES: usize = 2305;
pub(super) const MIN_ISLAND_TERRAIN_COLOR_BANDS: usize = 32;
pub(super) const MIN_ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS: usize = 24;
pub(super) const MIN_ISLAND_TERRAIN_MATERIAL_CHANNELS: usize = 3;
pub(super) const MIN_ISLAND_TERRAIN_MATERIAL_REGIONS: usize = 4;
pub(super) const MIN_ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS: usize = 44;
pub(super) const MIN_ISLAND_TERRAIN_RELIEF_RANGE_M: f32 = 0.8;
pub(super) const MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT: usize = IslandTerrainArchetype::COUNT;
pub(super) const MIN_ISLAND_CLIFF_COLOR_BANDS: usize = 9;
pub(super) const MIN_ISLAND_IMPOSTOR_MESH_VERTICES: usize = 140;
pub(super) const MIN_ISLAND_IMPOSTOR_COLOR_BANDS: usize = 18;
pub(super) const MIN_ISLAND_BODY_MESH_VERTICES: usize = 1600;
pub(super) const MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT: usize = 7;
pub(super) const AIR_CONTROL_RESPONSE_THRESHOLD_MPS: f32 = 4.0;
pub(super) const AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS: f32 = 0.20;
pub(super) const AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS: f32 = 18.0;
pub(super) const AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS: f32 = 10.0;
pub(super) const AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS: f32 = 10.0;
pub(super) const AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS: f32 = 20.0;
pub(super) const AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES: f32 = 8.0;
pub(super) const AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES: f32 = 22.0;
pub(super) const AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES: f32 = 36.0;
pub(super) const AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES: f32 = 45.0;
pub(super) const AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES: f32 = 60.0;
pub(super) const AIR_CONTROL_MAX_P95_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES: f32 =
    35.0;
pub(super) const AIR_CONTROL_MAX_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES: f32 = 50.0;
pub(super) const AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES: u32 = 1;
pub(super) const AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES: u32 = 1;
pub(super) const AIR_CONTROL_MIN_DESIRED_TRAVEL_HEADING_SAMPLES: u32 = 8;
pub(super) const AIR_CONTROL_MIN_DIRECTIONAL_DESIRED_TRAVEL_HEADING_SAMPLES: u32 = 1;
pub(super) const AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES: f32 = 45.0;
pub(super) const AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES: f32 = 65.0;
pub(super) const AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES: f32 = 36.0;
pub(super) const AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS: f32 = 4.0;
pub(super) const AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES: f32 = 8.0;
pub(super) const AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES: f32 = 12.0;
pub(super) const AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES: f32 = 0.01;
pub(super) const AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES: f32 = 2.0;
pub(super) const AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
pub(super) const AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES: f32 = 55.0;
pub(super) const AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES: f32 = 70.0;
pub(super) const CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS: f32 = 8.0;
pub(super) const CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES: f32 = 2.0;
pub(super) const MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES: f32 = 2.0;
pub(super) const MAX_GROUNDED_VISUAL_FOOT_GAP_M: f32 = 0.05;
pub(super) const AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS: f32 = 12.0;
pub(super) const AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS: f32 = 12.0;
pub(super) const AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS: f32 = 14.0;
pub(super) const AIR_CONTROL_MIN_POSE_AIR_TURN_SAMPLES: u32 = 4;
pub(super) const AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_TURN_SAMPLES: u32 = 1;
pub(super) const AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES: f32 = 45.0;
pub(super) const AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES: f32 = 100.0;
pub(super) const AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES: f32 = 35.0;
pub(super) const AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES: f32 = 8.0;
pub(super) const AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES: f32 = 8.0;
pub(super) const AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH: f32 = 0.25;
pub(super) const MIN_POSE_TEMPORAL_STABILITY_SAMPLES: u32 = 1;
pub(super) const MAX_POSE_PART_ROTATION_DELTA_DEGREES: f32 = 120.0;
pub(super) const MAX_POSE_PART_TRANSLATION_DELTA_M: f32 = 0.55;
pub(super) const LANDING_MIN_POSE_CROUCH_M: f32 = 0.05;
pub const LANDING_MIN_POSE_FLARE_DEGREES: f32 = 32.0;
pub(super) const AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES: f32 = 8.0;
pub const MIN_DYNAMIC_WIND_FLOW_SPEED_MPS: f32 = 8.0;
pub const MIN_DYNAMIC_WIND_FLOW_VARIATION: f32 = 0.12;
pub const MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE: f32 = 0.03;
pub const MIN_WIND_FORCE_DELTA_MPS: f32 = 0.04;
pub const MIN_CROSSWIND_FORCE_DELTA_MPS: f32 = 0.04;
pub const MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS: f32 = 0.03;
pub const MIN_WIND_FORCE_FLOW_SPEED_MPS: f32 = 6.0;
pub const MIN_WIND_FORCE_VARIATION: f32 = 0.12;
pub const MIN_WIND_FORCE_SAMPLE_COUNT: u32 = 2;
pub const MIN_CROSSWIND_FORCE_SAMPLE_COUNT: u32 = 2;
pub(super) const MIN_UPDRAFT_GUIDE_VISUAL_COUNT: usize = 70;
pub(super) const MIN_UPDRAFT_RIBBON_VISUAL_COUNT: usize = 6;
pub(super) const MIN_CROSSWIND_GUIDE_VISUAL_COUNT: usize = 72;
pub(super) const MIN_CROSSWIND_RIBBON_VISUAL_COUNT: usize = 8;
pub(super) const MIN_UPDRAFT_VISUAL_MOTION_M: f32 = 0.2;
pub(super) const MIN_UPDRAFT_VISUAL_RISE_M: f32 = 0.2;
pub(super) const MIN_CROSSWIND_VISUAL_MOTION_M: f32 = 0.3;
pub(super) const MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M: f32 = 0.3;
pub(super) const MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M: f32 = 0.3;
pub(super) const MIN_WORLD_COLLISION_PROXY_COUNT: usize = 24;
pub(super) const MIN_WORLD_COLLISION_CONTACT_SAMPLES: u32 = 10;
pub(super) const MIN_WORLD_COLLISION_CONTACT_SAMPLE_PUSH_M: f32 = 0.005;
pub(super) const MIN_WORLD_COLLISION_CONTACT_PUSH_M: f32 = 0.04;

#[derive(Clone, Copy, Debug)]
pub struct EvalThresholds {
    pub min_samples: u32,
    pub min_horizontal_distance_m: f32,
    pub min_max_altitude_m: f32,
    pub min_max_speed_mps: f32,
    pub min_gliding_samples: u32,
    pub min_grounded_samples: u32,
    pub min_lifted_samples: u32,
    pub min_sky_island_count: usize,
    pub min_active_island_count: usize,
    pub max_active_chunk_count: usize,
    pub min_near_lod_island_count: usize,
    pub min_mid_lod_island_count: usize,
    pub min_far_lod_island_count: usize,
    pub max_visible_island_terrain_count: usize,
    pub min_hidden_island_terrain_count: usize,
    pub min_visible_island_impostor_count: usize,
    pub max_visible_island_detail_count: usize,
    pub min_hidden_island_detail_count: usize,
    pub min_visible_route_beacon_count: usize,
    pub min_weather_cloud_count: usize,
    pub min_environment_motion_visual_count: usize,
    pub min_environment_motion_offset_m: f32,
    pub min_island_terrain_surface_count: usize,
    pub min_island_terrain_mesh_vertices: usize,
    pub min_island_terrain_color_bands: usize,
    pub min_island_terrain_relief_range_m: f32,
    pub min_island_terrain_archetype_count: usize,
    pub min_island_cliff_color_bands: usize,
    pub min_procedural_island_body_count: usize,
    pub max_primitive_island_body_count: usize,
    pub min_island_body_silhouette_segments: usize,
    pub max_resident_island_visual_count: usize,
    pub max_stream_visibility_changes_per_frame: usize,
    pub min_entity_count: usize,
    pub max_camera_distance_m: f32,
    pub min_camera_surface_clearance_m: f32,
    pub max_camera_player_angle_degrees: f32,
    pub max_camera_step_distance_m: f32,
    pub max_camera_rotation_delta_degrees: f32,
    pub max_camera_orbit_alignment_degrees: f32,
    pub max_abs_camera_view_yaw_degrees: f32,
    pub min_camera_obstruction_adjustment_m: f32,
    pub min_abs_camera_yaw_degrees: f32,
    pub min_camera_pitch_offset_degrees: f32,
    pub max_camera_pitch_offset_degrees: f32,
    pub min_objective_total_count: usize,
    pub min_completed_objective_count: usize,
    pub min_visual_asset_slot_count: usize,
    pub min_gltf_scene_asset_slot_count: usize,
    pub min_streaming_visual_asset_slot_count: usize,
    pub min_declared_animation_clip_count: usize,
    pub max_failed_visual_asset_scene_count: usize,
    pub min_power_up_count: usize,
    pub min_collected_power_up_count: usize,
    pub min_power_up_effect_samples: u32,
    pub require_target_landing: bool,
    pub max_final_target_distance_m: f32,
    pub min_target_landing_samples: u32,
}

impl EvalThresholds {
    pub(super) fn to_json(self, indent: &str) -> String {
        format!(
            "{{\n{indent}  \"min_samples\": {},\n{indent}  \"min_horizontal_distance_m\": {},\n{indent}  \"min_max_altitude_m\": {},\n{indent}  \"min_max_speed_mps\": {},\n{indent}  \"min_gliding_samples\": {},\n{indent}  \"min_grounded_samples\": {},\n{indent}  \"min_lifted_samples\": {},\n{indent}  \"min_sky_island_count\": {},\n{indent}  \"min_active_island_count\": {},\n{indent}  \"max_active_chunk_count\": {},\n{indent}  \"min_near_lod_island_count\": {},\n{indent}  \"min_mid_lod_island_count\": {},\n{indent}  \"min_far_lod_island_count\": {},\n{indent}  \"max_visible_island_terrain_count\": {},\n{indent}  \"min_hidden_island_terrain_count\": {},\n{indent}  \"min_visible_island_impostor_count\": {},\n{indent}  \"max_visible_island_detail_count\": {},\n{indent}  \"min_hidden_island_detail_count\": {},\n{indent}  \"min_visible_route_beacon_count\": {},\n{indent}  \"min_weather_cloud_count\": {},\n{indent}  \"min_environment_motion_visual_count\": {},\n{indent}  \"min_environment_motion_offset_m\": {},\n{indent}  \"min_island_terrain_surface_count\": {},\n{indent}  \"min_island_terrain_mesh_vertices\": {},\n{indent}  \"min_island_terrain_color_bands\": {},\n{indent}  \"min_island_terrain_relief_range_m\": {},\n{indent}  \"min_island_terrain_archetype_count\": {},\n{indent}  \"min_island_cliff_color_bands\": {},\n{indent}  \"min_procedural_island_body_count\": {},\n{indent}  \"max_primitive_island_body_count\": {},\n{indent}  \"min_island_body_silhouette_segments\": {},\n{indent}  \"max_resident_island_visual_count\": {},\n{indent}  \"max_stream_visibility_changes_per_frame\": {},\n{indent}  \"min_entity_count\": {},\n{indent}  \"max_camera_distance_m\": {},\n{indent}  \"min_camera_surface_clearance_m\": {},\n{indent}  \"max_camera_player_angle_degrees\": {},\n{indent}  \"max_camera_step_distance_m\": {},\n{indent}  \"max_camera_rotation_delta_degrees\": {},\n{indent}  \"max_camera_orbit_alignment_degrees\": {},\n{indent}  \"max_abs_camera_view_yaw_degrees\": {},\n{indent}  \"min_camera_obstruction_adjustment_m\": {},\n{indent}  \"min_abs_camera_yaw_degrees\": {},\n{indent}  \"min_camera_pitch_offset_degrees\": {},\n{indent}  \"max_camera_pitch_offset_degrees\": {},\n{indent}  \"min_objective_total_count\": {},\n{indent}  \"min_completed_objective_count\": {},\n{indent}  \"min_visual_asset_slot_count\": {},\n{indent}  \"min_gltf_scene_asset_slot_count\": {},\n{indent}  \"min_streaming_visual_asset_slot_count\": {},\n{indent}  \"min_declared_animation_clip_count\": {},\n{indent}  \"max_failed_visual_asset_scene_count\": {},\n{indent}  \"min_power_up_count\": {},\n{indent}  \"min_collected_power_up_count\": {},\n{indent}  \"min_power_up_effect_samples\": {},\n{indent}  \"require_target_landing\": {},\n{indent}  \"max_final_target_distance_m\": {},\n{indent}  \"min_target_landing_samples\": {}\n{indent}}}",
            self.min_samples,
            json_number(self.min_horizontal_distance_m),
            json_number(self.min_max_altitude_m),
            json_number(self.min_max_speed_mps),
            self.min_gliding_samples,
            self.min_grounded_samples,
            self.min_lifted_samples,
            self.min_sky_island_count,
            self.min_active_island_count,
            self.max_active_chunk_count,
            self.min_near_lod_island_count,
            self.min_mid_lod_island_count,
            self.min_far_lod_island_count,
            self.max_visible_island_terrain_count,
            self.min_hidden_island_terrain_count,
            self.min_visible_island_impostor_count,
            self.max_visible_island_detail_count,
            self.min_hidden_island_detail_count,
            self.min_visible_route_beacon_count,
            self.min_weather_cloud_count,
            self.min_environment_motion_visual_count,
            json_number(self.min_environment_motion_offset_m),
            self.min_island_terrain_surface_count,
            self.min_island_terrain_mesh_vertices,
            self.min_island_terrain_color_bands,
            json_number(self.min_island_terrain_relief_range_m),
            self.min_island_terrain_archetype_count,
            self.min_island_cliff_color_bands,
            self.min_procedural_island_body_count,
            self.max_primitive_island_body_count,
            self.min_island_body_silhouette_segments,
            self.max_resident_island_visual_count,
            self.max_stream_visibility_changes_per_frame,
            self.min_entity_count,
            json_number(self.max_camera_distance_m),
            json_number(self.min_camera_surface_clearance_m),
            json_number(self.max_camera_player_angle_degrees),
            json_number(self.max_camera_step_distance_m),
            json_number(self.max_camera_rotation_delta_degrees),
            json_number(self.max_camera_orbit_alignment_degrees),
            json_number(self.max_abs_camera_view_yaw_degrees),
            json_number(self.min_camera_obstruction_adjustment_m),
            json_number(self.min_abs_camera_yaw_degrees),
            json_number(self.min_camera_pitch_offset_degrees),
            json_number(self.max_camera_pitch_offset_degrees),
            self.min_objective_total_count,
            self.min_completed_objective_count,
            self.min_visual_asset_slot_count,
            self.min_gltf_scene_asset_slot_count,
            self.min_streaming_visual_asset_slot_count,
            self.min_declared_animation_clip_count,
            self.max_failed_visual_asset_scene_count,
            self.min_power_up_count,
            self.min_collected_power_up_count,
            self.min_power_up_effect_samples,
            self.require_target_landing,
            json_number(self.max_final_target_distance_m),
            self.min_target_landing_samples,
        )
    }
}
