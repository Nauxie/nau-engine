use crate::{
    thresholds::{
        EXPECTED_MATERIALS, MIN_MATERIAL_SAMPLE_HIT_RATIO, MIN_SAMPLE_PIXEL_HITS,
        MIN_TERRAIN_MATERIAL_SAMPLE_HIT_RATIO, SAMPLE_SEARCH_RADIUS_PX,
        WATER_INTERNAL_EDGE_LUMA_DELTA, water_sample_thresholds,
    },
    types::{MaterialAudit, SceneSampleAudit, WaterAggregateMetrics, WaterLocalMetrics},
};
use image::RgbImage;
use std::collections::{BTreeMap, BTreeSet};

const CONDITIONAL_MATERIALS: [&str; 4] = ["water", "stone", "wood", "flower"];

pub(crate) fn material_audits(samples: &[SceneSampleAudit]) -> Vec<MaterialAudit> {
    let water_metrics = aggregate_water_metrics(samples);
    let mut expected_materials = EXPECTED_MATERIALS.to_vec();
    for expected_material in CONDITIONAL_MATERIALS {
        if samples
            .iter()
            .any(|sample| sample.is_visible() && sample.expected_material == expected_material)
        {
            expected_materials.push(expected_material);
        }
    }
    if samples
        .iter()
        .any(|sample| sample.passed && sample.expected_material == "wind")
    {
        expected_materials.push("wind");
    }

    expected_materials
        .iter()
        .filter_map(|expected_material| {
            let visible_sample_count = samples
                .iter()
                .filter(|sample| {
                    sample.is_visible() && sample.expected_material == *expected_material
                })
                .count();
            if visible_sample_count == 0 {
                return None;
            }
            let sample_pixel_hit_count = samples
                .iter()
                .filter(|sample| sample.passed && sample.expected_material == *expected_material)
                .count();
            let min_sample_pixel_hit_count =
                min_material_sample_pixel_hit_count_for(expected_material, visible_sample_count);
            let hit_ratio = sample_pixel_hit_count as f64 / visible_sample_count as f64;

            Some(MaterialAudit {
                expected_material: (*expected_material).to_string(),
                visible_sample_count,
                sample_pixel_hit_count,
                min_sample_pixel_hit_count,
                hit_ratio,
                water_metrics: if *expected_material == "water" {
                    water_metrics.clone()
                } else {
                    None
                },
                passed: sample_pixel_hit_count >= min_sample_pixel_hit_count,
            })
        })
        .collect()
}

fn min_material_sample_pixel_hit_count_for(
    expected_material: &str,
    visible_sample_count: usize,
) -> usize {
    if expected_material == "wind" || CONDITIONAL_MATERIALS.contains(&expected_material) {
        return visible_sample_count.min(1);
    }
    if expected_material == "terrain" {
        return min_sample_hit_count_with_ratio(
            visible_sample_count,
            MIN_TERRAIN_MATERIAL_SAMPLE_HIT_RATIO,
        );
    }

    min_material_sample_pixel_hit_count(visible_sample_count)
}

pub(crate) fn min_material_sample_pixel_hit_count(visible_sample_count: usize) -> usize {
    min_sample_hit_count_with_ratio(visible_sample_count, MIN_MATERIAL_SAMPLE_HIT_RATIO)
}

pub(crate) fn min_sample_hit_count_with_ratio(visible_sample_count: usize, ratio: f64) -> usize {
    if visible_sample_count == 0 {
        0
    } else {
        (visible_sample_count as f64 * ratio).ceil().max(1.0) as usize
    }
}

#[cfg(test)]
pub(crate) fn sample_pixel_hits(
    image: &RgbImage,
    screen_x: f64,
    screen_y: f64,
    expected_material: &str,
    screenshot_scale: (f64, f64),
) -> usize {
    sample_pixel_hits_with_variant(
        image,
        screen_x,
        screen_y,
        expected_material,
        "terrain_unknown",
        screenshot_scale,
    )
}

