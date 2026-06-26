use super::super::super::EvalAccumulator;
use crate::eval::{
    summary::EvalCheck,
    thresholds::{EvalThresholds, *},
};

pub(super) fn append_content_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    thresholds: &EvalThresholds,
) {
    checks.extend([
        EvalCheck::at_least(
            "island_terrain_surface_count",
            acc.min_island_terrain_surface_count as f32,
            thresholds.min_island_terrain_surface_count as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "island_terrain_mesh_vertices",
            acc.min_island_terrain_mesh_vertices as f32,
            thresholds.min_island_terrain_mesh_vertices as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "island_terrain_color_bands",
            acc.min_island_terrain_color_bands as f32,
            thresholds.min_island_terrain_color_bands as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_material_weight_bands",
            acc.min_island_terrain_material_weight_bands as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_material_channels",
            acc.min_island_terrain_material_channels as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_CHANNELS as f32,
            "channels",
        ),
        EvalCheck::at_least(
            "island_terrain_material_regions",
            acc.min_island_terrain_material_regions as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_REGIONS as f32,
            "regions",
        ),
        EvalCheck::at_least(
            "island_terrain_texture_detail_bands",
            acc.min_island_terrain_texture_detail_bands as f32,
            MIN_ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_relief_range",
            acc.min_island_terrain_relief_range_m,
            thresholds.min_island_terrain_relief_range_m,
            "m",
        ),
        EvalCheck::at_least(
            "island_cliff_color_bands",
            acc.min_island_cliff_color_bands as f32,
            thresholds.min_island_cliff_color_bands as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_impostor_mesh_vertices",
            acc.min_island_impostor_mesh_vertices as f32,
            MIN_ISLAND_IMPOSTOR_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "island_impostor_color_bands",
            acc.min_island_impostor_color_bands as f32,
            MIN_ISLAND_IMPOSTOR_COLOR_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "procedural_island_body_count",
            acc.min_procedural_island_body_count as f32,
            thresholds.min_procedural_island_body_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "primitive_island_body_count",
            acc.max_primitive_island_body_count as f32,
            thresholds.max_primitive_island_body_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "island_body_silhouette_segments",
            acc.min_island_body_silhouette_segments as f32,
            thresholds.min_island_body_silhouette_segments as f32,
            "segments",
        ),
        EvalCheck::at_least(
            "island_body_mesh_vertices",
            acc.min_island_body_mesh_vertices as f32,
            MIN_ISLAND_BODY_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_ground_cover_patch_count",
            acc.min_generated_ground_cover_patch_count as f32,
            MIN_GENERATED_GROUND_COVER_PATCH_COUNT as f32,
            "patches",
        ),
        EvalCheck::at_least(
            "ground_cover_blade_count",
            acc.min_ground_cover_blade_count as f32,
            MIN_GROUND_COVER_BLADE_COUNT as f32,
            "blades",
        ),
        EvalCheck::at_least(
            "ground_cover_mesh_vertices",
            acc.min_ground_cover_mesh_vertices as f32,
            MIN_GROUND_COVER_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_tree_trunk_count",
            acc.min_generated_tree_trunk_count as f32,
            MIN_GENERATED_TREE_TRUNK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "generated_tree_canopy_count",
            acc.min_generated_tree_canopy_count as f32,
            MIN_GENERATED_TREE_CANOPY_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "tree_trunk_mesh_vertices",
            acc.min_tree_trunk_mesh_vertices as f32,
            MIN_TREE_TRUNK_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "tree_canopy_mesh_vertices",
            acc.min_tree_canopy_mesh_vertices as f32,
            MIN_TREE_CANOPY_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "detail_biome_palette_count",
            acc.min_detail_biome_palette_count as f32,
            MIN_DETAIL_BIOME_PALETTE_COUNT as f32,
            "palettes",
        ),
        EvalCheck::at_least(
            "generated_rock_count",
            acc.min_generated_rock_count as f32,
            MIN_GENERATED_ROCK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "rock_mesh_vertices",
            acc.min_rock_mesh_vertices as f32,
            MIN_ROCK_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_weather_cloud_count",
            acc.min_generated_weather_cloud_count as f32,
            MIN_GENERATED_WEATHER_CLOUD_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "generated_weather_cloud_bank_count",
            acc.min_generated_weather_cloud_bank_count as f32,
            MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "weather_cloud_bank_depth",
            acc.min_weather_cloud_bank_depth_m,
            MIN_WEATHER_CLOUD_BANK_DEPTH_M,
            "m",
        ),
        EvalCheck::at_least(
            "weather_cloud_lobe_count",
            acc.min_weather_cloud_lobe_count as f32,
            MIN_WEATHER_CLOUD_LOBE_COUNT as f32,
            "lobes",
        ),
        EvalCheck::at_least(
            "weather_cloud_bank_lobe_count",
            acc.min_max_weather_cloud_lobe_count as f32,
            MIN_MAX_WEATHER_CLOUD_LOBE_COUNT as f32,
            "lobes",
        ),
        EvalCheck::at_least(
            "weather_cloud_mesh_vertices",
            acc.min_weather_cloud_mesh_vertices as f32,
            MIN_WEATHER_CLOUD_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "weather_cloud_filament_ribbon_detail_count",
            acc.min_weather_cloud_filament_ribbon_detail_count as f32,
            MIN_WEATHER_CLOUD_FILAMENT_RIBBON_DETAIL_COUNT as f32,
            "ribbons",
        ),
    ]);
}
