use image::RgbImage;
use std::collections::HashSet;

use super::{pixel_rules::is_hud_region, thresholds::*};

pub(super) fn transparent_pixel_fraction(image: &image::DynamicImage) -> f64 {
    let rgba = image.to_rgba8();
    let pixel_count = rgba.width() as usize * rgba.height() as usize;
    let transparent_pixels = rgba.pixels().filter(|pixel| pixel.0[3] < u8::MAX).count();
    fraction(transparent_pixels, pixel_count)
}

pub(super) fn fraction(value: usize, total: usize) -> f64 {
    value as f64 / total.max(1) as f64
}

pub(super) fn variance(sum_sq: f64, sum: f64, count: f64) -> f64 {
    (sum_sq / count - (sum / count).powi(2)).max(0.0)
}

pub(super) fn edge_density(luma_values: &[f64], width: usize, height: usize) -> f64 {
    edge_density_in_region(luma_values, width, height, 1, width, 1, height)
}

pub(super) fn edge_density_in_region(
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
pub(super) struct TerrainSurfaceStats {
    pub(super) luma_span: f64,
    pub(super) internal_edge_density: f64,
    pub(super) isolated_speck_fraction: f64,
}

pub(super) fn terrain_surface_stats(
    luma_values: &[f64],
    terrain_mask: &[bool],
    width: usize,
    height: usize,
) -> TerrainSurfaceStats {
    let pixel_count = width.saturating_mul(height);
    if width == 0
        || height == 0
        || luma_values.len() != pixel_count
        || terrain_mask.len() != pixel_count
    {
        return TerrainSurfaceStats::default();
    }

    let mut terrain_luma = terrain_mask
        .iter()
        .enumerate()
        .filter_map(|(index, present)| present.then_some(luma_values[index]))
        .collect::<Vec<_>>();
    if terrain_luma.is_empty() {
        return TerrainSurfaceStats::default();
    }
    terrain_luma.sort_by(f64::total_cmp);
    let low_luma = sorted_percentile(&terrain_luma, TERRAIN_LUMA_SPAN_LOW_PERCENTILE);
    let high_luma = sorted_percentile(&terrain_luma, TERRAIN_LUMA_SPAN_HIGH_PERCENTILE);

    let mut internal_edge_pairs = 0usize;
    let mut internal_neighbor_pairs = 0usize;
    let edge_offsets = [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)];
    for (index, present) in terrain_mask.iter().enumerate() {
        if !present {
            continue;
        }
        let x = index % width;
        let y = index / width;
        for (x_offset, y_offset) in edge_offsets {
            let neighbor_x = x as isize + x_offset;
            let neighbor_y = y as isize + y_offset;
            if neighbor_x < 0
                || neighbor_y < 0
                || neighbor_x >= width as isize
                || neighbor_y >= height as isize
            {
                continue;
            }
            let neighbor_index = neighbor_y as usize * width + neighbor_x as usize;
            if !terrain_mask[neighbor_index] {
                continue;
            }

            internal_neighbor_pairs += 1;
            if (luma_values[index] - luma_values[neighbor_index]).abs()
                >= TERRAIN_INTERNAL_EDGE_LUMA_DELTA
            {
                internal_edge_pairs += 1;
            }
        }
    }

    let mut isolated_specks = 0usize;
    let mut speck_candidates = 0usize;
    let speck_offsets = [
        (-1isize, -1isize),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    for (index, present) in terrain_mask.iter().enumerate() {
        if !present {
            continue;
        }
        let x = index % width;
        let y = index / width;
        let mut in_mask_neighbors = 0usize;
        let mut high_contrast_neighbors = 0usize;

        for (x_offset, y_offset) in speck_offsets {
            let neighbor_x = x as isize + x_offset;
            let neighbor_y = y as isize + y_offset;
            if neighbor_x < 0
                || neighbor_y < 0
                || neighbor_x >= width as isize
                || neighbor_y >= height as isize
            {
                continue;
            }
            let neighbor_index = neighbor_y as usize * width + neighbor_x as usize;
            if !terrain_mask[neighbor_index] {
                continue;
            }

            in_mask_neighbors += 1;
            if (luma_values[index] - luma_values[neighbor_index]).abs()
                >= TERRAIN_ISOLATED_SPECK_LUMA_DELTA
            {
                high_contrast_neighbors += 1;
            }
        }

        if in_mask_neighbors >= MIN_TERRAIN_ISOLATED_SPECK_NEIGHBORS {
            speck_candidates += 1;
            if high_contrast_neighbors == in_mask_neighbors {
                isolated_specks += 1;
            }
        }
    }

    TerrainSurfaceStats {
        luma_span: (high_luma - low_luma).max(0.0),
        internal_edge_density: fraction(internal_edge_pairs, internal_neighbor_pairs),
        isolated_speck_fraction: fraction(isolated_specks, speck_candidates),
    }
}

fn sorted_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let index = ((sorted_values.len() - 1) as f64 * percentile.clamp(0.0, 1.0)).floor() as usize;
    sorted_values[index]
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SceneDetailStats {
    pub(super) detail_tile_fraction: f64,
    pub(super) flat_tile_fraction: f64,
    pub(super) detail_tile_count: usize,
    pub(super) flat_tile_count: usize,
    pub(super) candidate_tile_count: usize,
}

pub(super) fn scene_detail_stats(
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

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LowDetailSceneComponentStats {
    pub(super) dominant_component_fraction: f64,
}

pub(super) fn low_detail_scene_component_stats(
    image: &RgbImage,
    luma_values: &[f64],
    scene_mask: &[bool],
    width: usize,
    height: usize,
) -> LowDetailSceneComponentStats {
    if width == 0 || height == 0 || scene_mask.len() != width.saturating_mul(height) {
        return LowDetailSceneComponentStats::default();
    }

    let scene_pixel_count = scene_mask.iter().filter(|is_scene| **is_scene).count();
    if scene_pixel_count == 0 {
        return LowDetailSceneComponentStats::default();
    }

    let mut visited = vec![false; scene_mask.len()];
    let mut stack = Vec::new();
    let mut dominant_low_detail_pixels = 0usize;

    for index in 0..scene_mask.len() {
        if visited[index] || !scene_mask[index] {
            continue;
        }

        let mut component_pixels = 0usize;
        let mut edge_pixels = 0usize;
        let mut edge_samples = 0usize;
        visited[index] = true;
        stack.push(index);

        while let Some(current) = stack.pop() {
            component_pixels += 1;
            let x = current % width;
            let y = current / width;

            if x > 0 && scene_mask[current - 1] {
                if component_edge(image, luma_values, current, current - 1, width) {
                    edge_pixels += 1;
                }
                edge_samples += 1;
                push_marker_neighbor(current - 1, scene_mask, &mut visited, &mut stack);
            }
            if x + 1 < width {
                push_marker_neighbor(current + 1, scene_mask, &mut visited, &mut stack);
            }
            if y > 0 && scene_mask[current - width] {
                if component_edge(image, luma_values, current, current - width, width) {
                    edge_pixels += 1;
                }
                edge_samples += 1;
                push_marker_neighbor(current - width, scene_mask, &mut visited, &mut stack);
            }
            if y + 1 < height {
                push_marker_neighbor(current + width, scene_mask, &mut visited, &mut stack);
            }
        }

        if component_pixels < MIN_LOW_DETAIL_SCENE_COMPONENT_PIXELS {
            continue;
        }

        let edge_density = fraction(edge_pixels, edge_samples);
        if edge_density <= MAX_LOW_DETAIL_SCENE_COMPONENT_EDGE_DENSITY {
            dominant_low_detail_pixels = dominant_low_detail_pixels.max(component_pixels);
        }
    }

    LowDetailSceneComponentStats {
        dominant_component_fraction: fraction(dominant_low_detail_pixels, scene_pixel_count),
    }
}

fn component_edge(
    image: &RgbImage,
    luma_values: &[f64],
    current: usize,
    neighbor: usize,
    width: usize,
) -> bool {
    let luma_delta = (luma_values[current] - luma_values[neighbor]).abs();
    if luma_delta > 18.0 {
        return true;
    }

    let x = current % width;
    let y = current / width;
    let neighbor_x = neighbor % width;
    let neighbor_y = neighbor / width;
    let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
    let [neighbor_r, neighbor_g, neighbor_b] =
        image.get_pixel(neighbor_x as u32, neighbor_y as u32).0;
    let color_delta = (r as f64 - neighbor_r as f64).abs()
        + (g as f64 - neighbor_g as f64).abs()
        + (b as f64 - neighbor_b as f64).abs();

    color_delta >= MIN_LOW_DETAIL_SCENE_COMPONENT_COLOR_EDGE_DELTA
}

pub(super) const BORDER_REGION_COUNT: usize = 4;
pub(super) const ROUTE_MARKER_HUE_FAMILY_COUNT: usize = 4;
pub(super) const SCENE_MATERIAL_FAMILY_COUNT: usize = 4;

pub(super) fn border_regions(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> [Option<usize>; 4] {
    let x_band = (width * 8 / 100).max(1);
    let y_band = (height * 8 / 100).max(1);
    [
        (y < y_band).then_some(0),
        (y >= height.saturating_sub(y_band)).then_some(1),
        (x < x_band).then_some(2),
        (x >= width.saturating_sub(x_band)).then_some(3),
    ]
}

pub(super) fn inner_border_regions(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> [Option<usize>; 4] {
    let x_band = (width * 8 / 100).max(1);
    let y_band = (height * 8 / 100).max(1);
    [
        (y >= y_band && y < y_band * 2).then_some(0),
        (y >= height.saturating_sub(y_band * 2) && y < height.saturating_sub(y_band)).then_some(1),
        (x >= x_band && x < x_band * 2).then_some(2),
        (x >= width.saturating_sub(x_band * 2) && x < width.saturating_sub(x_band)).then_some(3),
    ]
}

pub(super) fn severe_clipping_fraction(
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

pub(super) fn border_edge_densities(
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

pub(super) fn route_marker_component_count(mask: &[bool], width: usize, height: usize) -> usize {
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

pub(super) fn distant_scene_component_count(mask: &[bool], width: usize, height: usize) -> usize {
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

pub(super) fn cloud_layer_component_count(mask: &[bool], width: usize, height: usize) -> usize {
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

pub(super) fn push_marker_neighbor(
    index: usize,
    mask: &[bool],
    visited: &mut [bool],
    stack: &mut Vec<usize>,
) {
    if !visited[index] && mask[index] {
        visited[index] = true;
        stack.push(index);
    }
}
