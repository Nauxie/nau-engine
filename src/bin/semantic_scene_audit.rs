use image::{ImageReader, RgbImage};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process,
};

const SAMPLE_SEARCH_RADIUS_PX: i32 = 20;
const MIN_SAMPLE_PIXEL_HITS: usize = 3;
const MIN_VISIBLE_SAMPLES_PER_CHECKPOINT: usize = 2;
const MIN_PASSED_SAMPLES_PER_CHECKPOINT: usize = 1;
const MIN_VISIBLE_MATERIALS_PER_CHECKPOINT: usize = 3;
const MIN_MATERIAL_SAMPLE_HIT_RATIO: f64 = 0.45;
const EXPECTED_MATERIALS: [&str; 4] = ["terrain", "foliage", "cloud", "distant_island"];

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin semantic_scene_audit -- <markers.json> [...]");
        process::exit(2);
    }

    let mut checkpoints = Vec::with_capacity(paths.len());
    for path in &paths {
        match audit_checkpoint_path(path) {
            Ok(checkpoint) => checkpoints.push(checkpoint),
            Err(error) => {
                eprintln!("failed to audit {}: {error}", path.display());
                process::exit(2);
            }
        }
    }

    let checks = report_checks(&checkpoints);
    let passed = checkpoints.iter().all(|checkpoint| checkpoint.passed)
        && checks.iter().all(|check| check.passed);
    println!("{}", report_json(passed, &checks, &checkpoints));
    if !passed {
        process::exit(1);
    }
}

#[derive(Clone, Debug)]
struct SceneSampleAudit {
    kind: String,
    label: String,
    expected_material: String,
    in_viewport: bool,
    screen_x: Option<f64>,
    screen_y: Option<f64>,
    semantic_pixel_hits: usize,
    passed: bool,
}

#[derive(Clone, Debug)]
struct CheckpointAudit {
    metadata_path: String,
    screenshot_path: String,
    checkpoint: String,
    visible_scene_sample_count: usize,
    scene_sample_pixel_hit_count: usize,
    visible_scene_material_count: usize,
    scene_material_pixel_hit_count: usize,
    passed: bool,
    samples: Vec<SceneSampleAudit>,
    materials: Vec<MaterialAudit>,
}

#[derive(Clone, Debug)]
struct MaterialAudit {
    expected_material: String,
    visible_sample_count: usize,
    sample_pixel_hit_count: usize,
    min_sample_pixel_hit_count: usize,
    hit_ratio: f64,
    passed: bool,
}

#[derive(Clone, Debug)]
struct Check {
    name: String,
    passed: bool,
    value: f64,
    comparator: &'static str,
    threshold: f64,
    unit: &'static str,
}