pub(crate) fn sample_pixel_hits_with_variant(
    image: &RgbImage,
    screen_x: f64,
    screen_y: f64,
    expected_material: &str,
    material_variant: &str,
    screenshot_scale: (f64, f64),
) -> usize {
    if !screen_x.is_finite() || !screen_y.is_finite() {
        return 0;
    }

    let width = image.width() as i32;
    let height = image.height() as i32;
    let scale_x = screenshot_scale.0.max(0.1);
    let scale_y = screenshot_scale.1.max(0.1);
    let center_x = (screen_x * scale_x).round() as i32;
    let center_y = (screen_y * scale_y).round() as i32;
    let radius_x = (SAMPLE_SEARCH_RADIUS_PX as f64 * scale_x).ceil() as i32;
    let radius_y = (SAMPLE_SEARCH_RADIUS_PX as f64 * scale_y).ceil() as i32;
    let mut hits = 0usize;

    for y in (center_y - radius_y).max(0)..=(center_y + radius_y).min(height.saturating_sub(1)) {
        for x in (center_x - radius_x).max(0)..=(center_x + radius_x).min(width.saturating_sub(1)) {
            let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
            if material_variant_matches(
                expected_material,
                material_variant,
                r as f64,
                g as f64,
                b as f64,
            ) {
                hits += 1;
            }
        }
    }

    hits
}

pub(crate) fn water_local_metrics(
    image: &RgbImage,
    screen_x: f64,
    screen_y: f64,
    sample_kind: &str,
    material_variant: &str,
    screenshot_scale: (f64, f64),
) -> WaterLocalMetrics {
    let thresholds = water_sample_thresholds(sample_kind, screenshot_scale);
    let mut metrics = WaterLocalMetrics {
        quality_required: thresholds.is_some(),
        ..Default::default()
    };
    if !screen_x.is_finite() || !screen_y.is_finite() || image.width() == 0 || image.height() == 0 {
        return metrics;
    }

    let width = image.width() as i32;
    let height = image.height() as i32;
    let scale_x = screenshot_scale.0.max(0.1);
    let scale_y = screenshot_scale.1.max(0.1);
    let center_x = (screen_x * scale_x).round() as i32;
    let center_y = (screen_y * scale_y).round() as i32;
    let radius_x = (SAMPLE_SEARCH_RADIUS_PX as f64 * scale_x).ceil() as i32;
    let radius_y = (SAMPLE_SEARCH_RADIUS_PX as f64 * scale_y).ceil() as i32;
    let min_window_x = (center_x - radius_x).max(0);
    let max_window_x = (center_x + radius_x).min(width - 1);
    let min_window_y = (center_y - radius_y).max(0);
    let max_window_y = (center_y + radius_y).min(height - 1);
    if min_window_x > max_window_x || min_window_y > max_window_y {
        return metrics;
    }

    let window_width = (max_window_x - min_window_x + 1) as usize;
    let window_height = (max_window_y - min_window_y + 1) as usize;
    let mut pixels = vec![None; window_width * window_height];

    for y in min_window_y..=max_window_y {
        for x in min_window_x..=max_window_x {
            let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
            if !water_sample_pixel_matches(
                sample_kind,
                material_variant,
                f64::from(r),
                f64::from(g),
                f64::from(b),
            ) {
                continue;
            }

            let luma = 0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
            let bucket = quantized_water_color_bucket(r, g, b);
            let pixel_index =
                (y - min_window_y) as usize * window_width + (x - min_window_x) as usize;
            pixels[pixel_index] = Some(WaterPixel { bucket, luma });
        }
    }

    let component_indices =
        select_water_component(&pixels, window_width, window_height, thresholds);
    metrics.local_hit_count = component_indices.len();
    if component_indices.is_empty() {
        apply_water_quality_thresholds(&mut metrics, thresholds);
        return metrics;
    }

    let mut component_pixels = vec![None; pixels.len()];
    let mut lumas = Vec::with_capacity(component_indices.len());
    let mut color_buckets = BTreeSet::new();
    let mut min_hit_x = usize::MAX;
    let mut max_hit_x = 0usize;
    let mut min_hit_y = usize::MAX;
    let mut max_hit_y = 0usize;
    for index in component_indices {
        let pixel = pixels[index].expect("selected water component pixel");
        component_pixels[index] = Some(pixel);
        lumas.push(pixel.luma);
        color_buckets.insert(pixel.bucket);
        let x = index % window_width;
        let y = index / window_width;
        min_hit_x = min_hit_x.min(x);
        max_hit_x = max_hit_x.max(x);
        min_hit_y = min_hit_y.min(y);
        max_hit_y = max_hit_y.max(y);
    }

    metrics.quantized_color_bucket_count = color_buckets.len();
    metrics.x_span = max_hit_x - min_hit_x + 1;
    metrics.y_span = max_hit_y - min_hit_y + 1;
    metrics.bounding_box_fill_ratio =
        metrics.local_hit_count as f64 / (metrics.x_span * metrics.y_span) as f64;
    lumas.sort_by(f64::total_cmp);
    let last = lumas.len() - 1;
    let p5_index = ((last as f64) * 0.05).floor() as usize;
    let p95_index = ((last as f64) * 0.95).ceil() as usize;
    metrics.luma_p95_p5 = lumas[p95_index.min(last)] - lumas[p5_index.min(last)];
    metrics.internal_edge_density =
        water_internal_edge_density(&component_pixels, window_width, window_height);
    apply_water_quality_thresholds(&mut metrics, thresholds);
    metrics
}

