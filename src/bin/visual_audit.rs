use image::{ImageReader, RgbImage};
use std::{
    collections::HashSet,
    env,
    fmt::Write as _,
    path::{Path, PathBuf},
    process,
};

const MIN_WIDTH: u32 = 640;
const MIN_HEIGHT: u32 = 360;
const MIN_MEAN_LUMA: f64 = 8.0;
const MAX_MEAN_LUMA: f64 = 247.0;
const MIN_LUMA_STDDEV: f64 = 4.5;
const MIN_COLORFULNESS: f64 = 6.0;
const MIN_QUANTIZED_COLORS: usize = 24;
const MIN_EDGE_DENSITY: f64 = 0.0025;
const MIN_SEQUENCE_TOP_SKY_FRACTION: f64 = 0.25;
const MIN_LOWER_SCENE_FRACTION: f64 = 0.25;
const MIN_CENTER_SCENE_FRACTION: f64 = 0.18;
const MIN_CENTER_EDGE_DENSITY: f64 = 0.02;
const DETAIL_TILE_COLUMNS: usize = 12;
const DETAIL_TILE_ROWS: usize = 8;
const MIN_SCENE_DETAIL_TILE_FRACTION: f64 = 0.30;
const MAX_FLAT_SCENE_TILE_FRACTION: f64 = 0.35;
const MIN_SCENE_CANDIDATE_TILE_COUNT: usize = 12;
const MIN_SCENE_TILE_PIXELS: usize = 24;
const MIN_SCENE_TILE_FRACTION: f64 = 0.18;
const MIN_DETAIL_TILE_COLOR_BUCKETS: usize = 5;
const MIN_DETAIL_TILE_LUMA_STDDEV: f64 = 5.0;
const MIN_DETAIL_TILE_EDGE_DENSITY: f64 = 0.018;
const MAX_FLAT_TILE_COLOR_BUCKETS: usize = 3;
const MAX_FLAT_TILE_LUMA_STDDEV: f64 = 4.0;
const MAX_FLAT_TILE_EDGE_DENSITY: f64 = 0.008;
const MIN_PLAYER_FOCUS_FRACTION: f64 = 0.0015;
const MIN_SEQUENCE_ROUTE_MARKER_FRACTION: f64 = 0.00008;
const MIN_SEQUENCE_ROUTE_MARKER_COMPONENTS: usize = 2;
const MIN_SEQUENCE_ROUTE_MARKER_HUE_FAMILIES: usize = 2;
const MIN_ROUTE_MARKER_COMPONENT_PIXELS: usize = 3;
const MIN_ROUTE_MARKER_HUE_FAMILY_PIXELS: usize = 8;
const MIN_SEQUENCE_DISTANT_SCENE_FRACTION: f64 = 0.004;
const MIN_SEQUENCE_DISTANT_SCENE_COMPONENTS: usize = 2;
const MIN_SEQUENCE_DISTANT_SCENE_COLOR_BUCKETS: usize = 6;
const MIN_SEQUENCE_SCENE_MATERIAL_FAMILIES: usize = 3;
const MIN_SCENE_MATERIAL_FAMILY_PIXELS: usize = 180;
const MIN_SEQUENCE_FOLIAGE_SCENE_FRACTION: f64 = 0.08;
const MIN_SEQUENCE_CLOUD_LAYER_FRACTION: f64 = 0.015;
const MIN_SEQUENCE_CLOUD_LAYER_COMPONENTS: usize = 2;
const MIN_CLOUD_LAYER_COMPONENT_PIXELS: usize = 36;
const MIN_CLOUD_LAYER_COMPONENT_WIDTH: usize = 8;
const MIN_CLOUD_LAYER_COMPONENT_HEIGHT: usize = 3;
const MIN_DISTANT_SCENE_COMPONENT_PIXELS: usize = 28;
const MIN_DISTANT_SCENE_COMPONENT_WIDTH: usize = 10;
const MIN_DISTANT_SCENE_COMPONENT_HEIGHT: usize = 3;
const MIN_DISTANT_SCENE_COMPONENT_ASPECT: f64 = 1.15;
const MAX_DISTANT_SCENE_COMPONENT_WIDTH_FRACTION: f64 = 0.58;
const MAX_DISTANT_SCENE_COMPONENT_HEIGHT_FRACTION: f64 = 0.34;
const MAX_TRANSPARENT_PIXEL_FRACTION: f64 = 0.0;
const MAX_FOREIGN_CANVAS_FRACTION: f64 = 0.08;
const MAX_HUD_TEXT_FRACTION: f64 = 0.06;
const MAX_SEVERE_CLIPPING_FRACTION: f64 = 0.82;
const MAX_CLIPPING_BORDER_EDGE_DENSITY: f64 = 0.08;
const MAX_CLIPPING_BORDER_COLOR_BUCKETS: usize = 12;

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin visual_audit -- <png> [<png> ...]");
        process::exit(2);
    }

    let mut audits = Vec::with_capacity(paths.len());
    for path in &paths {
        match audit_path(path) {
            Ok(audit) => audits.push(audit),
            Err(error) => {
                eprintln!("failed to audit {}: {error}", path.display());
                process::exit(2);
            }
        }
    }

    let report_checks = report_checks(&audits);
    let passed = report_passed(&audits, &report_checks);
    println!("{}", audit_report_json(passed, &report_checks, &audits));
    if !passed {
        process::exit(1);
    }
}

