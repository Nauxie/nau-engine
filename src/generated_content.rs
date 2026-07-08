mod detail_meshes;
mod island_meshes;
mod materials;
mod textures;

#[cfg(test)]
pub(crate) use detail_meshes::pond_surface_mesh;
#[cfg(test)]
pub(crate) use detail_meshes::{
    CAVE_MOUTH_ARCH_STONES, CLIFF_TOOTH_COUNT, CLIFF_TOOTH_TRIANGLES_PER_TOOTH,
    CLOUD_FILAMENT_RIBBON_VERTICES, CLOUD_FILAMENT_RIBBONS_PER_LOBE, DETAIL_CARD_VERTICES,
    GARDEN_RING_BANDS, GARDEN_RING_SEGMENTS, HANGING_ROOT_SEGMENTS, HANGING_ROOT_STRANDS,
    LAKE_BASIN_RIM_BANDS, LAKE_BASIN_RIM_SEGMENTS, LAKE_SURFACE_SEGMENTS,
    LANDING_GARDEN_MARKER_SEGMENTS, LAUNCH_BEACON_CRYSTAL_COUNT, OBSTRUCTION_SPIRE_RIB_COUNT,
    OBSTRUCTION_SPIRE_RINGS, OBSTRUCTION_SPIRE_SEGMENTS, POND_SURFACE_SEGMENTS, ROCK_MESH_RINGS,
    ROCK_MESH_SEGMENTS, ROUTE_CAIRN_STONE_COUNT, RUIN_ARCH_STONE_COUNT, TREE_BRANCH_SEGMENTS,
    TREE_CANOPY_LATITUDE_SEGMENTS, TREE_CANOPY_LONGITUDE_SEGMENTS, TREE_ROOT_FLARE_SEGMENTS,
    UNDERHANG_SHELF_SEGMENTS, WATERFALL_MIST_LOBES, WATERFALL_RIBBON_COLUMNS,
    WATERFALL_RIBBON_ROWS,
};
pub(crate) use detail_meshes::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, FirstExpeditionSilhouetteKind,
    IslandUnderRouteVisualKind, IslandWaterVisualKind, TREE_BRANCH_COUNT, TREE_CANOPY_CARD_COUNT,
    TREE_ROOT_FLARE_COUNT, TREE_TRUNK_RING_COUNT, TREE_TRUNK_SEGMENTS, cliff_tooth_ridge_mesh,
    cloud_cluster_mesh, cloud_filament_ribbon_detail_count,
    crosswind_flow_ribbon_centerline_offset, crosswind_flow_ribbon_mesh,
    first_expedition_silhouette_specs, garden_ring_mesh, island_lake_basin_visual_specs,
    island_under_route_visual_specs, island_water_visual_specs, landing_garden_marker_mesh,
    launch_beacon_mesh, obstruction_spire_mesh, player_airflow_streamline_mesh, rock_scatter_mesh,
    route_cairn_mesh, ruin_arch_mesh, tree_canopy_mesh, tree_trunk_mesh, updraft_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use detail_meshes::{
    cave_mouth_arch_mesh, hanging_root_curtain_mesh, lake_basin_rim_mesh, lake_surface_mesh,
    underhang_shelf_mesh, waterfall_mist_mesh, waterfall_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use island_meshes::ISLAND_TERRAIN_EDGE_SKIRT_DEPTH_M;
pub(crate) use island_meshes::{
    GROUND_COVER_BLADES_PER_PATCH, GROUND_COVER_PATCHES, ISLAND_BODY_SEGMENTS,
    ISLAND_TERRAIN_RINGS, IslandDetailMaterials, TERRAIN_BIOME_PALETTE_COUNT,
    VERTICES_PER_GROUND_BLADE, biome_detail_color_set, biome_detail_materials,
    island_body_mesh_diagnostics, island_cliff_mesh, island_ground_cover_mesh,
    island_impostor_cliff_mesh, island_impostor_mesh, island_impostor_mesh_diagnostics,
    island_impostor_terrain_mesh, island_impostor_underside_mesh,
    island_playable_normalized_offset, island_terrain_mesh, island_terrain_mesh_diagnostics,
    island_underside_mesh, island_visual_surface_position, mesh_normal_slope_band_count,
    mesh_terrain_material_channel_count, mesh_terrain_material_region_count,
    mesh_terrain_material_weight_band_count, mesh_vertex_color_band_count,
    mesh_vertical_band_count, mesh_y_range, terrain_biome_palette,
};
#[cfg(test)]
pub(crate) use island_meshes::{
    INDICES_PER_GROUND_BLADE, ISLAND_CLIFF_RINGS, ISLAND_CLIFF_STRATA_BANDS,
    ISLAND_IMPOSTOR_CLIFF_RINGS, ISLAND_IMPOSTOR_COLOR_BANDS, ISLAND_IMPOSTOR_SEGMENTS,
    ISLAND_IMPOSTOR_TERRAIN_RINGS, ISLAND_IMPOSTOR_UNDERSIDE_RINGS, ISLAND_TERRAIN_COLOR_BANDS,
    ISLAND_TERRAIN_HEIGHT_BANDS, ISLAND_TERRAIN_MATERIAL_CHANNELS, ISLAND_TERRAIN_MATERIAL_REGIONS,
    ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS, ISLAND_TERRAIN_NORMAL_SLOPE_BANDS,
    ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS, ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE,
    ISLAND_UNDERSIDE_RINGS, island_terrain_vertex_color,
};
pub(crate) use materials::{
    cloud_surface_material, cloud_veil_material, emissive_material, glider_airflow_material,
    ground_cover_material, terrain_surface_material, textured_material, updraft_column_material,
    updraft_ribbon_material, water_surface_material,
};
pub(crate) use textures::{
    TERRAIN_TEXTURE_SIZE, mix_color, procedural_terrain_surface_texture_data, random_unit,
    texture_detail_band_count, texture_edge_promille,
};
