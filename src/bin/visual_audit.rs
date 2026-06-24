use image::{ImageReader, RgbImage};
use std::{
    collections::HashSet,
    env,
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

    let passed = audits.iter().all(|audit| audit.passed);
    println!("{}", audit_report_json(passed, &audits));
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

    for pixel in image.pixels() {
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
    let edge_density = edge_density(&luma_values, width as usize, height as usize);

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
        passed,
        checks,
    })
}

fn variance(sum_sq: f64, sum: f64, count: f64) -> f64 {
    (sum_sq / count - (sum / count).powi(2)).max(0.0)
}

fn edge_density(luma_values: &[f64], width: usize, height: usize) -> f64 {
    if width < 2 || height < 2 {
        return 0.0;
    }

    let mut edge_pixels = 0usize;
    let mut sampled_pixels = 0usize;
    for y in (1..height).step_by(2) {
        for x in (1..width).step_by(2) {
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

fn audit_report_json(passed: bool, audits: &[ImageAudit]) -> String {
    let images = audits
        .iter()
        .map(image_audit_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!(
        "{{\n  \"passed\": {},\n  \"image_count\": {},\n  \"images\": [\n    {}\n  ]\n}}",
        passed,
        audits.len(),
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
        "{{\n      \"path\": {},\n      \"passed\": {},\n      \"width\": {},\n      \"height\": {},\n      \"mean_luma\": {},\n      \"luma_stddev\": {},\n      \"colorfulness\": {},\n      \"quantized_colors\": {},\n      \"edge_density\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
        json_string(&audit.path),
        audit.passed,
        audit.width,
        audit.height,
        json_number(audit.mean_luma),
        json_number(audit.luma_stddev),
        json_number(audit.colorfulness),
        audit.quantized_colors,
        json_number(audit.edge_density),
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
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
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
                let base = if checker { 42 } else { 174 };
                let r = (base + (x % 48)) as u8;
                let g = (70 + (y % 120)) as u8;
                let b = ((if checker { 180 } else { 58 }) + ((x + y) % 32)) as u8;
                image.put_pixel(x, y, Rgb([r, g, b]));
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
}
