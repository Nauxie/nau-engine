use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.min_island_terrain_surface_count = accumulator
        .min_island_terrain_surface_count
        .min(sample.island_terrain_surface_count);
    accumulator.min_island_terrain_mesh_vertices = accumulator
        .min_island_terrain_mesh_vertices
        .min(sample.min_island_terrain_mesh_vertices);
    accumulator.min_island_terrain_color_bands = accumulator
        .min_island_terrain_color_bands
        .min(sample.min_island_terrain_color_bands);
    accumulator.min_island_terrain_material_weight_bands = accumulator
        .min_island_terrain_material_weight_bands
        .min(sample.min_island_terrain_material_weight_bands);
    accumulator.min_island_terrain_material_channels = accumulator
        .min_island_terrain_material_channels
        .min(sample.min_island_terrain_material_channels);
    accumulator.min_island_terrain_material_regions = accumulator
        .min_island_terrain_material_regions
        .min(sample.min_island_terrain_material_regions);
    accumulator.min_island_terrain_texture_detail_bands = accumulator
        .min_island_terrain_texture_detail_bands
        .min(sample.min_island_terrain_texture_detail_bands);
    accumulator.min_island_terrain_relief_range_m = accumulator
        .min_island_terrain_relief_range_m
        .min(sample.min_island_terrain_relief_range_m);
    accumulator.min_island_terrain_archetype_count = accumulator
        .min_island_terrain_archetype_count
        .min(sample.island_terrain_archetype_count);
    accumulator.min_island_cliff_color_bands = accumulator
        .min_island_cliff_color_bands
        .min(sample.min_island_cliff_color_bands);
    accumulator.min_island_impostor_mesh_vertices = accumulator
        .min_island_impostor_mesh_vertices
        .min(sample.min_island_impostor_mesh_vertices);
    accumulator.min_island_impostor_color_bands = accumulator
        .min_island_impostor_color_bands
        .min(sample.min_island_impostor_color_bands);
    accumulator.min_procedural_island_body_count = accumulator
        .min_procedural_island_body_count
        .min(sample.procedural_island_body_count);
    accumulator.max_primitive_island_body_count = accumulator
        .max_primitive_island_body_count
        .max(sample.primitive_island_body_count);
    accumulator.min_island_body_silhouette_segments = accumulator
        .min_island_body_silhouette_segments
        .min(sample.min_island_body_silhouette_segments);
    accumulator.max_avg_island_body_silhouette_segments = accumulator
        .max_avg_island_body_silhouette_segments
        .max(sample.avg_island_body_silhouette_segments);
    accumulator.min_island_body_mesh_vertices = accumulator
        .min_island_body_mesh_vertices
        .min(sample.min_island_body_mesh_vertices);
    accumulator.max_island_body_mesh_vertices = accumulator
        .max_island_body_mesh_vertices
        .max(sample.max_island_body_mesh_vertices);
    accumulator.min_generated_ground_cover_patch_count = accumulator
        .min_generated_ground_cover_patch_count
        .min(sample.generated_ground_cover_patch_count);
    accumulator.min_ground_cover_blade_count = accumulator
        .min_ground_cover_blade_count
        .min(sample.min_ground_cover_blade_count);
    accumulator.min_ground_cover_mesh_vertices = accumulator
        .min_ground_cover_mesh_vertices
        .min(sample.min_ground_cover_mesh_vertices);
    accumulator.min_generated_tree_trunk_count = accumulator
        .min_generated_tree_trunk_count
        .min(sample.generated_tree_trunk_count);
    accumulator.min_generated_tree_canopy_count = accumulator
        .min_generated_tree_canopy_count
        .min(sample.generated_tree_canopy_count);
    accumulator.min_tree_trunk_mesh_vertices = accumulator
        .min_tree_trunk_mesh_vertices
        .min(sample.min_tree_trunk_mesh_vertices);
    accumulator.min_tree_canopy_mesh_vertices = accumulator
        .min_tree_canopy_mesh_vertices
        .min(sample.min_tree_canopy_mesh_vertices);
    accumulator.min_detail_biome_palette_count = accumulator
        .min_detail_biome_palette_count
        .min(sample.detail_biome_palette_count);
    accumulator.min_generated_rock_count = accumulator
        .min_generated_rock_count
        .min(sample.generated_rock_count);
    accumulator.min_rock_mesh_vertices = accumulator
        .min_rock_mesh_vertices
        .min(sample.min_rock_mesh_vertices);
    accumulator.min_generated_landmark_count = accumulator
        .min_generated_landmark_count
        .min(sample.generated_landmark_count);
    accumulator.min_generated_route_cairn_count = accumulator
        .min_generated_route_cairn_count
        .min(sample.generated_route_cairn_count);
    accumulator.min_generated_launch_beacon_count = accumulator
        .min_generated_launch_beacon_count
        .min(sample.generated_launch_beacon_count);
    accumulator.min_generated_landing_garden_marker_count = accumulator
        .min_generated_landing_garden_marker_count
        .min(sample.generated_landing_garden_marker_count);
    accumulator.min_generated_pond_surface_count = accumulator
        .min_generated_pond_surface_count
        .min(sample.generated_pond_surface_count);
    accumulator.min_landmark_mesh_vertices = accumulator
        .min_landmark_mesh_vertices
        .min(sample.min_landmark_mesh_vertices);
    accumulator.min_generated_weather_cloud_count = accumulator
        .min_generated_weather_cloud_count
        .min(sample.generated_weather_cloud_count);
    accumulator.min_generated_weather_cloud_bank_count = accumulator
        .min_generated_weather_cloud_bank_count
        .min(sample.generated_weather_cloud_bank_count);
    accumulator.min_weather_cloud_bank_depth_m = accumulator
        .min_weather_cloud_bank_depth_m
        .min(sample.min_weather_cloud_bank_depth_m);
    accumulator.min_weather_cloud_lobe_count = accumulator
        .min_weather_cloud_lobe_count
        .min(sample.min_weather_cloud_lobe_count);
    accumulator.min_max_weather_cloud_lobe_count = accumulator
        .min_max_weather_cloud_lobe_count
        .min(sample.max_weather_cloud_lobe_count);
    accumulator.min_weather_cloud_mesh_vertices = accumulator
        .min_weather_cloud_mesh_vertices
        .min(sample.min_weather_cloud_mesh_vertices);
    accumulator.min_weather_cloud_filament_ribbon_detail_count = accumulator
        .min_weather_cloud_filament_ribbon_detail_count
        .min(sample.min_weather_cloud_filament_ribbon_detail_count);
}