pub(crate) fn aggregate_water_metrics(
    samples: &[SceneSampleAudit],
) -> Option<WaterAggregateMetrics> {
    let visible_water = samples
        .iter()
        .filter(|sample| sample.is_visible() && sample.expected_material == "water")
        .collect::<Vec<_>>();
    if visible_water.is_empty() {
        return None;
    }

    let mut aggregate = WaterAggregateMetrics {
        visible_sample_count: visible_water.len(),
        ..Default::default()
    };
    let mut internal_edge_density_sum = 0.0;
    let mut metric_count = 0usize;
    let mut quality_evidence =
        BTreeMap::<(Option<&str>, &str, &str), WaterQualityEvidenceGroup>::new();

    for sample in visible_water {
        let quality_required = sample
            .water_local_metrics
            .as_ref()
            .map(|metrics| metrics.quality_required)
            .unwrap_or_else(|| water_sample_thresholds(&sample.kind, (1.0, 1.0)).is_some());
        aggregate.projected_quality_required_sample_count += usize::from(quality_required);

        let Some(metrics) = sample.water_local_metrics.as_ref() else {
            continue;
        };
        aggregate.total_local_hit_count += metrics.local_hit_count;
        aggregate.max_x_span = aggregate.max_x_span.max(metrics.x_span);
        aggregate.max_y_span = aggregate.max_y_span.max(metrics.y_span);
        aggregate.max_quantized_color_bucket_count = aggregate
            .max_quantized_color_bucket_count
            .max(metrics.quantized_color_bucket_count);
        aggregate.max_luma_p95_p5 = aggregate.max_luma_p95_p5.max(metrics.luma_p95_p5);
        internal_edge_density_sum += metrics.internal_edge_density;
        metric_count += 1;

        if quality_required && metrics.area_span_passed {
            let evidence = quality_evidence
                .entry((
                    sample.island_name.as_deref(),
                    sample.kind.as_str(),
                    sample.label.as_str(),
                ))
                .or_default();
            evidence.area_span_passed = true;
            evidence.color_bucket_passed |= metrics.color_bucket_passed;
            evidence.luma_variation_passed |= metrics.luma_variation_passed;
            evidence.internal_edge_density_passed |= metrics.internal_edge_density_passed;
            evidence.quality_passed |= metrics.passed;
        }
    }

    if metric_count > 0 {
        aggregate.mean_internal_edge_density = internal_edge_density_sum / metric_count as f64;
    }
    aggregate.quality_required_sample_count = quality_evidence.len();
    for evidence in quality_evidence.values() {
        aggregate.area_span_passed_sample_count += usize::from(evidence.area_span_passed);
        aggregate.color_bucket_passed_sample_count += usize::from(evidence.color_bucket_passed);
        aggregate.luma_variation_passed_sample_count += usize::from(evidence.luma_variation_passed);
        aggregate.internal_edge_density_passed_sample_count +=
            usize::from(evidence.internal_edge_density_passed);
        aggregate.quality_passed_sample_count += usize::from(evidence.quality_passed);
    }
    aggregate.passed =
        aggregate.quality_passed_sample_count == aggregate.quality_required_sample_count;
    Some(aggregate)
}