impl Check {
    fn at_least(name: impl Into<String>, value: f64, threshold: f64, unit: &'static str) -> Self {
        Self {
            name: name.into(),
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }
}

fn audit_checkpoint_path(path: &Path) -> Result<CheckpointAudit, String> {
    let metadata = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let parsed = serde_json::from_str::<Value>(&metadata).map_err(|error| error.to_string())?;
    let screenshot_path = parsed
        .get("screenshot")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing screenshot path".to_string())?;
    let screenshot_path = resolve_screenshot_path(path, screenshot_path);
    let image = ImageReader::open(&screenshot_path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgb8();
    let scale = screenshot_scale(&parsed, &image);

    let samples = parsed
        .get("scene_samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing scene_samples array".to_string())?
        .iter()
        .map(|sample| audit_scene_sample(sample, &image, scale))
        .collect::<Result<Vec<_>, _>>()?;
    let visible_scene_sample_count = samples.iter().filter(|sample| sample.in_viewport).count();
    let scene_sample_pixel_hit_count = samples.iter().filter(|sample| sample.passed).count();
    let materials = material_audits(&samples);
    let visible_scene_material_count = materials.len();
    let scene_material_pixel_hit_count =
        materials.iter().filter(|material| material.passed).count();
    let passed = visible_scene_sample_count >= MIN_VISIBLE_SAMPLES_PER_CHECKPOINT
        && scene_sample_pixel_hit_count >= MIN_PASSED_SAMPLES_PER_CHECKPOINT
        && visible_scene_material_count >= MIN_VISIBLE_MATERIALS_PER_CHECKPOINT
        && scene_material_pixel_hit_count >= visible_scene_material_count;
    let checkpoint = parsed
        .get("checkpoint")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(CheckpointAudit {
        metadata_path: path.to_string_lossy().into_owned(),
        screenshot_path: screenshot_path.to_string_lossy().into_owned(),
        checkpoint,
        visible_scene_sample_count,
        scene_sample_pixel_hit_count,
        visible_scene_material_count,
        scene_material_pixel_hit_count,
        passed,
        samples,
        materials,
    })
}

fn resolve_screenshot_path(metadata_path: &Path, screenshot_path: &str) -> PathBuf {
    let direct_path = PathBuf::from(screenshot_path);
    if direct_path.exists() || direct_path.is_absolute() {
        return direct_path;
    }

    if let Some(sibling_path) = metadata_path
        .parent()
        .map(|parent| parent.join(&direct_path))
        .filter(|path| path.exists())
    {
        return sibling_path;
    }

    if let Some(eval_relative_path) = metadata_path
        .parent()
        .and_then(Path::parent)
        .map(|eval_dir| eval_dir.join(&direct_path))
        .filter(|path| path.exists())
    {
        return eval_relative_path;
    }

    direct_path
}

fn screenshot_scale(metadata: &Value, image: &RgbImage) -> (f64, f64) {
    let viewport = metadata.get("viewport");
    let Some(viewport_width) = viewport
        .and_then(|viewport| viewport.get("width"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
    else {
        return (1.0, 1.0);
    };
    let Some(viewport_height) = viewport
        .and_then(|viewport| viewport.get("height"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
    else {
        return (1.0, 1.0);
    };

    (
        image.width() as f64 / viewport_width,
        image.height() as f64 / viewport_height,
    )
}

fn audit_scene_sample(
    sample: &Value,
    image: &RgbImage,
    screenshot_scale: (f64, f64),
) -> Result<SceneSampleAudit, String> {
    let kind = sample
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let label = sample
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let expected_material = sample
        .get("expected_material")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let in_viewport = sample
        .get("in_viewport")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let screen = sample.get("screen");
    let screen_x = screen
        .and_then(|screen| screen.get("x"))
        .and_then(Value::as_f64);
    let screen_y = screen
        .and_then(|screen| screen.get("y"))
        .and_then(Value::as_f64);
    let semantic_pixel_hits = match (in_viewport, screen_x, screen_y) {
        (true, Some(x), Some(y)) => {
            sample_pixel_hits(image, x, y, &expected_material, screenshot_scale)
        }
        _ => 0,
    };

    Ok(SceneSampleAudit {
        kind,
        label,
        expected_material,
        in_viewport,
        screen_x,
        screen_y,
        semantic_pixel_hits,
        passed: in_viewport && semantic_pixel_hits >= MIN_SAMPLE_PIXEL_HITS,
    })
}

fn material_audits(samples: &[SceneSampleAudit]) -> Vec<MaterialAudit> {
    EXPECTED_MATERIALS
        .iter()
        .filter_map(|expected_material| {
            let visible_sample_count = samples
                .iter()
                .filter(|sample| {
                    sample.in_viewport && sample.expected_material == *expected_material
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
                min_material_sample_pixel_hit_count(visible_sample_count);
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

fn min_material_sample_pixel_hit_count(visible_sample_count: usize) -> usize {
    if visible_sample_count == 0 {
        0
    } else {
        (visible_sample_count as f64 * MIN_MATERIAL_SAMPLE_HIT_RATIO)
            .ceil()
            .max(1.0) as usize
    }
}

fn sample_pixel_hits(
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

fn material_matches(expected_material: &str, r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let sky_like = is_sky_like(r, g, b, luma);
    match expected_material {
        "terrain" => is_scene_like(r, g, b, luma, sky_like),
        "foliage" => is_foliage_like(r, g, b, luma, sky_like),
        "cloud" => is_cloud_like(r, g, b, luma, sky_like),
        "distant_island" => is_distant_scene_like(r, g, b, luma, sky_like),
        _ => false,
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

    is_foliage_like(r, g, b, luma, sky_like)
        || is_earth_like(r, g, b)
        || is_rock_or_shadow_like(r, g, b, luma)
}

fn is_foliage_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    !sky_like && (18.0..=185.0).contains(&luma) && g >= 58.0 && g >= r * 0.72 && g >= b * 0.58
}

fn is_earth_like(r: f64, g: f64, b: f64) -> bool {
    r >= 50.0 && g >= 38.0 && r >= b + 8.0 && g >= b * 0.68
}

fn is_rock_or_shadow_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    (18.0..=155.0).contains(&luma) && (r - g).abs() <= 50.0 && b <= r.max(g) + 20.0
}

fn is_cloud_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
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
    !water_like
        && (is_foliage_like(r, g, b, luma, sky_like)
            || is_earth_like(r, g, b)
            || is_rock_or_shadow_like(r, g, b, luma))
}

fn report_checks(checkpoints: &[CheckpointAudit]) -> Vec<Check> {
    let passed_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.passed)
        .count();
    let material_family_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.visible_scene_material_count >= MIN_VISIBLE_MATERIALS_PER_CHECKPOINT
                && checkpoint.scene_material_pixel_hit_count
                    >= checkpoint.visible_scene_material_count
        })
        .count();
    let min_visible_material_count = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.visible_scene_material_count)
        .min()
        .unwrap_or(0);
    let material_counts = material_hit_counts(checkpoints);
    let mut checks = vec![Check::at_least(
        "checkpoint_scene_pixel_hits",
        passed_checkpoint_count as f64,
        checkpoints.len() as f64,
        "checkpoints",
    )];

    checks.push(Check::at_least(
        "checkpoint_scene_material_family_hits",
        material_family_checkpoint_count as f64,
        checkpoints.len() as f64,
        "checkpoints",
    ));
    checks.push(Check::at_least(
        "min_visible_scene_material_count",
        min_visible_material_count as f64,
        MIN_VISIBLE_MATERIALS_PER_CHECKPOINT as f64,
        "materials",
    ));

    for material in EXPECTED_MATERIALS {
        checks.push(Check::at_least(
            format!("{material}_scene_sample_pixel_hits"),
            *material_counts.get(material).unwrap_or(&0) as f64,
            1.0,
            "samples",
        ));
    }

    checks
}

fn material_hit_counts(checkpoints: &[CheckpointAudit]) -> BTreeMap<&str, usize> {
    let mut unique_hits = HashSet::new();
    for checkpoint in checkpoints {
        for sample in &checkpoint.samples {
            if sample.passed {
                unique_hits.insert((sample.expected_material.as_str(), sample.label.as_str()));
            }
        }
    }

    let mut counts = BTreeMap::new();
    for (material, _) in unique_hits {
        *counts.entry(material).or_default() += 1;
    }
    counts
}

fn report_json(passed: bool, checks: &[Check], checkpoints: &[CheckpointAudit]) -> String {
    let checkpoints_json = checkpoints
        .iter()
        .map(checkpoint_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let checks_json = checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");

    format!(
        "{{\n  \"passed\": {},\n  \"checkpoint_count\": {},\n  \"checks\": [\n    {}\n  ],\n  \"checkpoints\": [\n{}\n  ]\n}}",
        passed,
        checkpoints.len(),
        checks_json,
        checkpoints_json
    )
}

fn checkpoint_json(checkpoint: &CheckpointAudit) -> String {
    let samples_json = checkpoint
        .samples
        .iter()
        .map(sample_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let materials_json = checkpoint
        .materials
        .iter()
        .map(material_json)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "    {{\n      \"metadata_path\": {},\n      \"screenshot_path\": {},\n      \"checkpoint\": {},\n      \"passed\": {},\n      \"visible_scene_sample_count\": {},\n      \"scene_sample_pixel_hit_count\": {},\n      \"visible_scene_material_count\": {},\n      \"scene_material_pixel_hit_count\": {},\n      \"materials\": [\n{}\n      ],\n      \"samples\": [\n{}\n      ]\n    }}",
        json_string(&checkpoint.metadata_path),
        json_string(&checkpoint.screenshot_path),
        json_string(&checkpoint.checkpoint),
        checkpoint.passed,
        checkpoint.visible_scene_sample_count,
        checkpoint.scene_sample_pixel_hit_count,
        checkpoint.visible_scene_material_count,
        checkpoint.scene_material_pixel_hit_count,
        materials_json,
        samples_json
    )
}

fn material_json(material: &MaterialAudit) -> String {
    format!(
        "        {{\"expected_material\": {}, \"visible_sample_count\": {}, \"sample_pixel_hit_count\": {}, \"min_sample_pixel_hit_count\": {}, \"hit_ratio\": {}, \"passed\": {}}}",
        json_string(&material.expected_material),
        material.visible_sample_count,
        material.sample_pixel_hit_count,
        material.min_sample_pixel_hit_count,
        json_number(material.hit_ratio),
        material.passed
    )
}

fn sample_json(sample: &SceneSampleAudit) -> String {
    let screen = match (sample.screen_x, sample.screen_y) {
        (Some(x), Some(y)) => format!("{{\"x\": {}, \"y\": {}}}", json_number(x), json_number(y)),
        _ => "null".to_string(),
    };
    format!(
        "        {{\"kind\": {}, \"label\": {}, \"expected_material\": {}, \"in_viewport\": {}, \"screen\": {}, \"semantic_pixel_hits\": {}, \"passed\": {}}}",
        json_string(&sample.kind),
        json_string(&sample.label),
        json_string(&sample.expected_material),
        sample.in_viewport,
        screen,
        sample.semantic_pixel_hits,
        sample.passed
    )
}

fn check_json(check: &Check) -> String {
    format!(
        "{{\"name\": {}, \"passed\": {}, \"value\": {}, \"comparator\": {}, \"threshold\": {}, \"unit\": {}}}",
        json_string(&check.name),
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
                use std::fmt::Write as _;
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

    fn sample(expected_material: &str, x: f64, y: f64) -> Value {
        serde_json::json!({
            "kind": "test_sample",
            "label": expected_material,
            "expected_material": expected_material,
            "in_viewport": true,
            "screen": {"x": x, "y": y}
        })
    }

    #[test]
    fn sample_pixel_hits_classifies_expected_scene_materials() {
        let mut image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
        image.put_pixel(16, 16, Rgb([104, 82, 48]));
        image.put_pixel(17, 16, Rgb([92, 74, 46]));
        image.put_pixel(18, 16, Rgb([74, 68, 62]));
        image.put_pixel(16, 32, Rgb([44, 126, 32]));
        image.put_pixel(17, 32, Rgb([48, 132, 36]));
        image.put_pixel(18, 32, Rgb([52, 138, 34]));
        image.put_pixel(16, 48, Rgb([158, 166, 174]));
        image.put_pixel(17, 48, Rgb([166, 174, 184]));
        image.put_pixel(18, 48, Rgb([148, 158, 168]));

        assert!(
            sample_pixel_hits(&image, 17.0, 16.0, "terrain", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS
        );
        assert!(
            sample_pixel_hits(&image, 17.0, 32.0, "foliage", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS
        );
        assert!(
            sample_pixel_hits(&image, 17.0, 48.0, "cloud", (1.0, 1.0)) >= MIN_SAMPLE_PIXEL_HITS
        );
        assert!(
            sample_pixel_hits(&image, 17.0, 16.0, "distant_island", (1.0, 1.0))
                >= MIN_SAMPLE_PIXEL_HITS
        );
    }

    #[test]
    fn checkpoint_requires_projected_scene_sample_hits() {
        let image = RgbImage::from_pixel(64, 64, Rgb([130, 170, 220]));
        let audit = audit_scene_sample(&sample("terrain", 32.0, 32.0), &image, (1.0, 1.0))
            .expect("sample should parse");

        assert!(audit.in_viewport);
        assert!(!audit.passed);
    }

    #[test]
    fn semantic_scene_audit_scales_logical_viewport_to_retina_screenshot() {
        let temp_dir = unique_temp_dir("semantic_scene_retina");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(160, 120, Rgb([130, 170, 220]));
        image.put_pixel(40, 30, Rgb([104, 82, 48]));
        image.put_pixel(41, 30, Rgb([92, 74, 46]));
        image.put_pixel(42, 30, Rgb([74, 68, 62]));
        image.put_pixel(140, 110, Rgb([44, 126, 32]));
        image.put_pixel(141, 110, Rgb([48, 132, 36]));
        image.put_pixel(142, 110, Rgb([52, 138, 34]));
        image.put_pixel(120, 86, Rgb([158, 166, 174]));
        image.put_pixel(121, 86, Rgb([166, 174, 184]));
        image.put_pixel(122, 86, Rgb([148, 158, 168]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            semantic_metadata_json(&screenshot_path, 20.0, 15.0, 80.0, 60.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert_eq!(audit.visible_scene_sample_count, 3);
        assert_eq!(audit.scene_sample_pixel_hit_count, 3);
        assert_eq!(audit.visible_scene_material_count, 3);
        assert_eq!(audit.scene_material_pixel_hit_count, 3);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn checkpoint_requires_each_visible_material_family_to_hit() {
        let temp_dir = unique_temp_dir("semantic_scene_materials");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(80, 60, Rgb([130, 170, 220]));
        image.put_pixel(20, 15, Rgb([104, 82, 48]));
        image.put_pixel(21, 15, Rgb([92, 74, 46]));
        image.put_pixel(22, 15, Rgb([74, 68, 62]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            semantic_metadata_json(&screenshot_path, 20.0, 15.0, 80.0, 60.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(!audit.passed);
        assert_eq!(audit.visible_scene_material_count, 3);
        assert_eq!(audit.scene_material_pixel_hit_count, 1);
        assert!(
            audit
                .materials
                .iter()
                .any(|material| material.expected_material == "terrain" && material.passed)
        );
        assert!(
            audit
                .materials
                .iter()
                .any(|material| material.expected_material == "foliage" && !material.passed)
        );
        assert!(
            audit
                .materials
                .iter()
                .any(|material| material.expected_material == "cloud" && !material.passed)
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "nau_{name}_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    fn semantic_metadata_json(
        screenshot_path: &Path,
        x: f64,
        y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> String {
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"viewport\": {{\"width\": {}, \"height\": {}}}, \"scene_samples\": [{{\"kind\": \"terrain_surface\", \"label\": \"terrain\", \"expected_material\": \"terrain\", \"in_viewport\": true, \"screen\": {{\"x\": {}, \"y\": {}}}}}, {{\"kind\": \"tree_canopy\", \"label\": \"foliage\", \"expected_material\": \"foliage\", \"in_viewport\": true, \"screen\": {{\"x\": 70.0000, \"y\": 55.0000}}}}, {{\"kind\": \"weather_cloud\", \"label\": \"cloud\", \"expected_material\": \"cloud\", \"in_viewport\": true, \"screen\": {{\"x\": 60.0000, \"y\": 43.0000}}}}]}}",
            json_string(&screenshot_path.to_string_lossy()),
            json_number(viewport_width),
            json_number(viewport_height),
            json_number(x),
            json_number(y)
        )
    }
}
