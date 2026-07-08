#[derive(Clone, Debug)]
pub(crate) struct SceneSampleAudit {
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) expected_material: String,
    pub(crate) material_variant: String,
    pub(crate) in_viewport: bool,
    pub(crate) visibility: String,
    pub(crate) screen_x: Option<f64>,
    pub(crate) screen_y: Option<f64>,
    pub(crate) semantic_pixel_hits: usize,
    pub(crate) passed: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CheckpointAudit {
    pub(crate) metadata_path: String,
    pub(crate) screenshot_path: String,
    pub(crate) scenario: String,
    pub(crate) checkpoint: String,
    pub(crate) in_viewport_scene_sample_count: usize,
    pub(crate) occluded_scene_sample_count: usize,
    pub(crate) visible_scene_sample_count: usize,
    pub(crate) scene_sample_pixel_hit_count: usize,
    pub(crate) visible_scene_material_count: usize,
    pub(crate) scene_material_pixel_hit_count: usize,
    pub(crate) visible_scene_sample_kind_count: usize,
    pub(crate) scene_sample_kind_pixel_hit_count: usize,
    pub(crate) visible_terrain_material_variant_count: usize,
    pub(crate) terrain_material_variant_pixel_hit_count: usize,
    pub(crate) passed: bool,
    pub(crate) samples: Vec<SceneSampleAudit>,
    pub(crate) materials: Vec<MaterialAudit>,
}

#[derive(Clone, Debug)]
pub(crate) struct MaterialAudit {
    pub(crate) expected_material: String,
    pub(crate) visible_sample_count: usize,
    pub(crate) sample_pixel_hit_count: usize,
    pub(crate) min_sample_pixel_hit_count: usize,
    pub(crate) hit_ratio: f64,
    pub(crate) passed: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Check {
    pub(crate) name: String,
    pub(crate) passed: bool,
    pub(crate) value: f64,
    pub(crate) comparator: &'static str,
    pub(crate) threshold: f64,
    pub(crate) unit: &'static str,
}

impl Check {
    pub(crate) fn at_least(
        name: impl Into<String>,
        value: f64,
        threshold: f64,
        unit: &'static str,
    ) -> Self {
        Self {
            name: name.into(),
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }

    pub(crate) fn at_most(
        name: impl Into<String>,
        value: f64,
        threshold: f64,
        unit: &'static str,
    ) -> Self {
        Self {
            name: name.into(),
            passed: value <= threshold,
            value,
            comparator: "<=",
            threshold,
            unit,
        }
    }
}

impl SceneSampleAudit {
    pub(crate) fn is_visible(&self) -> bool {
        self.in_viewport && self.visibility == "visible"
    }
}