#[derive(Default)]
struct WaterQualityEvidenceGroup {
    area_span_passed: bool,
    color_bucket_passed: bool,
    luma_variation_passed: bool,
    internal_edge_density_passed: bool,
    quality_passed: bool,
}

#[derive(Clone, Copy)]
struct WaterPixel {
    bucket: u16,
    luma: f64,
}

fn select_water_component(
    pixels: &[Option<WaterPixel>],
    width: usize,
    height: usize,
    thresholds: Option<crate::thresholds::WaterSampleThresholds>,
) -> Vec<usize> {
    let mut visited = vec![false; pixels.len()];
    let mut stack = Vec::new();
    let mut best_component = Vec::new();
    let mut best_meets_span_floor = false;
    let mut best_fill_ratio = 0.0;

    for start in 0..pixels.len() {
        if visited[start] || pixels[start].is_none() {
            continue;
        }

        visited[start] = true;
        stack.push(start);
        let mut component = Vec::new();
        let mut min_x = usize::MAX;
        let mut max_x = 0usize;
        let mut min_y = usize::MAX;
        let mut max_y = 0usize;
        while let Some(index) = stack.pop() {
            component.push(index);
            let x = index % width;
            let y = index / width;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);

            if x > 0 {
                push_water_component_neighbor(index - 1, pixels, &mut visited, &mut stack);
            }
            if x + 1 < width {
                push_water_component_neighbor(index + 1, pixels, &mut visited, &mut stack);
            }
            if y > 0 {
                push_water_component_neighbor(index - width, pixels, &mut visited, &mut stack);
            }
            if y + 1 < height {
                push_water_component_neighbor(index + width, pixels, &mut visited, &mut stack);
            }
        }

        let x_span = max_x - min_x + 1;
        let y_span = max_y - min_y + 1;
        let meets_span_floor =
            thresholds.map_or(component.len() >= MIN_SAMPLE_PIXEL_HITS, |thresholds| {
                component.len() >= thresholds.min_local_hit_count
                    && x_span >= thresholds.min_x_span
                    && y_span >= thresholds.min_y_span
            });
        let fill_ratio = component.len() as f64 / (x_span * y_span) as f64;
        let replace = best_component.is_empty()
            || (meets_span_floor && !best_meets_span_floor)
            || (meets_span_floor == best_meets_span_floor
                && (component.len() > best_component.len()
                    || (component.len() == best_component.len() && fill_ratio > best_fill_ratio)));
        if replace {
            best_component = component;
            best_meets_span_floor = meets_span_floor;
            best_fill_ratio = fill_ratio;
        }
    }

    best_component
}

fn push_water_component_neighbor(
    index: usize,
    pixels: &[Option<WaterPixel>],
    visited: &mut [bool],
    stack: &mut Vec<usize>,
) {
    if !visited[index] && pixels[index].is_some() {
        visited[index] = true;
        stack.push(index);
    }
}

fn quantized_water_color_bucket(r: u8, g: u8, b: u8) -> u16 {
    ((u16::from(r) >> 3) << 10) | ((u16::from(g) >> 3) << 5) | (u16::from(b) >> 3)
}

