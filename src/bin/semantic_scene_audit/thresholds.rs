pub(crate) const SAMPLE_SEARCH_RADIUS_PX: i32 = 20;
pub(crate) const MIN_SAMPLE_PIXEL_HITS: usize = 3;
pub(crate) const MIN_VISIBLE_SAMPLES_PER_CHECKPOINT: usize = 2;
pub(crate) const MIN_PASSED_SAMPLES_PER_CHECKPOINT: usize = 1;
pub(crate) const MIN_VISIBLE_MATERIALS_PER_CHECKPOINT: usize = 3;
pub(crate) const MIN_MATERIAL_SAMPLE_HIT_RATIO: f64 = 0.45;
pub(crate) const EXPECTED_MATERIALS: [&str; 4] = ["terrain", "foliage", "cloud", "distant_island"];
