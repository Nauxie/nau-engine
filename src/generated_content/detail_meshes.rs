mod caves;
mod clouds;
mod effects;
mod flora;
mod formations;
mod hero_landmarks;
mod landmarks;
mod rocks;
mod ruins;
mod shared;
mod trees;
mod waterscapes;

#[cfg(test)]
pub(crate) use caves::{
    CAVE_MOUTH_ARCH_STONES, HANGING_ROOT_SEGMENTS, HANGING_ROOT_STRANDS, UNDERHANG_SHELF_SEGMENTS,
    cave_mouth_arch_mesh, hanging_root_curtain_mesh, underhang_shelf_mesh,
};
pub(crate) use caves::{IslandUnderRouteVisualKind, island_under_route_visual_specs};
pub(crate) use clouds::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, cloud_cluster_mesh,
    cloud_filament_ribbon_detail_count,
};
#[cfg(test)]
pub(crate) use clouds::{CLOUD_FILAMENT_RIBBON_VERTICES, CLOUD_FILAMENT_RIBBONS_PER_LOBE};
pub(crate) use effects::{
    crosswind_flow_ribbon_centerline_offset, crosswind_flow_ribbon_mesh,
    player_airflow_streamline_mesh, updraft_ribbon_mesh,
};
#[allow(unused_imports)]
pub(crate) use flora::{
    FloraMaterialRole, FloraVisualKind, IslandFloraVisualSpec, island_flora_visual_specs,
};
#[allow(unused_imports)]
pub(crate) use formations::{
    IslandRockFormationSpec, RockFormationKind, island_rock_formation_specs,
};
#[allow(unused_imports)]
pub(crate) use hero_landmarks::{IslandHeroLandmarkSpec, island_hero_landmark_spec};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use landmarks::{
    ARTIFACT_BANNER_STRIP_COUNT, ARTIFACT_BRIDGE_FRAGMENT_COUNT, ARTIFACT_GLYPH_STROKE_COUNT,
    ARTIFACT_PEBBLE_COUNT, ARTIFACT_REED_COUNT, ARTIFACT_RETAINING_WALL_SEGMENTS,
    ARTIFACT_STAIR_STEP_COUNT, artifact_banner_strips_mesh, artifact_bridge_fragment_mesh,
    artifact_glyph_slab_mesh, artifact_pebble_field_mesh, artifact_reed_patch_mesh,
    artifact_retaining_wall_mesh, artifact_stair_run_mesh,
};
pub(crate) use landmarks::{
    FirstExpeditionSilhouetteKind, IslandWaterFootprint, IslandWaterVisualKind,
    first_expedition_silhouette_specs, garden_ring_mesh, island_lake_basin_visual_specs,
    island_water_visual_specs, landing_garden_marker_mesh, launch_beacon_mesh, route_cairn_mesh,
    ruin_arch_mesh,
};
#[cfg(test)]
pub(crate) use landmarks::{
    GARDEN_RING_BANDS, GARDEN_RING_SEGMENTS, LAKE_BASIN_RIM_BANDS, LAKE_BASIN_RIM_SEGMENTS,
    LAKE_SURFACE_SEGMENTS, LANDING_GARDEN_MARKER_SEGMENTS, LAUNCH_BEACON_CRYSTAL_COUNT,
    POND_SURFACE_SEGMENTS, ROUTE_CAIRN_STONE_COUNT, RUIN_ARCH_STONE_COUNT, WATERFALL_MIST_LOBES,
    WATERFALL_RIBBON_COLUMNS, WATERFALL_RIBBON_ROWS,
};
#[allow(unused_imports)]
pub(crate) use landmarks::{
    IslandArtifactMaterial, IslandArtifactVisualKind, IslandArtifactVisualSpec,
    island_artifact_visual_specs,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use landmarks::{
    RIVER_CHANNEL_COLUMNS, RIVER_CHANNEL_SEGMENTS, river_channel_surface_mesh,
};
#[cfg(test)]
pub(crate) use landmarks::{
    lake_basin_rim_mesh, lake_surface_mesh, pond_surface_mesh, waterfall_mist_mesh,
    waterfall_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use rocks::{
    CLIFF_TOOTH_COUNT, CLIFF_TOOTH_TRIANGLES_PER_TOOTH, OBSTRUCTION_SPIRE_RIB_COUNT,
    OBSTRUCTION_SPIRE_RINGS, OBSTRUCTION_SPIRE_SEGMENTS, ROCK_MESH_RINGS, ROCK_MESH_SEGMENTS,
};
pub(crate) use rocks::{cliff_tooth_ridge_mesh, obstruction_spire_mesh, rock_scatter_mesh};
#[allow(unused_imports)]
pub(crate) use ruins::{IslandRuinComplexSpec, RuinComplexKind, island_ruin_complex_specs};
#[cfg(test)]
pub(crate) use shared::DETAIL_CARD_VERTICES;
pub(crate) use trees::{
    TREE_BRANCH_COUNT, TREE_CANOPY_CARD_COUNT, TREE_ROOT_FLARE_COUNT, TREE_TRUNK_RING_COUNT,
    TREE_TRUNK_SEGMENTS, tree_canopy_mesh, tree_canopy_mesh_for_species, tree_trunk_mesh,
    tree_trunk_mesh_for_species,
};
#[cfg(test)]
pub(crate) use trees::{
    TREE_BRANCH_SEGMENTS, TREE_CANOPY_LATITUDE_SEGMENTS, TREE_CANOPY_LONGITUDE_SEGMENTS,
    TREE_ROOT_FLARE_SEGMENTS,
};
#[allow(unused_imports)]
pub(crate) use waterscapes::{
    IslandWaterDetailSpec, WaterDetailKind, WaterDetailMaterialRole, island_water_detail_specs,
};