fn water_internal_edge_density(pixels: &[Option<WaterPixel>], width: usize, height: usize) -> f64 {
    let mut internal_pairs = 0usize;
    let mut edge_pairs = 0usize;

    for y in 0..height {
        for x in 0..width {
            let Some(pixel) = pixels[y * width + x] else {
                continue;
            };
            for (neighbor_x, neighbor_y) in [(x + 1, y), (x, y + 1)] {
                if neighbor_x >= width || neighbor_y >= height {
                    continue;
                }
                let Some(neighbor) = pixels[neighbor_y * width + neighbor_x] else {
                    continue;
                };
                internal_pairs += 1;
                if pixel.bucket != neighbor.bucket
                    || (pixel.luma - neighbor.luma).abs() >= WATER_INTERNAL_EDGE_LUMA_DELTA
                {
                    edge_pairs += 1;
                }
            }
        }
    }

    if internal_pairs == 0 {
        0.0
    } else {
        edge_pairs as f64 / internal_pairs as f64
    }
}

fn apply_water_quality_thresholds(
    metrics: &mut WaterLocalMetrics,
    thresholds: Option<crate::thresholds::WaterSampleThresholds>,
) {
    let Some(thresholds) = thresholds else {
        metrics.area_span_passed = metrics.local_hit_count >= MIN_SAMPLE_PIXEL_HITS;
        metrics.color_bucket_passed = true;
        metrics.luma_variation_passed = true;
        metrics.internal_edge_density_passed = true;
        metrics.passed = metrics.area_span_passed;
        return;
    };

    metrics.area_span_passed = metrics.local_hit_count >= thresholds.min_local_hit_count
        && metrics.x_span >= thresholds.min_x_span
        && metrics.y_span >= thresholds.min_y_span
        && metrics.bounding_box_fill_ratio >= thresholds.min_bounding_box_fill_ratio;
    metrics.color_bucket_passed =
        metrics.quantized_color_bucket_count >= thresholds.min_quantized_color_buckets;
    metrics.luma_variation_passed = metrics.luma_p95_p5 >= thresholds.min_luma_p95_p5;
    metrics.internal_edge_density_passed =
        metrics.internal_edge_density >= thresholds.min_internal_edge_density;
    metrics.passed = metrics.area_span_passed
        && metrics.color_bucket_passed
        && metrics.luma_variation_passed
        && metrics.internal_edge_density_passed;
}

pub(crate) fn material_variant_matches(
    expected_material: &str,
    material_variant: &str,
    r: f64,
    g: f64,
    b: f64,
) -> bool {
    if expected_material == "terrain" {
        return terrain_variant_matches(material_variant, r, g, b);
    }
    if expected_material == "water" {
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let lit_surface_water = (78.0..=150.0).contains(&r)
            && luma <= 190.0
            && g >= r + 45.0
            && b >= g + 8.0
            && b <= g + 48.0;
        if is_sky_like(r, g, b, luma) && !lit_surface_water {
            return false;
        }
    }

    material_matches(expected_material, r, g, b)
}

fn water_sample_pixel_matches(
    sample_kind: &str,
    material_variant: &str,
    r: f64,
    g: f64,
    b: f64,
) -> bool {
    material_variant_matches("water", material_variant, r, g, b)
        || (matches!(
            sample_kind,
            "water_surface" | "river_channel" | "waterfall_water"
        ) && is_translucent_surface_water_like(r, g, b))
        || (sample_kind == "waterfall_water" && is_waterfall_foam_like(r, g, b))
        || (sample_kind == "water_detail_plunge_pool" && is_additive_plunge_foam_like(r, g, b))
}

fn is_translucent_surface_water_like(r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    (35.0..=215.0).contains(&luma)
        && g >= r + 15.0
        && b >= r + 10.0
        && b >= g - 18.0
        && b <= g + 22.0
}

fn is_waterfall_foam_like(r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let saturation = r.max(g).max(b) - r.min(g).min(b);
    (160.0..=238.0).contains(&luma) && saturation <= 24.0 && g >= r && b >= r && b <= g + 12.0
}

fn is_additive_plunge_foam_like(r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let saturation = r.max(g).max(b) - r.min(g).min(b);
    (145.0..=248.0).contains(&luma) && saturation <= 16.0 && g >= r && b >= g - 4.0 && b <= g + 12.0
}

