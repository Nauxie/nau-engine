use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.first_sample = Some(sample.clone());
    accumulator.min_altitude_m = sample.altitude_m;
    accumulator.min_camera_surface_clearance_m = sample.camera_surface_clearance_m;
    accumulator.min_target_distance_m = sample.target_distance_m;
    accumulator.min_objective_distance_m = sample.objective.current_distance_m;
    accumulator.min_camera_pitch_degrees = sample.camera_pitch_degrees;
    accumulator.max_camera_pitch_degrees = sample.camera_pitch_degrees;
    accumulator.min_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
    accumulator.max_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
    accumulator.min_visible_power_up_count = sample.visible_power_up_count;
    accumulator.min_island_terrain_surface_count = sample.island_terrain_surface_count;
    accumulator.min_island_terrain_mesh_vertices = sample.min_island_terrain_mesh_vertices;
    accumulator.min_island_terrain_color_bands = sample.min_island_terrain_color_bands;
    accumulator.min_island_terrain_material_weight_bands =
        sample.min_island_terrain_material_weight_bands;
    accumulator.min_island_terrain_material_channels = sample.min_island_terrain_material_channels;
    accumulator.min_island_terrain_material_regions = sample.min_island_terrain_material_regions;
    accumulator.min_island_terrain_texture_detail_bands =
        sample.min_island_terrain_texture_detail_bands;
    accumulator.min_island_terrain_relief_range_m = sample.min_island_terrain_relief_range_m;
    accumulator.min_island_terrain_archetype_count = sample.island_terrain_archetype_count;
    accumulator.min_island_cliff_color_bands = sample.min_island_cliff_color_bands;
    accumulator.min_island_impostor_mesh_vertices = sample.min_island_impostor_mesh_vertices;
    accumulator.min_island_impostor_color_bands = sample.min_island_impostor_color_bands;
    accumulator.min_procedural_island_body_count = sample.procedural_island_body_count;
    accumulator.min_island_body_silhouette_segments = sample.min_island_body_silhouette_segments;
    accumulator.min_island_body_mesh_vertices = sample.min_island_body_mesh_vertices;
    accumulator.min_generated_ground_cover_patch_count = sample.generated_ground_cover_patch_count;
    accumulator.min_ground_cover_blade_count = sample.min_ground_cover_blade_count;
    accumulator.min_ground_cover_mesh_vertices = sample.min_ground_cover_mesh_vertices;
    accumulator.min_generated_tree_trunk_count = sample.generated_tree_trunk_count;
    accumulator.min_generated_tree_canopy_count = sample.generated_tree_canopy_count;
    accumulator.min_tree_trunk_mesh_vertices = sample.min_tree_trunk_mesh_vertices;
    accumulator.min_tree_canopy_mesh_vertices = sample.min_tree_canopy_mesh_vertices;
    accumulator.min_detail_biome_palette_count = sample.detail_biome_palette_count;
    accumulator.min_generated_rock_count = sample.generated_rock_count;
    accumulator.min_rock_mesh_vertices = sample.min_rock_mesh_vertices;
    accumulator.min_generated_landmark_count = sample.generated_landmark_count;
    accumulator.min_generated_route_cairn_count = sample.generated_route_cairn_count;
    accumulator.min_generated_launch_beacon_count = sample.generated_launch_beacon_count;
    accumulator.min_generated_landing_garden_marker_count =
        sample.generated_landing_garden_marker_count;
    accumulator.min_generated_pond_surface_count = sample.generated_pond_surface_count;
    accumulator.min_landmark_mesh_vertices = sample.min_landmark_mesh_vertices;
    accumulator.min_generated_weather_cloud_count = sample.generated_weather_cloud_count;
    accumulator.min_generated_weather_cloud_bank_count = sample.generated_weather_cloud_bank_count;
    accumulator.min_weather_cloud_bank_depth_m = sample.min_weather_cloud_bank_depth_m;
    accumulator.min_weather_cloud_lobe_count = sample.min_weather_cloud_lobe_count;
    accumulator.min_max_weather_cloud_lobe_count = sample.max_weather_cloud_lobe_count;
    accumulator.min_weather_cloud_mesh_vertices = sample.min_weather_cloud_mesh_vertices;
    accumulator.min_weather_cloud_filament_ribbon_detail_count =
        sample.min_weather_cloud_filament_ribbon_detail_count;
}
