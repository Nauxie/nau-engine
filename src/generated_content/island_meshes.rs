mod body;
mod constants;
mod ground_cover;
mod metrics;
mod normals;
mod palette;
mod shape;
mod terrain;

pub(crate) use body::{island_cliff_mesh, island_impostor_mesh, island_underside_mesh};
pub(crate) use constants::{
    GROUND_COVER_BLADES_PER_PATCH, GROUND_COVER_PATCHES, ISLAND_BODY_SEGMENTS,
    TERRAIN_BIOME_PALETTE_COUNT, VERTICES_PER_GROUND_BLADE,
};
#[cfg(test)]
pub(crate) use constants::{
    INDICES_PER_GROUND_BLADE, ISLAND_CLIFF_RINGS, ISLAND_CLIFF_STRATA_BANDS,
    ISLAND_IMPOSTOR_COLOR_BANDS, ISLAND_IMPOSTOR_SEGMENTS, ISLAND_TERRAIN_COLOR_BANDS,
    ISLAND_TERRAIN_MATERIAL_CHANNELS, ISLAND_TERRAIN_MATERIAL_REGIONS,
    ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS, ISLAND_TERRAIN_RINGS,
    ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS, ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE,
    ISLAND_UNDERSIDE_RINGS,
};
pub(crate) use ground_cover::island_ground_cover_mesh;
pub(crate) use metrics::{
    mesh_terrain_material_channel_count, mesh_terrain_material_region_count,
    mesh_terrain_material_weight_band_count, mesh_vertex_color_band_count, mesh_y_range,
};
#[cfg(test)]
pub(crate) use palette::island_terrain_vertex_color;
pub(crate) use palette::{
    IslandDetailMaterials, biome_detail_color_set, biome_detail_materials, terrain_biome_palette,
};
pub(crate) use shape::island_visual_surface_position;
pub(crate) use terrain::island_terrain_mesh;
