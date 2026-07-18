pub(crate) const TERRAIN_UV_TILES_PER_METER: f32 = 1.0 / 18.0;
pub(crate) const TERRAIN_BIOME_PALETTE_COUNT: usize = 5;
pub(crate) const GROUND_COVER_PATCHES: usize = 44;
pub(crate) const GROUND_COVER_BLADES_PER_PATCH: usize = 5;
pub(crate) const VERTICES_PER_GROUND_BLADE: usize = 6;
pub(crate) const INDICES_PER_GROUND_BLADE: usize = 12;

#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_COLOR_BANDS: usize = 5;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS: usize = 12;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_CHANNELS: usize = 3;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_REGIONS: usize = 4;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_HEIGHT_BANDS: usize = 19;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_NORMAL_SLOPE_BANDS: usize = 10;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS: usize = 5;
#[cfg(test)]
pub(crate) static ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE: usize = 0;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_NEAR_DETAIL_ENERGY_PROMILLE: usize = 1;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_MID_DETAIL_ENERGY_PROMILLE: usize = 4;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_MACRO_DETAIL_ENERGY_PROMILLE: usize = 9;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_MIN_NEAR_TO_MID_RATIO_PROMILLE: usize = 200;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_MAX_NEAR_TO_MID_RATIO_PROMILLE: usize = 450;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_MAX_ISOLATED_EDGE_PROMILLE: usize = 20;
#[cfg(test)]
pub(crate) const ISLAND_IMPOSTOR_COLOR_BANDS: usize = 18;
pub(crate) const ISLAND_CLIFF_STRATA_BANDS: usize = 9;

pub(crate) const ISLAND_TERRAIN_RINGS: usize = 24;
pub(crate) const ISLAND_TERRAIN_EDGE_SKIRT_DEPTH_M: f32 = 0.32;
pub(crate) const ISLAND_BODY_SEGMENTS: usize = 96;
pub(crate) const ISLAND_IMPOSTOR_SEGMENTS: usize = 48;
pub(crate) const ISLAND_IMPOSTOR_TERRAIN_RINGS: usize = 6;
pub(crate) const ISLAND_IMPOSTOR_CLIFF_RINGS: usize = 4;
pub(crate) const ISLAND_IMPOSTOR_UNDERSIDE_RINGS: usize = 3;
pub(crate) const ISLAND_CLIFF_RINGS: usize = 8;
pub(crate) const ISLAND_UNDERSIDE_RINGS: usize = 7;