fn audit_path(path: &Path) -> Result<ImageAudit, String> {
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
fn audit_image(path: String, image: RgbImage) -> Result<ImageAudit, String> {
    audit_image_with_alpha(path, image, 0.0)
}

fn audit_image_with_alpha(
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
    let scene_material_family_count = scene_material_family_pixels
        .into_iter()
        .filter(|pixels| *pixels >= MIN_SCENE_MATERIAL_FAMILY_PIXELS)
        .count();
    let foliage_scene_fraction = fraction(foliage_scene_pixels, scene_material_pixels);
    let cloud_layer_fraction = fraction(cloud_layer_pixels, cloud_layer_region_pixels);
    let cloud_layer_component_count =
        cloud_layer_component_count(&cloud_layer_mask, width_usize, height_usize);
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
        scene_material_family_count,
        foliage_scene_fraction,
        cloud_layer_fraction,
        cloud_layer_component_count,
        severe_clipping_fraction,
        transparent_pixel_fraction,
        foreign_canvas_fraction,
        hud_text_fraction,
        passed,
        checks,
    })
}

fn transparent_pixel_fraction(image: &image::DynamicImage) -> f64 {
    let rgba = image.to_rgba8();
    let pixel_count = rgba.width() as usize * rgba.height() as usize;
    let transparent_pixels = rgba.pixels().filter(|pixel| pixel.0[3] < u8::MAX).count();
    fraction(transparent_pixels, pixel_count)
}

fn fraction(value: usize, total: usize) -> f64 {
    value as f64 / total.max(1) as f64
}

fn variance(sum_sq: f64, sum: f64, count: f64) -> f64 {
    (sum_sq / count - (sum / count).powi(2)).max(0.0)
}

fn edge_density(luma_values: &[f64], width: usize, height: usize) -> f64 {
    edge_density_in_region(luma_values, width, height, 1, width, 1, height)
}

