pub(crate) const SAMPLE_SEARCH_RADIUS_PX: i32 = 20;
pub(crate) const MIN_SAMPLE_PIXEL_HITS: usize = 3;
pub(crate) const MIN_SUBSTANTIAL_WATER_LOCAL_HITS: usize = 24;
pub(crate) const MIN_HORIZONTAL_WATER_X_SPAN_PX: usize = 9;
pub(crate) const MIN_HORIZONTAL_WATER_Y_SPAN_PX: usize = 3;
pub(crate) const MIN_WATERFALL_X_SPAN_PX: usize = 3;
pub(crate) const MIN_WATERFALL_Y_SPAN_PX: usize = 9;
pub(crate) const MIN_WATER_BOUNDING_BOX_FILL_RATIO: f64 = 0.28;
pub(crate) const MIN_WATER_QUANTIZED_COLOR_BUCKETS: usize = 3;
pub(crate) const MIN_WATER_LUMA_P95_P5: f64 = 4.0;
pub(crate) const MIN_WATER_INTERNAL_EDGE_DENSITY: f64 = 0.02;
pub(crate) const WATER_INTERNAL_EDGE_LUMA_DELTA: f64 = 3.0;
pub(crate) const MIN_VISIBLE_SAMPLES_PER_CHECKPOINT: usize = 2;
pub(crate) const MIN_PASSED_SAMPLES_PER_CHECKPOINT: usize = 1;
pub(crate) const MIN_VISIBLE_MATERIALS_PER_CHECKPOINT: usize = 3;
pub(crate) const MIN_VISIBLE_SAMPLE_KINDS_PER_CHECKPOINT: usize = 3;
pub(crate) const MIN_MATERIAL_SAMPLE_HIT_RATIO: f64 = 0.45;
pub(crate) const MIN_TERRAIN_MATERIAL_SAMPLE_HIT_RATIO: f64 = 0.25;
pub(crate) const MIN_TERRAIN_MATERIAL_VARIANT_HIT_RATIO: f64 = 0.45;
pub(crate) const MIN_TERRAIN_PIXEL_COVERAGE: usize = 3_000;
pub(crate) const MIN_FOLIAGE_PIXEL_COVERAGE: usize = 5_000;
pub(crate) const MIN_CLOUD_PIXEL_COVERAGE: usize = 7_500;
pub(crate) const MIN_DISTANT_ISLAND_PIXEL_COVERAGE: usize = 10_000;
pub(crate) const MIN_WIND_PIXEL_COVERAGE_PER_VISIBLE_SAMPLE: usize = 12;
pub(crate) const MAX_WIND_PIXEL_COVERAGE_PER_CHECKPOINT: usize = 60_000;
pub(crate) const MAX_PLAYER_WIND_SHEAR_PIXEL_COVERAGE_PER_CHECKPOINT: usize = 8_000;
pub(crate) const MIN_VISIBLE_TERRAIN_MATERIAL_VARIANTS: usize = 3;
pub(crate) const MIN_PASSED_TERRAIN_MATERIAL_VARIANTS: usize = 3;
pub(crate) const MIN_TERRAIN_MATERIAL_VARIANT_PIXEL_COVERAGE: usize = 35;
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct WaterSampleThresholds {
    pub(crate) min_local_hit_count: usize,
    pub(crate) min_x_span: usize,
    pub(crate) min_y_span: usize,
    pub(crate) min_bounding_box_fill_ratio: f64,
    pub(crate) min_quantized_color_buckets: usize,
    pub(crate) min_luma_p95_p5: f64,
    pub(crate) min_internal_edge_density: f64,
}

pub(crate) fn water_sample_thresholds(
    kind: &str,
    screenshot_scale: (f64, f64),
) -> Option<WaterSampleThresholds> {
    let (base_x_span, base_y_span) = match kind {
        "water_surface" | "river_channel" => (
            MIN_HORIZONTAL_WATER_X_SPAN_PX,
            MIN_HORIZONTAL_WATER_Y_SPAN_PX,
        ),
        "waterfall_water" => (MIN_WATERFALL_X_SPAN_PX, MIN_WATERFALL_Y_SPAN_PX),
        _ => return None,
    };
    let scale_x = screenshot_scale.0.max(0.1);
    let scale_y = screenshot_scale.1.max(0.1);

    Some(WaterSampleThresholds {
        min_local_hit_count: ((MIN_SUBSTANTIAL_WATER_LOCAL_HITS as f64 * scale_x * scale_y).ceil()
            as usize)
            .max(MIN_SAMPLE_PIXEL_HITS),
        min_x_span: (base_x_span as f64 * scale_x).ceil() as usize,
        min_y_span: (base_y_span as f64 * scale_y).ceil() as usize,
        min_bounding_box_fill_ratio: MIN_WATER_BOUNDING_BOX_FILL_RATIO,
        min_quantized_color_buckets: MIN_WATER_QUANTIZED_COLOR_BUCKETS,
        min_luma_p95_p5: MIN_WATER_LUMA_P95_P5,
        min_internal_edge_density: MIN_WATER_INTERNAL_EDGE_DENSITY,
    })
}

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