pub(crate) fn material_matches(expected_material: &str, r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let sky_like = is_sky_like(r, g, b, luma);
    match expected_material {
        "terrain" => is_scene_like(r, g, b, luma, sky_like),
        "foliage" => is_foliage_like(r, g, b, luma, sky_like),
        "cloud" => is_cloud_like(r, g, b, luma, sky_like),
        "distant_island" => is_distant_scene_like(r, g, b, luma, sky_like),
        "wind" => is_wind_like(r, g, b, luma),
        "water" => is_water_like(r, g, b, luma),
        "stone" => is_stone_like(r, g, b, luma, sky_like),
        "wood" => is_wood_like(r, g, b, luma, sky_like),
        "flower" => is_flower_like(r, g, b, luma),
        _ => false,
    }
}

pub(crate) fn terrain_variant_matches(material_variant: &str, r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if is_non_surface_variant_color(r, g, b, luma) {
        return false;
    }

    match material_variant {
        "terrain_lush_meadow" => is_lush_meadow_variant(r, g, b, luma),
        "terrain_gold_meadow" => is_gold_meadow_variant(r, g, b, luma),
        "terrain_copper_clay" => is_copper_clay_variant(r, g, b, luma),
        "terrain_alpine_mist" => is_alpine_mist_variant(r, g, b, luma),
        "terrain_highland_grass" => is_highland_grass_variant(r, g, b, luma),
        _ => is_scene_like(r, g, b, luma, is_sky_like(r, g, b, luma)),
    }
}

fn is_non_surface_variant_color(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    let clear_blue_sky = luma >= 80.0 && b >= 128.0 && b >= r + 24.0 && b >= g + 10.0;
    let pale_neutral_cloud = luma >= 135.0 && saturation <= 30.0 && max_channel >= 145.0;

    luma <= 8.0 || luma >= 245.0 || clear_blue_sky || pale_neutral_cloud
}

fn is_lush_meadow_variant(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (28.0..=185.0).contains(&luma)
        && g >= 74.0
        && g >= r + 18.0
        && g >= b + 18.0
        && r <= 125.0
        && b <= 125.0
}

fn is_gold_meadow_variant(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (55.0..=205.0).contains(&luma)
        && r >= 72.0
        && g >= 88.0
        && b <= 132.0
        && g >= b + 24.0
        && r >= b + 14.0
        && g >= r - 16.0
}

fn is_copper_clay_variant(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (42.0..=185.0).contains(&luma)
        && r >= 78.0
        && g >= 52.0
        && b <= 122.0
        && r >= g + 8.0
        && r >= b + 24.0
        && g >= b + 6.0
}

fn is_alpine_mist_variant(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (38.0..=205.0).contains(&luma)
        && g >= 76.0
        && b >= 82.0
        && b >= r + 8.0
        && g >= r + 4.0
        && (b - g).abs() <= 44.0
}

fn is_highland_grass_variant(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (58.0..=205.0).contains(&luma)
        && r >= 82.0
        && g >= 86.0
        && b <= 132.0
        && g >= b + 14.0
        && r >= b + 12.0
        && (r - g).abs() <= 42.0
}

pub(crate) fn is_sky_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let blue_haze = b >= 105.0 && g >= 95.0 && b >= r + 8.0 && luma >= 80.0;
    let pale_cloud_haze = r >= 130.0 && g >= 140.0 && b >= 145.0 && b >= r - 4.0 && g >= r - 12.0;
    blue_haze || pale_cloud_haze
}

pub(crate) fn is_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if luma <= 8.0 || luma >= 245.0 {
        return false;
    }

    let water = luma <= 170.0
        && r <= 115.0
        && g >= 45.0
        && b >= 40.0
        && r <= g + 25.0
        && (g >= r + 8.0 || b >= r + 8.0);
    if water {
        return true;
    }
    if sky_like {
        return false;
    }

    is_foliage_like(r, g, b, luma, sky_like)
        || is_earth_like(r, g, b)
        || is_rock_or_shadow_like(r, g, b, luma)
}

pub(crate) fn is_foliage_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    !sky_like && (18.0..=185.0).contains(&luma) && g >= 58.0 && g >= r * 0.72 && g >= b * 0.58
}

