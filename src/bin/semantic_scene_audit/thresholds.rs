pub(crate) const SAMPLE_SEARCH_RADIUS_PX: i32 = 20;
pub(crate) const MIN_SAMPLE_PIXEL_HITS: usize = 3;
pub(crate) const MIN_VISIBLE_SAMPLES_PER_CHECKPOINT: usize = 2;
pub(crate) const MIN_PASSED_SAMPLES_PER_CHECKPOINT: usize = 1;
pub(crate) const MIN_VISIBLE_MATERIALS_PER_CHECKPOINT: usize = 3;
pub(crate) const MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT: usize = 3;
pub(crate) const MIN_MATERIAL_SAMPLE_HIT_RATIO: f64 = 0.45;
pub(crate) const MIN_TERRAIN_PIXEL_COVERAGE: usize = 3_000;
pub(crate) const MIN_FOLIAGE_PIXEL_COVERAGE: usize = 5_000;
pub(crate) const MIN_CLOUD_PIXEL_COVERAGE: usize = 7_500;
pub(crate) const MIN_DISTANT_ISLAND_PIXEL_COVERAGE: usize = 10_000;
pub(crate) const MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE: usize = 12;
pub(crate) const MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS: usize = 3;
pub(crate) const MIN_PASSED_TERRAIN_MATERIAL_VARIANTS: usize = 3;
pub(crate) const MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE: usize = 1_000;
pub(crate) const EXPECTED_MATERIALS: [&str; 4] = ["terrain", "foliage", "cloud", "distant_island"];
pub(crate) const EXPECTED_SCENE_SAMPLE_KINDS: [&str; 4] = [
    "terrain_surface",
    "tree_canopy",
    "weather_cloud",
    "distant_island",
];
pub(crate) const EXPECTED_TERRAIN_MATERIAL_VARIANTS: [&str; 5] = [
    "terrain_lush_meadow",
    "terrain_gold_meadow",
    "terrain_copper_clay",
    "terrain_alpine_mist",
    "terrain_highland_grass",
];

pub(crate) fn expected_material_pixel_coverage_floor(expected_material: &str) -> usize {
    match expected_material {
        "terrain" => MIN_TERRAIN_PIXEL_COVERAGE,
        "foliage" => MIN_FOLIAGE_PIXEL_COVERAGE,
        "cloud" => MIN_CLOUD_PIXEL_COVERAGE,
        "distant_island" => MIN_DISTANT_ISLAND_PIXEL_COVERAGE,
        _ => 0,
    }
}

pub(crate) fn expected_scene_kind_pixel_coverage_floor(kind: &str) -> usize {
    match kind {
        "terrain_surface" => MIN_TERRAIN_PIXEL_COVERAGE,
        "tree_canopy" => MIN_FOLIAGE_PIXEL_COVERAGE,
        "weather_cloud" => MIN_CLOUD_PIXEL_COVERAGE,
        "distant_island" => MIN_DISTANT_ISLAND_PIXEL_COVERAGE,
        _ => 0,
    }
}
