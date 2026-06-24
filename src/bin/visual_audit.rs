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
const MIN_FRAME_TOP_SKY_FRACTION: f64 = 0.04;
const MIN_SEQUENCE_TOP_SKY_FRACTION: f64 = 0.25;
const MIN_LOWER_SCENE_FRACTION: f64 = 0.25;
const MIN_CENTER_SCENE_FRACTION: f64 = 0.18;
const MIN_CENTER_EDGE_DENSITY: f64 = 0.02;
const MAX_HUD_TEXT_FRACTION: f64 = 0.06;

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
    let image = ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgb8();
    audit_image(path.to_string_lossy().into_owned(), image)
}

fn audit_image(path: String, image: RgbImage) -> Result<ImageAudit, String> {
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
    let mut hud_text_pixels = 0usize;

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
        if is_hud_region(x, y, width_usize, height_usize) && is_hud_text_like(r, g, b) {
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
    let hud_text_fraction = fraction(hud_text_pixels, pixel_count);

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
            "top_sky_fraction",
            top_sky_fraction,
            MIN_FRAME_TOP_SKY_FRACTION,
            "ratio",
        ),
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
        hud_text_fraction,
        passed,
        checks,
    })
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

fn is_hud_text_like(r: f64, g: f64, b: f64) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    max_channel >= 220.0 && max_channel - min_channel <= 24.0
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

    vec![Check::at_least(
        "max_top_sky_fraction",
        max_top_sky_fraction,
        MIN_SEQUENCE_TOP_SKY_FRACTION,
        "ratio",
    )]
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
        "{{\n      \"path\": {},\n      \"passed\": {},\n      \"width\": {},\n      \"height\": {},\n      \"mean_luma\": {},\n      \"luma_stddev\": {},\n      \"colorfulness\": {},\n      \"quantized_colors\": {},\n      \"edge_density\": {},\n      \"top_sky_fraction\": {},\n      \"lower_scene_fraction\": {},\n      \"center_scene_fraction\": {},\n      \"center_edge_density\": {},\n      \"hud_text_fraction\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
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
    fn audit_rejects_missing_sky() {
        let image = RgbImage::from_pixel(MIN_WIDTH, MIN_HEIGHT, Rgb([80, 130, 72]));

        let audit = audit_image("no_sky.png".to_string(), image).expect("audit should load");

        assert!(!audit.passed);
        assert!(
            audit
                .checks
                .iter()
                .any(|check| check.name == "top_sky_fraction" && !check.passed)
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

        let audit = audit_image("water.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.lower_scene_fraction >= MIN_LOWER_SCENE_FRACTION);
        assert!(audit.center_scene_fraction >= MIN_CENTER_SCENE_FRACTION);
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

        let audit = audit_image("clouds.png".to_string(), image).expect("audit should load");

        assert!(audit.passed, "{audit:?}");
        assert!(audit.hud_text_fraction <= MAX_HUD_TEXT_FRACTION);
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
