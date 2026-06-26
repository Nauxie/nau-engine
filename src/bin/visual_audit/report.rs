use std::fmt::Write as _;

use super::{
    thresholds::*,
    types::{Check, ImageAudit},
};

pub(super) fn report_checks(audits: &[ImageAudit]) -> Vec<Check> {
    let max_top_sky_fraction = audits
        .iter()
        .map(|audit| audit.top_sky_fraction)
        .fold(0.0, f64::max);
    let max_route_marker_fraction = audits
        .iter()
        .map(|audit| audit.route_marker_fraction)
        .fold(0.0, f64::max);
    let max_route_marker_component_count = audits
        .iter()
        .map(|audit| audit.route_marker_component_count)
        .max()
        .unwrap_or_default();
    let max_route_marker_hue_family_count = audits
        .iter()
        .map(|audit| audit.route_marker_hue_family_count)
        .max()
        .unwrap_or_default();
    let max_distant_scene_fraction = audits
        .iter()
        .map(|audit| audit.distant_scene_fraction)
        .fold(0.0, f64::max);
    let max_distant_scene_component_count = audits
        .iter()
        .map(|audit| audit.distant_scene_component_count)
        .max()
        .unwrap_or_default();
    let max_distant_scene_color_bucket_count = audits
        .iter()
        .map(|audit| audit.distant_scene_color_bucket_count)
        .max()
        .unwrap_or_default();
    let max_distant_scene_horizontal_span_fraction = audits
        .iter()
        .map(|audit| audit.distant_scene_horizontal_span_fraction)
        .fold(0.0, f64::max);
    let max_distant_scene_vertical_span_fraction = audits
        .iter()
        .map(|audit| audit.distant_scene_vertical_span_fraction)
        .fold(0.0, f64::max);
    let max_scene_material_family_count = audits
        .iter()
        .map(|audit| audit.scene_material_family_count)
        .max()
        .unwrap_or_default();
    let max_terrain_scene_fraction = audits
        .iter()
        .map(|audit| audit.terrain_scene_fraction)
        .fold(0.0, f64::max);
    let max_terrain_scene_tile_count = audits
        .iter()
        .map(|audit| audit.terrain_scene_tile_count)
        .max()
        .unwrap_or_default();
    let max_terrain_scene_color_bucket_count = audits
        .iter()
        .map(|audit| audit.terrain_scene_color_bucket_count)
        .max()
        .unwrap_or_default();
    let max_foliage_scene_fraction = audits
        .iter()
        .map(|audit| audit.foliage_scene_fraction)
        .fold(0.0, f64::max);
    let max_foliage_scene_tile_count = audits
        .iter()
        .map(|audit| audit.foliage_scene_tile_count)
        .max()
        .unwrap_or_default();
    let max_cloud_layer_fraction = audits
        .iter()
        .map(|audit| audit.cloud_layer_fraction)
        .fold(0.0, f64::max);
    let max_cloud_layer_component_count = audits
        .iter()
        .map(|audit| audit.cloud_layer_component_count)
        .max()
        .unwrap_or_default();
    let max_cloud_layer_horizontal_span_fraction = audits
        .iter()
        .map(|audit| audit.cloud_layer_horizontal_span_fraction)
        .fold(0.0, f64::max);
    let max_cloud_layer_vertical_span_fraction = audits
        .iter()
        .map(|audit| audit.cloud_layer_vertical_span_fraction)
        .fold(0.0, f64::max);

    vec![
        Check::at_least(
            "max_top_sky_fraction",
            max_top_sky_fraction,
            MIN_SEQUENCE_TOP_SKY_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_route_marker_fraction",
            max_route_marker_fraction,
            MIN_SEQUENCE_ROUTE_MARKER_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_route_marker_component_count",
            max_route_marker_component_count as f64,
            MIN_SEQUENCE_ROUTE_MARKER_COMPONENTS as f64,
            "components",
        ),
        Check::at_least(
            "max_route_marker_hue_family_count",
            max_route_marker_hue_family_count as f64,
            MIN_SEQUENCE_ROUTE_MARKER_HUE_FAMILIES as f64,
            "families",
        ),
        Check::at_least(
            "max_distant_scene_fraction",
            max_distant_scene_fraction,
            MIN_SEQUENCE_DISTANT_SCENE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_distant_scene_component_count",
            max_distant_scene_component_count as f64,
            MIN_SEQUENCE_DISTANT_SCENE_COMPONENTS as f64,
            "components",
        ),
        Check::at_least(
            "max_distant_scene_color_bucket_count",
            max_distant_scene_color_bucket_count as f64,
            MIN_SEQUENCE_DISTANT_SCENE_COLOR_BUCKETS as f64,
            "buckets",
        ),
        Check::at_least(
            "max_distant_scene_horizontal_span_fraction",
            max_distant_scene_horizontal_span_fraction,
            MIN_SEQUENCE_DISTANT_SCENE_HORIZONTAL_SPAN_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_distant_scene_vertical_span_fraction",
            max_distant_scene_vertical_span_fraction,
            MIN_SEQUENCE_DISTANT_SCENE_VERTICAL_SPAN_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_scene_material_family_count",
            max_scene_material_family_count as f64,
            MIN_SEQUENCE_SCENE_MATERIAL_FAMILIES as f64,
            "families",
        ),
        Check::at_least(
            "max_terrain_scene_fraction",
            max_terrain_scene_fraction,
            MIN_SEQUENCE_TERRAIN_SCENE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_terrain_scene_tile_count",
            max_terrain_scene_tile_count as f64,
            MIN_SEQUENCE_TERRAIN_SCENE_TILES as f64,
            "tiles",
        ),
        Check::at_least(
            "max_terrain_scene_color_bucket_count",
            max_terrain_scene_color_bucket_count as f64,
            MIN_SEQUENCE_TERRAIN_SCENE_COLOR_BUCKETS as f64,
            "buckets",
        ),
        Check::at_least(
            "max_foliage_scene_fraction",
            max_foliage_scene_fraction,
            MIN_SEQUENCE_FOLIAGE_SCENE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_foliage_scene_tile_count",
            max_foliage_scene_tile_count as f64,
            MIN_SEQUENCE_FOLIAGE_SCENE_TILES as f64,
            "tiles",
        ),
        Check::at_least(
            "max_cloud_layer_fraction",
            max_cloud_layer_fraction,
            MIN_SEQUENCE_CLOUD_LAYER_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_cloud_layer_component_count",
            max_cloud_layer_component_count as f64,
            MIN_SEQUENCE_CLOUD_LAYER_COMPONENTS as f64,
            "components",
        ),
        Check::at_least(
            "max_cloud_layer_horizontal_span_fraction",
            max_cloud_layer_horizontal_span_fraction,
            MIN_SEQUENCE_CLOUD_LAYER_HORIZONTAL_SPAN_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "max_cloud_layer_vertical_span_fraction",
            max_cloud_layer_vertical_span_fraction,
            MIN_SEQUENCE_CLOUD_LAYER_VERTICAL_SPAN_FRACTION,
            "ratio",
        ),
    ]
}