fn edge_density_in_region(
    luma_values: &[f64],
    width: usize,
    height: usize,
    x_start: usize,
    x_end: usize,
    y_start: usize,
    y_end: usize,
) -> f64 {
    if width < 2 || height < 2 {
        return 0.0;
    }

    let x_start = x_start.clamp(1, width - 1);
    let x_end = x_end.clamp(x_start + 1, width);
    let y_start = y_start.clamp(1, height - 1);
    let y_end = y_end.clamp(y_start + 1, height);
    let mut edge_pixels = 0usize;
    let mut sampled_pixels = 0usize;
    for y in (y_start..y_end).step_by(2) {
        for x in (x_start..x_end).step_by(2) {
            let index = y * width + x;
            let current = luma_values[index];
            let left = luma_values[index - 1];
            let up = luma_values[index - width];
            let gradient = (current - left).abs() + (current - up).abs();
            if gradient > 18.0 {
                edge_pixels += 1;
            }
            sampled_pixels += 1;
        }
    }

    edge_pixels as f64 / sampled_pixels.max(1) as f64
}

#[derive(Clone, Copy, Debug, Default)]
struct SceneDetailStats {
    detail_tile_fraction: f64,
    flat_tile_fraction: f64,
    detail_tile_count: usize,
    flat_tile_count: usize,
    candidate_tile_count: usize,
}

fn scene_detail_stats(
    image: &RgbImage,
    luma_values: &[f64],
    scene_mask: &[bool],
    width: usize,
    height: usize,
) -> SceneDetailStats {
    if width == 0 || height == 0 || scene_mask.len() != width.saturating_mul(height) {
        return SceneDetailStats::default();
    }

    let y_min = height / 3;
    let y_max = height * 9 / 10;
    let y_span = y_max.saturating_sub(y_min).max(1);
    let mut stats = SceneDetailStats::default();

    for row in 0..DETAIL_TILE_ROWS {
        let y_start = y_min + row * y_span / DETAIL_TILE_ROWS;
        let y_end = y_min + (row + 1) * y_span / DETAIL_TILE_ROWS;
        for column in 0..DETAIL_TILE_COLUMNS {
            let x_start = column * width / DETAIL_TILE_COLUMNS;
            let x_end = ((column + 1) * width / DETAIL_TILE_COLUMNS).min(width);
            let mut non_hud_pixels = 0usize;
            let mut scene_pixels = 0usize;
            let mut sum_luma = 0.0;
            let mut sum_luma_sq = 0.0;
            let mut color_buckets = HashSet::new();
            let mut edge_pixels = 0usize;
            let mut edge_samples = 0usize;

            for y in y_start..y_end {
                for x in x_start..x_end {
                    if is_hud_region(x, y, width, height) {
                        continue;
                    }
                    non_hud_pixels += 1;
                    let index = y * width + x;
                    if !scene_mask[index] {
                        continue;
                    }

                    scene_pixels += 1;
                    let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
                    let color_key =
                        ((r as u32 / 32) << 6) | ((g as u32 / 32) << 3) | (b as u32 / 32);
                    color_buckets.insert(color_key);
                    let luma = luma_values[index];
                    sum_luma += luma;
                    sum_luma_sq += luma * luma;

                    if x > x_start && y > y_start {
                        let gradient = (luma - luma_values[index - 1]).abs()
                            + (luma - luma_values[index - width]).abs();
                        if gradient > 18.0 {
                            edge_pixels += 1;
                        }
                        edge_samples += 1;
                    }
                }
            }

            let scene_fraction = fraction(scene_pixels, non_hud_pixels);
            if scene_pixels < MIN_SCENE_TILE_PIXELS || scene_fraction < MIN_SCENE_TILE_FRACTION {
                continue;
            }

            stats.candidate_tile_count += 1;
            let count = scene_pixels as f64;
            let luma_stddev = variance(sum_luma_sq, sum_luma, count).sqrt();
            let edge_density = fraction(edge_pixels, edge_samples);
            let detailed = color_buckets.len() >= MIN_DETAIL_TILE_COLOR_BUCKETS
                && (luma_stddev >= MIN_DETAIL_TILE_LUMA_STDDEV
                    || edge_density >= MIN_DETAIL_TILE_EDGE_DENSITY);
            if detailed {
                stats.detail_tile_count += 1;
            }

            let flat = scene_fraction >= 0.45
                && color_buckets.len() <= MAX_FLAT_TILE_COLOR_BUCKETS
                && luma_stddev <= MAX_FLAT_TILE_LUMA_STDDEV
                && edge_density <= MAX_FLAT_TILE_EDGE_DENSITY;
            if flat {
                stats.flat_tile_count += 1;
            }
        }
    }

    stats.detail_tile_fraction = fraction(stats.detail_tile_count, stats.candidate_tile_count);
    stats.flat_tile_fraction = fraction(stats.flat_tile_count, stats.candidate_tile_count);
    stats
}

