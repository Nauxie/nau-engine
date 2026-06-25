#[derive(Debug)]
pub(super) struct ImageAudit {
    pub(super) path: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) mean_luma: f64,
    pub(super) luma_stddev: f64,
    pub(super) colorfulness: f64,
    pub(super) quantized_colors: usize,
    pub(super) edge_density: f64,
    pub(super) top_sky_fraction: f64,
    pub(super) lower_scene_fraction: f64,
    pub(super) center_scene_fraction: f64,
    pub(super) center_edge_density: f64,
    pub(super) scene_detail_tile_fraction: f64,
    pub(super) flat_scene_tile_fraction: f64,
    pub(super) scene_detail_tile_count: usize,
    pub(super) flat_scene_tile_count: usize,
    pub(super) scene_candidate_tile_count: usize,
    pub(super) player_focus_fraction: f64,
    pub(super) player_warm_focus_fraction: f64,
    pub(super) route_marker_fraction: f64,
    pub(super) route_marker_component_count: usize,
    pub(super) route_marker_hue_family_count: usize,
    pub(super) distant_scene_fraction: f64,
    pub(super) distant_scene_component_count: usize,
    pub(super) distant_scene_color_bucket_count: usize,
    pub(super) scene_material_family_count: usize,
    pub(super) foliage_scene_fraction: f64,
    pub(super) cloud_layer_fraction: f64,
    pub(super) cloud_layer_component_count: usize,
    pub(super) severe_clipping_fraction: f64,
    pub(super) transparent_pixel_fraction: f64,
    pub(super) foreign_canvas_fraction: f64,
    pub(super) hud_text_fraction: f64,
    pub(super) passed: bool,
    pub(super) checks: Vec<Check>,
}

#[derive(Debug)]
pub(super) struct Check {
    pub(super) name: &'static str,
    pub(super) passed: bool,
    pub(super) value: f64,
    pub(super) comparator: &'static str,
    pub(super) threshold: f64,
    pub(super) unit: &'static str,
}

impl Check {
    pub(super) fn at_least(
        name: &'static str,
        value: f64,
        threshold: f64,
        unit: &'static str,
    ) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }

    pub(super) fn at_most(
        name: &'static str,
        value: f64,
        threshold: f64,
        unit: &'static str,
    ) -> Self {
        Self {
            name,
            passed: value <= threshold,
            value,
            comparator: "<=",
            threshold,
            unit,
        }
    }
}