pub(super) fn report_passed(audits: &[ImageAudit], report_checks: &[Check]) -> bool {
    audits.iter().all(|audit| audit.passed) && report_checks.iter().all(|check| check.passed)
}

pub(super) fn audit_report_json(
    passed: bool,
    report_checks: &[Check],
    audits: &[ImageAudit],
) -> String {
    let checks = report_checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let images = audits
        .iter()
        .map(image_audit_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!(
        "{{\n  \"passed\": {},\n  \"image_count\": {},\n  \"checks\": [\n    {}\n  ],\n  \"images\": [\n    {}\n  ]\n}}",
        passed,
        audits.len(),
        checks,
        images
    )
}

fn image_audit_json(audit: &ImageAudit) -> String {
    let checks = audit
        .checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n      ");
    format!(
        "{{\n      \"path\": {},\n      \"passed\": {},\n      \"width\": {},\n      \"height\": {},\n      \"mean_luma\": {},\n      \"luma_stddev\": {},\n      \"colorfulness\": {},\n      \"quantized_colors\": {},\n      \"edge_density\": {},\n      \"top_sky_fraction\": {},\n      \"lower_scene_fraction\": {},\n      \"center_scene_fraction\": {},\n      \"center_edge_density\": {},\n      \"scene_detail_tile_fraction\": {},\n      \"flat_scene_tile_fraction\": {},\n      \"dominant_low_detail_scene_component_fraction\": {},\n      \"scene_detail_tile_count\": {},\n      \"flat_scene_tile_count\": {},\n      \"scene_candidate_tile_count\": {},\n      \"player_focus_fraction\": {},\n      \"player_warm_focus_fraction\": {},\n      \"route_marker_fraction\": {},\n      \"route_marker_component_count\": {},\n      \"route_marker_hue_family_count\": {},\n      \"distant_scene_fraction\": {},\n      \"distant_scene_component_count\": {},\n      \"distant_scene_color_bucket_count\": {},\n      \"distant_scene_horizontal_span_fraction\": {},\n      \"distant_scene_vertical_span_fraction\": {},\n      \"scene_material_family_count\": {},\n      \"terrain_scene_fraction\": {},\n      \"terrain_scene_tile_count\": {},\n      \"terrain_scene_color_bucket_count\": {},\n      \"foliage_scene_fraction\": {},\n      \"foliage_scene_tile_count\": {},\n      \"cloud_layer_fraction\": {},\n      \"cloud_layer_component_count\": {},\n      \"cloud_layer_horizontal_span_fraction\": {},\n      \"cloud_layer_vertical_span_fraction\": {},\n      \"severe_clipping_fraction\": {},\n      \"transparent_pixel_fraction\": {},\n      \"foreign_canvas_fraction\": {},\n      \"hud_text_fraction\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
        json_string(&audit.path),
        audit.passed,
        audit.width,
        audit.height,
        json_number(audit.mean_luma),
        json_number(audit.luma_stddev),
        json_number(audit.colorfulness),
        audit.quantized_colors,
        json_number(audit.edge_density),
        json_number(audit.top_sky_fraction),
        json_number(audit.lower_scene_fraction),
        json_number(audit.center_scene_fraction),
        json_number(audit.center_edge_density),
        json_number(audit.scene_detail_tile_fraction),
        json_number(audit.flat_scene_tile_fraction),
        json_number(audit.dominant_low_detail_scene_component_fraction),
        audit.scene_detail_tile_count,
        audit.flat_scene_tile_count,
        audit.scene_candidate_tile_count,
        json_number(audit.player_focus_fraction),
        json_number(audit.player_warm_focus_fraction),
        json_number(audit.route_marker_fraction),
        audit.route_marker_component_count,
        audit.route_marker_hue_family_count,
        json_number(audit.distant_scene_fraction),
        audit.distant_scene_component_count,
        audit.distant_scene_color_bucket_count,
        json_number(audit.distant_scene_horizontal_span_fraction),
        json_number(audit.distant_scene_vertical_span_fraction),
        audit.scene_material_family_count,
        json_number(audit.terrain_scene_fraction),
        audit.terrain_scene_tile_count,
        audit.terrain_scene_color_bucket_count,
        json_number(audit.foliage_scene_fraction),
        audit.foliage_scene_tile_count,
        json_number(audit.cloud_layer_fraction),
        audit.cloud_layer_component_count,
        json_number(audit.cloud_layer_horizontal_span_fraction),
        json_number(audit.cloud_layer_vertical_span_fraction),
        json_number(audit.severe_clipping_fraction),
        json_number(audit.transparent_pixel_fraction),
        json_number(audit.foreign_canvas_fraction),
        json_number(audit.hud_text_fraction),
        checks,
    )
}

fn check_json(check: &Check) -> String {
    format!(
        "{{\"name\": {}, \"passed\": {}, \"value\": {}, \"comparator\": {}, \"threshold\": {}, \"unit\": {}}}",
        json_string(check.name),
        check.passed,
        json_number(check.value),
        json_string(check.comparator),
        json_number(check.threshold),
        json_string(check.unit)
    )
}

fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

pub(super) fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character <= '\u{1f}' => {
                write!(&mut escaped, "\\u{:04x}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            character => escaped.push(character),
        }
    }
    format!("\"{escaped}\"")
}