const BORDER_REGION_COUNT: usize = 4;
const ROUTE_MARKER_HUE_FAMILY_COUNT: usize = 4;
const SCENE_MATERIAL_FAMILY_COUNT: usize = 4;

fn border_regions(x: usize, y: usize, width: usize, height: usize) -> [Option<usize>; 4] {
    let x_band = (width * 8 / 100).max(1);
    let y_band = (height * 8 / 100).max(1);
    [
        (y < y_band).then_some(0),
        (y >= height.saturating_sub(y_band)).then_some(1),
        (x < x_band).then_some(2),
        (x >= width.saturating_sub(x_band)).then_some(3),
    ]
}

fn inner_border_regions(x: usize, y: usize, width: usize, height: usize) -> [Option<usize>; 4] {
    let x_band = (width * 8 / 100).max(1);
    let y_band = (height * 8 / 100).max(1);
    [
        (y >= y_band && y < y_band * 2).then_some(0),
        (y >= height.saturating_sub(y_band * 2) && y < height.saturating_sub(y_band)).then_some(1),
        (x >= x_band && x < x_band * 2).then_some(2),
        (x >= width.saturating_sub(x_band * 2) && x < width.saturating_sub(x_band)).then_some(3),
    ]
}

fn severe_clipping_fraction(
    values: &[usize; BORDER_REGION_COUNT],
    totals: &[usize; BORDER_REGION_COUNT],
    inner_values: &[usize; BORDER_REGION_COUNT],
    inner_totals: &[usize; BORDER_REGION_COUNT],
    color_bucket_counts: [usize; BORDER_REGION_COUNT],
    edge_densities: [f64; BORDER_REGION_COUNT],
) -> f64 {
    values
        .iter()
        .zip(totals)
        .zip(color_bucket_counts)
        .zip(edge_densities)
        .enumerate()
        .filter(|(_, ((value_total, color_bucket_count), edge_density))| {
            let (_, total) = value_total;
            **total > 0
                && *color_bucket_count <= MAX_CLIPPING_BORDER_COLOR_BUCKETS
                && *edge_density <= MAX_CLIPPING_BORDER_EDGE_DENSITY
        })
        .map(|(region, (((value, total), _), _))| {
            let border_fraction = fraction(*value, *total);
            let inner_continuation_fraction = fraction(inner_values[region], inner_totals[region]);
            border_fraction * (1.0 - inner_continuation_fraction)
        })
        .fold(0.0, f64::max)
}

fn border_edge_densities(
    luma_values: &[f64],
    width: usize,
    height: usize,
) -> [f64; BORDER_REGION_COUNT] {
    let x_band = (width * 8 / 100).max(1);
    let y_band = (height * 8 / 100).max(1);
    [
        edge_density_in_region(
            luma_values,
            width,
            height,
            width * 36 / 100,
            width,
            1,
            y_band,
        ),
        edge_density_in_region(
            luma_values,
            width,
            height,
            1,
            width,
            height.saturating_sub(y_band),
            height,
        ),
        edge_density_in_region(
            luma_values,
            width,
            height,
            1,
            x_band,
            height * 88 / 100,
            height,
        ),
        edge_density_in_region(
            luma_values,
            width,
            height,
            width.saturating_sub(x_band),
            width,
            1,
            height,
        ),
    ]
}

