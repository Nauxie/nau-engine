mod caves;
mod clouds;
mod effects;
mod landmarks;
mod rocks;
mod shared;
mod trees;

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
    crosswind_flow_ribbon_centerline_offset, crosswind_flow_ribbon_mesh, glider_airflow_trail_mesh,
    updraft_ribbon_mesh,
};
pub(crate) use landmarks::{
    IslandWaterVisualKind, island_water_visual_specs, landing_garden_marker_mesh,
    launch_beacon_mesh, route_cairn_mesh, ruin_arch_mesh,
};
#[cfg(test)]
pub(crate) use landmarks::{
    LAKE_SURFACE_SEGMENTS, LANDING_GARDEN_MARKER_SEGMENTS, LAUNCH_BEACON_CRYSTAL_COUNT,
    POND_SURFACE_SEGMENTS, ROUTE_CAIRN_STONE_COUNT, RUIN_ARCH_STONE_COUNT, WATERFALL_MIST_LOBES,
    WATERFALL_RIBBON_COLUMNS, WATERFALL_RIBBON_ROWS,
};
#[cfg(test)]
pub(crate) use landmarks::{
    lake_surface_mesh, pond_surface_mesh, waterfall_mist_mesh, waterfall_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use rocks::{
    CLIFF_TOOTH_COUNT, CLIFF_TOOTH_TRIANGLES_PER_TOOTH, OBSTRUCTION_SPIRE_RIB_COUNT,
    OBSTRUCTION_SPIRE_RINGS, OBSTRUCTION_SPIRE_SEGMENTS, ROCK_MESH_RINGS, ROCK_MESH_SEGMENTS,
};
pub(crate) use rocks::{cliff_tooth_ridge_mesh, obstruction_spire_mesh, rock_scatter_mesh};
#[cfg(test)]
pub(crate) use shared::DETAIL_CARD_VERTICES;
pub(crate) use trees::{
    TREE_BRANCH_COUNT, TREE_CANOPY_CARD_COUNT, TREE_ROOT_FLARE_COUNT, TREE_TRUNK_RING_COUNT,
    TREE_TRUNK_SEGMENTS, tree_canopy_mesh, tree_trunk_mesh,
};
#[cfg(test)]
pub(crate) use trees::{
    TREE_BRANCH_SEGMENTS, TREE_CANOPY_LATITUDE_SEGMENTS, TREE_CANOPY_LONGITUDE_SEGMENTS,
    TREE_ROOT_FLARE_SEGMENTS,
};
