use crate::{
    thresholds::{EXPECTED_MATERIALS, MIN_MATERIAL_SAMPLE_HIT_RATIO, SAMPLE_SEARCH_RADIUS_PX},
    types::{MaterialAudit, SceneSampleAudit},
};
use image::RgbImage;

pub(crate) fn material_audits(samples: &[SceneSampleAudit]) -> Vec<MaterialAudit> {
    let mut expected_materials = EXPECTED_MATERIALS.to_vec();
    if samples
        .iter()
        .any(|sample| sample.is_visible() && sample.expected_material == "wind")
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
                passed: sample_pixel_hit_count >= min_sample_pixel_hit_count,
            })
        })
        .collect()
}

fn min_material_sample_pixel_hit_count_for(
    expected_material: &str,
    visible_sample_count: usize,
) -> usize {
    if expected_material == "wind" {
        return visible_sample_count.min(1);
    }

    min_material_sample_pixel_hit_count(visible_sample_count)
}

pub(crate) fn min_material_sample_pixel_hit_count(visible_sample_count: usize) -> usize {
    if visible_sample_count == 0 {
        0
    } else {
        (visible_sample_count as f64 * MIN_MATERIAL_SAMPLE_HIT_RATIO)
            .ceil()
            .max(1.0) as usize
    }
}

pub(crate) fn sample_pixel_hits(
    image: &RgbImage,
    screen_x: f64,
    screen_y: f64,
    expected_material: &str,
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
            if material_matches(expected_material, r as f64, g as f64, b as f64) {
                hits += 1;
            }
        }
    }

    hits
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
        _ => false,
    }
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