fn route_marker_component_count(mask: &[bool], width: usize, height: usize) -> usize {
    if width == 0 || height == 0 || mask.len() != width.saturating_mul(height) {
        return 0;
    }

    let mut visited = vec![false; mask.len()];
    let mut stack = Vec::new();
    let mut components = 0usize;
    for index in 0..mask.len() {
        if visited[index] || !mask[index] {
            continue;
        }

        let mut pixel_count = 0usize;
        visited[index] = true;
        stack.push(index);
        while let Some(current) = stack.pop() {
            pixel_count += 1;
            let x = current % width;
            let y = current / width;

            if x > 0 {
                push_marker_neighbor(current - 1, mask, &mut visited, &mut stack);
            }
            if x + 1 < width {
                push_marker_neighbor(current + 1, mask, &mut visited, &mut stack);
            }
            if y > 0 {
                push_marker_neighbor(current - width, mask, &mut visited, &mut stack);
            }
            if y + 1 < height {
                push_marker_neighbor(current + width, mask, &mut visited, &mut stack);
            }
        }

        if pixel_count >= MIN_ROUTE_MARKER_COMPONENT_PIXELS {
            components += 1;
        }
    }

    components
}

fn distant_scene_component_count(mask: &[bool], width: usize, height: usize) -> usize {
    if width == 0 || height == 0 || mask.len() != width.saturating_mul(height) {
        return 0;
    }

    let mut visited = vec![false; mask.len()];
    let mut stack = Vec::new();
    let mut components = 0usize;
    for index in 0..mask.len() {
        if visited[index] || !mask[index] {
            continue;
        }

        let mut pixel_count = 0usize;
        let mut min_x = width;
        let mut max_x = 0usize;
        let mut min_y = height;
        let mut max_y = 0usize;
        visited[index] = true;
        stack.push(index);
        while let Some(current) = stack.pop() {
            pixel_count += 1;
            let x = current % width;
            let y = current / width;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);

            if x > 0 {
                push_marker_neighbor(current - 1, mask, &mut visited, &mut stack);
            }
            if x + 1 < width {
                push_marker_neighbor(current + 1, mask, &mut visited, &mut stack);
            }
            if y > 0 {
                push_marker_neighbor(current - width, mask, &mut visited, &mut stack);
            }
            if y + 1 < height {
                push_marker_neighbor(current + width, mask, &mut visited, &mut stack);
            }
        }

        let component_width = max_x.saturating_sub(min_x) + 1;
        let component_height = max_y.saturating_sub(min_y) + 1;
        let aspect = component_width as f64 / component_height.max(1) as f64;
        let max_width = (width as f64 * MAX_DISTANT_SCENE_COMPONENT_WIDTH_FRACTION) as usize;
        let max_height = (height as f64 * MAX_DISTANT_SCENE_COMPONENT_HEIGHT_FRACTION) as usize;
        let readable_component = pixel_count >= MIN_DISTANT_SCENE_COMPONENT_PIXELS
            && component_width >= MIN_DISTANT_SCENE_COMPONENT_WIDTH
            && component_height >= MIN_DISTANT_SCENE_COMPONENT_HEIGHT
            && component_width <= max_width.max(MIN_DISTANT_SCENE_COMPONENT_WIDTH)
            && component_height <= max_height.max(MIN_DISTANT_SCENE_COMPONENT_HEIGHT)
            && aspect >= MIN_DISTANT_SCENE_COMPONENT_ASPECT;

        if readable_component {
            components += 1;
        }
    }

    components
}

