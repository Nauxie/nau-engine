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
const MIN_SEQUENCE_ROUTE_MARKER_HUE_FAMILIES: usize = 1;
const MIN_ROUTE_MARKER_COMPONENT_PIXELS: usize = 3;
const MIN_ROUTE_MARKER_HUE_FAMILY_PIXELS: usize = 8;
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
        "{{\n      \"path\": {},\n      \"passed\": {},\n      \"width\": {},\n      \"height\": {},\n      \"mean_luma\": {},\n      \"luma_stddev\": {},\n      \"colorfulness\": {},\n      \"quantized_colors\": {},\n      \"edge_density\": {},\n      \"top_sky_fraction\": {},\n      \"lower_scene_fraction\": {},\n      \"center_scene_fraction\": {},\n      \"center_edge_density\": {},\n      \"scene_detail_tile_fraction\": {},\n      \"flat_scene_tile_fraction\": {},\n      \"scene_detail_tile_count\": {},\n      \"flat_scene_tile_count\": {},\n      \"scene_candidate_tile_count\": {},\n      \"player_focus_fraction\": {},\n      \"player_warm_focus_fraction\": {},\n      \"route_marker_fraction\": {},\n      \"route_marker_component_count\": {},\n      \"route_marker_hue_family_count\": {},\n      \"severe_clipping_fraction\": {},\n      \"transparent_pixel_fraction\": {},\n      \"foreign_canvas_fraction\": {},\n      \"hud_text_fraction\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
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
mod tests {
    use super::*;
    use image::Rgb;

    fn paint_readability_signals(image: &mut RgbImage) {
        let width = image.width();
        let height = image.height();
        let center_x = width / 2;

        for y in height * 54 / 100..height * 72 / 100 {
            for x in center_x - width / 80..=center_x + width / 80 {
                image.put_pixel(x, y, Rgb([24, 28, 26]));
            }
        }
        for y in height * 48 / 100..height * 56 / 100 {
            for x in center_x - width / 22..=center_x + width / 22 {
                image.put_pixel(x, y, Rgb([188, 84, 34]));
            }
        }
        for y in height * 48 / 100..height * 72 / 100 {
            let x = width * 68 / 100 + (y % 3);
            image.put_pixel(x, y, Rgb([246, 58, 142]));
            image.put_pixel(x + 8, y, Rgb([64, 226, 92]));
        }
    }

