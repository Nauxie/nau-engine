use image::{ImageReader, RgbImage};
use std::{collections::HashSet, path::Path};

use super::{
    image_metrics::*,
    pixel_rules::*,
    thresholds::*,
    types::{Check, ImageAudit},
};

pub(super) fn audit_path(path: &Path) -> Result<ImageAudit, String> {
    let decoded = ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?;
    let transparent_pixel_fraction = transparent_pixel_fraction(&decoded);
    audit_image_with_alpha(
        path.to_string_lossy().into_owned(),
        decoded.to_rgb8(),
        transparent_pixel_fraction,
    )
}

#[cfg(test)]
pub(super) fn audit_image(path: String, image: RgbImage) -> Result<ImageAudit, String> {
    audit_image_with_alpha(path, image, 0.0)
}

pub(super) fn audit_image_with_alpha(
    path: String,
    image: RgbImage,
    transparent_pixel_fraction: f64,
) -> Result<ImageAudit, String> {
    let (width, height) = image.dimensions();
    let pixel_count = (width as usize).saturating_mul(height as usize);
    if pixel_count == 0 {
        return Err("image has no pixels".to_string());
    }

    let mut luma_values = Vec::with_capacity(pixel_count);
    let mut sum_luma = 0.0;
    let mut sum_luma_sq = 0.0;
    let mut sum_rg = 0.0;
    let mut sum_rg_sq = 0.0;
    let mut sum_yb = 0.0;
    let mut sum_yb_sq = 0.0;
    let mut quantized_colors = HashSet::new();
    let mut top_sky_pixels = 0usize;
    let mut top_pixels = 0usize;
    let mut lower_scene_pixels = 0usize;
    let mut lower_pixels = 0usize;
    let mut center_scene_pixels = 0usize;
    let mut center_pixels = 0usize;
    let mut player_focus_pixels = 0usize;
    let mut player_warm_focus_pixels = 0usize;
    let mut player_focus_region_pixels = 0usize;
    let mut route_marker_pixels = 0usize;
    let mut route_marker_region_pixels = 0usize;
    let mut route_marker_hue_family_pixels = [0usize; ROUTE_MARKER_HUE_FAMILY_COUNT];
    let mut route_marker_mask = vec![false; pixel_count];
    let mut distant_scene_pixels = 0usize;
    let mut distant_scene_region_pixels = 0usize;
    let mut distant_scene_color_buckets = HashSet::new();
    let mut distant_scene_mask = vec![false; pixel_count];
    let mut scene_material_family_pixels = [0usize; SCENE_MATERIAL_FAMILY_COUNT];
    let mut scene_material_pixels = 0usize;
    let mut foliage_scene_pixels = 0usize;
    let mut foliage_scene_mask = vec![false; pixel_count];
    let mut cloud_layer_pixels = 0usize;
    let mut cloud_layer_region_pixels = 0usize;
    let mut cloud_layer_mask = vec![false; pixel_count];
    let mut border_occluder_pixels = [0usize; BORDER_REGION_COUNT];
    let mut border_region_pixels = [0usize; BORDER_REGION_COUNT];
    let mut inner_border_occluder_pixels = [0usize; BORDER_REGION_COUNT];
    let mut inner_border_region_pixels = [0usize; BORDER_REGION_COUNT];
    let mut border_color_buckets =
        std::array::from_fn::<_, BORDER_REGION_COUNT, _>(|_| HashSet::<u32>::new());
    let mut foreign_canvas_pixels = 0usize;
    let mut foreign_canvas_region_pixels = 0usize;
    let mut hud_text_pixels = 0usize;
    let mut scene_mask = vec![false; pixel_count];

    let width_usize = width as usize;
    let height_usize = height as usize;
    let top_limit = height_usize / 3;
    let lower_start = height_usize / 2;
    let center_x_start = width_usize / 4;
    let center_x_end = width_usize - center_x_start;
    let center_y_start = height_usize / 3;
    let center_y_end = height_usize * 9 / 10;

    for (index, pixel) in image.pixels().enumerate() {
        let [r, g, b] = pixel.0;
        let r = r as f64;
        let g = g as f64;
        let b = b as f64;
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let rg = r - g;
        let yb = (r + g) * 0.5 - b;

        sum_luma += luma;
        sum_luma_sq += luma * luma;
        sum_rg += rg;
        sum_rg_sq += rg * rg;
        sum_yb += yb;
        sum_yb_sq += yb * yb;
        luma_values.push(luma);

        let key = ((r as u32 / 32) << 6) | ((g as u32 / 32) << 3) | (b as u32 / 32);
        quantized_colors.insert(key);

        let x = index % width_usize;
        let y = index / width_usize;
        let sky_like = is_sky_like(r, g, b, luma);
        let scene_like = is_scene_like(r, g, b, luma, sky_like);
        let hud_region = is_hud_region(x, y, width_usize, height_usize);
        let route_marker_like =
            !hud_region && y >= top_limit && is_route_marker_like(r, g, b, luma);
        let distant_scene_region = !hud_region
            && is_distant_scene_region(x, y, width_usize, height_usize)
            && !is_player_focus_region(x, y, width_usize, height_usize);
        let distant_scene_like = distant_scene_region
            && !route_marker_like
            && is_distant_scene_like(r, g, b, luma, sky_like);
        let material_region = !hud_region
            && y >= top_limit
            && y < height_usize * 9 / 10
            && !route_marker_like
            && !is_player_focus_region(x, y, width_usize, height_usize);
        let cloud_layer_region = !hud_region
            && !is_player_focus_region(x, y, width_usize, height_usize)
            && is_cloud_layer_region(x, y, width_usize, height_usize);
        scene_mask[index] = scene_like && !hud_region;

        if y < top_limit {
            top_pixels += 1;
            if sky_like {
                top_sky_pixels += 1;
            }
        }
        if y >= lower_start {
            lower_pixels += 1;
            if scene_like {
                lower_scene_pixels += 1;
            }
        }
        if x >= center_x_start && x < center_x_end && y >= center_y_start && y < center_y_end {
            center_pixels += 1;
            if scene_like {
                center_scene_pixels += 1;
            }
        }
        if is_player_focus_region(x, y, width_usize, height_usize) {
            player_focus_region_pixels += 1;
            if is_player_warm_like(r, g, b) {
                player_warm_focus_pixels += 1;
            }
            if is_player_focus_like(r, g, b, luma) {
                player_focus_pixels += 1;
            }
        }
        if route_marker_like {
            route_marker_mask[index] = true;
            route_marker_region_pixels += 1;
            route_marker_pixels += 1;
            if let Some(family) = route_marker_hue_family(r, g, b) {
                route_marker_hue_family_pixels[family] += 1;
            }
        } else if !hud_region && y >= top_limit {
            route_marker_region_pixels += 1;
        }
        if distant_scene_region {
            distant_scene_region_pixels += 1;
            if distant_scene_like {
                distant_scene_mask[index] = true;
                distant_scene_pixels += 1;
                distant_scene_color_buckets.insert(key);
            }
        }
        if material_region && let Some(family) = scene_material_family(r, g, b, luma, sky_like) {
            scene_material_pixels += 1;
            scene_material_family_pixels[family] += 1;
            if family == 1 {
                foliage_scene_pixels += 1;
                foliage_scene_mask[index] = true;
            }
        }
        if cloud_layer_region {
            cloud_layer_region_pixels += 1;
            if !route_marker_like && is_cloud_layer_like(r, g, b, luma, sky_like) {
                cloud_layer_mask[index] = true;
                cloud_layer_pixels += 1;
            }
        }
        if !hud_region {
            foreign_canvas_region_pixels += 1;
            if is_foreign_canvas_like(r, g, b, luma, sky_like) {
                foreign_canvas_pixels += 1;
            }

            let clipping_occluder = is_clipping_occluder_like(r, g, b, luma, sky_like, scene_like);
            for region in border_regions(x, y, width_usize, height_usize)
                .into_iter()
                .flatten()
            {
                border_region_pixels[region] += 1;
                border_color_buckets[region].insert(key);
                if clipping_occluder {
                    border_occluder_pixels[region] += 1;
                }
            }
            for region in inner_border_regions(x, y, width_usize, height_usize)
                .into_iter()
                .flatten()
            {
                inner_border_region_pixels[region] += 1;
                if clipping_occluder {
                    inner_border_occluder_pixels[region] += 1;
                }
            }
        }
        if hud_region && is_hud_text_like(r, g, b) {
            hud_text_pixels += 1;
        }
    }

    let count = pixel_count as f64;
    let mean_luma = sum_luma / count;
    let luma_stddev = variance(sum_luma_sq, sum_luma, count).sqrt();
    let mean_rg = sum_rg / count;
    let mean_yb = sum_yb / count;
    let std_rg = variance(sum_rg_sq, sum_rg, count).sqrt();
    let std_yb = variance(sum_yb_sq, sum_yb, count).sqrt();
    let colorfulness = (std_rg * std_rg + std_yb * std_yb).sqrt()
        + 0.3 * (mean_rg * mean_rg + mean_yb * mean_yb).sqrt();
    let edge_density = edge_density(&luma_values, width_usize, height_usize);
    let center_edge_density = edge_density_in_region(
        &luma_values,
        width_usize,
        height_usize,
        center_x_start,
        center_x_end,
        center_y_start,
        center_y_end,
    );
    let top_sky_fraction = fraction(top_sky_pixels, top_pixels);
    let lower_scene_fraction = fraction(lower_scene_pixels, lower_pixels);
    let center_scene_fraction = fraction(center_scene_pixels, center_pixels);
    let player_focus_fraction = fraction(player_focus_pixels, player_focus_region_pixels);
    let player_warm_focus_fraction = fraction(player_warm_focus_pixels, player_focus_region_pixels);
    let route_marker_fraction = fraction(route_marker_pixels, route_marker_region_pixels);
    let route_marker_component_count =
        route_marker_component_count(&route_marker_mask, width_usize, height_usize);
    let route_marker_hue_family_count = route_marker_hue_family_pixels
        .into_iter()
        .filter(|pixels| *pixels >= MIN_ROUTE_MARKER_HUE_FAMILY_PIXELS)
        .count();
    let distant_scene_fraction = fraction(distant_scene_pixels, distant_scene_region_pixels);
    let distant_scene_component_count =
        distant_scene_component_count(&distant_scene_mask, width_usize, height_usize);
    let distant_scene_color_bucket_count = distant_scene_color_buckets.len();
    let distant_scene_bounds = mask_bounds(&distant_scene_mask, width_usize, height_usize);
    let distant_scene_horizontal_span_fraction =
        horizontal_span_fraction(distant_scene_bounds, width_usize);
    let distant_scene_vertical_span_fraction =
        vertical_span_fraction(distant_scene_bounds, height_usize);
    let scene_material_family_count = scene_material_family_pixels
        .into_iter()
        .filter(|pixels| *pixels >= MIN_SCENE_MATERIAL_FAMILY_PIXELS)
        .count();
    let foliage_scene_fraction = fraction(foliage_scene_pixels, scene_material_pixels);
    let foliage_scene_tile_count = mask_tile_count(
        &foliage_scene_mask,
        width_usize,
        height_usize,
        DETAIL_TILE_COLUMNS,
        DETAIL_TILE_ROWS,
        MIN_FOLIAGE_SCENE_TILE_PIXELS,
    );
    let cloud_layer_fraction = fraction(cloud_layer_pixels, cloud_layer_region_pixels);
    let cloud_layer_component_count =
        cloud_layer_component_count(&cloud_layer_mask, width_usize, height_usize);
    let cloud_layer_bounds = mask_bounds(&cloud_layer_mask, width_usize, height_usize);
    let cloud_layer_horizontal_span_fraction =
        horizontal_span_fraction(cloud_layer_bounds, width_usize);
    let cloud_layer_vertical_span_fraction =
        vertical_span_fraction(cloud_layer_bounds, height_usize);
    let severe_clipping_fraction = severe_clipping_fraction(
        &border_occluder_pixels,
        &border_region_pixels,
        &inner_border_occluder_pixels,
        &inner_border_region_pixels,
        border_color_buckets.map(|buckets| buckets.len()),
        border_edge_densities(&luma_values, width_usize, height_usize),
    );
    let foreign_canvas_fraction = fraction(foreign_canvas_pixels, foreign_canvas_region_pixels);
    let hud_text_fraction = fraction(hud_text_pixels, pixel_count);
    let scene_detail =
        scene_detail_stats(&image, &luma_values, &scene_mask, width_usize, height_usize);
    let low_detail_scene_component =
        low_detail_scene_component_stats(&luma_values, &scene_mask, width_usize, height_usize);

    let checks = vec![
        Check::at_least("width", width as f64, MIN_WIDTH as f64, "px"),
        Check::at_least("height", height as f64, MIN_HEIGHT as f64, "px"),
        Check::at_least("mean_luma", mean_luma, MIN_MEAN_LUMA, "luma"),
        Check::at_most("mean_luma", mean_luma, MAX_MEAN_LUMA, "luma"),
        Check::at_least("luma_stddev", luma_stddev, MIN_LUMA_STDDEV, "luma"),
        Check::at_least("colorfulness", colorfulness, MIN_COLORFULNESS, "score"),
        Check::at_least(
            "quantized_colors",
            quantized_colors.len() as f64,
            MIN_QUANTIZED_COLORS as f64,
            "buckets",
        ),
        Check::at_least("edge_density", edge_density, MIN_EDGE_DENSITY, "ratio"),
        Check::at_least(
            "lower_scene_fraction",
            lower_scene_fraction,
            MIN_LOWER_SCENE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "center_scene_fraction",
            center_scene_fraction,
            MIN_CENTER_SCENE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "center_edge_density",
            center_edge_density,
            MIN_CENTER_EDGE_DENSITY,
            "ratio",
        ),
        Check::at_least(
            "scene_detail_tile_fraction",
            scene_detail.detail_tile_fraction,
            MIN_SCENE_DETAIL_TILE_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "scene_candidate_tile_count",
            scene_detail.candidate_tile_count as f64,
            MIN_SCENE_CANDIDATE_TILE_COUNT as f64,
            "tiles",
        ),
        Check::at_most(
            "flat_scene_tile_fraction",
            scene_detail.flat_tile_fraction,
            MAX_FLAT_SCENE_TILE_FRACTION,
            "ratio",
        ),
        Check::at_most(
            "dominant_low_detail_scene_component_fraction",
            low_detail_scene_component.dominant_component_fraction,
            MAX_DOMINANT_LOW_DETAIL_SCENE_COMPONENT_FRACTION,
            "ratio",
        ),
        Check::at_least(
            "player_focus_fraction",
            player_focus_fraction,
            MIN_PLAYER_FOCUS_FRACTION,
            "ratio",
        ),
        Check::at_most(
            "severe_clipping_fraction",
            severe_clipping_fraction,
            MAX_SEVERE_CLIPPING_FRACTION,
            "ratio",
        ),
        Check::at_most(
            "transparent_pixel_fraction",
            transparent_pixel_fraction,
            MAX_TRANSPARENT_PIXEL_FRACTION,
            "ratio",
        ),
        Check::at_most(
            "foreign_canvas_fraction",
            foreign_canvas_fraction,
            MAX_FOREIGN_CANVAS_FRACTION,
            "ratio",
        ),
        Check::at_most(
            "hud_text_fraction",
            hud_text_fraction,
            MAX_HUD_TEXT_FRACTION,
            "ratio",
        ),
    ];
    let passed = checks.iter().all(|check| check.passed);

    Ok(ImageAudit {
        path,
        width,
        height,
        mean_luma,
        luma_stddev,
        colorfulness,
        quantized_colors: quantized_colors.len(),
        edge_density,
        top_sky_fraction,
        lower_scene_fraction,
        center_scene_fraction,
        center_edge_density,
        scene_detail_tile_fraction: scene_detail.detail_tile_fraction,
        flat_scene_tile_fraction: scene_detail.flat_tile_fraction,
        dominant_low_detail_scene_component_fraction: low_detail_scene_component
            .dominant_component_fraction,
        scene_detail_tile_count: scene_detail.detail_tile_count,
        flat_scene_tile_count: scene_detail.flat_tile_count,
        scene_candidate_tile_count: scene_detail.candidate_tile_count,
        player_focus_fraction,
        player_warm_focus_fraction,
        route_marker_fraction,
        route_marker_component_count,
        route_marker_hue_family_count,
        distant_scene_fraction,
        distant_scene_component_count,
        distant_scene_color_bucket_count,
        distant_scene_horizontal_span_fraction,
        distant_scene_vertical_span_fraction,
        scene_material_family_count,
        foliage_scene_fraction,
        foliage_scene_tile_count,
        cloud_layer_fraction,
        cloud_layer_component_count,
        cloud_layer_horizontal_span_fraction,
        cloud_layer_vertical_span_fraction,
        severe_clipping_fraction,
        transparent_pixel_fraction,
        foreign_canvas_fraction,
        hud_text_fraction,
        passed,
        checks,
    })
}