fn cloud_layer_component_count(mask: &[bool], width: usize, height: usize) -> usize {
    if width == 0 || height == 0 || mask.len() != width.saturating_mul(height) {
        return 0;
    }

    let mut visited = vec![false; mask.len()];
    let mut stack = Vec::new();
    let mut components = 0usize;
    for index in 0..mask.len() {
        if visited[index] || !mask[index] {
            continue;
        }

        let mut pixel_count = 0usize;
        let mut min_x = width;
        let mut max_x = 0usize;
        let mut min_y = height;
        let mut max_y = 0usize;
        visited[index] = true;
        stack.push(index);
        while let Some(current) = stack.pop() {
            pixel_count += 1;
            let x = current % width;
            let y = current / width;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);

            if x > 0 {
                push_marker_neighbor(current - 1, mask, &mut visited, &mut stack);
            }
            if x + 1 < width {
                push_marker_neighbor(current + 1, mask, &mut visited, &mut stack);
            }
            if y > 0 {
                push_marker_neighbor(current - width, mask, &mut visited, &mut stack);
            }
            if y + 1 < height {
                push_marker_neighbor(current + width, mask, &mut visited, &mut stack);
            }
        }

        let component_width = max_x.saturating_sub(min_x) + 1;
        let component_height = max_y.saturating_sub(min_y) + 1;
        let readable_component = pixel_count >= MIN_CLOUD_LAYER_COMPONENT_PIXELS
            && component_width >= MIN_CLOUD_LAYER_COMPONENT_WIDTH
            && component_height >= MIN_CLOUD_LAYER_COMPONENT_HEIGHT;

        if readable_component {
            components += 1;
        }
    }

    components
}

fn push_marker_neighbor(index: usize, mask: &[bool], visited: &mut [bool], stack: &mut Vec<usize>) {
    if !visited[index] && mask[index] {
        visited[index] = true;
        stack.push(index);
    }
}

fn is_sky_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let blue_haze = b >= 105.0 && g >= 95.0 && b >= r + 8.0 && luma >= 80.0;
    let pale_cloud_haze = r >= 130.0 && g >= 140.0 && b >= 145.0 && b >= r - 4.0 && g >= r - 12.0;
    blue_haze || pale_cloud_haze
}

fn is_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
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

    let foliage = g >= 60.0 && g >= r * 0.75 && g >= b * 0.65;
    let earth = r >= 55.0 && g >= 40.0 && r >= b + 10.0 && g >= b * 0.75;
    let rock_or_shadow = (18.0..=150.0).contains(&luma) && (r - g).abs() <= 45.0;

    foliage || earth || rock_or_shadow
}

fn scene_material_family(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> Option<usize> {
    if sky_like || luma <= 8.0 || luma >= 235.0 {
        return None;
    }

    let water = luma <= 170.0
        && r <= 115.0
        && g >= 45.0
        && b >= 40.0
        && r <= g + 25.0
        && (g >= r + 8.0 || b >= r + 8.0);
    if water {
        return Some(0);
    }

    let foliage = g >= 60.0 && g >= r * 0.75 && g >= b * 0.65;
    if foliage {
        return Some(1);
    }

    let earth = r >= 55.0 && g >= 40.0 && r >= b + 10.0 && g >= b * 0.75;
    if earth {
        return Some(2);
    }

    let rock_or_shadow = (18.0..=150.0).contains(&luma) && (r - g).abs() <= 45.0;
    rock_or_shadow.then_some(3)
}

fn is_hud_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let left_panel = x < width * 36 / 100 && y < height * 88 / 100;
    let bottom_help = y >= height * 88 / 100;
    left_panel || bottom_help
}

fn is_player_focus_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let center_x = width / 2;
    let x_radius = width * 18 / 100;
    let y_start = height * 36 / 100;
    let y_end = height * 82 / 100;
    x >= center_x.saturating_sub(x_radius) && x <= center_x + x_radius && y >= y_start && y <= y_end
}

fn is_player_focus_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let dark_body = (10.0..=85.0).contains(&luma) && r <= 95.0 && g <= 95.0 && b <= 105.0;
    is_player_warm_like(r, g, b) || dark_body
}

fn is_player_warm_like(r: f64, g: f64, b: f64) -> bool {
    r >= 115.0 && (35.0..=125.0).contains(&g) && b <= 95.0 && r >= g + 35.0
}

