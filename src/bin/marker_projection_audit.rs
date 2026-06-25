use image::{ImageReader, RgbImage};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const MARKER_SEARCH_RADIUS_PX: i32 = 22;
const MIN_MARKER_PIXEL_HITS: usize = 2;

fn main() {
    let paths = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: cargo run --bin marker_projection_audit -- <markers.json> [...]");
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

    let passed = checkpoints.iter().all(|checkpoint| checkpoint.passed);
    println!("{}", report_json(passed, &checkpoints));
    if !passed {
        process::exit(1);
    }
}

#[derive(Clone, Debug)]
struct MarkerAudit {
    kind: String,
    label: String,
    current_objective: bool,
    in_viewport: bool,
    screen_x: Option<f64>,
    screen_y: Option<f64>,
    marker_pixel_hits: usize,
    passed: bool,
}

#[derive(Clone, Debug)]
struct CheckpointAudit {
    metadata_path: String,
    screenshot_path: String,
    checkpoint: String,
    visible_marker_count: usize,
    marker_pixel_hit_count: usize,
    passed: bool,
    markers: Vec<MarkerAudit>,
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

    let markers = parsed
        .get("markers")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing markers array".to_string())?
        .iter()
        .map(|marker| audit_marker(marker, &image))
        .collect::<Result<Vec<_>, _>>()?;
    let visible_marker_count = markers.iter().filter(|marker| marker.in_viewport).count();
    let marker_pixel_hit_count = markers.iter().filter(|marker| marker.passed).count();
    let passed = visible_marker_count > 0 && marker_pixel_hit_count > 0;
    let checkpoint = parsed
        .get("checkpoint")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(CheckpointAudit {
        metadata_path: path.to_string_lossy().into_owned(),
        screenshot_path: screenshot_path.to_string_lossy().into_owned(),
        checkpoint,
        visible_marker_count,
        marker_pixel_hit_count,
        passed,
        markers,
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

fn audit_marker(marker: &Value, image: &RgbImage) -> Result<MarkerAudit, String> {
    let kind = marker
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let label = marker
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let current_objective = marker
        .get("current_objective")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_viewport = marker
        .get("in_viewport")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let screen = marker.get("screen");
    let screen_x = screen
        .and_then(|screen| screen.get("x"))
        .and_then(Value::as_f64);
    let screen_y = screen
        .and_then(|screen| screen.get("y"))
        .and_then(Value::as_f64);
    let marker_pixel_hits = match (in_viewport, screen_x, screen_y) {
        (true, Some(x), Some(y)) => marker_pixel_hits(image, x, y),
        _ => 0,
    };

    Ok(MarkerAudit {
        kind,
        label,
        current_objective,
        in_viewport,
        screen_x,
        screen_y,
        marker_pixel_hits,
        passed: in_viewport && marker_pixel_hits >= MIN_MARKER_PIXEL_HITS,
    })
}

fn marker_pixel_hits(image: &RgbImage, screen_x: f64, screen_y: f64) -> usize {
    if !screen_x.is_finite() || !screen_y.is_finite() {
        return 0;
    }

    let width = image.width() as i32;
    let height = image.height() as i32;
    let center_x = screen_x.round() as i32;
    let center_y = screen_y.round() as i32;
    let mut hits = 0usize;

    for y in (center_y - MARKER_SEARCH_RADIUS_PX).max(0)
        ..=(center_y + MARKER_SEARCH_RADIUS_PX).min(height.saturating_sub(1))
    {
        for x in (center_x - MARKER_SEARCH_RADIUS_PX).max(0)
            ..=(center_x + MARKER_SEARCH_RADIUS_PX).min(width.saturating_sub(1))
        {
            let [r, g, b] = image.get_pixel(x as u32, y as u32).0;
            if is_route_marker_like(r as f64, g as f64, b as f64) {
                hits += 1;
            }
        }
    }

    hits
}

fn is_route_marker_like(r: f64, g: f64, b: f64) -> bool {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
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

fn report_json(passed: bool, checkpoints: &[CheckpointAudit]) -> String {
    let checkpoint_count = checkpoints.len();
    let passed_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.passed)
        .count();
    let checkpoints_json = checkpoints
        .iter()
        .map(checkpoint_json)
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "{{\n  \"passed\": {},\n  \"checkpoint_count\": {},\n  \"checks\": [\n    {{\"name\": \"checkpoint_marker_pixel_hits\", \"passed\": {}, \"value\": {}, \"comparator\": \">=\", \"threshold\": {}, \"unit\": \"checkpoints\"}}\n  ],\n  \"checkpoints\": [\n{}\n  ]\n}}",
        passed,
        checkpoint_count,
        passed_checkpoint_count >= checkpoint_count,
        passed_checkpoint_count,
        checkpoint_count,
        checkpoints_json
    )
}

fn checkpoint_json(checkpoint: &CheckpointAudit) -> String {
    let markers_json = checkpoint
        .markers
        .iter()
        .map(marker_json)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "    {{\n      \"metadata_path\": {},\n      \"screenshot_path\": {},\n      \"checkpoint\": {},\n      \"passed\": {},\n      \"visible_marker_count\": {},\n      \"marker_pixel_hit_count\": {},\n      \"markers\": [\n{}\n      ]\n    }}",
        json_string(&checkpoint.metadata_path),
        json_string(&checkpoint.screenshot_path),
        json_string(&checkpoint.checkpoint),
        checkpoint.passed,
        checkpoint.visible_marker_count,
        checkpoint.marker_pixel_hit_count,
        markers_json
    )
}

fn marker_json(marker: &MarkerAudit) -> String {
    let screen = match (marker.screen_x, marker.screen_y) {
        (Some(x), Some(y)) => format!("{{\"x\": {}, \"y\": {}}}", json_number(x), json_number(y)),
        _ => "null".to_string(),
    };
    format!(
        "        {{\"kind\": {}, \"label\": {}, \"current_objective\": {}, \"in_viewport\": {}, \"screen\": {}, \"marker_pixel_hits\": {}, \"passed\": {}}}",
        json_string(&marker.kind),
        json_string(&marker.label),
        marker.current_objective,
        marker.in_viewport,
        screen,
        marker.marker_pixel_hits,
        marker.passed
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
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn marker_projection_audit_accepts_pixels_near_visible_marker() {
        let temp_dir = unique_temp_dir("marker_projection_accepts");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let mut image = RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]));
        image.put_pixel(40, 30, Rgb([246, 58, 142]));
        image.put_pixel(41, 30, Rgb([246, 58, 142]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json(&screenshot_path, 40.0, 30.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(audit.passed);
        assert_eq!(audit.marker_pixel_hit_count, 1);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn marker_projection_audit_rejects_missing_projected_pixels() {
        let temp_dir = unique_temp_dir("marker_projection_rejects");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        let image = RgbImage::from_pixel(80, 60, Rgb([72, 118, 172]));
        image.save(&screenshot_path).expect("screenshot");
        fs::write(
            &metadata_path,
            marker_metadata_json(&screenshot_path, 40.0, 30.0),
        )
        .expect("metadata");

        let audit = audit_checkpoint_path(&metadata_path).expect("audit");

        assert!(!audit.passed);
        assert_eq!(audit.marker_pixel_hit_count, 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "nau_{name}_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    fn marker_metadata_json(screenshot_path: &Path, x: f64, y: f64) -> String {
        format!(
            "{{\"passed\": true, \"checkpoint\": \"test\", \"screenshot\": {}, \"markers\": [{{\"kind\": \"route_cairn\", \"label\": \"test\", \"current_objective\": false, \"in_viewport\": true, \"screen\": {{\"x\": {}, \"y\": {}}}}}]}}",
            json_string(&screenshot_path.to_string_lossy()),
            json_number(x),
            json_number(y)
        )
    }
}