fn mask_bounds(mask: &[bool], width: usize, height: usize) -> Option<(usize, usize, usize, usize)> {
    if width == 0 || height == 0 || mask.len() != width.saturating_mul(height) {
        return None;
    }

    let mut min_x = width;
    let mut max_x = 0usize;
    let mut min_y = height;
    let mut max_y = 0usize;
    let mut found = false;
    for (index, present) in mask.iter().enumerate() {
        if !present {
            continue;
        }
        let x = index % width;
        let y = index / width;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        found = true;
    }

    found.then_some((min_x, max_x, min_y, max_y))
}

fn horizontal_span_fraction(bounds: Option<(usize, usize, usize, usize)>, width: usize) -> f64 {
    bounds
        .map(|(min_x, max_x, _, _)| fraction(max_x.saturating_sub(min_x) + 1, width))
        .unwrap_or(0.0)
}

fn vertical_span_fraction(bounds: Option<(usize, usize, usize, usize)>, height: usize) -> f64 {
    bounds
        .map(|(_, _, min_y, max_y)| fraction(max_y.saturating_sub(min_y) + 1, height))
        .unwrap_or(0.0)
}

fn mask_tile_count(
    mask: &[bool],
    width: usize,
    height: usize,
    columns: usize,
    rows: usize,
    min_pixels_per_tile: usize,
) -> usize {
    if width == 0
        || height == 0
        || columns == 0
        || rows == 0
        || mask.len() != width.saturating_mul(height)
    {
        return 0;
    }

    let mut tile_count = 0usize;
    for row in 0..rows {
        let y_start = row * height / rows;
        let y_end = (row + 1) * height / rows;
        for column in 0..columns {
            let x_start = column * width / columns;
            let x_end = (column + 1) * width / columns;
            let mut tile_pixels = 0usize;
            for y in y_start..y_end {
                let row_start = y * width;
                for x in x_start..x_end {
                    if mask[row_start + x] {
                        tile_pixels += 1;
                    }
                }
            }
            if tile_pixels >= min_pixels_per_tile {
                tile_count += 1;
            }
        }
    }

    tile_count
}