    #[test]
    fn audit_passes_textured_color_image() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
                } else if checker {
                    (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
                } else {
                    (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let audit = audit_image("synthetic.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
    }

    #[test]
    fn audit_rejects_flat_image() {
        let image = RgbImage::from_pixel(MIN_WIDTH, MIN_HEIGHT, Rgb([16, 16, 16]));

        let audit = audit_image("flat.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "luma_stddev" && !check.passed)
        );
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "edge_density" && !check.passed)
        );
    }

    #[test]
    fn audit_rejects_non_opaque_image() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
                } else if checker {
                    (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
                } else {
                    (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let audit = audit_image_with_alpha("transparent.png".to_string(), image, 0.01)
            .expect("audit should load");

        assert!(!audit.passed);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "transparent_pixel_fraction" && !check.passed)
        );
    }

    #[test]
    fn audit_rejects_large_foreign_canvas_regions() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
                } else if checker {
                    (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
                } else {
                    (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        for y in 0..MIN_HEIGHT * 86 / 100 {
            for x in MIN_WIDTH * 42 / 100..MIN_WIDTH {
                image.put_pixel(x, y, Rgb([246, 242, 232]));
            }
        }
        paint_readability_signals(&mut image);

        let audit =
            audit_image("foreign_canvas.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(audit.foreign_canvas_fraction > MAX_FOREIGN_CANVAS_FRACTION);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "foreign_canvas_fraction" && !check.passed)
        );
    }

    #[test]
    fn audit_rejects_large_low_detail_scene_surface() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126, 158, 196)
                } else {
                    (92, 74, 52)
                };
                image.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        paint_readability_signals(&mut image);

        let audit =
            audit_image("low_detail_scene.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(audit.scene_candidate_tile_count > 0);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "scene_detail_tile_fraction" && !check.passed)
        );
    }

    #[test]
    fn report_rejects_readable_sequence_without_sky() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if checker {
                    (46 + (x % 90), 104 + (y % 86), 56 + ((x + y) % 54))
                } else {
                    (108 + (x % 76), 78 + (y % 62), 50 + ((x + y) % 48))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let audit = audit_image("no_sky.png".to_string(), image).expect("audit should load");
        let checks = report_checks(std::slice::from_ref(&audit));

        assert!(audit.passed, "{audit:?}");
        assert!(!report_passed(std::slice::from_ref(&audit), &checks));
        assert!(
            checks
                .iter()
                .any(|check| check.name == "max_top_sky_fraction" && !check.passed)
        );
    }

    #[test]
    fn report_allows_low_sky_close_frame_when_checkpoint_has_sky() {
        let mut close_image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 18 {
                    (124 + (x % 32), 154 + (y % 32), 190 + ((x + y) % 32))
                } else if y < MIN_HEIGHT / 3 {
                    (72 + (x % 74), 60 + (y % 58), 42 + ((x + y) % 42))
                } else if checker {
                    (42 + (x % 95), 105 + (y % 90), 54 + ((x + y) % 56))
                } else {
                    (112 + (x % 82), 82 + (y % 64), 50 + ((x + y) % 52))
                };
                close_image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut close_image);

        let close_audit =
            audit_image("close.png".to_string(), close_image).expect("audit should load");
        assert!(close_audit.passed, "{close_audit:?}");
        assert!(close_audit.top_sky_fraction < MIN_SEQUENCE_TOP_SKY_FRACTION);

        let close_only_checks = report_checks(std::slice::from_ref(&close_audit));
        assert!(!report_passed(
            std::slice::from_ref(&close_audit),
            &close_only_checks
        ));

        let mut checkpoint_image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (132 + (x % 32), 162 + (y % 32), 198 + ((x + y) % 32))
                } else if checker {
                    (44 + (x % 90), 110 + (y % 90), 58 + ((x + y) % 52))
                } else {
                    (118 + (x % 78), 84 + (y % 58), 52 + ((x + y) % 48))
                };
                checkpoint_image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut checkpoint_image);

        let checkpoint_audit =
            audit_image("checkpoint.png".to_string(), checkpoint_image).expect("audit should load");
        let audits = vec![close_audit, checkpoint_audit];
        let checks = report_checks(&audits);

        assert!(report_passed(&audits, &checks), "{checks:?}");
    }

    #[test]
    fn audit_rejects_sky_only_frame() {
        let image = RgbImage::from_pixel(MIN_WIDTH, MIN_HEIGHT, Rgb([136, 170, 208]));

        let audit = audit_image("sky_only.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "lower_scene_fraction" && !check.passed)
        );
    }

    #[test]
    fn audit_passes_water_heavy_scene() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (125 + (x % 38), 154 + (y % 46), 190 + ((x + y) % 46))
                } else {
                    let wave = ((x / 5 + y / 3) % 2) * 42;
                    (34 + (x % 28), 88 + wave + (y % 40), 112 + wave + (x % 52))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let audit = audit_image("water.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.lower_scene_fraction >= MIN_LOWER_SCENE_FRACTION);
        assert!(audit.center_scene_fraction >= MIN_CENTER_SCENE_FRACTION);
    }

    #[test]
    fn audit_allows_dark_player_without_warm_focus_pixels() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
                } else if checker {
                    (42 + (x % 70), 96 + (y % 60), 58 + ((x + y) % 48))
                } else {
                    (52 + (x % 52), 64 + (y % 48), 72 + ((x + y) % 36))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }

        let center_x = MIN_WIDTH / 2;
        for y in MIN_HEIGHT * 50 / 100..MIN_HEIGHT * 72 / 100 {
            for x in center_x - MIN_WIDTH / 70..=center_x + MIN_WIDTH / 70 {
                image.put_pixel(x, y, Rgb([24, 28, 26]));
            }
        }

        let audit = audit_image("dark_player.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.player_focus_fraction >= MIN_PLAYER_FOCUS_FRACTION);
        assert_eq!(audit.player_warm_focus_fraction, 0.0);
    }

    #[test]
    fn audit_does_not_count_bright_clouds_outside_hud_regions_as_hud_text() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    if x > MIN_WIDTH / 2 && y < MIN_HEIGHT / 5 {
                        (232, 235, 238)
                    } else {
                        (124 + (x % 42), 156 + (y % 44), 194 + ((x + y) % 44))
                    }
                } else if checker {
                    (48 + (x % 95), 102 + (y % 95), 55 + ((x + y) % 60))
                } else {
                    (105 + (x % 85), 76 + (y % 66), 48 + ((x + y) % 54))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let audit = audit_image("clouds.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.hud_text_fraction <= MAX_HUD_TEXT_FRACTION);
    }

    #[test]
    fn audit_rejects_missing_player_focus() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
                } else {
                    (54 + (x % 90), 112 + (y % 78), 58 + ((x + y) % 48))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }

        let audit =
            audit_image("missing_player.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "player_focus_fraction" && !check.passed)
        );
    }

    #[test]
    fn audit_rejects_severe_border_clipping() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
                } else if checker {
                    (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
                } else {
                    (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let top_band = MIN_HEIGHT * 8 / 100;
        for y in 0..top_band {
            for x in MIN_WIDTH * 36 / 100..MIN_WIDTH {
                image.put_pixel(x, y, Rgb([74, 62, 46]));
            }
        }

        let audit = audit_image("clipped.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(audit.severe_clipping_fraction > MAX_SEVERE_CLIPPING_FRACTION);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "severe_clipping_fraction" && !check.passed)
        );
    }

    #[test]
    fn audit_allows_continuous_scene_surface_at_border() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
                } else if checker {
                    (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
                } else {
                    (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        paint_readability_signals(&mut image);

        let top_inner_band = MIN_HEIGHT * 16 / 100;
        for y in 0..top_inner_band {
            for x in MIN_WIDTH * 36 / 100..MIN_WIDTH {
                image.put_pixel(x, y, Rgb([74, 62, 46]));
            }
        }

        let audit = audit_image("continuous_border_surface.png".to_string(), image)
            .expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.severe_clipping_fraction <= MAX_SEVERE_CLIPPING_FRACTION);
    }

    #[test]
    fn report_rejects_missing_route_markers() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 34), 158 + (y % 34), 196 + ((x + y) % 34))
                } else if checker {
                    (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
                } else {
                    (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }
        let center_x = MIN_WIDTH / 2;
        for y in MIN_HEIGHT * 54 / 100..MIN_HEIGHT * 72 / 100 {
            for x in center_x - MIN_WIDTH / 80..=center_x + MIN_WIDTH / 80 {
                image.put_pixel(x, y, Rgb([24, 28, 26]));
            }
        }
        for y in MIN_HEIGHT * 48 / 100..MIN_HEIGHT * 56 / 100 {
            for x in center_x - MIN_WIDTH / 22..=center_x + MIN_WIDTH / 22 {
                image.put_pixel(x, y, Rgb([188, 84, 34]));
            }
        }

        let audit =
            audit_image("missing_route_marker.png".to_string(), image).expect("audit should load");
        assert!(audit.passed, "{audit:?}");
        let checks = report_checks(std::slice::from_ref(&audit));

        assert!(!report_passed(std::slice::from_ref(&audit), &checks));
        assert!(
            checks
                .iter()
                .any(|check| check.name == "max_route_marker_fraction" && !check.passed)
        );
    }

    #[test]
    fn report_rejects_single_blob_route_marker_identity() {
        let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
        for y in 0..MIN_HEIGHT {
            for x in 0..MIN_WIDTH {
                let checker = (x + y) % 2 == 0;
                let (r, g, b) = if y < MIN_HEIGHT / 3 {
                    (126 + (x % 34), 158 + (y % 34), 196 + ((x + y) % 34))
                } else if checker {
                    (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
                } else {
                    (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
                };
                image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
            }
        }

        let center_x = MIN_WIDTH / 2;
        for y in MIN_HEIGHT * 54 / 100..MIN_HEIGHT * 72 / 100 {
            for x in center_x - MIN_WIDTH / 80..=center_x + MIN_WIDTH / 80 {
                image.put_pixel(x, y, Rgb([24, 28, 26]));
            }
        }
        for y in MIN_HEIGHT * 48 / 100..MIN_HEIGHT * 56 / 100 {
            for x in center_x - MIN_WIDTH / 22..=center_x + MIN_WIDTH / 22 {
                image.put_pixel(x, y, Rgb([188, 84, 34]));
            }
        }
        for y in MIN_HEIGHT * 50 / 100..MIN_HEIGHT * 62 / 100 {
            for x in MIN_WIDTH * 66 / 100..MIN_WIDTH * 70 / 100 {
                image.put_pixel(x, y, Rgb([246, 184, 48]));
            }
        }

        let audit =
            audit_image("single_route_marker.png".to_string(), image).expect("audit should load");
        assert!(audit.passed, "{audit:?}");
        assert!(audit.route_marker_fraction >= MIN_SEQUENCE_ROUTE_MARKER_FRACTION);
        assert_eq!(audit.route_marker_component_count, 1);
        assert_eq!(audit.route_marker_hue_family_count, 1);

        let checks = report_checks(std::slice::from_ref(&audit));
        assert!(!report_passed(std::slice::from_ref(&audit), &checks));
        assert!(
            checks
                .iter()
                .any(|check| check.name == "max_route_marker_component_count" && !check.passed)
        );
        assert_eq!(checks.iter().filter(|check| !check.passed).count(), 1);
    }

    #[test]
    fn json_string_escapes_control_characters() {
        let escaped = json_string("quote\" slash\\ newline\n carriage\r tab\t unit\u{1f}");

        assert_eq!(
            escaped,
            "\"quote\\\" slash\\\\ newline\\n carriage\\r tab\\t unit\\u001f\""
        );
    }
}
