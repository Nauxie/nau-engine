mod clouds;
mod effects;
mod landmarks;
mod rocks;
mod shared;
mod trees;

pub(crate) use clouds::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, cloud_cluster_mesh,
    cloud_filament_ribbon_detail_count,
};
#[cfg(test)]
pub(crate) use clouds::{CLOUD_FILAMENT_RIBBON_VERTICES, CLOUD_FILAMENT_RIBBONS_PER_LOBE};
pub(crate) use effects::{glider_airflow_trail_mesh, updraft_ribbon_mesh};
#[cfg(test)]
pub(crate) use landmarks::{
    LANDING_GARDEN_MARKER_SEGMENTS, LAUNCH_BEACON_CRYSTAL_COUNT, POND_SURFACE_SEGMENTS,
    ROUTE_CAIRN_STONE_COUNT,
};
pub(crate) use landmarks::{
    landing_garden_marker_mesh, launch_beacon_mesh, pond_surface_mesh, route_cairn_mesh,
};
pub(crate) use rocks::rock_scatter_mesh;
#[cfg(test)]
pub(crate) use rocks::{ROCK_MESH_RINGS, ROCK_MESH_SEGMENTS};
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