pub(crate) fn is_earth_like(r: f64, g: f64, b: f64) -> bool {
    r >= 50.0 && g >= 38.0 && r >= b + 8.0 && g >= b * 0.68
}

pub(crate) fn is_rock_or_shadow_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (18.0..=155.0).contains(&luma) && (r - g).abs() <= 50.0 && b <= r.max(g) + 20.0
}

pub(crate) fn is_water_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    if !(18.0..=235.0).contains(&luma) {
        return false;
    }

    let shadowed_teal = luma <= 45.0
        && r <= 35.0
        && g <= 50.0
        && b <= 65.0
        && g >= r + 8.0
        && b >= r + 12.0
        && b >= g - 8.0
        && b <= g + 42.0;
    let deep_shadow_blue = luma <= 55.0
        && r <= 45.0
        && g <= 60.0
        && b <= 75.0
        && g >= r + 5.0
        && b >= g + 5.0
        && b >= r + 12.0;
    let muted_teal = luma <= 100.0
        && r <= 75.0
        && (45.0..=95.0).contains(&g)
        && (48.0..=110.0).contains(&b)
        && g >= r + 7.0
        && b >= r + 10.0
        && b >= g - 10.0
        && b <= g + 35.0;
    let deep_blue = r <= 105.0 && g >= 65.0 && b >= 120.0 && g >= r + 35.0 && b >= g + 28.0;
    let bright_cyan = g >= r + 45.0 && b >= g + 8.0 && b <= g + 48.0 && b >= 150.0;
    shadowed_teal || deep_shadow_blue || muted_teal || deep_blue || bright_cyan
}

pub(crate) fn is_stone_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if sky_like || !(18.0..=210.0).contains(&luma) || is_water_like(r, g, b, luma) {
        return false;
    }

    let green_foliage = g >= r + 22.0 && g >= b + 18.0;
    !green_foliage
        && (is_rock_or_shadow_like(r, g, b, luma)
            || (is_earth_like(r, g, b) && r.max(g).max(b) - r.min(g).min(b) <= 105.0))
}

pub(crate) fn is_wood_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    !sky_like
        && (20.0..=190.0).contains(&luma)
        && r >= 48.0
        && g >= 30.0
        && r >= g * 0.92
        && r >= b + 12.0
        && g >= b * 0.62
}

pub(crate) fn is_flower_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (35.0..=225.0).contains(&luma) && r >= 105.0 && r >= g + 35.0 && r >= b + 35.0 && b >= g + 16.0
}

pub(crate) fn is_cloud_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if !(72.0..=238.0).contains(&luma) {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    let blue_sky = sky_like && b >= r + 22.0 && b >= g + 10.0 && saturation >= 40.0;
    if blue_sky {
        return false;
    }

    let pale_cloud = sky_like && luma >= 118.0 && saturation <= 72.0 && b <= r + 28.0;
    let gray_bank =
        saturation <= 44.0 && r >= 68.0 && g >= 68.0 && b >= 68.0 && b + 20.0 >= r && b + 20.0 >= g;
    let warm_haze_bank = saturation <= 54.0 && r >= 86.0 && g >= 78.0 && b >= 68.0 && r + 16.0 >= b;

    pale_cloud || gray_bank || warm_haze_bank
}

pub(crate) fn is_distant_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if sky_like || !(12.0..=210.0).contains(&luma) {
        return false;
    }

    let water_like =
        r <= 115.0 && g >= 45.0 && b >= 40.0 && r <= g + 25.0 && (g >= r + 8.0 || b >= r + 8.0);
    !water_like
        && (is_foliage_like(r, g, b, luma, sky_like)
            || is_earth_like(r, g, b)
            || is_rock_or_shadow_like(r, g, b, luma))
}

pub(crate) fn is_wind_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    if !(55.0..=245.0).contains(&luma) {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    g >= 100.0
        && b >= 145.0
        && r <= 190.0
        && g >= r + 55.0
        && b >= r + 70.0
        && b + 18.0 >= g
        && saturation >= 70.0
}