fn is_distant_scene_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let y_start = height * 16 / 100;
    let y_end = height * 64 / 100;
    let x_margin = width * 4 / 100;
    x >= x_margin && x < width.saturating_sub(x_margin) && y >= y_start && y < y_end
}

fn is_cloud_layer_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let y_start = height * 4 / 100;
    let y_end = height * 55 / 100;
    let x_margin = width * 4 / 100;
    x >= x_margin && x < width.saturating_sub(x_margin) && y >= y_start && y < y_end
}

fn is_cloud_layer_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
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

fn is_distant_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if sky_like || !(12.0..=210.0).contains(&luma) {
        return false;
    }

    let water_like =
        r <= 115.0 && g >= 45.0 && b >= 40.0 && r <= g + 25.0 && (g >= r + 8.0 || b >= r + 8.0);
    if water_like {
        return false;
    }

    let foliage = g >= 58.0 && g >= r * 0.72 && g >= b * 0.58;
    let earth = r >= 50.0 && g >= 38.0 && r >= b + 8.0 && g >= b * 0.68;
    let rock_or_shadow =
        (18.0..=155.0).contains(&luma) && (r - g).abs() <= 50.0 && b <= r.max(g) + 20.0;

    foliage || earth || rock_or_shadow
}

fn is_route_marker_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    if luma < 90.0 {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    max_channel >= 190.0
        && saturation >= 90.0
        && (r >= g + 60.0 || g >= r + 60.0 || b >= g + 50.0 || b >= r + 50.0)
}

fn route_marker_hue_family(r: f64, g: f64, b: f64) -> Option<usize> {
    if b >= r + 45.0 && b >= g + 18.0 {
        Some(0)
    } else if g >= r + 50.0 && g >= b + 8.0 {
        Some(1)
    } else if r >= 180.0 && g >= 130.0 && b <= 130.0 {
        Some(2)
    } else if r >= g + 45.0 && b >= g + 25.0 {
        Some(3)
    } else {
        None
    }
}

fn is_clipping_occluder_like(
    r: f64,
    g: f64,
    b: f64,
    luma: f64,
    sky_like: bool,
    scene_like: bool,
) -> bool {
    if sky_like || is_route_marker_like(r, g, b, luma) || luma <= 8.0 || luma >= 230.0 {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let dark_close_surface = luma <= 70.0 && max_channel - min_channel <= 70.0;
    let earth_or_rock = r >= 48.0 && g >= 36.0 && r >= b + 8.0 && g >= b * 0.7 && luma <= 180.0;
    let foliage = g >= 55.0 && g >= r * 0.75 && g >= b * 0.65 && luma <= 170.0;

    scene_like && (dark_close_surface || earth_or_rock || foliage)
}

fn is_hud_text_like(r: f64, g: f64, b: f64) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    max_channel >= 220.0 && max_channel - min_channel <= 24.0
}

fn is_foreign_canvas_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    !sky_like && luma >= 210.0 && max_channel - min_channel <= 36.0
}

#[derive(Debug)]
struct ImageAudit {
    path: String,
    width: u32,
    height: u32,
    mean_luma: f64,
    luma_stddev: f64,
    colorfulness: f64,
    quantized_colors: usize,
    edge_density: f64,
    top_sky_fraction: f64,
    lower_scene_fraction: f64,
    center_scene_fraction: f64,
    center_edge_density: f64,
    scene_detail_tile_fraction: f64,
    flat_scene_tile_fraction: f64,
    scene_detail_tile_count: usize,
    flat_scene_tile_count: usize,
    scene_candidate_tile_count: usize,
    player_focus_fraction: f64,
    player_warm_focus_fraction: f64,
    route_marker_fraction: f64,
    route_marker_component_count: usize,
    route_marker_hue_family_count: usize,
    distant_scene_fraction: f64,
    distant_scene_component_count: usize,
    distant_scene_color_bucket_count: usize,
    scene_material_family_count: usize,
    foliage_scene_fraction: f64,
    cloud_layer_fraction: f64,
    cloud_layer_component_count: usize,
    severe_clipping_fraction: f64,
    transparent_pixel_fraction: f64,
    foreign_canvas_fraction: f64,
    hud_text_fraction: f64,
    passed: bool,
    checks: Vec<Check>,
}

#[derive(Debug)]
struct Check {
    name: &'static str,
    passed: bool,
    value: f64,
    comparator: &'static str,
    threshold: f64,
    unit: &'static str,
}

impl Check {
    fn at_least(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }

    fn at_most(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Self {
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

fn report_checks(audits: &[ImageAudit]) -> Vec<Check> {
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
    let max_scene_material_family_count = audits
        .iter()
        .map(|audit| audit.scene_material_family_count)
        .max()
        .unwrap_or_default();
    let max_foliage_scene_fraction = audits
        .iter()
        .map(|audit| audit.foliage_scene_fraction)
        .fold(0.0, f64::max);
    let max_cloud_layer_fraction = audits
        .iter()
        .map(|audit| audit.cloud_layer_fraction)
        .fold(0.0, f64::max);
    let max_cloud_layer_component_count = audits
        .iter()
        .map(|audit| audit.cloud_layer_component_count)
        .max()
        .unwrap_or_default();

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
            "max_scene_material_family_count",
            max_scene_material_family_count as f64,
            MIN_SEQUENCE_SCENE_MATERIAL_FAMILIES as f64,
            "families",
        ),
        Check::at_least(
            "max_foliage_scene_fraction",
            max_foliage_scene_fraction,
            MIN_SEQUENCE_FOLIAGE_SCENE_FRACTION,
            "ratio",
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
    ]
}

fn report_passed(audits: &[ImageAudit], report_checks: &[Check]) -> bool {
    audits.iter().all(|audit| audit.passed) && report_checks.iter().all(|check| check.passed)
}

fn audit_report_json(passed: bool, report_checks: &[Check], audits: &[ImageAudit]) -> String {
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
        "{{\n      \"path\": {},\n      \"passed\": {},\n      \"width\": {},\n      \"height\": {},\n      \"mean_luma\": {},\n      \"luma_stddev\": {},\n      \"colorfulness\": {},\n      \"quantized_colors\": {},\n      \"edge_density\": {},\n      \"top_sky_fraction\": {},\n      \"lower_scene_fraction\": {},\n      \"center_scene_fraction\": {},\n      \"center_edge_density\": {},\n      \"scene_detail_tile_fraction\": {},\n      \"flat_scene_tile_fraction\": {},\n      \"scene_detail_tile_count\": {},\n      \"flat_scene_tile_count\": {},\n      \"scene_candidate_tile_count\": {},\n      \"player_focus_fraction\": {},\n      \"player_warm_focus_fraction\": {},\n      \"route_marker_fraction\": {},\n      \"route_marker_component_count\": {},\n      \"route_marker_hue_family_count\": {},\n      \"distant_scene_fraction\": {},\n      \"distant_scene_component_count\": {},\n      \"distant_scene_color_bucket_count\": {},\n      \"scene_material_family_count\": {},\n      \"foliage_scene_fraction\": {},\n      \"cloud_layer_fraction\": {},\n      \"cloud_layer_component_count\": {},\n      \"severe_clipping_fraction\": {},\n      \"transparent_pixel_fraction\": {},\n      \"foreign_canvas_fraction\": {},\n      \"hud_text_fraction\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
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
        audit.scene_material_family_count,
        json_number(audit.foliage_scene_fraction),
        json_number(audit.cloud_layer_fraction),
        audit.cloud_layer_component_count,
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

fn json_string(value: &str) -> String {
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

#[cfg(test)]
#[path = "visual_audit/tests.rs"]
mod tests;
